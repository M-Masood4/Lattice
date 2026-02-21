use crate::Result;
use async_trait::async_trait;
use shared::models::{Recommendation, TradeExecution};

/// Email notification data
#[derive(Debug, Clone)]
pub struct EmailNotification {
    pub to: String,
    pub subject: String,
    pub body: String,
}

/// Trait for email service providers
#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(&self, notification: EmailNotification) -> Result<()>;
}

/// Mock email service for testing/MVP
pub struct MockEmailService;

#[async_trait]
impl EmailService for MockEmailService {
    async fn send_email(&self, notification: EmailNotification) -> Result<()> {
        // In production, this would integrate with AWS SES, SendGrid, etc.
        tracing::info!(
            "Mock email sent to {}: {}",
            notification.to,
            notification.subject
        );
        Ok(())
    }
}

/// Build email for whale movement notification
pub fn build_whale_movement_email(
    user_email: &str,
    recommendation: &Recommendation,
    whale_address: &str,
    token_symbol: &str,
) -> EmailNotification {
    let subject = format!("🐋 Whale Alert: {} Movement Detected", token_symbol);
    
    let body = format!(
        r#"
Hello,

A significant whale movement has been detected:

Whale Address: {}
Token: {}
Recommendation: {}
Confidence: {}%

Analysis:
{}

{}

---
This is an automated notification from Solana Whale Tracker.
To manage your notification preferences, visit your dashboard.
        "#,
        whale_address,
        token_symbol,
        recommendation.action,
        recommendation.confidence,
        recommendation.reasoning,
        if let Some(amount) = &recommendation.suggested_amount {
            format!("Suggested Amount: {}", amount)
        } else {
            String::new()
        }
    );

    EmailNotification {
        to: user_email.to_string(),
        subject,
        body,
    }
}

/// Build email for trade execution notification
pub fn build_trade_execution_email(
    user_email: &str,
    trade: &TradeExecution,
) -> EmailNotification {
    let subject = format!("✅ Trade Executed: {} {}", trade.action, trade.token_mint);
    
    let body = format!(
        r#"
Hello,

Your auto-trader has executed a trade:

Action: {}
Token: {}
Amount: {}
Transaction: {}
Status: {}

{}

---
This is an automated notification from Solana Whale Tracker.
To manage your auto-trader settings, visit your dashboard.
        "#,
        trade.action,
        trade.token_mint,
        trade.amount,
        trade.transaction_signature,
        trade.status,
        if let Some(value) = trade.total_value_usd {
            format!("Total Value: ${:.2}", value)
        } else {
            String::new()
        }
    );

    EmailNotification {
        to: user_email.to_string(),
        subject,
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_build_whale_movement_email() {
        let recommendation = Recommendation {
            id: Uuid::new_v4(),
            movement_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            action: "BUY".to_string(),
            confidence: 85,
            reasoning: "Strong accumulation signal".to_string(),
            suggested_amount: Some("100".to_string()),
            timeframe: Some("short-term".to_string()),
            risks: None,
            created_at: chrono::Utc::now(),
        };

        let email = build_whale_movement_email(
            "user@example.com",
            &recommendation,
            "whale123abc",
            "SOL",
        );

        assert_eq!(email.to, "user@example.com");
        assert!(email.subject.contains("SOL"));
        assert!(email.body.contains("whale123abc"));
        assert!(email.body.contains("BUY"));
        assert!(email.body.contains("85%"));
        assert!(email.body.contains("Strong accumulation signal"));
    }

    #[test]
    fn test_build_trade_execution_email() {
        let trade = TradeExecution {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            recommendation_id: Some(Uuid::new_v4()),
            transaction_signature: "sig123".to_string(),
            action: "SELL".to_string(),
            token_mint: "USDC".to_string(),
            amount: "500".to_string(),
            price_usd: Some(1.0),
            total_value_usd: Some(500.0),
            status: "CONFIRMED".to_string(),
            executed_at: chrono::Utc::now(),
            confirmed_at: Some(chrono::Utc::now()),
        };

        let email = build_trade_execution_email("user@example.com", &trade);

        assert_eq!(email.to, "user@example.com");
        assert!(email.subject.contains("SELL"));
        assert!(email.subject.contains("USDC"));
        assert!(email.body.contains("sig123"));
        assert!(email.body.contains("$500.00"));
    }

    #[tokio::test]
    async fn test_mock_email_service() {
        let service = MockEmailService;
        
        let notification = EmailNotification {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body: "Test body".to_string(),
        };

        let result = service.send_email(notification).await;
        assert!(result.is_ok());
    }
}
