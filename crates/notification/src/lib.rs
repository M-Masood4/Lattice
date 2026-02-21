use shared::models::{Notification, NotificationPreferences, Recommendation, TradeExecution};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

pub mod error;
pub mod email;

pub use error::{NotificationError, Result};
pub use email::{EmailNotification, EmailService, MockEmailService};

/// In-memory notification storage (for MVP - would use database in production)
type NotificationStore = Arc<RwLock<Vec<Notification>>>;
type PreferencesStore = Arc<RwLock<std::collections::HashMap<Uuid, NotificationPreferences>>>;

/// Notification service for managing user notifications
pub struct NotificationService {
    notifications: NotificationStore,
    preferences: PreferencesStore,
    email_service: Option<Arc<dyn EmailService>>,
}

impl NotificationService {
    /// Create a new notification service
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(RwLock::new(Vec::new())),
            preferences: Arc::new(RwLock::new(std::collections::HashMap::new())),
            email_service: None,
        }
    }

    /// Create a new notification service with email support
    pub fn with_email_service(email_service: Arc<dyn EmailService>) -> Self {
        Self {
            notifications: Arc::new(RwLock::new(Vec::new())),
            preferences: Arc::new(RwLock::new(std::collections::HashMap::new())),
            email_service: Some(email_service),
        }
    }

    /// Create a notification for a whale movement recommendation
    pub async fn create_whale_movement_notification(
        &self,
        user_id: Uuid,
        user_email: Option<&str>,
        recommendation: &Recommendation,
        whale_address: &str,
        token_symbol: &str,
    ) -> Result<Notification> {
        let notification = Notification {
            id: Uuid::new_v4(),
            user_id,
            notification_type: "WHALE_MOVEMENT".to_string(),
            title: format!("Whale Movement Detected: {}", token_symbol),
            message: format!(
                "A whale ({}) made a significant move. Recommendation: {} ({}% confidence)",
                &whale_address[..8.min(whale_address.len())],
                recommendation.action,
                recommendation.confidence
            ),
            data: Some(serde_json::json!({
                "whale_address": whale_address,
                "recommendation_id": recommendation.id,
                "action": recommendation.action,
                "confidence": recommendation.confidence,
                "reasoning": recommendation.reasoning,
            })),
            priority: if recommendation.confidence >= 80 {
                "HIGH".to_string()
            } else if recommendation.confidence >= 60 {
                "MEDIUM".to_string()
            } else {
                "LOW".to_string()
            },
            read: false,
            created_at: chrono::Utc::now(),
        };

        self.store_notification(&notification).await?;
        
        // Send email if enabled and email service is available
        if let (Some(email), Some(email_service)) = (user_email, &self.email_service) {
            let prefs = self.get_preferences(user_id).await?;
            if prefs.email_enabled {
                let email_notification = email::build_whale_movement_email(
                    email,
                    recommendation,
                    whale_address,
                    token_symbol,
                );
                if let Err(e) = email_service.send_email(email_notification).await {
                    tracing::warn!("Failed to send email notification: {:?}", e);
                }
            }
        }

        info!("Created whale movement notification for user {}", user_id);
        Ok(notification)
    }

    /// Create a notification for a trade execution
    pub async fn create_trade_notification(
        &self,
        user_id: Uuid,
        user_email: Option<&str>,
        trade: &TradeExecution,
    ) -> Result<Notification> {
        let notification = Notification {
            id: Uuid::new_v4(),
            user_id,
            notification_type: "TRADE_EXECUTED".to_string(),
            title: format!("Trade Executed: {}", trade.action),
            message: format!(
                "Auto-trader executed a {} trade for {} tokens",
                trade.action, trade.token_mint
            ),
            data: Some(serde_json::json!({
                "trade_id": trade.id,
                "action": trade.action,
                "token_mint": trade.token_mint,
                "amount": trade.amount,
                "transaction_signature": trade.transaction_signature,
            })),
            priority: "HIGH".to_string(),
            read: false,
            created_at: chrono::Utc::now(),
        };

        self.store_notification(&notification).await?;
        
        // Send email if enabled and email service is available
        if let (Some(email), Some(email_service)) = (user_email, &self.email_service) {
            let prefs = self.get_preferences(user_id).await?;
            if prefs.email_enabled {
                let email_notification = email::build_trade_execution_email(email, trade);
                if let Err(e) = email_service.send_email(email_notification).await {
                    tracing::warn!("Failed to send email notification: {:?}", e);
                }
            }
        }

        info!("Created trade notification for user {}", user_id);
        Ok(notification)
    }

    /// Store a notification in memory
    async fn store_notification(&self, notification: &Notification) -> Result<()> {
        let mut notifications = self.notifications.write().await;
        notifications.push(notification.clone());
        Ok(())
    }

    /// Get user's notification preferences
    pub async fn get_preferences(&self, user_id: Uuid) -> Result<NotificationPreferences> {
        let preferences = self.preferences.read().await;
        
        match preferences.get(&user_id) {
            Some(prefs) => Ok(prefs.clone()),
            None => {
                drop(preferences); // Release read lock
                let default_prefs = self.create_default_preferences(user_id).await?;
                Ok(default_prefs)
            }
        }
    }

    /// Create default notification preferences for a new user
    async fn create_default_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationPreferences> {
        let prefs = NotificationPreferences {
            user_id,
            in_app_enabled: true,
            email_enabled: false,
            push_enabled: false,
            frequency: "REALTIME".to_string(),
            minimum_movement_percent: 5.0,
            minimum_confidence: 70,
        };

        let mut preferences = self.preferences.write().await;
        preferences.insert(user_id, prefs.clone());

        info!("Created default notification preferences for user {}", user_id);
        Ok(prefs)
    }

    /// Update user's notification preferences
    pub async fn update_preferences(
        &self,
        prefs: &NotificationPreferences,
    ) -> Result<()> {
        let mut preferences = self.preferences.write().await;
        preferences.insert(prefs.user_id, prefs.clone());

        info!("Updated notification preferences for user {}", prefs.user_id);
        Ok(())
    }

    /// Get unread notifications for a user
    pub async fn get_unread_notifications(&self, user_id: Uuid) -> Result<Vec<Notification>> {
        let notifications = self.notifications.read().await;
        
        let unread: Vec<Notification> = notifications
            .iter()
            .filter(|n| n.user_id == user_id && !n.read)
            .cloned()
            .collect();

        Ok(unread)
    }

    /// Mark a notification as read
    pub async fn mark_as_read(&self, notification_id: Uuid) -> Result<()> {
        let mut notifications = self.notifications.write().await;
        
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == notification_id) {
            notification.read = true;
        }

        Ok(())
    }

    /// Check if notification should be sent based on user preferences
    pub async fn should_notify(
        &self,
        user_id: Uuid,
        movement_percent: f64,
        confidence: i32,
    ) -> Result<bool> {
        let prefs = self.get_preferences(user_id).await?;

        // Check if in-app notifications are enabled
        if !prefs.in_app_enabled {
            return Ok(false);
        }

        // Check minimum movement threshold
        if movement_percent < prefs.minimum_movement_percent {
            return Ok(false);
        }

        // Check minimum confidence threshold
        if confidence < prefs.minimum_confidence {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get all notifications for a user (read and unread)
    pub async fn get_all_notifications(&self, user_id: Uuid) -> Result<Vec<Notification>> {
        let notifications = self.notifications.read().await;
        
        let user_notifications: Vec<Notification> = notifications
            .iter()
            .filter(|n| n.user_id == user_id)
            .cloned()
            .collect();

        Ok(user_notifications)
    }

    /// Get notification count for a user
    pub async fn get_notification_count(&self, user_id: Uuid) -> Result<(usize, usize)> {
        let notifications = self.notifications.read().await;
        
        let total = notifications.iter().filter(|n| n.user_id == user_id).count();
        let unread = notifications
            .iter()
            .filter(|n| n.user_id == user_id && !n.read)
            .count();

        Ok((total, unread))
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> Result<()> {
        let mut notifications = self.notifications.write().await;
        
        for notification in notifications.iter_mut() {
            if notification.user_id == user_id && !notification.read {
                notification.read = true;
            }
        }

        info!("Marked all notifications as read for user {}", user_id);
        Ok(())
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_priority_high_confidence() {
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Strong signal".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let priority = if recommendation.confidence >= 80 {
            "HIGH"
        } else if recommendation.confidence >= 60 {
            "MEDIUM"
        } else {
            "LOW"
        };

        assert_eq!(priority, "HIGH");
    }

    #[test]
    fn test_notification_priority_medium_confidence() {
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            action: "HOLD".to_string(),
            confidence: 65,
            reasoning: "Moderate signal".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let priority = if recommendation.confidence >= 80 {
            "HIGH"
        } else if recommendation.confidence >= 60 {
            "MEDIUM"
        } else {
            "LOW"
        };

        assert_eq!(priority, "MEDIUM");
    }

    #[test]
    fn test_default_preferences() {
        let user_id = Uuid::new_v4();
        let prefs = NotificationPreferences {
            user_id,
            in_app_enabled: true,
            email_enabled: false,
            push_enabled: false,
            frequency: "REALTIME".to_string(),
            minimum_movement_percent: 5.0,
            minimum_confidence: 70,
        };

        assert!(prefs.in_app_enabled);
        assert!(!prefs.email_enabled);
        assert!(!prefs.push_enabled);
        assert_eq!(prefs.frequency, "REALTIME");
        assert_eq!(prefs.minimum_movement_percent, 5.0);
        assert_eq!(prefs.minimum_confidence, 70);
    }

    #[tokio::test]
    async fn test_create_and_retrieve_notification() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();
        
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Strong whale accumulation".to_string(),
            suggested_amount: Some("100".to_string()),
            timeframe: Some("short-term".to_string()),
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let notification = service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale123", "SOL")
            .await
            .unwrap();

        assert_eq!(notification.user_id, user_id);
        assert_eq!(notification.notification_type, "WHALE_MOVEMENT");
        assert_eq!(notification.priority, "HIGH");
        assert!(!notification.read);

        let unread = service.get_unread_notifications(user_id).await.unwrap();
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].id, notification.id);
    }

    #[tokio::test]
    async fn test_mark_notification_as_read() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();
        
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "SELL".to_string(),
            confidence: 75,
            reasoning: "Whale dumping".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let notification = service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale456", "USDC")
            .await
            .unwrap();

        service.mark_as_read(notification.id).await.unwrap();

        let unread = service.get_unread_notifications(user_id).await.unwrap();
        assert_eq!(unread.len(), 0);
    }

    #[tokio::test]
    async fn test_default_preferences_creation() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();

        let prefs = service.get_preferences(user_id).await.unwrap();

        assert_eq!(prefs.user_id, user_id);
        assert!(prefs.in_app_enabled);
        assert!(!prefs.email_enabled);
        assert!(!prefs.push_enabled);
        assert_eq!(prefs.frequency, "REALTIME");
    }

    #[tokio::test]
    async fn test_update_preferences() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();

        let mut prefs = service.get_preferences(user_id).await.unwrap();
        prefs.email_enabled = true;
        prefs.minimum_confidence = 80;

        service.update_preferences(&prefs).await.unwrap();

        let updated_prefs = service.get_preferences(user_id).await.unwrap();
        assert!(updated_prefs.email_enabled);
        assert_eq!(updated_prefs.minimum_confidence, 80);
    }

    #[tokio::test]
    async fn test_should_notify_filters() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();

        // Get default preferences (min movement 5%, min confidence 70)
        let _prefs = service.get_preferences(user_id).await.unwrap();

        // Should notify - meets all criteria
        assert!(service.should_notify(user_id, 10.0, 75).await.unwrap());

        // Should not notify - movement too small
        assert!(!service.should_notify(user_id, 3.0, 75).await.unwrap());

        // Should not notify - confidence too low
        assert!(!service.should_notify(user_id, 10.0, 50).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_all_notifications() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();
        
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        // Create two notifications
        service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale1", "SOL")
            .await
            .unwrap();
        service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale2", "USDC")
            .await
            .unwrap();

        let all = service.get_all_notifications(user_id).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_get_notification_count() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();
        
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "SELL".to_string(),
            confidence: 75,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        // Create three notifications
        let n1 = service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale1", "SOL")
            .await
            .unwrap();
        service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale2", "USDC")
            .await
            .unwrap();
        service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale3", "BTC")
            .await
            .unwrap();

        // Mark one as read
        service.mark_as_read(n1.id).await.unwrap();

        let (total, unread) = service.get_notification_count(user_id).await.unwrap();
        assert_eq!(total, 3);
        assert_eq!(unread, 2);
    }

    #[tokio::test]
    async fn test_mark_all_as_read() {
        let service = NotificationService::new();
        let user_id = Uuid::new_v4();
        
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "HOLD".to_string(),
            confidence: 70,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        // Create multiple notifications
        service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale1", "SOL")
            .await
            .unwrap();
        service
            .create_whale_movement_notification(user_id, None, &recommendation, "whale2", "USDC")
            .await
            .unwrap();

        // Mark all as read
        service.mark_all_as_read(user_id).await.unwrap();

        let unread = service.get_unread_notifications(user_id).await.unwrap();
        assert_eq!(unread.len(), 0);
    }

    #[tokio::test]
    async fn test_email_notification_with_service() {
        let email_service = Arc::new(MockEmailService) as Arc<dyn EmailService>;
        let service = NotificationService::with_email_service(email_service);
        let user_id = Uuid::new_v4();
        
        // Enable email notifications
        let mut prefs = service.get_preferences(user_id).await.unwrap();
        prefs.email_enabled = true;
        service.update_preferences(&prefs).await.unwrap();
        
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id,
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Test".to_string(),
            suggested_amount: None,
            timeframe: None,
            risks: None,
            created_at: chrono::Utc::now(),
        };

        // Should send email notification
        let notification = service
            .create_whale_movement_notification(
                user_id,
                Some("user@example.com"),
                &recommendation,
                "whale123",
                "SOL",
            )
            .await
            .unwrap();

        assert_eq!(notification.user_id, user_id);
    }
}
