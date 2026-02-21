use crate::{AnalysisContextBuilder, ClaudeClient, Result};
use aws_sdk_sqs::{Client as SqsClient, types::Message};
use serde::{Deserialize, Serialize};
use shared::models::{Portfolio, Recommendation, UserSettings, WhaleMovement};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Event published by monitoring engine when whale movement detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleMovementEvent {
    pub movement: WhaleMovement,
    pub whale_address: String,
    pub token_symbol: String,
    pub affected_user_ids: Vec<Uuid>,
}

/// Consumer for whale movement events from SQS
pub struct WhaleMovementConsumer {
    sqs_client: SqsClient,
    queue_url: String,
    claude_client: ClaudeClient,
    context_builder: AnalysisContextBuilder,
}

impl WhaleMovementConsumer {
    /// Create a new whale movement consumer
    pub fn new(
        sqs_client: SqsClient,
        queue_url: String,
        claude_api_key: String,
    ) -> Self {
        Self {
            sqs_client,
            queue_url,
            claude_client: ClaudeClient::new(claude_api_key),
            context_builder: AnalysisContextBuilder::new(),
        }
    }

    /// Start consuming messages from the queue
    pub async fn start(&self) -> Result<()> {
        info!("Starting whale movement consumer on queue: {}", self.queue_url);

        loop {
            match self.poll_messages().await {
                Ok(messages) => {
                    if messages.is_empty() {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }

                    for message in messages {
                        if let Err(e) = self.process_message(message).await {
                            error!("Failed to process message: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to poll messages: {:?}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    /// Poll messages from SQS queue
    async fn poll_messages(&self) -> Result<Vec<Message>> {
        let response = self
            .sqs_client
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(20) // Long polling
            .send()
            .await
            .map_err(|e| crate::AIServiceError::ApiError(format!("SQS receive error: {}", e)))?;

        Ok(response.messages.unwrap_or_default())
    }

    /// Process a single message
    async fn process_message(&self, message: Message) -> Result<()> {
        let body = message.body().ok_or_else(|| {
            crate::AIServiceError::ParseError("Message has no body".to_string())
        })?;

        info!("Processing whale movement message");

        // Parse the event
        let event: WhaleMovementEvent = serde_json::from_str(body).map_err(|e| {
            crate::AIServiceError::ParseError(format!("Failed to parse event: {}", e))
        })?;

        // Process for each affected user
        for user_id in &event.affected_user_ids {
            match self.process_for_user(&event, *user_id).await {
                Ok(recommendation) => {
                    info!(
                        "Generated recommendation {} for user {}",
                        recommendation.id, user_id
                    );
                    // In a real implementation, this would:
                    // 1. Store recommendation in database
                    // 2. Publish to notification service
                    // 3. Check if auto-trader should execute
                }
                Err(e) => {
                    error!("Failed to process for user {}: {:?}", user_id, e);
                    // Continue processing other users
                }
            }
        }

        // Delete message from queue after successful processing
        if let Some(receipt_handle) = message.receipt_handle() {
            self.sqs_client
                .delete_message()
                .queue_url(&self.queue_url)
                .receipt_handle(receipt_handle)
                .send()
                .await
                .map_err(|e| {
                    crate::AIServiceError::ApiError(format!("Failed to delete message: {}", e))
                })?;
        }

        Ok(())
    }

    /// Process whale movement for a specific user
    async fn process_for_user(
        &self,
        event: &WhaleMovementEvent,
        user_id: Uuid,
    ) -> Result<Recommendation> {
        // In a real implementation, this would fetch from database
        // For now, we'll create mock data
        let user_portfolio = self.fetch_user_portfolio(user_id).await?;
        let user_settings = self.fetch_user_settings(user_id).await?;

        // Build analysis context
        let context = self
            .context_builder
            .build_context(
                &event.movement,
                event.whale_address.clone(),
                event.token_symbol.clone(),
                user_portfolio,
                &user_settings,
            )
            .await?;

        // Analyze with Claude
        let recommendation = self
            .claude_client
            .analyze_movement(&event.movement, user_id, context)
            .await?;

        Ok(recommendation)
    }

    /// Fetch user portfolio (placeholder - would query database in real implementation)
    async fn fetch_user_portfolio(&self, _user_id: Uuid) -> Result<Portfolio> {
        // Placeholder implementation
        warn!("Using mock portfolio data - implement database query");
        Ok(Portfolio {
            wallet_address: "mock_wallet".to_string(),
            assets: vec![],
            total_value_usd: 10000.0,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Fetch user settings (placeholder - would query database in real implementation)
    async fn fetch_user_settings(&self, user_id: Uuid) -> Result<UserSettings> {
        // Placeholder implementation
        warn!("Using mock user settings - implement database query");
        Ok(UserSettings {
            user_id,
            auto_trader_enabled: false,
            max_trade_percentage: 5.0,
            max_daily_trades: 10,
            stop_loss_percentage: 10.0,
            risk_tolerance: "MEDIUM".to_string(),
            updated_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_movement_event_serialization() {
        let event = WhaleMovementEvent {
            movement: WhaleMovement {
                id: Uuid::new_v4(),
                whale_id: Uuid::new_v4(),
                transaction_signature: "sig123".to_string(),
                movement_type: "BUY".to_string(),
                token_mint: "SOL".to_string(),
                amount: "1000".to_string(),
                percent_of_position: Some(15.5),
                detected_at: chrono::Utc::now(),
            },
            whale_address: "whale123".to_string(),
            token_symbol: "SOL".to_string(),
            affected_user_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WhaleMovementEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.whale_address, "whale123");
        assert_eq!(deserialized.token_symbol, "SOL");
        assert_eq!(deserialized.affected_user_ids.len(), 2);
    }
}
