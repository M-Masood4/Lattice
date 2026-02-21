use aws_sdk_sqs::Client as SqsClient;
use serde::{Deserialize, Serialize};
use shared::{Error, Result};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Message queue client for publishing whale movement events
/// 
/// **Validates: Requirements 3.2**
pub struct MessageQueueClient {
    sqs_client: SqsClient,
    queue_url: String,
}

/// Whale movement event to be published to the message queue
/// 
/// This event includes all information needed for downstream processing:
/// - Whale movement details (address, transaction, type, amount)
/// - Affected user IDs who are tracking this whale
/// 
/// **Validates: Requirements 3.2**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleMovementEvent {
    /// Unique identifier for this movement
    pub movement_id: Uuid,
    /// Whale account address
    pub whale_address: String,
    /// Solana transaction signature
    pub transaction_signature: String,
    /// Movement type: BUY or SELL
    pub movement_type: String,
    /// Token mint address
    pub token_mint: String,
    /// Amount moved (as string to preserve precision)
    pub amount: String,
    /// Percentage of whale's total position
    pub percent_of_position: f64,
    /// Timestamp when movement was detected
    pub detected_at: chrono::DateTime<chrono::Utc>,
    /// List of user IDs who are tracking this whale
    /// This allows downstream services to process recommendations for each affected user
    pub affected_user_ids: Vec<Uuid>,
}

impl MessageQueueClient {
    /// Create a new message queue client
    /// 
    /// # Arguments
    /// * `queue_url` - The AWS SQS queue URL (e.g., https://sqs.us-east-1.amazonaws.com/123456789012/whale-movements)
    /// 
    /// # Note
    /// This function uses the default AWS credential chain:
    /// 1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
    /// 2. AWS credentials file (~/.aws/credentials)
    /// 3. IAM role (if running on EC2/ECS/Lambda)
    /// 
    /// For production use, IAM roles with least-privilege policies are recommended.
    pub async fn new(queue_url: String) -> Result<Self> {
        info!("Initializing message queue client for queue: {}", queue_url);

        // Load AWS configuration from environment
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let sqs_client = SqsClient::new(&config);

        // Verify queue exists (non-destructive check)
        match sqs_client.get_queue_attributes()
            .queue_url(&queue_url)
            .send()
            .await
        {
            Ok(_) => {
                info!("Successfully connected to SQS queue");
            }
            Err(e) => {
                error!("Failed to verify SQS queue: {}", e);
                return Err(Error::Internal(format!(
                    "Failed to connect to SQS queue: {}. Please verify the queue URL and AWS credentials.",
                    e
                )));
            }
        }

        Ok(Self {
            sqs_client,
            queue_url,
        })
    }

    /// Publish a whale movement event to the message queue
    /// 
    /// This method serializes the event to JSON and sends it to the configured SQS queue.
    /// The event will be processed by downstream services (AI analysis, notifications, etc.)
    /// 
    /// **Validates: Requirements 3.2**
    pub async fn publish_movement(&self, event: WhaleMovementEvent) -> Result<()> {
        debug!(
            "Publishing whale movement event: {} for whale {} affecting {} users",
            event.movement_id,
            event.whale_address,
            event.affected_user_ids.len()
        );

        // Serialize event to JSON
        let message_body = serde_json::to_string(&event).map_err(|e| {
            Error::Internal(format!("Failed to serialize whale movement event: {}", e))
        })?;

        // Send message to SQS
        match self
            .sqs_client
            .send_message()
            .queue_url(&self.queue_url)
            .message_body(&message_body)
            // Add message attributes for filtering/routing
            .message_attributes(
                "whale_address",
                aws_sdk_sqs::types::MessageAttributeValue::builder()
                    .data_type("String")
                    .string_value(&event.whale_address)
                    .build()
                    .map_err(|e| Error::Internal(format!("Failed to build message attribute: {}", e)))?,
            )
            .message_attributes(
                "movement_type",
                aws_sdk_sqs::types::MessageAttributeValue::builder()
                    .data_type("String")
                    .string_value(&event.movement_type)
                    .build()
                    .map_err(|e| Error::Internal(format!("Failed to build message attribute: {}", e)))?,
            )
            .message_attributes(
                "token_mint",
                aws_sdk_sqs::types::MessageAttributeValue::builder()
                    .data_type("String")
                    .string_value(&event.token_mint)
                    .build()
                    .map_err(|e| Error::Internal(format!("Failed to build message attribute: {}", e)))?,
            )
            .send()
            .await
        {
            Ok(response) => {
                let message_id = response.message_id().unwrap_or("unknown");
                info!(
                    "Successfully published whale movement event {} to queue (message_id: {})",
                    event.movement_id, message_id
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to publish whale movement event {}: {}",
                    event.movement_id, e
                );
                Err(Error::Internal(format!(
                    "Failed to send message to SQS: {}",
                    e
                )))
            }
        }
    }

    /// Publish multiple whale movement events in a batch
    /// 
    /// This is more efficient than publishing events one at a time.
    /// SQS supports up to 10 messages per batch.
    /// 
    /// **Validates: Requirements 3.2**
    pub async fn publish_movements_batch(&self, events: Vec<WhaleMovementEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        info!("Publishing batch of {} whale movement events", events.len());

        // SQS batch limit is 10 messages
        const BATCH_SIZE: usize = 10;

        for chunk in events.chunks(BATCH_SIZE) {
            let mut entries = Vec::new();

            for (idx, event) in chunk.iter().enumerate() {
                // Serialize event to JSON
                let message_body = serde_json::to_string(event).map_err(|e| {
                    Error::Internal(format!("Failed to serialize whale movement event: {}", e))
                })?;

                // Create batch entry
                let entry = aws_sdk_sqs::types::SendMessageBatchRequestEntry::builder()
                    .id(format!("msg_{}", idx))
                    .message_body(message_body)
                    .message_attributes(
                        "whale_address",
                        aws_sdk_sqs::types::MessageAttributeValue::builder()
                            .data_type("String")
                            .string_value(&event.whale_address)
                            .build()
                            .map_err(|e| Error::Internal(format!("Failed to build message attribute: {}", e)))?,
                    )
                    .message_attributes(
                        "movement_type",
                        aws_sdk_sqs::types::MessageAttributeValue::builder()
                            .data_type("String")
                            .string_value(&event.movement_type)
                            .build()
                            .map_err(|e| Error::Internal(format!("Failed to build message attribute: {}", e)))?,
                    )
                    .build()
                    .map_err(|e| Error::Internal(format!("Failed to build batch entry: {}", e)))?;

                entries.push(entry);
            }

            // Send batch
            match self
                .sqs_client
                .send_message_batch()
                .queue_url(&self.queue_url)
                .set_entries(Some(entries))
                .send()
                .await
            {
                Ok(response) => {
                    let successful = response.successful().len();
                    let failed = response.failed().len();

                    if failed > 0 {
                        warn!(
                            "Batch send partially failed: {} successful, {} failed",
                            successful, failed
                        );
                        for failure in response.failed() {
                            error!(
                                "Failed to send message {}: {} - {}",
                                failure.id(),
                                failure.code(),
                                failure.message().unwrap_or("no message")
                            );
                        }
                    } else {
                        debug!("Successfully published batch of {} events", successful);
                    }
                }
                Err(e) => {
                    error!("Failed to publish batch of whale movement events: {}", e);
                    return Err(Error::Internal(format!(
                        "Failed to send batch to SQS: {}",
                        e
                    )));
                }
            }
        }

        info!("Completed batch publishing of {} events", events.len());
        Ok(())
    }

    /// Get the queue URL
    pub fn queue_url(&self) -> &str {
        &self.queue_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_movement_event_serialization() {
        let event = WhaleMovementEvent {
            movement_id: Uuid::new_v4(),
            whale_address: "11111111111111111111111111111111".to_string(),
            transaction_signature: "5j7s6NiJS3JAkvgkoc18WVAsiSaci2pxB2A6ueCJP4tprA2TFg9wSyTLeYouxPBJEMzJinENTkpA52YStRW5Dia7".to_string(),
            movement_type: "SELL".to_string(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: "1000000".to_string(),
            percent_of_position: 10.5,
            detected_at: chrono::Utc::now(),
            affected_user_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };

        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("whale_address"));
        assert!(json.contains("affected_user_ids"));

        // Test deserialization
        let deserialized: WhaleMovementEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.whale_address, event.whale_address);
        assert_eq!(deserialized.affected_user_ids.len(), 2);
    }

    #[test]
    fn test_whale_movement_event_structure() {
        let user_id_1 = Uuid::new_v4();
        let user_id_2 = Uuid::new_v4();

        let event = WhaleMovementEvent {
            movement_id: Uuid::new_v4(),
            whale_address: "test_address".to_string(),
            transaction_signature: "test_sig".to_string(),
            movement_type: "BUY".to_string(),
            token_mint: "test_mint".to_string(),
            amount: "5000000".to_string(),
            percent_of_position: 7.5,
            detected_at: chrono::Utc::now(),
            affected_user_ids: vec![user_id_1, user_id_2],
        };

        assert_eq!(event.movement_type, "BUY");
        assert_eq!(event.affected_user_ids.len(), 2);
        assert!(event.affected_user_ids.contains(&user_id_1));
        assert!(event.affected_user_ids.contains(&user_id_2));
    }
}
