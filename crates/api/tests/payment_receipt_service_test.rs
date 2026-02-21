use api::{PaymentReceiptService, ReceiptService, TransactionType};
use blockchain::{Blockchain, MultiChainClient};
use database::{create_pool, run_migrations};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

/// Helper to create a test database pool
async fn setup_test_db() -> Result<database::DbPool, String> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/whale_tracker_test".to_string());
    
    let pool = match create_pool(&database_url, 5).await {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to create pool: {}", e)),
    };
    
    // Test the connection
    match pool.get().await {
        Ok(_) => {},
        Err(e) => return Err(format!("Failed to get database connection: {}", e)),
    }
    
    // Try to run migrations, but ignore if tables already exist
    let _ = run_migrations(&pool).await;
    
    Ok(pool)
}

/// Helper to create a test payment receipt service
async fn create_test_service() -> Result<(PaymentReceiptService, database::DbPool), String> {
    let db_pool = setup_test_db().await?;

    // Configure MultiChainClient with test RPC endpoints
    let blockchain_client = Arc::new(
        MultiChainClient::new()
            .with_solana("https://api.devnet.solana.com".to_string(), None)
            .with_ethereum("https://eth-sepolia.public.blastapi.io".to_string(), None)
            .with_bsc("https://bsc-testnet.public.blastapi.io".to_string(), None)
            .with_polygon("https://polygon-mumbai.public.blastapi.io".to_string(), None)
    );
    
    let receipt_service = Arc::new(ReceiptService::new(db_pool.clone(), blockchain_client));
    let payment_receipt_service = PaymentReceiptService::new(db_pool.clone(), receipt_service);

    Ok((payment_receipt_service, db_pool))
}

/// Helper to create a test user and return the user_id
async fn create_test_user(db: &database::DbPool) -> Uuid {
    let user_id = Uuid::new_v4();
    let client = db.get().await.expect("Failed to get db connection");
    
    client
        .execute(
            r#"
            INSERT INTO users (id, email, password_hash, created_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (id) DO NOTHING
            "#,
            &[
                &user_id,
                &format!("test_{}@example.com", user_id),
                &"dummy_hash",
            ],
        )
        .await
        .expect("Failed to insert test user");
    
    user_id
}

#[tokio::test]
async fn test_generate_payment_receipt() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let payment_id = Uuid::new_v4();
    let amount = Decimal::new(10050, 2); // 100.50
    let currency = "USD".to_string();
    let sender = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string();
    let recipient = "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string();
    let blockchain = Blockchain::Ethereum;
    let network_fee = Some(Decimal::new(21, 2)); // 0.21
    let platform_fee = Some(Decimal::new(50, 2)); // 0.50

    let result = service
        .generate_payment_receipt(
            payment_id,
            amount,
            currency.clone(),
            sender.clone(),
            recipient.clone(),
            blockchain,
            network_fee,
            platform_fee,
        )
        .await;

    assert!(result.is_ok(), "Failed to generate payment receipt: {:?}", result.err());
    let receipt = result.unwrap();

    // Verify all required fields are present
    assert_eq!(receipt.transaction_id, payment_id);
    assert_eq!(receipt.transaction_type, TransactionType::Payment);
    assert_eq!(receipt.amount, amount);
    assert_eq!(receipt.currency, currency);
    assert_eq!(receipt.sender, sender);
    assert_eq!(receipt.recipient, recipient);

    // Verify fees
    assert_eq!(receipt.fees.network_fee, network_fee);
    assert_eq!(receipt.fees.platform_fee, platform_fee);
    assert_eq!(receipt.fees.provider_fee, None);
    assert_eq!(receipt.fees.total, Decimal::new(71, 2)); // 0.21 + 0.50 = 0.71

    // Verify exchange rate is None for payments
    assert_eq!(receipt.exchange_rate, None);

    // Verify blockchain confirmation
    assert_eq!(receipt.confirmation.blockchain, blockchain);
    assert!(!receipt.confirmation.transaction_hash.is_empty());
}

#[tokio::test]
async fn test_generate_trade_receipt() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let trade_id = Uuid::new_v4();
    let amount = Decimal::new(5, 0); // 5 tokens
    let token_symbol = "SOL".to_string();
    let price_usd = Some(Decimal::new(10050, 2)); // 100.50 USD per token
    let sender = "wallet1".to_string();
    let recipient = "wallet2".to_string();
    let blockchain = Blockchain::Solana;
    let network_fee = Some(Decimal::new(5000, 6)); // 0.005 SOL
    let platform_fee = Some(Decimal::new(1, 2)); // 0.01 SOL

    let result = service
        .generate_trade_receipt(
            trade_id,
            amount,
            token_symbol.clone(),
            price_usd,
            sender.clone(),
            recipient.clone(),
            blockchain,
            network_fee,
            platform_fee,
        )
        .await;

    assert!(result.is_ok(), "Failed to generate trade receipt");
    let receipt = result.unwrap();

    // Verify all required fields
    assert_eq!(receipt.transaction_id, trade_id);
    assert_eq!(receipt.transaction_type, TransactionType::Trade);
    assert_eq!(receipt.amount, amount);
    assert_eq!(receipt.currency, token_symbol);
    assert_eq!(receipt.sender, sender);
    assert_eq!(receipt.recipient, recipient);

    // Verify exchange rate (price) is present
    assert_eq!(receipt.exchange_rate, price_usd);

    // Verify fees
    assert_eq!(receipt.fees.network_fee, network_fee);
    assert_eq!(receipt.fees.platform_fee, platform_fee);
    assert_eq!(receipt.fees.provider_fee, None);

    // Verify blockchain confirmation
    assert_eq!(receipt.confirmation.blockchain, blockchain);
    assert!(!receipt.confirmation.transaction_hash.is_empty());
}

#[tokio::test]
#[ignore = "Requires migration 20240101000021 to be reapplied with VARCHAR(100) for currency column"]
async fn test_generate_conversion_receipt() {
    let (service, db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let conversion_id = Uuid::new_v4();
    let from_amount = Decimal::new(100, 0); // 100 USDC
    let from_asset = "USDC".to_string();
    let to_amount = Decimal::new(1, 0); // 1 SOL
    let to_asset = "SOL".to_string();
    let exchange_rate = Decimal::new(100, 0); // 100 USDC per SOL
    let sender = "wallet1".to_string();
    let recipient = "wallet2".to_string();
    let blockchain = Blockchain::Solana;
    let network_fee = Some(Decimal::new(5000, 6)); // 0.005 SOL
    let platform_fee = Some(Decimal::new(50, 2)); // 0.50 USDC
    let provider_fee = Some(Decimal::new(25, 2)); // 0.25 USDC

    // Create the conversion record first (required for foreign key)
    let user_id = create_test_user(&db).await;
    let client = db.get().await.expect("Failed to get db connection");
    client
        .execute(
            r#"
            INSERT INTO conversions 
            (id, user_id, from_asset, to_asset, from_amount, to_amount, exchange_rate, provider, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            "#,
            &[
                &conversion_id,
                &user_id,
                &from_asset,
                &to_asset,
                &from_amount,
                &to_amount,
                &exchange_rate,
                &"SIDESHIFT",
                &"COMPLETED",
            ],
        )
        .await
        .expect("Failed to insert conversion record");

    let result = service
        .generate_conversion_receipt(
            conversion_id,
            from_amount,
            from_asset.clone(),
            to_amount,
            to_asset.clone(),
            exchange_rate,
            sender.clone(),
            recipient.clone(),
            blockchain,
            network_fee,
            platform_fee,
            provider_fee,
        )
        .await;

    assert!(result.is_ok(), "Failed to generate conversion receipt: {:?}", result.err());
    let receipt = result.unwrap();

    // Verify all required fields
    assert_eq!(receipt.transaction_id, conversion_id);
    assert_eq!(receipt.transaction_type, TransactionType::Conversion);
    assert_eq!(receipt.amount, from_amount);
    assert!(receipt.currency.contains(&from_asset));
    assert!(receipt.currency.contains(&to_asset));
    assert_eq!(receipt.sender, sender);
    assert_eq!(receipt.recipient, recipient);

    // Verify exchange rate is present
    assert_eq!(receipt.exchange_rate, Some(exchange_rate));

    // Verify all fees are present
    assert_eq!(receipt.fees.network_fee, network_fee);
    assert_eq!(receipt.fees.platform_fee, platform_fee);
    assert_eq!(receipt.fees.provider_fee, provider_fee);
    assert_eq!(receipt.fees.total, Decimal::new(80, 2)); // 0.005 + 0.50 + 0.25 = 0.755 (rounded)

    // Verify blockchain confirmation
    assert_eq!(receipt.confirmation.blockchain, blockchain);
    assert!(!receipt.confirmation.transaction_hash.is_empty());
}

#[tokio::test]
async fn test_receipt_has_all_required_fields() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let payment_id = Uuid::new_v4();
    let result = service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    let receipt = result.unwrap();

    // Verify all required fields per requirements 13.1 and 13.2
    // Transaction ID
    assert_eq!(receipt.transaction_id, payment_id);

    // Timestamp
    assert!(receipt.timestamp <= chrono::Utc::now());

    // Amount
    assert_eq!(receipt.amount, Decimal::new(100, 0));

    // Fees (even if zero)
    assert!(receipt.fees.total >= Decimal::ZERO);

    // Exchange rate (optional, but field exists)
    // For payments, this should be None
    assert_eq!(receipt.exchange_rate, None);

    // Blockchain confirmation
    assert!(!receipt.confirmation.transaction_hash.is_empty());
    assert_eq!(receipt.confirmation.blockchain, Blockchain::Ethereum);
}

#[tokio::test]
async fn test_get_receipt_by_id() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // First create a receipt
    let payment_id = Uuid::new_v4();
    let created_receipt = service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create receipt");

    // Now retrieve it by ID
    let result = service.get_receipt(created_receipt.id).await;

    assert!(result.is_ok(), "Failed to get receipt by ID");
    let retrieved_receipt = result.unwrap();

    // Verify it's the same receipt
    assert_eq!(retrieved_receipt.id, created_receipt.id);
    assert_eq!(retrieved_receipt.transaction_id, payment_id);
    assert_eq!(retrieved_receipt.amount, created_receipt.amount);
}

#[tokio::test]
async fn test_get_receipt_by_transaction() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create a trade receipt
    let trade_id = Uuid::new_v4();
    let created_receipt = service
        .generate_trade_receipt(
            trade_id,
            Decimal::new(10, 0),
            "SOL".to_string(),
            Some(Decimal::new(100, 0)),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            None,
            None,
        )
        .await
        .expect("Failed to create trade receipt");

    // Retrieve by transaction ID
    let result = service
        .get_receipt_by_transaction(TransactionType::Trade, trade_id)
        .await;

    assert!(result.is_ok(), "Failed to get receipt by transaction");
    let retrieved_receipt = result.unwrap();

    assert!(retrieved_receipt.is_some());
    let receipt = retrieved_receipt.unwrap();
    assert_eq!(receipt.id, created_receipt.id);
    assert_eq!(receipt.transaction_id, trade_id);
    assert_eq!(receipt.transaction_type, TransactionType::Trade);
}

#[tokio::test]
async fn test_verify_receipt() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create a receipt
    let payment_id = Uuid::new_v4();
    let created_receipt = service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create receipt");

    // Verify the receipt
    let result = service.verify_receipt(created_receipt.id).await;

    assert!(result.is_ok(), "Failed to verify receipt");
    let verified_receipt = result.unwrap();

    // Verification status should be updated
    assert_eq!(verified_receipt.id, created_receipt.id);
    // Note: In the test implementation, verification will succeed
    // because we're using simulated blockchain transactions
}

#[tokio::test]
#[ignore = "Requires migration 20240101000021 to be reapplied with VARCHAR(100) for currency column"]
async fn test_fees_calculation() {
    let (service, db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Test with all fees
    let conversion_id = Uuid::new_v4();
    
    // Create the conversion record first
    let user_id = create_test_user(&db).await;
    let client = db.get().await.expect("Failed to get db connection");
    client
        .execute(
            r#"
            INSERT INTO conversions 
            (id, user_id, from_asset, to_asset, from_amount, to_amount, exchange_rate, provider, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            "#,
            &[
                &conversion_id,
                &user_id,
                &"USDC",
                &"SOL",
                &Decimal::new(100, 0),
                &Decimal::new(1, 0),
                &Decimal::new(100, 0),
                &"SIDESHIFT",
                &"COMPLETED",
            ],
        )
        .await
        .expect("Failed to insert conversion record");
    
    let receipt = service
        .generate_conversion_receipt(
            conversion_id,
            Decimal::new(100, 0),
            "USDC".to_string(),
            Decimal::new(1, 0),
            "SOL".to_string(),
            Decimal::new(100, 0),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            Some(Decimal::new(10, 2)),  // 0.10
            Some(Decimal::new(20, 2)),  // 0.20
            Some(Decimal::new(30, 2)),  // 0.30
        )
        .await
        .expect("Failed to create conversion receipt");

    // Total should be sum of all fees
    assert_eq!(receipt.fees.total, Decimal::new(60, 2)); // 0.60

    // Test with partial fees
    let payment_id = Uuid::new_v4();
    let receipt = service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            Some(Decimal::new(15, 2)), // 0.15
            None,
        )
        .await
        .expect("Failed to create payment receipt");

    assert_eq!(receipt.fees.total, Decimal::new(15, 2)); // 0.15

    // Test with no fees
    let trade_id = Uuid::new_v4();
    let receipt = service
        .generate_trade_receipt(
            trade_id,
            Decimal::new(5, 0),
            "SOL".to_string(),
            None,
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            None,
            None,
        )
        .await
        .expect("Failed to create trade receipt");

    assert_eq!(receipt.fees.total, Decimal::ZERO);
}

#[tokio::test]
async fn test_multiple_blockchains() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let blockchains = vec![
        Blockchain::Solana,
        Blockchain::Ethereum,
        Blockchain::BinanceSmartChain,
        Blockchain::Polygon,
    ];

    for blockchain in blockchains {
        let payment_id = Uuid::new_v4();
        let result = service
            .generate_payment_receipt(
                payment_id,
                Decimal::new(100, 0),
                "USD".to_string(),
                "sender".to_string(),
                "recipient".to_string(),
                blockchain,
                None,
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Failed to create receipt for blockchain: {:?}",
            blockchain
        );
        let receipt = result.unwrap();
        assert_eq!(receipt.confirmation.blockchain, blockchain);
    }
}

#[test]
fn test_transaction_type_enum() {
    assert_eq!(TransactionType::Payment.as_str(), "PAYMENT");
    assert_eq!(TransactionType::Trade.as_str(), "TRADE");
    assert_eq!(TransactionType::Conversion.as_str(), "CONVERSION");
}

#[tokio::test]
async fn test_receipt_timestamp_accuracy() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let before = chrono::Utc::now();
    
    let payment_id = Uuid::new_v4();
    let receipt = service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create receipt");

    let after = chrono::Utc::now();

    // Timestamp should be between before and after
    assert!(receipt.timestamp >= before);
    assert!(receipt.timestamp <= after);
}

#[tokio::test]
#[ignore = "Requires migration 20240101000021 to be reapplied with VARCHAR(100) for currency column"]
async fn test_conversion_receipt_exchange_rate() {
    let (service, db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let conversion_id = Uuid::new_v4();
    let exchange_rate = Decimal::new(12345, 2); // 123.45

    // Create the conversion record first
    let user_id = create_test_user(&db).await;
    let client = db.get().await.expect("Failed to get db connection");
    client
        .execute(
            r#"
            INSERT INTO conversions 
            (id, user_id, from_asset, to_asset, from_amount, to_amount, exchange_rate, provider, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            "#,
            &[
                &conversion_id,
                &user_id,
                &"USDC",
                &"SOL",
                &Decimal::new(100, 0),
                &Decimal::new(1, 0),
                &exchange_rate,
                &"SIDESHIFT",
                &"COMPLETED",
            ],
        )
        .await
        .expect("Failed to insert conversion record");

    let receipt = service
        .generate_conversion_receipt(
            conversion_id,
            Decimal::new(100, 0),
            "USDC".to_string(),
            Decimal::new(1, 0),
            "SOL".to_string(),
            exchange_rate,
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create conversion receipt");

    // Exchange rate should be present for conversions
    assert!(receipt.exchange_rate.is_some());
    assert_eq!(receipt.exchange_rate.unwrap(), exchange_rate);
}

#[tokio::test]
async fn test_search_receipts_no_filters() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create multiple receipts
    for i in 0..5 {
        let payment_id = Uuid::new_v4();
        service
            .generate_payment_receipt(
                payment_id,
                Decimal::new(100 + i, 0),
                "USD".to_string(),
                "sender".to_string(),
                "recipient".to_string(),
                Blockchain::Ethereum,
                None,
                None,
            )
            .await
            .expect("Failed to create payment receipt");
    }

    // Search without filters
    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: None,
        start_date: None,
        end_date: None,
    };

    let pagination = api::Pagination {
        page: 0,
        page_size: 10,
    };

    let result = service.search_receipts(filters, pagination).await;

    if let Err(ref e) = result {
        eprintln!("Search error: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to search receipts: {:?}", result.err());
    let results = result.unwrap();

    // Should have at least 5 receipts
    assert!(results.receipts.len() >= 5);
    assert!(results.total_count >= 5);
    assert_eq!(results.page, 0);
    assert_eq!(results.page_size, 10);
}

#[tokio::test]
async fn test_search_receipts_by_type() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create different types of receipts
    let payment_id = Uuid::new_v4();
    service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create payment receipt");

    let trade_id = Uuid::new_v4();
    service
        .generate_trade_receipt(
            trade_id,
            Decimal::new(10, 0),
            "SOL".to_string(),
            Some(Decimal::new(100, 0)),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            None,
            None,
        )
        .await
        .expect("Failed to create trade receipt");

    // Search for only payments
    let filters = api::ReceiptSearchFilters {
        transaction_type: Some(TransactionType::Payment),
        asset: None,
        start_date: None,
        end_date: None,
    };

    let pagination = api::Pagination::default();

    let result = service.search_receipts(filters, pagination).await;

    assert!(result.is_ok(), "Failed to search receipts");
    let results = result.unwrap();

    // All results should be payments
    for receipt in &results.receipts {
        assert_eq!(receipt.transaction_type, TransactionType::Payment);
    }
}

#[tokio::test]
async fn test_search_receipts_by_asset() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create receipts with different assets
    let payment_id = Uuid::new_v4();
    service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create USD receipt");

    let trade_id = Uuid::new_v4();
    service
        .generate_trade_receipt(
            trade_id,
            Decimal::new(10, 0),
            "SOL".to_string(),
            Some(Decimal::new(100, 0)),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            None,
            None,
        )
        .await
        .expect("Failed to create SOL receipt");

    // Search for SOL receipts
    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: Some("SOL".to_string()),
        start_date: None,
        end_date: None,
    };

    let pagination = api::Pagination::default();

    let result = service.search_receipts(filters, pagination).await;

    assert!(result.is_ok(), "Failed to search receipts");
    let results = result.unwrap();

    // All results should contain SOL
    for receipt in &results.receipts {
        assert!(receipt.currency.contains("SOL"));
    }
}

#[tokio::test]
#[ignore] // TODO: Fix DateTime serialization for date range filters
async fn test_search_receipts_by_date_range() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let now = chrono::Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);
    let two_hours_ago = now - chrono::Duration::hours(2);

    // Create a receipt
    let payment_id = Uuid::new_v4();
    service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create receipt");

    // Search for receipts in the last hour
    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: None,
        start_date: Some(one_hour_ago),
        end_date: Some(now),
    };

    let pagination = api::Pagination::default();

    let result = service.search_receipts(filters, pagination).await;

    if let Err(ref e) = result {
        eprintln!("Search error in date range test: {:?}", e);
    }
    assert!(result.is_ok(), "Failed to search receipts: {:?}", result.err());
    let results = result.unwrap();

    // Should find the receipt we just created
    assert!(results.receipts.len() > 0, "Expected to find at least one receipt in date range");

    // Search for receipts older than 2 hours (should find nothing recent)
    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: None,
        start_date: None,
        end_date: Some(two_hours_ago),
    };

    let result = service.search_receipts(filters, pagination).await;

    assert!(result.is_ok(), "Failed to search receipts");
    // This might have old receipts from other tests, so we just verify it doesn't error
}

#[tokio::test]
async fn test_search_receipts_pagination() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create 15 receipts
    for i in 0..15 {
        let payment_id = Uuid::new_v4();
        service
            .generate_payment_receipt(
                payment_id,
                Decimal::new(100 + i, 0),
                "USD".to_string(),
                "sender".to_string(),
                "recipient".to_string(),
                Blockchain::Ethereum,
                None,
                None,
            )
            .await
            .expect("Failed to create receipt");
    }

    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: None,
        start_date: None,
        end_date: None,
    };

    // Get first page (5 items)
    let pagination = api::Pagination {
        page: 0,
        page_size: 5,
    };

    let result = service.search_receipts(filters.clone(), pagination).await;

    assert!(result.is_ok(), "Failed to search receipts page 0");
    let page0 = result.unwrap();

    assert_eq!(page0.receipts.len(), 5);
    assert!(page0.total_count >= 15);
    assert_eq!(page0.page, 0);
    assert_eq!(page0.page_size, 5);
    assert!(page0.total_pages >= 3);

    // Get second page
    let pagination = api::Pagination {
        page: 1,
        page_size: 5,
    };

    let result = service.search_receipts(filters.clone(), pagination).await;

    assert!(result.is_ok(), "Failed to search receipts page 1");
    let page1 = result.unwrap();

    assert_eq!(page1.receipts.len(), 5);
    assert_eq!(page1.page, 1);

    // Verify pages have different receipts
    let page0_ids: Vec<Uuid> = page0.receipts.iter().map(|r| r.id).collect();
    let page1_ids: Vec<Uuid> = page1.receipts.iter().map(|r| r.id).collect();

    for id in &page1_ids {
        assert!(!page0_ids.contains(id), "Pages should have different receipts");
    }
}

#[tokio::test]
#[ignore] // TODO: Fix DateTime serialization for date range filters
async fn test_search_receipts_combined_filters() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let now = chrono::Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);

    // Create a conversion receipt
    let conversion_id = Uuid::new_v4();
    service
        .generate_conversion_receipt(
            conversion_id,
            Decimal::new(100, 0),
            "USDC".to_string(),
            Decimal::new(1, 0),
            "SOL".to_string(),
            Decimal::new(100, 0),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create conversion receipt");

    // Search with multiple filters
    let filters = api::ReceiptSearchFilters {
        transaction_type: Some(TransactionType::Conversion),
        asset: Some("SOL".to_string()),
        start_date: Some(one_hour_ago),
        end_date: Some(now),
    };

    let pagination = api::Pagination::default();

    let result = service.search_receipts(filters, pagination).await;

    assert!(result.is_ok(), "Failed to search receipts with combined filters");
    let results = result.unwrap();

    // Should find the conversion receipt
    assert!(results.receipts.len() > 0);

    // Verify all results match the filters
    for receipt in &results.receipts {
        assert_eq!(receipt.transaction_type, TransactionType::Conversion);
        assert!(receipt.currency.contains("SOL"));
        assert!(receipt.timestamp >= one_hour_ago);
        assert!(receipt.timestamp <= now);
    }
}

#[tokio::test]
async fn test_search_receipts_empty_results() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Search for a non-existent asset
    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: Some("NONEXISTENT_ASSET_XYZ".to_string()),
        start_date: None,
        end_date: None,
    };

    let pagination = api::Pagination::default();

    let result = service.search_receipts(filters, pagination).await;

    assert!(result.is_ok(), "Search should succeed even with no results");
    let results = result.unwrap();

    assert_eq!(results.receipts.len(), 0);
    assert_eq!(results.total_count, 0);
    assert_eq!(results.total_pages, 0);
}


#[tokio::test]
async fn test_export_receipt_pdf() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create a receipt
    let payment_id = Uuid::new_v4();
    let receipt = service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender_address".to_string(),
            "recipient_address".to_string(),
            Blockchain::Ethereum,
            Some(Decimal::new(5, 2)),
            Some(Decimal::new(10, 2)),
        )
        .await
        .expect("Failed to create receipt");

    // Export as PDF
    let result = service.export_receipt_pdf(receipt.id).await;

    assert!(result.is_ok(), "Failed to export PDF: {:?}", result.err());
    let pdf_bytes = result.unwrap();

    // Verify PDF has content
    assert!(pdf_bytes.len() > 0, "PDF should have content");
    
    // PDF files start with %PDF
    assert!(pdf_bytes.starts_with(b"%PDF"), "Should be a valid PDF file");
}

#[tokio::test]
async fn test_export_receipts_csv() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create multiple receipts
    for i in 0..3 {
        let payment_id = Uuid::new_v4();
        service
            .generate_payment_receipt(
                payment_id,
                Decimal::new(100 + i, 0),
                "USD".to_string(),
                "sender".to_string(),
                "recipient".to_string(),
                Blockchain::Ethereum,
                None,
                None,
            )
            .await
            .expect("Failed to create receipt");
    }

    // Export as CSV
    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: None,
        start_date: None,
        end_date: None,
    };

    let result = service.export_receipts_csv(filters).await;

    assert!(result.is_ok(), "Failed to export CSV: {:?}", result.err());
    let csv_bytes = result.unwrap();

    // Verify CSV has content
    assert!(csv_bytes.len() > 0, "CSV should have content");
    
    // Convert to string and verify it's valid CSV
    let csv_string = String::from_utf8(csv_bytes).expect("CSV should be valid UTF-8");
    
    // Should have header row
    assert!(csv_string.contains("Receipt ID"), "CSV should have header");
    assert!(csv_string.contains("Transaction ID"), "CSV should have header");
    
    // Should have at least 3 data rows (plus header)
    let line_count = csv_string.lines().count();
    assert!(line_count >= 4, "CSV should have at least 4 lines (header + 3 receipts), got {}", line_count);
}

#[tokio::test]
async fn test_export_csv_with_filters() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create receipts of different types
    let payment_id = Uuid::new_v4();
    service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create payment receipt");

    let trade_id = Uuid::new_v4();
    service
        .generate_trade_receipt(
            trade_id,
            Decimal::new(10, 0),
            "SOL".to_string(),
            Some(Decimal::new(100, 0)),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Solana,
            None,
            None,
        )
        .await
        .expect("Failed to create trade receipt");

    // Export only payments
    let filters = api::ReceiptSearchFilters {
        transaction_type: Some(api::TransactionType::Payment),
        asset: None,
        start_date: None,
        end_date: None,
    };

    let result = service.export_receipts_csv(filters).await;

    assert!(result.is_ok(), "Failed to export filtered CSV");
    let csv_bytes = result.unwrap();
    let csv_string = String::from_utf8(csv_bytes).expect("CSV should be valid UTF-8");

    // Should contain PAYMENT type
    assert!(csv_string.contains("PAYMENT"), "CSV should contain PAYMENT type");
    
    // Count occurrences - should have at least one PAYMENT
    let payment_count = csv_string.matches("PAYMENT").count();
    assert!(payment_count >= 1, "Should have at least one PAYMENT in CSV");
}

#[tokio::test]
async fn test_export_pdf_nonexistent_receipt() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let nonexistent_id = Uuid::new_v4();
    let result = service.export_receipt_pdf(nonexistent_id).await;

    assert!(result.is_err(), "Should fail for nonexistent receipt");
}

#[tokio::test]
async fn test_export_csv_empty_results() {
    let (service, _db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Export with filter that matches nothing
    let filters = api::ReceiptSearchFilters {
        transaction_type: None,
        asset: Some("NONEXISTENT_ASSET_XYZ".to_string()),
        start_date: None,
        end_date: None,
    };

    let result = service.export_receipts_csv(filters).await;

    assert!(result.is_ok(), "Should succeed even with no results");
    let csv_bytes = result.unwrap();
    let csv_string = String::from_utf8(csv_bytes).expect("CSV should be valid UTF-8");

    // Should have header but no data rows
    let line_count = csv_string.lines().count();
    assert_eq!(line_count, 1, "CSV should have only header row");
}


#[tokio::test]
#[ignore] // Requires migration 20240101000025 to be applied
async fn test_archive_old_receipts() {
    let (service, db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create a recent receipt (should not be archived)
    let recent_payment_id = Uuid::new_v4();
    service
        .generate_payment_receipt(
            recent_payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create recent receipt");

    // Manually create an old receipt (7+ years old) by inserting directly into database
    let old_payment_id = Uuid::new_v4();
    let seven_years_ago = chrono::Utc::now() - chrono::Duration::days(7 * 365 + 30);
    
    let client = db.get().await.expect("Failed to get db connection");
    client
        .execute(
            r#"
            INSERT INTO blockchain_receipts 
            (payment_id, amount, currency, sender, recipient, blockchain, transaction_hash, verification_status, created_at, archived)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            &[
                &old_payment_id,
                &Decimal::new(100, 0),
                &"USD",
                &"sender",
                &"recipient",
                &"Ethereum",
                &"0xold_transaction_hash",
                &"CONFIRMED",
                &seven_years_ago,
                &false,
            ],
        )
        .await
        .expect("Failed to insert old receipt");

    // Run archival
    let result = service.archive_old_receipts().await;

    assert!(result.is_ok(), "Failed to archive old receipts: {:?}", result.err());
    let archived_count = result.unwrap();

    // Should have archived at least 1 receipt (the old one)
    assert!(archived_count >= 1, "Should have archived at least 1 receipt, got {}", archived_count);

    // Verify the old receipt is now archived
    let row = client
        .query_one(
            "SELECT archived FROM blockchain_receipts WHERE payment_id = $1",
            &[&old_payment_id],
        )
        .await
        .expect("Failed to query old receipt");

    let is_archived: bool = row.get(0);
    assert!(is_archived, "Old receipt should be archived");

    // Verify the recent receipt is NOT archived
    let recent_receipt = service.get_receipt_by_transaction(
        api::TransactionType::Payment,
        recent_payment_id,
    ).await.expect("Failed to get recent receipt");

    assert!(recent_receipt.is_some(), "Recent receipt should exist");
}

#[tokio::test]
#[ignore] // Requires migration 20240101000025 to be applied
async fn test_get_archived_count() {
    let (service, db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Get initial count
    let initial_count = service.get_archived_count().await.expect("Failed to get archived count");

    // Create and archive an old receipt
    let old_payment_id = Uuid::new_v4();
    let seven_years_ago = chrono::Utc::now() - chrono::Duration::days(7 * 365 + 30);
    
    let client = db.get().await.expect("Failed to get db connection");
    client
        .execute(
            r#"
            INSERT INTO blockchain_receipts 
            (payment_id, amount, currency, sender, recipient, blockchain, transaction_hash, verification_status, created_at, archived)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            &[
                &old_payment_id,
                &Decimal::new(100, 0),
                &"USD",
                &"sender",
                &"recipient",
                &"Ethereum",
                &"0xold_transaction_hash",
                &"CONFIRMED",
                &seven_years_ago,
                &false,
            ],
        )
        .await
        .expect("Failed to insert old receipt");

    // Archive it
    service.archive_old_receipts().await.expect("Failed to archive");

    // Get new count
    let new_count = service.get_archived_count().await.expect("Failed to get archived count");

    // Should have increased by at least 1
    assert!(new_count > initial_count, "Archived count should have increased");
}

#[tokio::test]
#[ignore] // This test requires the database trigger to be in place
async fn test_cannot_delete_recent_receipt() {
    let (service, db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create a recent receipt
    let payment_id = Uuid::new_v4();
    let receipt = service
        .generate_payment_receipt(
            payment_id,
            Decimal::new(100, 0),
            "USD".to_string(),
            "sender".to_string(),
            "recipient".to_string(),
            Blockchain::Ethereum,
            None,
            None,
        )
        .await
        .expect("Failed to create receipt");

    // Try to delete it (should fail due to retention policy)
    let client = db.get().await.expect("Failed to get db connection");
    let result = client
        .execute(
            "DELETE FROM blockchain_receipts WHERE id = $1",
            &[&receipt.id],
        )
        .await;

    // Should fail with retention policy error
    assert!(result.is_err(), "Should not be able to delete recent receipt");
    
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("7 years") || error_msg.contains("retention"),
            "Error should mention retention policy: {}",
            error_msg
        );
    }
}

#[tokio::test]
#[ignore] // Requires migration 20240101000025 to be applied
async fn test_archive_receipts_idempotent() {
    let (service, db) = match create_test_service().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    // Create an old receipt
    let old_payment_id = Uuid::new_v4();
    let seven_years_ago = chrono::Utc::now() - chrono::Duration::days(7 * 365 + 30);
    
    let client = db.get().await.expect("Failed to get db connection");
    client
        .execute(
            r#"
            INSERT INTO blockchain_receipts 
            (payment_id, amount, currency, sender, recipient, blockchain, transaction_hash, verification_status, created_at, archived)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            &[
                &old_payment_id,
                &Decimal::new(100, 0),
                &"USD",
                &"sender",
                &"recipient",
                &"Ethereum",
                &"0xold_transaction_hash",
                &"CONFIRMED",
                &seven_years_ago,
                &false,
            ],
        )
        .await
        .expect("Failed to insert old receipt");

    // Run archival first time
    let first_count = service.archive_old_receipts().await.expect("Failed to archive");
    assert!(first_count >= 1, "Should archive at least 1 receipt");

    // Run archival second time (should not archive anything new)
    let second_count = service.archive_old_receipts().await.expect("Failed to archive");
    assert_eq!(second_count, 0, "Should not archive already archived receipts");
}
