use shared::models::Subscription;
use std::collections::HashMap;
use std::sync::Arc;
use stripe::{
    Client, CreateSubscription, CreateSubscriptionItems, Customer, EventObject, EventType,
    Subscription as StripeSubscription, SubscriptionId, SubscriptionStatus, Webhook,
};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

pub mod error;

pub use error::{PaymentError, Result};

/// Subscription tier configuration
#[derive(Debug, Clone)]
pub struct SubscriptionTier {
    pub name: String,
    pub stripe_price_id: String,
    pub features: TierFeatures,
}

#[derive(Debug, Clone)]
pub struct TierFeatures {
    pub max_whales: usize,
    pub auto_trader_enabled: bool,
    pub advanced_analytics: bool,
    pub priority_support: bool,
}

/// Payment service for managing subscriptions via Stripe
pub struct PaymentService {
    stripe_client: Client,
    tiers: HashMap<String, SubscriptionTier>,
    subscriptions: Arc<RwLock<HashMap<Uuid, Subscription>>>,
    webhook_secret: String,
}

impl PaymentService {
    /// Create a new payment service
    pub fn new(stripe_api_key: String, webhook_secret: String) -> Result<Self> {
        let stripe_client = Client::new(stripe_api_key);

        // Define subscription tiers
        let mut tiers = HashMap::new();

        tiers.insert(
            "BASIC".to_string(),
            SubscriptionTier {
                name: "BASIC".to_string(),
                stripe_price_id: "price_basic".to_string(), // Replace with actual Stripe price ID
                features: TierFeatures {
                    max_whales: 10,
                    auto_trader_enabled: false,
                    advanced_analytics: false,
                    priority_support: false,
                },
            },
        );

        tiers.insert(
            "PREMIUM".to_string(),
            SubscriptionTier {
                name: "PREMIUM".to_string(),
                stripe_price_id: "price_premium".to_string(), // Replace with actual Stripe price ID
                features: TierFeatures {
                    max_whales: 100,
                    auto_trader_enabled: true,
                    advanced_analytics: true,
                    priority_support: true,
                },
            },
        );

        Ok(Self {
            stripe_client,
            tiers,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            webhook_secret,
        })
    }

    /// Create a new subscription for a user
    pub async fn create_subscription(
        &self,
        user_id: Uuid,
        user_email: &str,
        tier_name: &str,
    ) -> Result<Subscription> {
        // Get tier configuration
        let tier = self
            .tiers
            .get(tier_name)
            .ok_or_else(|| PaymentError::InvalidTier(tier_name.to_string()))?;

        // Create or retrieve Stripe customer
        let customer = self.create_or_get_customer(user_email).await?;

        // Create subscription in Stripe
        let mut create_sub = CreateSubscription::new(customer.id.clone());
        create_sub.items = Some(vec![CreateSubscriptionItems {
            price: Some(tier.stripe_price_id.clone()),
            ..Default::default()
        }]);

        let stripe_subscription = StripeSubscription::create(&self.stripe_client, create_sub)
            .await
            .map_err(PaymentError::from)?;

        // Convert to our subscription model
        let subscription = Subscription {
            id: Uuid::new_v4(),
            user_id,
            stripe_subscription_id: stripe_subscription.id.to_string(),
            tier: tier_name.to_string(),
            status: self.convert_stripe_status(&stripe_subscription.status),
            current_period_end: chrono::DateTime::from_timestamp(
                stripe_subscription.current_period_end,
                0,
            )
            .unwrap_or_else(chrono::Utc::now),
            cancel_at_period_end: stripe_subscription.cancel_at_period_end,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Store subscription
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(user_id, subscription.clone());

        info!(
            "Created subscription {} for user {} with tier {}",
            subscription.id, user_id, tier_name
        );

        Ok(subscription)
    }

    /// Cancel a subscription
    pub async fn cancel_subscription(&self, user_id: Uuid) -> Result<()> {
        let subscriptions = self.subscriptions.read().await;
        let subscription = subscriptions
            .get(&user_id)
            .ok_or_else(|| PaymentError::SubscriptionNotFound(user_id.to_string()))?;

        // Cancel in Stripe
        let stripe_sub_id: SubscriptionId = subscription.stripe_subscription_id.parse()
            .map_err(|_| PaymentError::PaymentFailed("Invalid subscription ID".to_string()))?;
        StripeSubscription::cancel(&self.stripe_client, &stripe_sub_id, Default::default())
            .await
            .map_err(PaymentError::from)?;

        drop(subscriptions);

        // Update local subscription
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(sub) = subscriptions.get_mut(&user_id) {
            sub.status = "CANCELED".to_string();
            sub.cancel_at_period_end = true;
            sub.updated_at = chrono::Utc::now();
        }

        info!("Canceled subscription for user {}", user_id);

        Ok(())
    }

    /// Check if user has active subscription
    pub async fn has_active_subscription(&self, user_id: Uuid, tier: &str) -> Result<bool> {
        let subscriptions = self.subscriptions.read().await;

        if let Some(subscription) = subscriptions.get(&user_id) {
            Ok(subscription.status == "ACTIVE" && subscription.tier == tier)
        } else {
            Ok(false)
        }
    }

    /// Get subscription for user
    pub async fn get_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>> {
        let subscriptions = self.subscriptions.read().await;
        Ok(subscriptions.get(&user_id).cloned())
    }

    /// Handle Stripe webhook event
    pub async fn handle_webhook(&self, payload: &str, signature: &str) -> Result<()> {
        // Verify webhook signature
        let event = Webhook::construct_event(payload, signature, &self.webhook_secret)
            .map_err(|e| PaymentError::WebhookVerificationFailed(e.to_string()))?;

        info!("Received webhook event: {:?}", event.type_);

        match event.type_ {
            EventType::CustomerSubscriptionCreated => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    self.handle_subscription_created(subscription).await?;
                }
            }
            EventType::CustomerSubscriptionUpdated => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    self.handle_subscription_updated(subscription).await?;
                }
            }
            EventType::CustomerSubscriptionDeleted => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    self.handle_subscription_deleted(subscription).await?;
                }
            }
            EventType::InvoicePaymentFailed => {
                if let EventObject::Invoice(invoice) = event.data.object {
                    self.handle_payment_failed(invoice).await?;
                }
            }
            _ => {
                info!("Unhandled webhook event type: {:?}", event.type_);
            }
        }

        Ok(())
    }

    /// Create or retrieve Stripe customer
    async fn create_or_get_customer(&self, email: &str) -> Result<Customer> {
        // In production, check if customer exists first
        // For MVP, create new customer
        let mut create_customer = stripe::CreateCustomer::new();
        create_customer.email = Some(email);

        Customer::create(&self.stripe_client, create_customer)
            .await
            .map_err(PaymentError::from)
    }

    /// Handle subscription created event
    async fn handle_subscription_created(&self, subscription: StripeSubscription) -> Result<()> {
        info!(
            "Subscription created: {}",
            subscription.id
        );
        // Update local subscription status
        self.update_subscription_from_stripe(subscription).await
    }

    /// Handle subscription updated event
    async fn handle_subscription_updated(&self, subscription: StripeSubscription) -> Result<()> {
        info!(
            "Subscription updated: {}",
            subscription.id
        );
        self.update_subscription_from_stripe(subscription).await
    }

    /// Handle subscription deleted event
    async fn handle_subscription_deleted(&self, subscription: StripeSubscription) -> Result<()> {
        info!(
            "Subscription deleted: {}",
            subscription.id
        );
        self.update_subscription_from_stripe(subscription).await
    }

    /// Handle payment failed event
    async fn handle_payment_failed(&self, invoice: stripe::Invoice) -> Result<()> {
        warn!(
            "Payment failed for invoice: {}",
            invoice.id
        );
        // In production, send notification to user
        Ok(())
    }

    /// Update local subscription from Stripe subscription
    async fn update_subscription_from_stripe(&self, stripe_sub: StripeSubscription) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;

        // Find subscription by Stripe ID
        for (_, subscription) in subscriptions.iter_mut() {
            if stripe_sub.id == subscription.stripe_subscription_id {
                subscription.status = self.convert_stripe_status(&stripe_sub.status);
                subscription.current_period_end = chrono::DateTime::from_timestamp(
                    stripe_sub.current_period_end,
                    0,
                )
                .unwrap_or_else(chrono::Utc::now);
                subscription.cancel_at_period_end = stripe_sub.cancel_at_period_end;
                subscription.updated_at = chrono::Utc::now();
                break;
            }
        }

        Ok(())
    }

    /// Convert Stripe subscription status to our status
    fn convert_stripe_status(&self, status: &SubscriptionStatus) -> String {
        match status {
            SubscriptionStatus::Active => "ACTIVE".to_string(),
            SubscriptionStatus::PastDue => "PAST_DUE".to_string(),
            SubscriptionStatus::Canceled => "CANCELED".to_string(),
            SubscriptionStatus::Trialing => "TRIALING".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    /// Get tier features
    pub fn get_tier_features(&self, tier_name: &str) -> Option<&TierFeatures> {
        self.tiers.get(tier_name).map(|t| &t.features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> PaymentService {
        PaymentService::new(
            "sk_test_mock".to_string(),
            "whsec_test_mock".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_payment_service_creation() {
        let service = create_test_service();
        assert_eq!(service.tiers.len(), 2);
        assert!(service.tiers.contains_key("BASIC"));
        assert!(service.tiers.contains_key("PREMIUM"));
    }

    #[test]
    fn test_tier_features() {
        let service = create_test_service();

        let basic = service.get_tier_features("BASIC").unwrap();
        assert_eq!(basic.max_whales, 10);
        assert!(!basic.auto_trader_enabled);

        let premium = service.get_tier_features("PREMIUM").unwrap();
        assert_eq!(premium.max_whales, 100);
        assert!(premium.auto_trader_enabled);
    }

    #[tokio::test]
    async fn test_has_active_subscription_none() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let result = service
            .has_active_subscription(user_id, "PREMIUM")
            .await
            .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_get_subscription_none() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let result = service.get_subscription(user_id).await.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_convert_stripe_status() {
        let service = create_test_service();

        assert_eq!(
            service.convert_stripe_status(&SubscriptionStatus::Active),
            "ACTIVE"
        );
        assert_eq!(
            service.convert_stripe_status(&SubscriptionStatus::PastDue),
            "PAST_DUE"
        );
        assert_eq!(
            service.convert_stripe_status(&SubscriptionStatus::Canceled),
            "CANCELED"
        );
        assert_eq!(
            service.convert_stripe_status(&SubscriptionStatus::Trialing),
            "TRIALING"
        );
    }
}
