use shared::models::{Recommendation, Subscription, TradeExecution, UserSettings};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

pub mod error;
pub mod validation;
pub mod transaction;

pub use error::{TradingError, Result};
pub use validation::{TradeValidator, ValidationResult};
pub use transaction::{TransactionBuilder, SolanaTransaction, TransactionStatus};

// Re-export notification types for convenience
pub use notification::{NotificationService, NotificationError};

/// Trade request from user or auto-trader
#[derive(Debug, Clone)]
pub struct TradeRequest {
    pub user_id: Uuid,
    pub action: String, // BUY or SELL
    pub token_mint: String,
    pub amount: String,
    pub slippage_tolerance: f64,
    pub recommendation_id: Option<Uuid>,
}

/// Trading service for executing trades
pub struct TradingService {
    validator: TradeValidator,
    transaction_builder: TransactionBuilder,
    trade_history: Arc<RwLock<Vec<TradeExecution>>>,
    daily_trade_counts: Arc<RwLock<HashMap<Uuid, usize>>>,
    notification_service: Option<Arc<NotificationService>>,
}

impl TradingService {
    /// Create a new trading service
    pub fn new() -> Self {
        Self {
            validator: TradeValidator::new(),
            transaction_builder: TransactionBuilder::new(),
            trade_history: Arc::new(RwLock::new(Vec::new())),
            daily_trade_counts: Arc::new(RwLock::new(HashMap::new())),
            notification_service: None,
        }
    }

    /// Create a new trading service with notification support
    pub fn with_notification_service(notification_service: Arc<NotificationService>) -> Self {
        Self {
            validator: TradeValidator::new(),
            transaction_builder: TransactionBuilder::new(),
            trade_history: Arc::new(RwLock::new(Vec::new())),
            daily_trade_counts: Arc::new(RwLock::new(HashMap::new())),
            notification_service: Some(notification_service),
        }
    }

    /// Validate a trade request against user limits and safety checks
    pub async fn validate_trade(
        &self,
        trade: &TradeRequest,
        user_settings: &UserSettings,
        subscription: Option<&Subscription>,
        portfolio_value: f64,
    ) -> Result<ValidationResult> {
        // Get daily trade count
        let daily_count = self.get_daily_trade_count(trade.user_id).await;

        // Validate the trade
        let result = self.validator.validate(
            trade,
            user_settings,
            subscription,
            portfolio_value,
            daily_count,
        )?;

        Ok(result)
    }

    /// Check if auto-trader should execute based on recommendation
    pub async fn should_auto_trade(
        &self,
        user_settings: &UserSettings,
        recommendation: &Recommendation,
        subscription: Option<&Subscription>,
    ) -> Result<bool> {
        // Check if auto-trader is enabled
        if !user_settings.auto_trader_enabled {
            return Ok(false);
        }

        // Check if user has premium subscription
        if subscription.is_none() {
            warn!(
                "Auto-trader disabled for user {} - no premium subscription",
                user_settings.user_id
            );
            return Ok(false);
        }

        let sub = subscription.unwrap();
        if sub.tier != "PREMIUM" || sub.status != "ACTIVE" {
            warn!(
                "Auto-trader disabled for user {} - invalid subscription",
                user_settings.user_id
            );
            return Ok(false);
        }

        // Check confidence threshold (>= 80%)
        if recommendation.confidence < 80 {
            info!(
                "Auto-trader skipping trade for user {} - confidence too low ({}%)",
                user_settings.user_id, recommendation.confidence
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Log a trade execution and send notification
    pub async fn log_trade(&self, execution: &TradeExecution, user_email: Option<&str>) -> Result<()> {
        let mut history = self.trade_history.write().await;
        history.push(execution.clone());

        // Increment daily trade count
        let mut counts = self.daily_trade_counts.write().await;
        *counts.entry(execution.user_id).or_insert(0) += 1;

        info!(
            "Logged trade {} for user {}: {} {} {}",
            execution.id, execution.user_id, execution.action, execution.amount, execution.token_mint
        );

        // Send notification if notification service is available
        if let Some(notification_service) = &self.notification_service {
            if let Err(e) = notification_service
                .create_trade_notification(execution.user_id, user_email, execution)
                .await
            {
                warn!("Failed to send trade notification: {:?}", e);
            }
        }

        Ok(())
    }

    /// Get trade history for a user
    pub async fn get_trade_history(&self, user_id: Uuid) -> Result<Vec<TradeExecution>> {
        let history = self.trade_history.read().await;
        let user_trades: Vec<TradeExecution> = history
            .iter()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();

        Ok(user_trades)
    }

    /// Get daily trade count for a user
    async fn get_daily_trade_count(&self, user_id: Uuid) -> usize {
        let counts = self.daily_trade_counts.read().await;
        *counts.get(&user_id).unwrap_or(&0)
    }

    /// Reset daily trade counts (should be called daily)
    pub async fn reset_daily_counts(&self) {
        let mut counts = self.daily_trade_counts.write().await;
        counts.clear();
        info!("Reset daily trade counts");
    }

    /// Execute a trade for auto-trader
    pub async fn execute_auto_trade(
        &self,
        recommendation: &Recommendation,
        user_settings: &UserSettings,
        subscription: Option<&Subscription>,
        portfolio_value: f64,
        user_email: Option<&str>,
    ) -> Result<TradeExecution> {
        // Check if auto-trader should execute
        if !self
            .should_auto_trade(user_settings, recommendation, subscription)
            .await?
        {
            return Err(TradingError::ValidationError(
                "Auto-trader conditions not met".to_string(),
            ));
        }

        // Determine trade amount from recommendation
        let amount = recommendation
            .suggested_amount
            .clone()
            .unwrap_or_else(|| "0".to_string());

        // Create trade request
        let trade_request = TradeRequest {
            user_id: user_settings.user_id,
            action: recommendation.action.clone(),
            token_mint: "SOL".to_string(), // Simplified for MVP
            amount,
            slippage_tolerance: 1.0,
            recommendation_id: Some(recommendation.id),
        };

        // Validate trade
        let daily_count = self.get_daily_trade_count(user_settings.user_id).await;
        let validation = self.validator.validate(
            &trade_request,
            user_settings,
            subscription,
            portfolio_value,
            daily_count,
        )?;

        if !validation.valid {
            return Err(TradingError::ValidationError(format!(
                "Trade validation failed: {:?}",
                validation.errors
            )));
        }

        // Build transaction
        let transaction = self
            .transaction_builder
            .build_transaction(&trade_request)
            .await?;

        // Validate signature
        if !self
            .transaction_builder
            .validate_signature(&transaction.signature)?
        {
            return Err(TradingError::TransactionError(
                "Invalid transaction signature".to_string(),
            ));
        }

        // Submit transaction
        let tx_signature = transaction.submit().await?;

        // Monitor confirmation
        let status = transaction.check_status().await?;

        // Create trade execution record
        let execution = TradeExecution {
            id: Uuid::new_v4(),
            user_id: user_settings.user_id,
            recommendation_id: Some(recommendation.id),
            transaction_signature: tx_signature,
            action: trade_request.action,
            token_mint: trade_request.token_mint,
            amount: trade_request.amount,
            price_usd: None, // Would be populated from actual trade
            total_value_usd: None,
            status: match status {
                TransactionStatus::Confirmed => "CONFIRMED".to_string(),
                TransactionStatus::Pending => "PENDING".to_string(),
                TransactionStatus::Failed => "FAILED".to_string(),
            },
            executed_at: chrono::Utc::now(),
            confirmed_at: if status == TransactionStatus::Confirmed {
                Some(chrono::Utc::now())
            } else {
                None
            },
        };

        // Log the trade and send notification
        self.log_trade(&execution, user_email).await?;

        info!(
            "Auto-trader executed trade {} for user {}",
            execution.id, user_settings.user_id
        );

        Ok(execution)
    }
}

impl Default for TradingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_settings(auto_trader_enabled: bool) -> UserSettings {
        UserSettings {
            user_id: Uuid::new_v4(),
            auto_trader_enabled,
            max_trade_percentage: 5.0,
            max_daily_trades: 10,
            stop_loss_percentage: 10.0,
            risk_tolerance: "MEDIUM".to_string(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn create_test_subscription(tier: &str, status: &str) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            stripe_subscription_id: "sub_test".to_string(),
            tier: tier.to_string(),
            status: status.to_string(),
            current_period_end: chrono::Utc::now() + chrono::Duration::days(30),
            cancel_at_period_end: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_should_auto_trade_disabled() {
        let service = TradingService::new();
        let settings = create_test_settings(false);
        let subscription = create_test_subscription("PREMIUM", "ACTIVE");

        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id: settings.user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let result = service
            .should_auto_trade(&settings, &recommendation, Some(&subscription))
            .await
            .unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_should_auto_trade_no_subscription() {
        let service = TradingService::new();
        let settings = create_test_settings(true);

        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id: settings.user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let result = service
            .should_auto_trade(&settings, &recommendation, None)
            .await
            .unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_should_auto_trade_low_confidence() {
        let service = TradingService::new();
        let settings = create_test_settings(true);
        let subscription = create_test_subscription("PREMIUM", "ACTIVE");

        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id: settings.user_id,
            action: "BUY".to_string(),
            confidence: 70,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let result = service
            .should_auto_trade(&settings, &recommendation, Some(&subscription))
            .await
            .unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_should_auto_trade_success() {
        let service = TradingService::new();
        let settings = create_test_settings(true);
        let subscription = create_test_subscription("PREMIUM", "ACTIVE");

        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id: settings.user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let result = service
            .should_auto_trade(&settings, &recommendation, Some(&subscription))
            .await
            .unwrap();

        assert!(result);
    }

    #[tokio::test]
    async fn test_log_and_retrieve_trade() {
        let service = TradingService::new();
        let user_id = Uuid::new_v4();

        let trade = TradeExecution {
            id: Uuid::new_v4(),
            user_id,
            recommendation_id: Some(Uuid::new_v4()),
            transaction_signature: "sig123".to_string(),
            action: "BUY".to_string(),
            token_mint: "SOL".to_string(),
            amount: "10".to_string(),
            price_usd: Some(100.0),
            total_value_usd: Some(1000.0),
            status: "CONFIRMED".to_string(),
            executed_at: chrono::Utc::now(),
            confirmed_at: Some(chrono::Utc::now()),
        };

        service.log_trade(&trade, None).await.unwrap();

        let history = service.get_trade_history(user_id).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, trade.id);
    }

    #[tokio::test]
    async fn test_daily_trade_count() {
        let service = TradingService::new();
        let user_id = Uuid::new_v4();

        // Log multiple trades
        for i in 0..3 {
            let trade = TradeExecution {
                id: Uuid::new_v4(),
                user_id,
                recommendation_id: None,
                transaction_signature: format!("sig{}", i),
                action: "BUY".to_string(),
                token_mint: "SOL".to_string(),
                amount: "10".to_string(),
                price_usd: Some(100.0),
                total_value_usd: Some(1000.0),
                status: "CONFIRMED".to_string(),
                executed_at: chrono::Utc::now(),
                confirmed_at: Some(chrono::Utc::now()),
            };
            service.log_trade(&trade, None).await.unwrap();
        }

        let count = service.get_daily_trade_count(user_id).await;
        assert_eq!(count, 3);

        // Reset counts
        service.reset_daily_counts().await;
        let count_after_reset = service.get_daily_trade_count(user_id).await;
        assert_eq!(count_after_reset, 0);
    }

    #[tokio::test]
    async fn test_execute_auto_trade_success() {
        let service = TradingService::new();
        let user_id = Uuid::new_v4();
        let settings = create_test_settings(true);
        let subscription = create_test_subscription("PREMIUM", "ACTIVE");

        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Strong signal".to_string(),
            suggested_amount: Some("100".to_string()),
            timeframe: Some("short-term".to_string()),
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let result = service
            .execute_auto_trade(&recommendation, &settings, Some(&subscription), 10000.0, None)
            .await;

        assert!(result.is_ok());
        let execution = result.unwrap();
        assert_eq!(execution.user_id, settings.user_id);
        assert_eq!(execution.action, "BUY");
        assert_eq!(execution.status, "CONFIRMED");
    }

    #[tokio::test]
    async fn test_execute_auto_trade_disabled() {
        let service = TradingService::new();
        let user_id = Uuid::new_v4();
        let settings = create_test_settings(false); // Auto-trader disabled
        let subscription = create_test_subscription("PREMIUM", "ACTIVE");

        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Strong signal".to_string(),
            suggested_amount: Some("100".to_string()),
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let result = service
            .execute_auto_trade(&recommendation, &settings, Some(&subscription), 10000.0, None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_trade_notification_integration() {
        use notification::MockEmailService;
        
        let notification_service = Arc::new(NotificationService::with_email_service(
            Arc::new(MockEmailService),
        ));
        let service = TradingService::with_notification_service(notification_service.clone());
        let user_id = Uuid::new_v4();

        let trade = TradeExecution {
            id: Uuid::new_v4(),
            user_id,
            recommendation_id: Some(Uuid::new_v4()),
            transaction_signature: "sig_test".to_string(),
            action: "BUY".to_string(),
            token_mint: "SOL".to_string(),
            amount: "100".to_string(),
            price_usd: Some(150.0),
            total_value_usd: Some(15000.0),
            status: "CONFIRMED".to_string(),
            executed_at: chrono::Utc::now(),
            confirmed_at: Some(chrono::Utc::now()),
        };

        // Log trade with notification
        service.log_trade(&trade, Some("user@example.com")).await.unwrap();

        // Verify notification was created
        let notifications = notification_service.get_unread_notifications(user_id).await.unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].notification_type, "TRADE_EXECUTED");
        assert_eq!(notifications[0].priority, "HIGH");
    }

    #[tokio::test]
    async fn test_execute_auto_trade_with_notification() {
        use notification::MockEmailService;
        
        let notification_service = Arc::new(NotificationService::with_email_service(
            Arc::new(MockEmailService),
        ));
        let service = TradingService::with_notification_service(notification_service.clone());
        let user_id = Uuid::new_v4();
        let mut settings = create_test_settings(true);
        settings.user_id = user_id;
        let subscription = create_test_subscription("PREMIUM", "ACTIVE");

        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Strong signal".to_string(),
            suggested_amount: Some("100".to_string()),
            timeframe: Some("short-term".to_string()),
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let result = service
            .execute_auto_trade(&recommendation, &settings, Some(&subscription), 10000.0, Some("user@example.com"))
            .await;

        assert!(result.is_ok());
        
        // Verify notification was created
        let notifications = notification_service.get_unread_notifications(user_id).await.unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].notification_type, "TRADE_EXECUTED");
    }
}
