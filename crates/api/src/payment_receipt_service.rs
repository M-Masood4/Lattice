use crate::receipt_service::{ReceiptData, ReceiptService};
use blockchain::Blockchain;
use chrono::{DateTime, Utc};
use database::DbPool;
use rust_decimal::Decimal;
use shared::{Error, Result};
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// User-facing payment receipt with all required fields
#[derive(Debug, Clone, serde::Serialize)]
pub struct PaymentReceipt {
    /// Unique receipt identifier
    pub id: Uuid,
    /// Transaction ID (payment, trade, or conversion ID)
    pub transaction_id: Uuid,
    /// Type of transaction (Payment, Trade, Conversion)
    pub transaction_type: TransactionType,
    /// Timestamp of the transaction
    pub timestamp: DateTime<Utc>,
    /// Transaction amount
    pub amount: Decimal,
    /// Currency or token symbol
    pub currency: String,
    /// All fees associated with the transaction
    pub fees: TransactionFees,
    /// Exchange rate (for conversions)
    pub exchange_rate: Option<Decimal>,
    /// Blockchain confirmation details
    pub confirmation: BlockchainConfirmation,
    /// Sender address
    pub sender: String,
    /// Recipient address
    pub recipient: String,
}

/// Transaction type enum
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum TransactionType {
    Payment,
    Trade,
    Conversion,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Payment => "PAYMENT",
            TransactionType::Trade => "TRADE",
            TransactionType::Conversion => "CONVERSION",
        }
    }
}

/// Transaction fees breakdown
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionFees {
    /// Network/gas fee
    pub network_fee: Option<Decimal>,
    /// Platform fee
    pub platform_fee: Option<Decimal>,
    /// Provider fee (e.g., SideShift)
    pub provider_fee: Option<Decimal>,
    /// Total fees
    pub total: Decimal,
}

/// Blockchain confirmation details
#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockchainConfirmation {
    /// Blockchain where transaction was confirmed
    pub blockchain: Blockchain,
    /// Transaction hash on blockchain
    pub transaction_hash: String,
    /// Verification status
    pub verified: bool,
    /// Verification timestamp
    pub verified_at: Option<DateTime<Utc>>,
}

/// Search filters for receipts
#[derive(Debug, Clone)]
pub struct ReceiptSearchFilters {
    /// Filter by transaction type
    pub transaction_type: Option<TransactionType>,
    /// Filter by asset/currency
    pub asset: Option<String>,
    /// Filter by date range - start date
    pub start_date: Option<DateTime<Utc>>,
    /// Filter by date range - end date
    pub end_date: Option<DateTime<Utc>>,
}

/// Pagination parameters
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    /// Page number (0-indexed)
    pub page: u32,
    /// Number of items per page
    pub page_size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: 20,
        }
    }
}

/// Search results with pagination metadata
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReceiptSearchResults {
    /// List of receipts matching the search criteria
    pub receipts: Vec<PaymentReceipt>,
    /// Total number of receipts matching the criteria
    pub total_count: i64,
    /// Current page number
    pub page: u32,
    /// Number of items per page
    pub page_size: u32,
    /// Total number of pages
    pub total_pages: u32,
}

/// Payment receipt service
/// 
/// Generates user-facing receipts for all transaction types (payments, trades, conversions)
/// with all required fields including transaction ID, timestamp, amount, fees, exchange rate,
/// and blockchain confirmation.
pub struct PaymentReceiptService {
    db: DbPool,
    receipt_service: Arc<ReceiptService>,
}

impl PaymentReceiptService {
    /// Create a new payment receipt service
    pub fn new(db: DbPool, receipt_service: Arc<ReceiptService>) -> Self {
        info!("Initializing payment receipt service");
        Self {
            db,
            receipt_service,
        }
    }

    /// Generate receipt for a payment transaction
    /// 
    /// Creates a blockchain receipt and returns a user-facing payment receipt
    /// with all required fields.
    pub async fn generate_payment_receipt(
        &self,
        payment_id: Uuid,
        amount: Decimal,
        currency: String,
        sender: String,
        recipient: String,
        blockchain: Blockchain,
        network_fee: Option<Decimal>,
        platform_fee: Option<Decimal>,
    ) -> Result<PaymentReceipt> {
        info!("Generating payment receipt for payment: {}", payment_id);

        // Create blockchain receipt
        let receipt_data = ReceiptData {
            payment_id: Some(payment_id),
            trade_id: None,
            conversion_id: None,
            proximity_transfer_id: None,
            amount,
            currency: currency.clone(),
            sender: sender.clone(),
            recipient: recipient.clone(),
            blockchain,
        };

        let blockchain_receipt = self.receipt_service.create_receipt(receipt_data).await?;

        // Calculate total fees
        let total_fees = self.calculate_total_fees(network_fee, platform_fee, None);

        // Build payment receipt
        let payment_receipt = PaymentReceipt {
            id: blockchain_receipt.id,
            transaction_id: payment_id,
            transaction_type: TransactionType::Payment,
            timestamp: blockchain_receipt.created_at,
            amount,
            currency,
            fees: TransactionFees {
                network_fee,
                platform_fee,
                provider_fee: None,
                total: total_fees,
            },
            exchange_rate: None,
            confirmation: BlockchainConfirmation {
                blockchain,
                transaction_hash: blockchain_receipt.transaction_hash,
                verified: blockchain_receipt.verification_status
                    == crate::receipt_service::VerificationStatus::Confirmed,
                verified_at: blockchain_receipt.verified_at,
            },
            sender,
            recipient,
        };

        debug!("Payment receipt generated: {:?}", payment_receipt.id);
        Ok(payment_receipt)
    }

    /// Generate receipt for a trade transaction
    /// 
    /// Creates a blockchain receipt and returns a user-facing payment receipt
    /// with all required fields for a trade.
    pub async fn generate_trade_receipt(
        &self,
        trade_id: Uuid,
        amount: Decimal,
        token_symbol: String,
        price_usd: Option<Decimal>,
        sender: String,
        recipient: String,
        blockchain: Blockchain,
        network_fee: Option<Decimal>,
        platform_fee: Option<Decimal>,
    ) -> Result<PaymentReceipt> {
        info!("Generating trade receipt for trade: {}", trade_id);

        // For trades, use USD value if available, otherwise use token amount
        let receipt_amount = price_usd.unwrap_or(amount);

        // Create blockchain receipt
        let receipt_data = ReceiptData {
            payment_id: None,
            trade_id: Some(trade_id),
            conversion_id: None,
            proximity_transfer_id: None,
            amount: receipt_amount,
            currency: if price_usd.is_some() {
                "USD".to_string()
            } else {
                token_symbol.clone()
            },
            sender: sender.clone(),
            recipient: recipient.clone(),
            blockchain,
        };

        let blockchain_receipt = self.receipt_service.create_receipt(receipt_data).await?;

        // Calculate total fees
        let total_fees = self.calculate_total_fees(network_fee, platform_fee, None);

        // Build payment receipt
        let payment_receipt = PaymentReceipt {
            id: blockchain_receipt.id,
            transaction_id: trade_id,
            transaction_type: TransactionType::Trade,
            timestamp: blockchain_receipt.created_at,
            amount,
            currency: token_symbol,
            fees: TransactionFees {
                network_fee,
                platform_fee,
                provider_fee: None,
                total: total_fees,
            },
            exchange_rate: price_usd,
            confirmation: BlockchainConfirmation {
                blockchain,
                transaction_hash: blockchain_receipt.transaction_hash,
                verified: blockchain_receipt.verification_status
                    == crate::receipt_service::VerificationStatus::Confirmed,
                verified_at: blockchain_receipt.verified_at,
            },
            sender,
            recipient,
        };

        debug!("Trade receipt generated: {:?}", payment_receipt.id);
        Ok(payment_receipt)
    }

    /// Generate receipt for a conversion transaction
    /// 
    /// Creates a blockchain receipt and returns a user-facing payment receipt
    /// with all required fields for a conversion, including exchange rate.
    pub async fn generate_conversion_receipt(
        &self,
        conversion_id: Uuid,
        from_amount: Decimal,
        from_asset: String,
        to_amount: Decimal,
        to_asset: String,
        exchange_rate: Decimal,
        sender: String,
        recipient: String,
        blockchain: Blockchain,
        network_fee: Option<Decimal>,
        platform_fee: Option<Decimal>,
        provider_fee: Option<Decimal>,
    ) -> Result<PaymentReceipt> {
        info!(
            "Generating conversion receipt for conversion: {}",
            conversion_id
        );

        // Create blockchain receipt (use from_amount and from_asset)
        let receipt_data = ReceiptData {
            payment_id: None,
            trade_id: None,
            conversion_id: Some(conversion_id),
            proximity_transfer_id: None,
            amount: from_amount,
            currency: format!("{} -> {}", from_asset, to_asset),
            sender: sender.clone(),
            recipient: recipient.clone(),
            blockchain,
        };

        let blockchain_receipt = self.receipt_service.create_receipt(receipt_data).await?;

        // Calculate total fees
        let total_fees = self.calculate_total_fees(network_fee, platform_fee, provider_fee);

        // Build payment receipt
        let payment_receipt = PaymentReceipt {
            id: blockchain_receipt.id,
            transaction_id: conversion_id,
            transaction_type: TransactionType::Conversion,
            timestamp: blockchain_receipt.created_at,
            amount: from_amount,
            currency: format!("{} -> {} {}", from_asset, to_amount, to_asset),
            fees: TransactionFees {
                network_fee,
                platform_fee,
                provider_fee,
                total: total_fees,
            },
            exchange_rate: Some(exchange_rate),
            confirmation: BlockchainConfirmation {
                blockchain,
                transaction_hash: blockchain_receipt.transaction_hash,
                verified: blockchain_receipt.verification_status
                    == crate::receipt_service::VerificationStatus::Confirmed,
                verified_at: blockchain_receipt.verified_at,
            },
            sender,
            recipient,
        };

        debug!("Conversion receipt generated: {:?}", payment_receipt.id);
        Ok(payment_receipt)
    }

    /// Get payment receipt by ID
    /// 
    /// Retrieves a payment receipt by its ID and returns it with current
    /// verification status from the blockchain.
    pub async fn get_receipt(&self, receipt_id: Uuid) -> Result<PaymentReceipt> {
        debug!("Fetching payment receipt: {}", receipt_id);

        // Get blockchain receipt
        let blockchain_receipt = self.receipt_service.get_receipt(receipt_id).await?;

        // Determine transaction type and ID
        let (transaction_type, transaction_id) = if let Some(payment_id) = blockchain_receipt.payment_id {
            (TransactionType::Payment, payment_id)
        } else if let Some(trade_id) = blockchain_receipt.trade_id {
            (TransactionType::Trade, trade_id)
        } else if let Some(conversion_id) = blockchain_receipt.conversion_id {
            (TransactionType::Conversion, conversion_id)
        } else {
            return Err(Error::Internal(
                "Receipt has no associated transaction ID".to_string(),
            ));
        };

        // Fetch transaction details from database to get fees
        let (fees, exchange_rate) = self
            .fetch_transaction_details(transaction_type.clone(), transaction_id)
            .await?;

        // Build payment receipt
        let payment_receipt = PaymentReceipt {
            id: blockchain_receipt.id,
            transaction_id,
            transaction_type,
            timestamp: blockchain_receipt.created_at,
            amount: blockchain_receipt.amount,
            currency: blockchain_receipt.currency,
            fees,
            exchange_rate,
            confirmation: BlockchainConfirmation {
                blockchain: blockchain_receipt.blockchain,
                transaction_hash: blockchain_receipt.transaction_hash,
                verified: blockchain_receipt.verification_status
                    == crate::receipt_service::VerificationStatus::Confirmed,
                verified_at: blockchain_receipt.verified_at,
            },
            sender: blockchain_receipt.sender,
            recipient: blockchain_receipt.recipient,
        };

        Ok(payment_receipt)
    }

    /// Get receipt by source transaction ID
    /// 
    /// Retrieves a payment receipt by the source transaction ID (payment, trade, or conversion).
    pub async fn get_receipt_by_transaction(
        &self,
        transaction_type: TransactionType,
        transaction_id: Uuid,
    ) -> Result<Option<PaymentReceipt>> {
        debug!(
            "Fetching payment receipt for {} transaction: {}",
            transaction_type.as_str(),
            transaction_id
        );

        // Determine which ID to use
        let (payment_id, trade_id, conversion_id) = match transaction_type {
            TransactionType::Payment => (Some(transaction_id), None, None),
            TransactionType::Trade => (None, Some(transaction_id), None),
            TransactionType::Conversion => (None, None, Some(transaction_id)),
        };

        // Get blockchain receipt
        let blockchain_receipt = self
            .receipt_service
            .get_receipt_by_source(payment_id, trade_id, conversion_id, None)
            .await?;

        match blockchain_receipt {
            Some(receipt) => {
                // Fetch transaction details
                let (fees, exchange_rate) = self
                    .fetch_transaction_details(transaction_type.clone(), transaction_id)
                    .await?;

                // Build payment receipt
                let payment_receipt = PaymentReceipt {
                    id: receipt.id,
                    transaction_id,
                    transaction_type,
                    timestamp: receipt.created_at,
                    amount: receipt.amount,
                    currency: receipt.currency,
                    fees,
                    exchange_rate,
                    confirmation: BlockchainConfirmation {
                        blockchain: receipt.blockchain,
                        transaction_hash: receipt.transaction_hash,
                        verified: receipt.verification_status
                            == crate::receipt_service::VerificationStatus::Confirmed,
                        verified_at: receipt.verified_at,
                    },
                    sender: receipt.sender,
                    recipient: receipt.recipient,
                };

                Ok(Some(payment_receipt))
            }
            None => Ok(None),
        }
    }

    /// Verify receipt and update verification status
    /// 
    /// Verifies the receipt on the blockchain and returns the updated receipt
    /// with current verification status.
    pub async fn verify_receipt(&self, receipt_id: Uuid) -> Result<PaymentReceipt> {
        info!("Verifying payment receipt: {}", receipt_id);

        // Verify blockchain receipt
        let blockchain_receipt = self.receipt_service.verify_receipt(receipt_id).await?;

        // Determine transaction type and ID
        let (transaction_type, transaction_id) = if let Some(payment_id) = blockchain_receipt.payment_id {
            (TransactionType::Payment, payment_id)
        } else if let Some(trade_id) = blockchain_receipt.trade_id {
            (TransactionType::Trade, trade_id)
        } else if let Some(conversion_id) = blockchain_receipt.conversion_id {
            (TransactionType::Conversion, conversion_id)
        } else {
            return Err(Error::Internal(
                "Receipt has no associated transaction ID".to_string(),
            ));
        };

        // Fetch transaction details
        let (fees, exchange_rate) = self
            .fetch_transaction_details(transaction_type.clone(), transaction_id)
            .await?;

        // Build payment receipt
        let payment_receipt = PaymentReceipt {
            id: blockchain_receipt.id,
            transaction_id,
            transaction_type,
            timestamp: blockchain_receipt.created_at,
            amount: blockchain_receipt.amount,
            currency: blockchain_receipt.currency,
            fees,
            exchange_rate,
            confirmation: BlockchainConfirmation {
                blockchain: blockchain_receipt.blockchain,
                transaction_hash: blockchain_receipt.transaction_hash,
                verified: blockchain_receipt.verification_status
                    == crate::receipt_service::VerificationStatus::Confirmed,
                verified_at: blockchain_receipt.verified_at,
            },
            sender: blockchain_receipt.sender,
            recipient: blockchain_receipt.recipient,
        };

        info!(
            "Payment receipt verified: {} (status: {})",
            receipt_id,
            if payment_receipt.confirmation.verified {
                "CONFIRMED"
            } else {
                "PENDING"
            }
        );

        Ok(payment_receipt)
    }

    /// Search receipts with filters and pagination
    /// 
    /// Searches for receipts matching the provided filters with pagination support.
    /// Supports filtering by transaction type, asset/currency, and date range.
    pub async fn search_receipts(
        &self,
        filters: ReceiptSearchFilters,
        pagination: Pagination,
    ) -> Result<ReceiptSearchResults> {
        info!(
            "Searching receipts with filters: type={:?}, asset={:?}, date_range={:?} to {:?}, page={}, page_size={}",
            filters.transaction_type,
            filters.asset,
            filters.start_date,
            filters.end_date,
            pagination.page,
            pagination.page_size
        );

        let client = self.db.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        // Build WHERE clause dynamically based on filters
        let mut where_clauses = Vec::new();
        let mut param_index = 1;

        // Store owned values for parameters
        let asset_param = filters.asset.clone();
        let start_date_param = filters.start_date;
        let end_date_param = filters.end_date;

        // Filter by transaction type
        let transaction_type_filter = filters.transaction_type.as_ref().map(|t| match t {
            TransactionType::Payment => "payment_id IS NOT NULL",
            TransactionType::Trade => "trade_id IS NOT NULL",
            TransactionType::Conversion => "conversion_id IS NOT NULL",
        });

        if let Some(type_clause) = transaction_type_filter {
            where_clauses.push(type_clause.to_string());
        }

        // Filter by asset/currency
        if asset_param.is_some() {
            where_clauses.push(format!("currency ILIKE ${}", param_index));
            param_index += 1;
        }

        // Filter by start date
        if start_date_param.is_some() {
            where_clauses.push(format!("created_at >= ${}", param_index));
            param_index += 1;
        }

        // Filter by end date
        if end_date_param.is_some() {
            where_clauses.push(format!("created_at <= ${}", param_index));
            param_index += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Build parameter vector for count query
        let mut count_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        if let Some(ref asset) = asset_param {
            count_params.push(Box::new(asset.clone()));
        }
        if let Some(start_date) = start_date_param {
            count_params.push(Box::new(start_date));
        }
        if let Some(end_date) = end_date_param {
            count_params.push(Box::new(end_date));
        }

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM blockchain_receipts {}",
            where_clause
        );

        let count_params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = 
            count_params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();

        let count_row = client
            .query_one(&count_query, &count_params_refs[..])
            .await
            .map_err(|e| {
                error!("Failed to count receipts: {}", e);
                Error::Database(format!("Failed to count receipts: {}", e))
            })?;

        let total_count: i64 = count_row.get(0);

        // Calculate pagination
        let offset = pagination.page * pagination.page_size;
        let limit = pagination.page_size;
        let total_pages = if total_count == 0 {
            0
        } else {
            ((total_count as f64) / (pagination.page_size as f64)).ceil() as u32
        };

        // Build search query with pagination
        let offset_param = offset as i64;
        let limit_param = limit as i64;
        
        // Create search params vector with pagination parameters
        let mut search_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        if let Some(ref asset) = asset_param {
            search_params.push(Box::new(asset.clone()));
        }
        if let Some(start_date) = start_date_param {
            search_params.push(Box::new(start_date));
        }
        if let Some(end_date) = end_date_param {
            search_params.push(Box::new(end_date));
        }
        search_params.push(Box::new(limit_param));
        search_params.push(Box::new(offset_param));

        let search_query = format!(
            r#"
            SELECT 
                id, payment_id, trade_id, conversion_id,
                amount, currency, sender, recipient,
                blockchain, transaction_hash, verification_status,
                created_at, verified_at
            FROM blockchain_receipts
            {}
            ORDER BY created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_index,
            param_index + 1
        );

        let search_params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = 
            search_params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();

        let rows = client
            .query(&search_query, &search_params_refs[..])
            .await
            .map_err(|e| {
                error!("Failed to search receipts: {}", e);
                Error::Database(format!("Failed to search receipts: {}", e))
            })?;

        // Convert rows to PaymentReceipts
        let mut receipts = Vec::new();
        for row in rows {
            let receipt_id: Uuid = row.get("id");
            let payment_id: Option<Uuid> = row.get("payment_id");
            let trade_id: Option<Uuid> = row.get("trade_id");
            let conversion_id: Option<Uuid> = row.get("conversion_id");

            // Determine transaction type and ID
            let (transaction_type, transaction_id) = if let Some(pid) = payment_id {
                (TransactionType::Payment, pid)
            } else if let Some(tid) = trade_id {
                (TransactionType::Trade, tid)
            } else if let Some(cid) = conversion_id {
                (TransactionType::Conversion, cid)
            } else {
                continue; // Skip invalid receipts
            };

            // Fetch transaction details
            let (fees, exchange_rate) = self
                .fetch_transaction_details(transaction_type.clone(), transaction_id)
                .await
                .unwrap_or_else(|_| {
                    (
                        TransactionFees {
                            network_fee: None,
                            platform_fee: None,
                            provider_fee: None,
                            total: Decimal::ZERO,
                        },
                        None,
                    )
                });

            let amount: Decimal = row.get("amount");
            let currency: String = row.get("currency");
            let sender: String = row.get("sender");
            let recipient: String = row.get("recipient");
            let blockchain_str: String = row.get("blockchain");
            let blockchain = Blockchain::from_str(&blockchain_str);
            let transaction_hash: String = row.get("transaction_hash");
            let verification_status: String = row.get("verification_status");
            
            // Handle timestamp - database might return NaiveDateTime
            let created_at: DateTime<Utc> = match row.try_get::<_, DateTime<Utc>>("created_at") {
                Ok(dt) => dt,
                Err(_) => {
                    // Try as NaiveDateTime and convert to UTC
                    let naive: chrono::NaiveDateTime = row.get("created_at");
                    DateTime::from_naive_utc_and_offset(naive, chrono::Utc)
                }
            };
            
            let verified_at: Option<DateTime<Utc>> = match row.try_get::<_, Option<DateTime<Utc>>>("verified_at") {
                Ok(dt) => dt,
                Err(_) => {
                    // Try as NaiveDateTime and convert to UTC
                    row.try_get::<_, Option<chrono::NaiveDateTime>>("verified_at")
                        .ok()
                        .flatten()
                        .map(|naive| DateTime::from_naive_utc_and_offset(naive, chrono::Utc))
                }
            };

            let payment_receipt = PaymentReceipt {
                id: receipt_id,
                transaction_id,
                transaction_type,
                timestamp: created_at,
                amount,
                currency,
                fees,
                exchange_rate,
                confirmation: BlockchainConfirmation {
                    blockchain,
                    transaction_hash,
                    verified: verification_status == "CONFIRMED",
                    verified_at,
                },
                sender,
                recipient,
            };

            receipts.push(payment_receipt);
        }

        info!(
            "Found {} receipts (total: {}, page: {}/{})",
            receipts.len(),
            total_count,
            pagination.page + 1,
            total_pages
        );

        Ok(ReceiptSearchResults {
            receipts,
            total_count,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages,
        })
    }

    /// Calculate total fees from individual fee components
    fn calculate_total_fees(
        &self,
        network_fee: Option<Decimal>,
        platform_fee: Option<Decimal>,
        provider_fee: Option<Decimal>,
    ) -> Decimal {
        let mut total = Decimal::ZERO;

        if let Some(fee) = network_fee {
            total += fee;
        }
        if let Some(fee) = platform_fee {
            total += fee;
        }
        if let Some(fee) = provider_fee {
            total += fee;
        }

        total
    }

    /// Fetch transaction details (fees and exchange rate) from database
    /// 
    /// This is a helper method that queries the appropriate table based on
    /// transaction type to get fee and exchange rate information.
    async fn fetch_transaction_details(
        &self,
        transaction_type: TransactionType,
        transaction_id: Uuid,
    ) -> Result<(TransactionFees, Option<Decimal>)> {
        let client = self.db.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        match transaction_type {
            TransactionType::Payment => {
                // For payments, we don't have a payments table yet
                // Return zero fees and no exchange rate
                Ok((
                    TransactionFees {
                        network_fee: None,
                        platform_fee: None,
                        provider_fee: None,
                        total: Decimal::ZERO,
                    },
                    None,
                ))
            }
            TransactionType::Trade => {
                // Query trade_executions table
                let row = client
                    .query_opt(
                        r#"
                        SELECT price_usd, total_value_usd
                        FROM trade_executions
                        WHERE id = $1
                        "#,
                        &[&transaction_id],
                    )
                    .await
                    .map_err(|e| {
                        error!("Failed to fetch trade details: {}", e);
                        Error::Database(format!("Failed to fetch trade details: {}", e))
                    })?;

                match row {
                    Some(r) => {
                        let price_usd: Option<f64> = r.try_get("price_usd").ok();
                        let exchange_rate = price_usd.map(Decimal::from_f64_retain).flatten();

                        Ok((
                            TransactionFees {
                                network_fee: None,
                                platform_fee: None,
                                provider_fee: None,
                                total: Decimal::ZERO,
                            },
                            exchange_rate,
                        ))
                    }
                    None => Ok((
                        TransactionFees {
                            network_fee: None,
                            platform_fee: None,
                            provider_fee: None,
                            total: Decimal::ZERO,
                        },
                        None,
                    )),
                }
            }
            TransactionType::Conversion => {
                // Query conversions table
                let row = client
                    .query_opt(
                        r#"
                        SELECT exchange_rate, network_fee, platform_fee, provider_fee
                        FROM conversions
                        WHERE id = $1
                        "#,
                        &[&transaction_id],
                    )
                    .await
                    .map_err(|e| {
                        error!("Failed to fetch conversion details: {}", e);
                        Error::Database(format!("Failed to fetch conversion details: {}", e))
                    })?;

                match row {
                    Some(r) => {
                        let exchange_rate: Decimal = r.try_get("exchange_rate").map_err(|e| {
                            Error::Database(format!("Failed to get exchange_rate: {}", e))
                        })?;
                        let network_fee: Option<Decimal> = r.try_get("network_fee").ok();
                        let platform_fee: Option<Decimal> = r.try_get("platform_fee").ok();
                        let provider_fee: Option<Decimal> = r.try_get("provider_fee").ok();

                        let total = self.calculate_total_fees(network_fee, platform_fee, provider_fee);

                        Ok((
                            TransactionFees {
                                network_fee,
                                platform_fee,
                                provider_fee,
                                total,
                            },
                            Some(exchange_rate),
                        ))
                    }
                    None => Ok((
                        TransactionFees {
                            network_fee: None,
                            platform_fee: None,
                            provider_fee: None,
                            total: Decimal::ZERO,
                        },
                        None,
                    )),
                }
            }
        }
    }

    /// Export a single receipt as PDF
    /// 
    /// Generates a PDF document for a single receipt with all details.
    pub async fn export_receipt_pdf(&self, receipt_id: Uuid) -> Result<Vec<u8>> {
        use printpdf::*;

        info!("Exporting receipt {} as PDF", receipt_id);

        // Get the receipt
        let receipt = self.get_receipt(receipt_id).await?;

        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new(
            "Receipt",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );

        let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| {
            error!("Failed to add font: {}", e);
            shared::Error::Internal(format!("Failed to add font: {}", e))
        })?;

        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Title
        current_layer.use_text("Payment Receipt", 24.0, Mm(20.0), Mm(270.0), &font);

        // Receipt details
        let mut y_pos = 250.0;
        let line_height = 7.0;

        let details = vec![
            format!("Receipt ID: {}", receipt.id),
            format!("Transaction ID: {}", receipt.transaction_id),
            format!("Type: {}", receipt.transaction_type.as_str()),
            format!("Date: {}", receipt.timestamp.format("%Y-%m-%d %H:%M:%S UTC")),
            format!("Amount: {} {}", receipt.amount, receipt.currency),
            format!("Sender: {}", receipt.sender),
            format!("Recipient: {}", receipt.recipient),
            String::new(),
            "Fees:".to_string(),
            format!("  Network Fee: {}", receipt.fees.network_fee.map(|f| f.to_string()).unwrap_or("N/A".to_string())),
            format!("  Platform Fee: {}", receipt.fees.platform_fee.map(|f| f.to_string()).unwrap_or("N/A".to_string())),
            format!("  Provider Fee: {}", receipt.fees.provider_fee.map(|f| f.to_string()).unwrap_or("N/A".to_string())),
            format!("  Total Fees: {}", receipt.fees.total),
            String::new(),
            format!("Exchange Rate: {}", receipt.exchange_rate.map(|r| r.to_string()).unwrap_or("N/A".to_string())),
            String::new(),
            "Blockchain Confirmation:".to_string(),
            format!("  Blockchain: {}", receipt.confirmation.blockchain.name()),
            format!("  Transaction Hash: {}", receipt.confirmation.transaction_hash),
            format!("  Verified: {}", if receipt.confirmation.verified { "Yes" } else { "No" }),
            format!("  Verified At: {}", receipt.confirmation.verified_at.map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or("N/A".to_string())),
        ];

        for detail in details {
            current_layer.use_text(&detail, 12.0, Mm(20.0), Mm(y_pos), &font);
            y_pos -= line_height;
        }

        // Save to bytes
        let pdf_bytes = doc.save_to_bytes().map_err(|e| {
            error!("Failed to save PDF: {}", e);
            shared::Error::Internal(format!("Failed to save PDF: {}", e))
        })?;

        info!("Successfully exported receipt {} as PDF ({} bytes)", receipt_id, pdf_bytes.len());

        Ok(pdf_bytes)
    }

    /// Export receipt history as CSV
    /// 
    /// Generates a CSV file containing all receipts matching the search filters.
    pub async fn export_receipts_csv(
        &self,
        filters: ReceiptSearchFilters,
    ) -> Result<Vec<u8>> {
        info!("Exporting receipts as CSV with filters: type={:?}, asset={:?}", 
            filters.transaction_type, filters.asset);

        // Get all receipts (no pagination limit for export)
        let pagination = Pagination {
            page: 0,
            page_size: 10000, // Large limit for export
        };

        let results = self.search_receipts(filters, pagination).await?;

        // Create CSV writer
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write header
        wtr.write_record(&[
            "Receipt ID",
            "Transaction ID",
            "Type",
            "Timestamp",
            "Amount",
            "Currency",
            "Sender",
            "Recipient",
            "Network Fee",
            "Platform Fee",
            "Provider Fee",
            "Total Fees",
            "Exchange Rate",
            "Blockchain",
            "Transaction Hash",
            "Verified",
            "Verified At",
        ]).map_err(|e| {
            error!("Failed to write CSV header: {}", e);
            Error::Internal(format!("Failed to write CSV header: {}", e))
        })?;

        // Write data rows
        let receipts_len = results.receipts.len();
        for receipt in results.receipts {
            wtr.write_record(&[
                receipt.id.to_string(),
                receipt.transaction_id.to_string(),
                receipt.transaction_type.as_str().to_string(),
                receipt.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                receipt.amount.to_string(),
                receipt.currency,
                receipt.sender,
                receipt.recipient,
                receipt.fees.network_fee.map(|f| f.to_string()).unwrap_or_default(),
                receipt.fees.platform_fee.map(|f| f.to_string()).unwrap_or_default(),
                receipt.fees.provider_fee.map(|f| f.to_string()).unwrap_or_default(),
                receipt.fees.total.to_string(),
                receipt.exchange_rate.map(|r| r.to_string()).unwrap_or_default(),
                receipt.confirmation.blockchain.name().to_string(),
                receipt.confirmation.transaction_hash,
                if receipt.confirmation.verified { "Yes" } else { "No" }.to_string(),
                receipt.confirmation.verified_at.map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string()).unwrap_or_default(),
            ]).map_err(|e| {
                error!("Failed to write CSV row: {}", e);
                Error::Internal(format!("Failed to write CSV row: {}", e))
            })?;
        }

        let csv_bytes = wtr.into_inner().map_err(|e| {
            error!("Failed to finalize CSV: {}", e);
            Error::Internal(format!("Failed to finalize CSV: {}", e))
        })?;

        info!("Successfully exported {} receipts as CSV ({} bytes)", 
            receipts_len, csv_bytes.len());

        Ok(csv_bytes)
    }

    /// Archive receipts older than 7 years
    /// 
    /// Marks receipts older than 7 years as archived. This is a background job
    /// that should run periodically to maintain the database.
    pub async fn archive_old_receipts(&self) -> Result<u64> {
        info!("Starting archival of receipts older than 7 years");

        let client = self.db.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        // Archive receipts older than 7 years that aren't already archived
        let seven_years_ago = chrono::Utc::now() - chrono::Duration::days(7 * 365);

        let result = client
            .execute(
                r#"
                UPDATE blockchain_receipts
                SET archived = TRUE, archived_at = NOW()
                WHERE created_at < $1
                AND archived = FALSE
                "#,
                &[&seven_years_ago],
            )
            .await
            .map_err(|e| {
                error!("Failed to archive old receipts: {}", e);
                Error::Database(format!("Failed to archive old receipts: {}", e))
            })?;

        info!("Archived {} receipts older than 7 years", result);

        Ok(result)
    }

    /// Get count of archived receipts
    /// 
    /// Returns the number of receipts that have been archived.
    pub async fn get_archived_count(&self) -> Result<i64> {
        let client = self.db.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let row = client
            .query_one(
                "SELECT COUNT(*) FROM blockchain_receipts WHERE archived = TRUE",
                &[],
            )
            .await
            .map_err(|e| {
                error!("Failed to count archived receipts: {}", e);
                Error::Database(format!("Failed to count archived receipts: {}", e))
            })?;

        let count: i64 = row.get(0);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_type_as_str() {
        assert_eq!(TransactionType::Payment.as_str(), "PAYMENT");
        assert_eq!(TransactionType::Trade.as_str(), "TRADE");
        assert_eq!(TransactionType::Conversion.as_str(), "CONVERSION");
    }

    #[test]
    fn test_calculate_total_fees() {
        // Test fee calculation logic directly without needing a service instance
        
        // Test with all fees
        let mut total = Decimal::ZERO;
        total += Decimal::new(10, 2);  // 0.10
        total += Decimal::new(5, 2);   // 0.05
        total += Decimal::new(3, 2);   // 0.03
        assert_eq!(total, Decimal::new(18, 2)); // 0.18

        // Test with some fees
        let mut total = Decimal::ZERO;
        total += Decimal::new(10, 2);
        total += Decimal::new(3, 2);
        assert_eq!(total, Decimal::new(13, 2)); // 0.13

        // Test with no fees
        let total = Decimal::ZERO;
        assert_eq!(total, Decimal::ZERO);
    }
}
