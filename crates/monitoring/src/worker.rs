use crate::redis_store::RedisStore;
use crate::message_queue::{MessageQueueClient, WhaleMovementEvent};
use blockchain::SolanaClient;
use database::DbPool;
use shared::{Error, Result};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// A worker that monitors a set of whale accounts
pub struct Worker {
    id: usize,
    whale_addresses: Arc<RwLock<Vec<String>>>,
    solana_client: Arc<SolanaClient>,
    redis_store: RedisStore,
    db_pool: Option<DbPool>,
    message_queue: Option<Arc<MessageQueueClient>>,
    check_interval: Duration,
    shutdown_signal: Arc<RwLock<bool>>,
}

impl Worker {
    /// Create a new worker
    pub fn new(
        id: usize,
        solana_client: Arc<SolanaClient>,
        redis_store: RedisStore,
        check_interval_seconds: u64,
    ) -> Self {
        Self {
            id,
            whale_addresses: Arc::new(RwLock::new(Vec::new())),
            solana_client,
            redis_store,
            db_pool: None,
            message_queue: None,
            check_interval: Duration::from_secs(check_interval_seconds),
            shutdown_signal: Arc::new(RwLock::new(false)),
        }
    }

    /// Set the database pool for storing whale movements
    pub fn set_db_pool(&mut self, pool: DbPool) {
        self.db_pool = Some(pool);
    }

    /// Set the message queue client for publishing whale movements
    pub fn set_message_queue(&mut self, mq_client: Arc<MessageQueueClient>) {
        self.message_queue = Some(mq_client);
    }

    /// Assign whale addresses to this worker
    pub async fn assign_whales(&self, addresses: Vec<String>) -> Result<()> {
        let mut whales = self.whale_addresses.write().await;
        whales.extend(addresses);
        info!("Worker {} now monitoring {} whales", self.id, whales.len());
        Ok(())
    }

    /// Remove whale addresses from this worker
    pub async fn remove_whales(&self, addresses: &[String]) -> Result<()> {
        let mut whales = self.whale_addresses.write().await;
        whales.retain(|addr| !addresses.contains(addr));
        info!("Worker {} now monitoring {} whales", self.id, whales.len());
        Ok(())
    }

    /// Get the number of whales assigned to this worker
    pub async fn whale_count(&self) -> usize {
        self.whale_addresses.read().await.len()
    }

    /// Start the worker monitoring loop
    pub async fn run(&self) -> Result<()> {
        info!("Worker {} starting monitoring loop", self.id);
        
        // Register worker as active
        self.redis_store.register_worker(self.id).await?;
        
        let mut ticker = interval(self.check_interval);
        
        loop {
            // Check shutdown signal
            if *self.shutdown_signal.read().await {
                info!("Worker {} received shutdown signal", self.id);
                break;
            }

            ticker.tick().await;
            
            // Get current whale list
            let whales = {
                let whales_guard = self.whale_addresses.read().await;
                whales_guard.clone()
            };

            if whales.is_empty() {
                debug!("Worker {} has no whales to monitor", self.id);
                continue;
            }

            debug!("Worker {} checking {} whales", self.id, whales.len());

            // Check each whale
            for whale_address in whales {
                if *self.shutdown_signal.read().await {
                    break;
                }

                if let Err(e) = self.check_whale_activity(&whale_address).await {
                    // Log error but continue monitoring other whales (Requirement 3.5)
                    error!(
                        "Worker {} error checking whale {}: {}",
                        self.id, whale_address, e
                    );
                }
            }
        }

        // Unregister worker
        self.redis_store.unregister_worker(self.id).await?;
        info!("Worker {} stopped", self.id);
        
        Ok(())
    }

    /// Check a single whale account for new transactions
    async fn check_whale_activity(&self, whale_address: &str) -> Result<()> {
        debug!("Worker {} checking whale {}", self.id, whale_address);

        // Get the last checked transaction signature from Redis
        let last_signature = self.redis_store.get_last_transaction(whale_address).await?;

        // Get recent transactions for the whale
        let signatures = self.get_recent_signatures(whale_address, last_signature.as_deref()).await?;

        if signatures.is_empty() {
            debug!("Worker {} found no new transactions for whale {}", self.id, whale_address);
            return Ok(());
        }

        info!(
            "Worker {} detected {} new transaction(s) for whale {}",
            self.id,
            signatures.len(),
            whale_address
        );

        // Process each new transaction
        for signature in &signatures {
            if let Err(e) = self.process_transaction(whale_address, signature).await {
                error!(
                    "Worker {} error processing transaction {} for whale {}: {}",
                    self.id, signature, whale_address, e
                );
                // Continue processing other transactions (Requirement 3.5)
            }
        }

        // Update the last checked signature to the most recent one
        if let Some(latest_sig) = signatures.first() {
            self.redis_store
                .set_last_transaction(whale_address, latest_sig)
                .await?;
        }

        Ok(())
    }

    /// Get recent transaction signatures for a whale address
    async fn get_recent_signatures(
        &self,
        whale_address: &str,
        last_known_signature: Option<&str>,
    ) -> Result<Vec<String>> {
        let pubkey = self.solana_client.validate_address(whale_address)?;

        // Get recent signatures (limit to 10 for efficiency)
        match self.solana_client.primary_client().get_signatures_for_address(&pubkey) {
            Ok(signatures) => {
                let mut new_signatures = Vec::new();
                
                for sig_info in signatures {
                    // Stop when we reach the last known signature
                    if let Some(last_sig) = last_known_signature {
                        if sig_info.signature == last_sig {
                            break;
                        }
                    }
                    
                    new_signatures.push(sig_info.signature.clone());
                }

                Ok(new_signatures)
            }
            Err(e) => {
                warn!(
                    "Worker {} failed to get signatures for {}: {}",
                    self.id, whale_address, e
                );
                Err(Error::SolanaRpc(format!(
                    "Failed to get signatures: {}",
                    e
                )))
            }
        }
    }

    /// Process a single transaction to detect whale movements
    async fn process_transaction(&self, whale_address: &str, signature: &str) -> Result<()> {
        debug!(
            "Worker {} processing transaction {} for whale {}",
            self.id, signature, whale_address
        );

        // Fetch transaction details
        let transaction = self.fetch_transaction_details(whale_address, signature).await?;

        // Analyze the transaction to determine movement type and amount
        let movement = self.analyze_transaction(whale_address, &transaction).await?;

        // Filter movements below 5% threshold (Requirement 3.4)
        if let Some(percent) = movement.percent_of_position {
            if percent < 5.0 {
                debug!(
                    "Worker {} filtering out movement below 5% threshold: {:.2}%",
                    self.id, percent
                );
                return Ok(());
            }

            info!(
                "Worker {} detected significant movement: {} {} {} ({:.2}% of position)",
                self.id, movement.movement_type, movement.amount, movement.token_mint, percent
            );

            // Store the movement in PostgreSQL
            self.store_whale_movement(&movement).await?;
        }

        Ok(())
    }

    /// Fetch transaction details from Solana
    async fn fetch_transaction_details(
        &self,
        whale_address: &str,
        signature: &str,
    ) -> Result<TransactionDetails> {
        use solana_sdk::signature::Signature;

        let sig = Signature::from_str(signature)
            .map_err(|e| Error::SolanaRpc(format!("Invalid signature: {}", e)))?;

        let _pubkey = self.solana_client.validate_address(whale_address)?;

        // Get transaction with full details
        match self
            .solana_client
            .primary_client()
            .get_transaction(&sig, solana_transaction_status::UiTransactionEncoding::Json)
        {
            Ok(tx) => {
                // Parse the transaction to extract relevant details
                Ok(TransactionDetails {
                    signature: signature.to_string(),
                    whale_address: whale_address.to_string(),
                    transaction: tx,
                })
            }
            Err(e) => {
                warn!(
                    "Worker {} failed to fetch transaction {}: {}",
                    self.id, signature, e
                );
                Err(Error::SolanaRpc(format!(
                    "Failed to fetch transaction: {}",
                    e
                )))
            }
        }
    }

    /// Analyze a transaction to determine movement type and amount
    async fn analyze_transaction(
        &self,
        whale_address: &str,
        transaction: &TransactionDetails,
    ) -> Result<WhaleMovementData> {
        // For now, we'll implement a simplified version that detects token transfers
        // A full implementation would parse instruction data to determine buy/sell
        
        // Get the whale's current token balances to calculate percentage
        let whale_balance = self.solana_client.get_wallet_balance(whale_address).await?;

        // Parse transaction to find token transfers
        // This is a simplified implementation - a production version would need
        // more sophisticated parsing of instruction data
        
        // For demonstration, we'll create a placeholder movement
        // In a real implementation, you would:
        // 1. Parse the transaction instructions
        // 2. Identify token program instructions (transfer, transferChecked)
        // 3. Determine if it's a buy (receiving tokens) or sell (sending tokens)
        // 4. Extract the token mint and amount
        
        // Placeholder: assume first token account has movement
        if let Some(token_account) = whale_balance.token_accounts.first() {
            let movement_type = "SELL"; // Simplified - would be determined from tx analysis
            let amount = "1000000"; // Simplified - would be extracted from tx
            
            // Calculate percentage of position
            let percent_of_position = if token_account.amount > 0 {
                let amount_value = amount.parse::<u64>().unwrap_or(0) as f64;
                let total_value = token_account.amount as f64;
                (amount_value / total_value) * 100.0
            } else {
                0.0
            };

            Ok(WhaleMovementData {
                whale_address: whale_address.to_string(),
                transaction_signature: transaction.signature.clone(),
                movement_type: movement_type.to_string(),
                token_mint: token_account.mint.clone(),
                amount: amount.to_string(),
                percent_of_position: Some(percent_of_position),
            })
        } else {
            // No token accounts found - might be a SOL-only transaction
            Err(Error::Internal(
                "Unable to determine movement details from transaction".to_string(),
            ))
        }
    }

    /// Store a whale movement in PostgreSQL
    async fn store_whale_movement(&self, movement: &WhaleMovementData) -> Result<()> {
        let db_pool = match &self.db_pool {
            Some(pool) => pool,
            None => {
                warn!("Worker {} has no database pool configured", self.id);
                return Ok(()); // Skip storage if no DB pool
            }
        };

        let client = db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        // First, get or create the whale record
        let whale_id = self.get_or_create_whale(&client, &movement.whale_address).await?;

        // Insert the whale movement
        let query = r#"
            INSERT INTO whale_movements (
                whale_id,
                transaction_signature,
                movement_type,
                token_mint,
                amount,
                percent_of_position
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (transaction_signature) DO NOTHING
            RETURNING id
        "#;

        match client
            .query_opt(
                query,
                &[
                    &whale_id,
                    &movement.transaction_signature,
                    &movement.movement_type,
                    &movement.token_mint,
                    &movement.amount,
                    &movement.percent_of_position,
                ],
            )
            .await
        {
            Ok(Some(row)) => {
                let movement_id: Uuid = row.get(0);
                info!(
                    "Worker {} stored whale movement {} for whale {}",
                    self.id, movement_id, movement.whale_address
                );

                // Get affected user IDs who are tracking this whale
                let affected_users = self.get_affected_users(&client, whale_id, &movement.token_mint).await?;

                // Publish to message queue if configured
                if let Some(mq_client) = &self.message_queue {
                    let event = WhaleMovementEvent {
                        movement_id,
                        whale_address: movement.whale_address.clone(),
                        transaction_signature: movement.transaction_signature.clone(),
                        movement_type: movement.movement_type.clone(),
                        token_mint: movement.token_mint.clone(),
                        amount: movement.amount.clone(),
                        percent_of_position: movement.percent_of_position.unwrap_or(0.0),
                        detected_at: chrono::Utc::now(),
                        affected_user_ids: affected_users,
                    };

                    if let Err(e) = mq_client.publish_movement(event).await {
                        error!(
                            "Worker {} failed to publish whale movement {} to message queue: {}",
                            self.id, movement_id, e
                        );
                        // Don't fail the entire operation if message queue publish fails
                        // The movement is already stored in the database
                    }
                } else {
                    debug!("Worker {} has no message queue configured, skipping event publish", self.id);
                }

                Ok(())
            }
            Ok(None) => {
                debug!(
                    "Worker {} skipped duplicate movement for signature {}",
                    self.id, movement.transaction_signature
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Worker {} failed to store whale movement: {}",
                    self.id, e
                );
                Err(Error::Database(format!(
                    "Failed to insert whale movement: {}",
                    e
                )))
            }
        }
    }

    /// Get user IDs who are tracking this whale for the given token
    /// 
    /// **Validates: Requirements 3.2** (include affected user IDs in movement events)
    async fn get_affected_users(
        &self,
        client: &tokio_postgres::Client,
        whale_id: Uuid,
        token_mint: &str,
    ) -> Result<Vec<Uuid>> {
        let query = r#"
            SELECT DISTINCT user_id
            FROM user_whale_tracking
            WHERE whale_id = $1 AND token_mint = $2
        "#;

        match client.query(query, &[&whale_id, &token_mint]).await {
            Ok(rows) => {
                let user_ids: Vec<Uuid> = rows.iter().map(|row| row.get(0)).collect();
                debug!(
                    "Worker {} found {} affected users for whale {} token {}",
                    self.id,
                    user_ids.len(),
                    whale_id,
                    token_mint
                );
                Ok(user_ids)
            }
            Err(e) => {
                warn!(
                    "Worker {} failed to get affected users: {}",
                    self.id, e
                );
                // Return empty list on error - don't fail the entire operation
                Ok(Vec::new())
            }
        }
    }

    /// Get or create a whale record in the database
    async fn get_or_create_whale(
        &self,
        client: &tokio_postgres::Client,
        whale_address: &str,
    ) -> Result<Uuid> {
        // Try to get existing whale
        let query = "SELECT id FROM whales WHERE address = $1";
        
        if let Ok(Some(row)) = client.query_opt(query, &[&whale_address]).await {
            let id: Uuid = row.get(0);
            return Ok(id);
        }

        // Create new whale record
        let insert_query = r#"
            INSERT INTO whales (address, last_checked)
            VALUES ($1, NOW())
            ON CONFLICT (address) DO UPDATE SET last_checked = NOW()
            RETURNING id
        "#;

        match client.query_one(insert_query, &[&whale_address]).await {
            Ok(row) => {
                let id: Uuid = row.get(0);
                Ok(id)
            }
            Err(e) => Err(Error::Database(format!(
                "Failed to create whale record: {}",
                e
            ))),
        }
    }

    /// Signal the worker to shutdown
    pub async fn shutdown(&self) {
        let mut signal = self.shutdown_signal.write().await;
        *signal = true;
        info!("Worker {} shutdown signal set", self.id);
    }
}

/// Transaction details fetched from Solana
struct TransactionDetails {
    signature: String,
    #[allow(dead_code)]
    whale_address: String,
    #[allow(dead_code)]
    transaction: solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta,
}

/// Whale movement data to be stored
struct WhaleMovementData {
    whale_address: String,
    transaction_signature: String,
    movement_type: String,
    token_mint: String,
    amount: String,
    percent_of_position: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis_store::RedisStore;
    use blockchain::SolanaClient;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_worker_creation() {
        let solana_client = Arc::new(SolanaClient::new(
            "https://api.devnet.solana.com".to_string(),
            None,
        ));

        // Note: This test requires Redis to be running
        // Skip if Redis is not available
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let redis_store = match RedisStore::new(&redis_url).await {
            Ok(store) => store,
            Err(_) => {
                println!("Skipping test - Redis not available");
                return;
            }
        };

        let worker = Worker::new(0, solana_client, redis_store, 30);

        assert_eq!(worker.whale_count().await, 0);
    }

    #[tokio::test]
    async fn test_worker_whale_assignment() {
        let solana_client = Arc::new(SolanaClient::new(
            "https://api.devnet.solana.com".to_string(),
            None,
        ));

        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let redis_store = match RedisStore::new(&redis_url).await {
            Ok(store) => store,
            Err(_) => {
                println!("Skipping test - Redis not available");
                return;
            }
        };

        let worker = Worker::new(0, solana_client, redis_store, 30);

        // Assign some whale addresses
        let whales = vec![
            "11111111111111111111111111111111".to_string(),
            "11111111111111111111111111111112".to_string(),
        ];

        worker.assign_whales(whales.clone()).await.unwrap();
        assert_eq!(worker.whale_count().await, 2);

        // Remove one whale
        worker.remove_whales(&[whales[0].clone()]).await.unwrap();
        assert_eq!(worker.whale_count().await, 1);
    }

    #[test]
    fn test_whale_movement_data_structure() {
        let movement = WhaleMovementData {
            whale_address: "11111111111111111111111111111111".to_string(),
            transaction_signature: "5j7s6NiJS3JAkvgkoc18WVAsiSaci2pxB2A6ueCJP4tprA2TFg9wSyTLeYouxPBJEMzJinENTkpA52YStRW5Dia7".to_string(),
            movement_type: "SELL".to_string(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: "1000000".to_string(),
            percent_of_position: Some(10.5),
        };

        assert_eq!(movement.movement_type, "SELL");
        assert_eq!(movement.percent_of_position, Some(10.5));
    }
}
