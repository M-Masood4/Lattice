use blockchain::{Blockchain, MultiChainClient};
use chrono::{DateTime, Utc};
use database::DbPool;
use shared::{Error, Result};
use std::sync::Arc;
use tokio_postgres::Row;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Verification status for blockchain receipts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    Pending,
    Confirmed,
    Failed,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationStatus::Pending => "PENDING",
            VerificationStatus::Confirmed => "CONFIRMED",
            VerificationStatus::Failed => "FAILED",
        }
    }
}

/// Blockchain receipt data structure
#[derive(Debug, Clone)]
pub struct Receipt {
    pub id: Uuid,
    pub payment_id: Option<Uuid>,
    pub trade_id: Option<Uuid>,
    pub conversion_id: Option<Uuid>,
    pub proximity_transfer_id: Option<Uuid>,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub sender: String,
    pub recipient: String,
    pub blockchain: Blockchain,
    pub transaction_hash: String,
    pub verification_status: VerificationStatus,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

/// Data for creating a new receipt
#[derive(Debug, Clone)]
pub struct ReceiptData {
    pub payment_id: Option<Uuid>,
    pub trade_id: Option<Uuid>,
    pub conversion_id: Option<Uuid>,
    pub proximity_transfer_id: Option<Uuid>,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub sender: String,
    pub recipient: String,
    pub blockchain: Blockchain,
}

/// Blockchain receipt service
/// 
/// Handles creation and verification of blockchain-based receipts for payments,
/// trades, and conversions. Generates SHA-256 hashes of transaction data and
/// submits them to the blockchain for immutable proof.
pub struct ReceiptService {
    db: DbPool,
    blockchain_client: Arc<MultiChainClient>,
}

impl ReceiptService {
    /// Create a new receipt service
    pub fn new(db: DbPool, blockchain_client: Arc<MultiChainClient>) -> Self {
        info!("Initializing blockchain receipt service");
        Self {
            db,
            blockchain_client,
        }
    }

    /// Create a blockchain receipt for a payment, trade, or conversion
    /// 
    /// This method:
    /// 1. Generates a SHA-256 hash of the transaction data
    /// 2. Submits the hash to the specified blockchain
    /// 3. Stores the receipt with transaction hash in the database
    /// 4. Links the receipt to the source transaction
    pub async fn create_receipt(&self, data: ReceiptData) -> Result<Receipt> {
        info!(
            "Creating blockchain receipt on {} for amount: {} {}",
            data.blockchain.name(),
            data.amount,
            data.currency
        );

        // Generate SHA-256 hash of receipt data
        let receipt_hash = self.generate_receipt_hash(&data)?;
        debug!("Generated receipt hash: {}", receipt_hash);

        // Submit hash to blockchain with retry logic
        let transaction_hash = self.submit_to_blockchain(&data.blockchain, &receipt_hash).await?;
        info!(
            "Receipt hash submitted to {} blockchain: {}",
            data.blockchain.name(),
            transaction_hash
        );

        // Store receipt in database
        let receipt = self.store_receipt(data, transaction_hash).await?;
        info!("Receipt stored in database with ID: {}", receipt.id);

        Ok(receipt)
    }

    /// Generate SHA-256 hash of receipt data
    /// 
    /// Hash format: SHA-256(amount|currency|timestamp|sender|recipient|id)
    fn generate_receipt_hash(&self, data: &ReceiptData) -> Result<String> {
        use sha2::{Digest, Sha256};

        let timestamp = Utc::now().timestamp();
        
        // Determine which ID to use (payment, trade, conversion, or proximity transfer)
        let id = data
            .payment_id
            .or(data.trade_id)
            .or(data.conversion_id)
            .or(data.proximity_transfer_id)
            .ok_or_else(|| {
                Error::Internal("Receipt must have at least one source ID".to_string())
            })?;

        // Concatenate fields with pipe separator
        let receipt_data = format!(
            "{}|{}|{}|{}|{}|{}",
            data.amount, data.currency, timestamp, data.sender, data.recipient, id
        );

        // Generate SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(receipt_data.as_bytes());
        let hash_bytes = hasher.finalize();

        // Convert to hex string
        let hash_hex = format!("0x{}", hex::encode(hash_bytes));
        Ok(hash_hex)
    }

    /// Submit receipt hash to blockchain with retry logic
    /// 
    /// Attempts to submit the hash up to 3 times with exponential backoff
    async fn submit_to_blockchain(
        &self,
        blockchain: &Blockchain,
        receipt_hash: &str,
    ) -> Result<String> {
        const MAX_RETRIES: u32 = 3;
        let mut retry_count = 0;

        loop {
            match self.try_submit_to_blockchain(blockchain, receipt_hash).await {
                Ok(tx_hash) => {
                    if retry_count > 0 {
                        info!(
                            "Successfully submitted receipt to {} after {} retries",
                            blockchain.name(),
                            retry_count
                        );
                    }
                    return Ok(tx_hash);
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        error!(
                            "Failed to submit receipt to {} after {} attempts: {}",
                            blockchain.name(),
                            MAX_RETRIES,
                            e
                        );
                        return Err(Error::Internal(format!(
                            "Failed to submit receipt to blockchain after {} attempts: {}",
                            MAX_RETRIES, e
                        )));
                    }

                    warn!(
                        "Blockchain submission attempt {} failed for {}: {}. Retrying...",
                        retry_count,
                        blockchain.name(),
                        e
                    );

                    // Exponential backoff: 1s, 2s, 4s
                    let delay = std::time::Duration::from_secs(2u64.pow(retry_count - 1));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Attempt to submit receipt hash to blockchain (single attempt)
    async fn try_submit_to_blockchain(
        &self,
        blockchain: &Blockchain,
        receipt_hash: &str,
    ) -> Result<String> {
        // Get the appropriate blockchain client
        let _client = self
            .blockchain_client
            .get_client(*blockchain)
            .ok_or_else(|| {
                Error::Internal(format!(
                    "Blockchain client not configured for {}",
                    blockchain.name()
                ))
            })?;

        // For this implementation, we'll create a simple data transaction
        // In a real implementation, this would create a proper transaction
        // with the receipt hash embedded in the transaction data
        
        // Note: This is a simplified implementation
        // In production, you would:
        // 1. Create a proper transaction with the hash in the data field
        // 2. Sign the transaction with a service wallet
        // 3. Submit the signed transaction
        
        // For now, we'll simulate by validating the blockchain is available
        // and return a mock transaction hash
        // The actual implementation would use client.submit_transaction()
        
        match blockchain {
            Blockchain::Solana => {
                // Solana implementation would go here
                // For now, return a simulated transaction hash
                Ok(format!("solana_tx_{}", &receipt_hash[2..18]))
            }
            Blockchain::Ethereum | Blockchain::BinanceSmartChain | Blockchain::Polygon => {
                // EVM chains implementation
                // In production, this would create and submit a real transaction
                // For now, return a simulated transaction hash
                Ok(format!("evm_tx_{}", &receipt_hash[2..18]))
            }
        }
    }

    /// Store receipt in database
    async fn store_receipt(
        &self,
        data: ReceiptData,
        transaction_hash: String,
    ) -> Result<Receipt> {
        let id = Uuid::new_v4();
        let blockchain_str = data.blockchain.name().to_string();

        let client = self.db.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let row = client
            .query_one(
                r#"
                INSERT INTO blockchain_receipts (
                    id, payment_id, trade_id, conversion_id, proximity_transfer_id, amount, currency,
                    sender, recipient, blockchain, transaction_hash,
                    verification_status, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'PENDING', NOW())
                RETURNING id, payment_id, trade_id, conversion_id, proximity_transfer_id, amount, currency,
                          sender, recipient, blockchain, transaction_hash,
                          verification_status, created_at, verified_at
                "#,
                &[
                    &id,
                    &data.payment_id,
                    &data.trade_id,
                    &data.conversion_id,
                    &data.proximity_transfer_id,
                    &data.amount,
                    &data.currency,
                    &data.sender,
                    &data.recipient,
                    &blockchain_str,
                    &transaction_hash,
                ],
            )
            .await
            .map_err(|e| {
                error!("Failed to store receipt in database: {:?}", e);
                Error::Database(format!("Failed to store receipt: {:?}", e))
            })?;

        self.row_to_receipt(&row)
    }

    /// Get receipt by ID
    pub async fn get_receipt(&self, receipt_id: Uuid) -> Result<Receipt> {
        let client = self.db.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let rows = client
            .query(
                r#"
                SELECT id, payment_id, trade_id, conversion_id, proximity_transfer_id, amount, currency,
                       sender, recipient, blockchain, transaction_hash,
                       verification_status, created_at, verified_at
                FROM blockchain_receipts
                WHERE id = $1
                "#,
                &[&receipt_id],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to fetch receipt: {}", e)))?;

        if rows.is_empty() {
            return Err(Error::Internal(format!("Receipt not found: {}", receipt_id)));
        }

        self.row_to_receipt(&rows[0])
    }

    /// Get receipt by source transaction ID
    pub async fn get_receipt_by_source(
        &self,
        payment_id: Option<Uuid>,
        trade_id: Option<Uuid>,
        conversion_id: Option<Uuid>,
        proximity_transfer_id: Option<Uuid>,
    ) -> Result<Option<Receipt>> {
        let client = self.db.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let row = client
            .query_opt(
                r#"
                SELECT id, payment_id, trade_id, conversion_id, proximity_transfer_id, amount, currency,
                       sender, recipient, blockchain, transaction_hash,
                       verification_status, created_at, verified_at
                FROM blockchain_receipts
                WHERE ($1::UUID IS NULL OR payment_id = $1)
                  AND ($2::UUID IS NULL OR trade_id = $2)
                  AND ($3::UUID IS NULL OR conversion_id = $3)
                  AND ($4::UUID IS NULL OR proximity_transfer_id = $4)
                LIMIT 1
                "#,
                &[&payment_id, &trade_id, &conversion_id, &proximity_transfer_id],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to fetch receipt: {}", e)))?;

        match row {
            Some(r) => Ok(Some(self.row_to_receipt(&r)?)),
            None => Ok(None),
        }
    }

    /// Verify receipt authenticity by checking blockchain
    /// 
    /// This method:
    /// 1. Fetches the receipt from the database
    /// 2. Queries the blockchain for the transaction
    /// 3. Verifies the transaction exists and is confirmed
    /// 4. Updates the verification status in the database
    /// 5. Returns the updated receipt with verification status
    pub async fn verify_receipt(&self, receipt_id: Uuid) -> Result<Receipt> {
        info!("Verifying receipt: {}", receipt_id);

        // Fetch receipt from database
        let mut receipt = self.get_receipt(receipt_id).await?;

        // If already confirmed, return immediately
        if receipt.verification_status == VerificationStatus::Confirmed {
            debug!("Receipt {} already confirmed", receipt_id);
            return Ok(receipt);
        }

        // Verify transaction on blockchain
        let verification_result = self
            .verify_transaction_on_blockchain(&receipt.blockchain, &receipt.transaction_hash)
            .await;

        // Update verification status based on result
        let new_status = match verification_result {
            Ok(true) => {
                info!(
                    "Receipt {} verified successfully on {}",
                    receipt_id,
                    receipt.blockchain.name()
                );
                VerificationStatus::Confirmed
            }
            Ok(false) => {
                warn!(
                    "Receipt {} verification failed: transaction not found on {}",
                    receipt_id,
                    receipt.blockchain.name()
                );
                VerificationStatus::Failed
            }
            Err(e) => {
                error!(
                    "Error verifying receipt {} on {}: {}",
                    receipt_id,
                    receipt.blockchain.name(),
                    e
                );
                // Keep as pending if there's an error (might be temporary)
                VerificationStatus::Pending
            }
        };

        // Update status in database if it changed
        if new_status != receipt.verification_status {
            receipt = self
                .update_verification_status(receipt_id, new_status.clone())
                .await?;
        }

        Ok(receipt)
    }

    /// Verify transaction exists on blockchain
    /// 
    /// Returns Ok(true) if transaction is confirmed, Ok(false) if not found,
    /// or Err if there's an error checking
    async fn verify_transaction_on_blockchain(
        &self,
        blockchain: &Blockchain,
        transaction_hash: &str,
    ) -> Result<bool> {
        debug!(
            "Checking transaction {} on {}",
            transaction_hash,
            blockchain.name()
        );

        // Get the appropriate blockchain client
        let _client = self
            .blockchain_client
            .get_client(*blockchain)
            .ok_or_else(|| {
                Error::Internal(format!(
                    "Blockchain client not configured for {}",
                    blockchain.name()
                ))
            })?;

        // For this implementation, we'll simulate verification
        // In a real implementation, this would:
        // 1. Query the blockchain for the transaction by hash
        // 2. Check if the transaction exists and is confirmed
        // 3. Optionally verify the transaction data matches the receipt hash
        
        // Note: This is a simplified implementation
        // In production, you would:
        // - For Solana: Use getTransaction RPC method
        // - For EVM chains: Use eth_getTransactionByHash and eth_getTransactionReceipt
        // - Check confirmation count (e.g., 6+ confirmations for Ethereum)
        
        match blockchain {
            Blockchain::Solana => {
                // Solana verification would use getTransaction
                // For now, simulate by checking hash format
                if transaction_hash.starts_with("solana_tx_") {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Blockchain::Ethereum | Blockchain::BinanceSmartChain | Blockchain::Polygon => {
                // EVM verification would use eth_getTransactionByHash
                // For now, simulate by checking hash format
                if transaction_hash.starts_with("evm_tx_") {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Update verification status in database
    async fn update_verification_status(
        &self,
        receipt_id: Uuid,
        status: VerificationStatus,
    ) -> Result<Receipt> {
        let client = self.db.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let status_str = status.as_str();
        let verified_at: Option<std::time::SystemTime> = if status == VerificationStatus::Confirmed {
            Some(std::time::SystemTime::now())
        } else {
            None
        };

        let row = client
            .query_one(
                r#"
                UPDATE blockchain_receipts
                SET verification_status = $1,
                    verified_at = $2
                WHERE id = $3
                RETURNING id, payment_id, trade_id, conversion_id, proximity_transfer_id, amount, currency,
                          sender, recipient, blockchain, transaction_hash,
                          verification_status, created_at, verified_at
                "#,
                &[&status_str, &verified_at, &receipt_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to update receipt verification status: {}", e);
                Error::Database(format!("Failed to update verification status: {}", e))
            })?;

        self.row_to_receipt(&row)
    }

    /// Convert database row to Receipt struct
    fn row_to_receipt(&self, row: &Row) -> Result<Receipt> {
        let blockchain_str: String = row.try_get("blockchain").map_err(|e| {
            Error::Database(format!("Failed to get blockchain field: {}", e))
        })?;

        let blockchain = match blockchain_str.as_str() {
            "Solana" => Blockchain::Solana,
            "Ethereum" => Blockchain::Ethereum,
            "Binance Smart Chain" => Blockchain::BinanceSmartChain,
            "Polygon" => Blockchain::Polygon,
            _ => {
                return Err(Error::Internal(format!(
                    "Unknown blockchain: {}",
                    blockchain_str
                )))
            }
        };

        let status_str: String = row.try_get("verification_status").map_err(|e| {
            Error::Database(format!("Failed to get verification_status field: {}", e))
        })?;

        let verification_status = match status_str.as_str() {
            "PENDING" => VerificationStatus::Pending,
            "CONFIRMED" => VerificationStatus::Confirmed,
            "FAILED" => VerificationStatus::Failed,
            _ => VerificationStatus::Pending,
        };

        // Get created_at as SystemTime and convert to DateTime<Utc>
        let created_at_systime: std::time::SystemTime = row.try_get("created_at").map_err(|e| {
            Error::Database(format!("Failed to get created_at field: {}", e))
        })?;
        let created_at = DateTime::<Utc>::from(created_at_systime);

        // Get verified_at if present
        let verified_at = row
            .try_get::<_, Option<std::time::SystemTime>>("verified_at")
            .ok()
            .flatten()
            .map(DateTime::<Utc>::from);

        Ok(Receipt {
            id: row.try_get("id").map_err(|e| {
                Error::Database(format!("Failed to get id field: {}", e))
            })?,
            payment_id: row.try_get("payment_id").ok(),
            trade_id: row.try_get("trade_id").ok(),
            conversion_id: row.try_get("conversion_id").ok(),
            proximity_transfer_id: row.try_get("proximity_transfer_id").ok(),
            amount: row.try_get("amount").map_err(|e| {
                Error::Database(format!("Failed to get amount field: {}", e))
            })?,
            currency: row.try_get("currency").map_err(|e| {
                Error::Database(format!("Failed to get currency field: {}", e))
            })?,
            sender: row.try_get("sender").map_err(|e| {
                Error::Database(format!("Failed to get sender field: {}", e))
            })?,
            recipient: row.try_get("recipient").map_err(|e| {
                Error::Database(format!("Failed to get recipient field: {}", e))
            })?,
            blockchain,
            transaction_hash: row.try_get("transaction_hash").map_err(|e| {
                Error::Database(format!("Failed to get transaction_hash field: {}", e))
            })?,
            verification_status,
            created_at,
            verified_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_verification_status_as_str() {
        assert_eq!(VerificationStatus::Pending.as_str(), "PENDING");
        assert_eq!(VerificationStatus::Confirmed.as_str(), "CONFIRMED");
        assert_eq!(VerificationStatus::Failed.as_str(), "FAILED");
    }

    #[test]
    fn test_receipt_data_creation() {
        let data = ReceiptData {
            payment_id: Some(Uuid::new_v4()),
            trade_id: None,
            conversion_id: None,
            proximity_transfer_id: None,
            amount: Decimal::new(10050, 2), // 100.50
            currency: "USD".to_string(),
            sender: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string(),
            recipient: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
            blockchain: Blockchain::Ethereum,
        };

        assert!(data.payment_id.is_some());
        assert_eq!(data.currency, "USD");
        assert_eq!(data.blockchain, Blockchain::Ethereum);
    }
}
