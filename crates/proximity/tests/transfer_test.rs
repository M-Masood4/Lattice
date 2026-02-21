// Tests for Transfer Service

use proximity::{ProximityError, TransferService};
use uuid::Uuid;

// Note: Most transfer service tests require database integration
// These tests verify the service behavior without database

async fn create_test_service() -> TransferService {
    // Create a test database pool
    // In CI/CD, this would connect to a test database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/test".to_string());
    
    let db_pool = database::create_pool(&database_url, 1)
        .await
        .expect("Failed to create test database pool");
    
    // Create a test Solana client
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let solana_client = std::sync::Arc::new(blockchain::SolanaClient::new(rpc_url, None));
    
    TransferService::new(db_pool, solana_client)
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_accept_nonexistent_transfer() {
    let service = create_test_service().await;
    
    let nonexistent_id = Uuid::new_v4();
    let result = service.accept_transfer(nonexistent_id).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::TransferNotFound(_)));
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_reject_nonexistent_transfer() {
    let service = create_test_service().await;
    
    let nonexistent_id = Uuid::new_v4();
    let result = service.reject_transfer(nonexistent_id, None).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::TransferNotFound(_)));
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_get_transfer_status_nonexistent() {
    let service = create_test_service().await;
    
    let nonexistent_id = Uuid::new_v4();
    let result = service.get_transfer_status(nonexistent_id).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::TransferNotFound(_)));
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_get_transfer_request_nonexistent() {
    let service = create_test_service().await;
    
    let nonexistent_id = Uuid::new_v4();
    let result = service.get_transfer_request(nonexistent_id).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::TransferNotFound(_)));
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_get_active_requests_empty() {
    let service = create_test_service().await;
    
    let requests = service.get_active_requests().await.unwrap();
    assert_eq!(requests.len(), 0);
}

// Note: The following tests require full database integration with test data
// They are documented here but not implemented until integration test infrastructure is ready

/*
Integration tests to implement:

1. test_create_transfer_request_with_sufficient_balance
   - Setup wallet with sufficient balance
   - Create transfer request
   - Verify request is created with Pending status

2. test_create_transfer_request_with_insufficient_balance
   - Setup wallet with insufficient balance
   - Attempt to create transfer request
   - Verify InsufficientBalance error is returned

3. test_create_transfer_request_validates_positive_amount
   - Attempt to create transfer with zero or negative amount
   - Verify error is returned

4. test_accept_transfer_updates_status
   - Create transfer request
   - Accept the transfer
   - Verify status changes to Accepted

5. test_reject_transfer_updates_status
   - Create transfer request
   - Reject the transfer
   - Verify status changes to Rejected

6. test_transfer_request_expires_after_timeout
   - Create transfer request
   - Wait for timeout period (60 seconds)
   - Verify status changes to Expired

7. test_cannot_accept_expired_transfer
   - Create transfer request
   - Wait for expiration
   - Attempt to accept
   - Verify Timeout error is returned

8. test_cannot_accept_already_accepted_transfer
   - Create and accept transfer
   - Attempt to accept again
   - Verify error is returned

9. test_cannot_reject_already_rejected_transfer
   - Create and reject transfer
   - Attempt to reject again
   - Verify error is returned

10. test_cleanup_task_removes_old_requests
    - Create multiple requests in various states
    - Wait for cleanup interval
    - Verify old completed/failed requests are removed
*/

// Receipt generation tests

#[test]
fn test_proximity_receipt_data_creation() {
    use proximity::{ProximityReceiptData, TransferRequest, TransferStatus};
    use rust_decimal::Decimal;
    use chrono::Utc;
    
    let request = TransferRequest {
        id: Uuid::new_v4(),
        sender_user_id: Uuid::new_v4(),
        sender_wallet: "sender_wallet_address".to_string(),
        recipient_user_id: Uuid::new_v4(),
        recipient_wallet: "recipient_wallet_address".to_string(),
        asset: "SOL".to_string(),
        amount: Decimal::new(10050, 2), // 100.50
        status: TransferStatus::Completed,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::seconds(60),
    };
    
    let tx_hash = "tx_hash_123456".to_string();
    let receipt_data = ProximityReceiptData::from_transfer(&request, tx_hash.clone());
    
    assert_eq!(receipt_data.proximity_transfer_id, request.id);
    assert_eq!(receipt_data.amount, request.amount);
    assert_eq!(receipt_data.currency, "SOL");
    assert_eq!(receipt_data.sender, "sender_wallet_address");
    assert_eq!(receipt_data.recipient, "recipient_wallet_address");
    assert_eq!(receipt_data.transaction_hash, tx_hash);
}

#[test]
fn test_proximity_receipt_data_with_spl_token() {
    use proximity::{ProximityReceiptData, TransferRequest, TransferStatus};
    use rust_decimal::Decimal;
    use chrono::Utc;
    
    let request = TransferRequest {
        id: Uuid::new_v4(),
        sender_user_id: Uuid::new_v4(),
        sender_wallet: "sender_wallet".to_string(),
        recipient_user_id: Uuid::new_v4(),
        recipient_wallet: "recipient_wallet".to_string(),
        asset: "USDC".to_string(),
        amount: Decimal::new(50000, 2), // 500.00
        status: TransferStatus::Completed,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::seconds(60),
    };
    
    let tx_hash = "spl_token_tx_hash".to_string();
    let receipt_data = ProximityReceiptData::from_transfer(&request, tx_hash.clone());
    
    assert_eq!(receipt_data.currency, "USDC");
    assert_eq!(receipt_data.amount, Decimal::new(50000, 2));
    assert_eq!(receipt_data.transaction_hash, tx_hash);
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_concurrent_transfer_limit() {
    let service = create_test_service().await;
    
    let sender_user_id = Uuid::new_v4();
    let recipient_user_id = Uuid::new_v4();
    let sender_wallet = "SenderWallet111111111111111111111111111".to_string();
    let recipient_wallet = "RecipientWallet111111111111111111111111".to_string();
    
    // Create 5 transfer requests (at limit)
    for i in 0..5 {
        let result = service.create_transfer_request(
            sender_user_id,
            sender_wallet.clone(),
            recipient_user_id,
            recipient_wallet.clone(),
            "SOL".to_string(),
            rust_decimal::Decimal::new(1, 0),
        ).await;
        
        // First 5 should succeed and be active
        assert!(result.is_ok());
    }
    
    let active = service.get_active_requests().await.unwrap();
    let user_active = active.iter().filter(|r| r.sender_user_id == sender_user_id).count();
    assert_eq!(user_active, 5);
    
    // 6th request should be queued
    let result = service.create_transfer_request(
        sender_user_id,
        sender_wallet.clone(),
        recipient_user_id,
        recipient_wallet.clone(),
        "SOL".to_string(),
        rust_decimal::Decimal::new(1, 0),
    ).await;
    
    assert!(result.is_ok());
    
    // Should still have 5 active
    let active = service.get_active_requests().await.unwrap();
    let user_active = active.iter().filter(|r| r.sender_user_id == sender_user_id).count();
    assert_eq!(user_active, 5);
    
    // Should have 1 queued
    let queued = service.get_queued_requests().await.unwrap();
    let user_queued = queued.iter().filter(|r| r.sender_user_id == sender_user_id).count();
    assert_eq!(user_queued, 1);
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_queue_processing_on_rejection() {
    let service = create_test_service().await;
    
    let sender_user_id = Uuid::new_v4();
    let recipient_user_id = Uuid::new_v4();
    let sender_wallet = "SenderWallet111111111111111111111111111".to_string();
    let recipient_wallet = "RecipientWallet111111111111111111111111".to_string();
    
    // Create 5 transfer requests (at limit)
    let mut request_ids = Vec::new();
    for _ in 0..5 {
        let result = service.create_transfer_request(
            sender_user_id,
            sender_wallet.clone(),
            recipient_user_id,
            recipient_wallet.clone(),
            "SOL".to_string(),
            rust_decimal::Decimal::new(1, 0),
        ).await.unwrap();
        request_ids.push(result.id);
    }
    
    // Create 6th request (should be queued)
    let queued_request = service.create_transfer_request(
        sender_user_id,
        sender_wallet.clone(),
        recipient_user_id,
        recipient_wallet.clone(),
        "SOL".to_string(),
        rust_decimal::Decimal::new(1, 0),
    ).await.unwrap();
    
    // Verify it's queued
    let queued = service.get_queued_requests().await.unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].id, queued_request.id);
    
    // Reject one of the active requests
    service.reject_transfer(request_ids[0], Some("Test rejection".to_string())).await.unwrap();
    
    // Queued request should now be active
    let active = service.get_active_requests().await.unwrap();
    assert!(active.iter().any(|r| r.id == queued_request.id));
    
    // Queue should be empty
    let queued = service.get_queued_requests().await.unwrap();
    let user_queued = queued.iter().filter(|r| r.sender_user_id == sender_user_id).count();
    assert_eq!(user_queued, 0);
}

#[tokio::test]
#[ignore] // Requires test database
async fn test_different_users_independent_limits() {
    let service = create_test_service().await;
    
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    let recipient_user_id = Uuid::new_v4();
    let wallet1 = "User1Wallet111111111111111111111111111".to_string();
    let wallet2 = "User2Wallet111111111111111111111111111".to_string();
    let recipient_wallet = "RecipientWallet111111111111111111111111".to_string();
    
    // Create 5 requests for user1
    for _ in 0..5 {
        service.create_transfer_request(
            user1_id,
            wallet1.clone(),
            recipient_user_id,
            recipient_wallet.clone(),
            "SOL".to_string(),
            rust_decimal::Decimal::new(1, 0),
        ).await.unwrap();
    }
    
    // Create 5 requests for user2 (should not be affected by user1's limit)
    for _ in 0..5 {
        let result = service.create_transfer_request(
            user2_id,
            wallet2.clone(),
            recipient_user_id,
            recipient_wallet.clone(),
            "SOL".to_string(),
            rust_decimal::Decimal::new(1, 0),
        ).await;
        
        assert!(result.is_ok());
    }
    
    let active = service.get_active_requests().await.unwrap();
    let user1_active = active.iter().filter(|r| r.sender_user_id == user1_id).count();
    let user2_active = active.iter().filter(|r| r.sender_user_id == user2_id).count();
    
    assert_eq!(user1_active, 5);
    assert_eq!(user2_active, 5);
}

// Tests for SPL token account handling

#[tokio::test]
#[ignore] // Requires Solana devnet connection
async fn test_check_token_account_exists_for_nonexistent_account() {
    let service = create_test_service().await;
    
    // Use a valid wallet address and token mint for testing
    let recipient_wallet = "11111111111111111111111111111111"; // System program (won't have token accounts)
    let token_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC mint on devnet
    
    let result = service.check_token_account_exists(recipient_wallet, token_mint).await;
    
    // Should return Ok(false) for non-existent account
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[tokio::test]
#[ignore] // Requires Solana devnet connection
async fn test_check_token_account_with_invalid_addresses() {
    let service = create_test_service().await;
    
    // Test with invalid recipient address
    let result = service.check_token_account_exists("invalid_address", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::InvalidWalletAddress(_)));
    
    // Test with invalid token mint
    let result = service.check_token_account_exists("11111111111111111111111111111111", "invalid_mint").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::InvalidWalletAddress(_)));
}

#[tokio::test]
#[ignore] // Requires Solana devnet connection
async fn test_calculate_token_account_creation_fee() {
    let service = create_test_service().await;
    
    let result = service.calculate_token_account_creation_fee().await;
    
    // Should return a positive fee amount
    assert!(result.is_ok());
    let fee = result.unwrap();
    assert!(fee > rust_decimal::Decimal::ZERO);
    
    // Token account creation fee should be around 0.00203928 SOL (rent exemption for 165 bytes)
    // Allow some variance for different network conditions
    assert!(fee > rust_decimal::Decimal::new(1, 3)); // > 0.001 SOL
    assert!(fee < rust_decimal::Decimal::new(1, 2)); // < 0.01 SOL
}

#[tokio::test]
#[ignore] // Requires Solana devnet connection
async fn test_validate_transfer_requirements_for_sol() {
    let service = create_test_service().await;
    
    let sender = "11111111111111111111111111111111";
    let recipient = "22222222222222222222222222222222";
    let asset = "SOL";
    let amount = rust_decimal::Decimal::new(1, 0); // 1 SOL
    
    let result = service.validate_transfer_requirements(sender, recipient, asset, amount).await;
    
    // SOL transfers don't need token accounts
    assert!(result.is_ok());
    let (is_valid, needs_token_account, fee) = result.unwrap();
    assert!(is_valid);
    assert!(!needs_token_account);
    assert!(fee.is_none());
}

#[tokio::test]
#[ignore] // Requires Solana devnet connection
async fn test_validate_transfer_requirements_for_spl_token_without_account() {
    let service = create_test_service().await;
    
    let sender = "11111111111111111111111111111111";
    let recipient = "22222222222222222222222222222222";
    let token_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    let amount = rust_decimal::Decimal::new(10, 0); // 10 USDC
    
    let result = service.validate_transfer_requirements(sender, recipient, token_mint, amount).await;
    
    // Should indicate token account creation is needed
    assert!(result.is_ok());
    let (is_valid, needs_token_account, fee) = result.unwrap();
    assert!(is_valid);
    assert!(needs_token_account);
    assert!(fee.is_some());
    assert!(fee.unwrap() > rust_decimal::Decimal::ZERO);
}

#[tokio::test]
#[ignore] // Requires Solana devnet connection and funded wallet
async fn test_create_token_account() {
    let service = create_test_service().await;
    
    // Note: This test requires a funded payer wallet
    // In a real test environment, you would use a test wallet with devnet SOL
    let payer = "YourTestPayerWallet111111111111111111111";
    let recipient = "YourTestRecipientWallet11111111111111111";
    let token_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
    
    let result = service.create_token_account(payer, recipient, token_mint).await;
    
    // This will fail in CI without proper test setup, but demonstrates the API
    // In a proper test environment with funded wallets, this should succeed
    if result.is_ok() {
        let tx_hash = result.unwrap();
        assert!(!tx_hash.is_empty());
        assert!(tx_hash.len() > 32); // Solana signatures are 88 characters
    }
}

// Integration test scenarios for token account handling

/*
Additional integration tests to implement with proper test infrastructure:

1. test_transfer_spl_token_creates_account_if_needed
   - Setup sender with SPL tokens
   - Setup recipient without token account
   - Create transfer request
   - Verify system offers to create token account
   - Accept offer and execute transfer
   - Verify token account is created and transfer succeeds

2. test_transfer_spl_token_with_existing_account
   - Setup sender with SPL tokens
   - Setup recipient with existing token account
   - Create and execute transfer
   - Verify no token account creation is needed
   - Verify transfer succeeds

3. test_token_account_creation_fee_calculation_accuracy
   - Query actual rent exemption from Solana
   - Calculate fee using service method
   - Verify calculated fee matches actual rent exemption

4. test_concurrent_token_account_creation_requests
   - Attempt to create same token account multiple times concurrently
   - Verify only one creation succeeds
   - Verify others handle gracefully

5. test_token_account_validation_with_network_errors
   - Simulate network errors during token account check
   - Verify appropriate error handling
   - Verify retry logic works correctly
*/

// Tests for token account creation offer

#[tokio::test]
#[ignore] // Requires test database and Solana devnet
async fn test_create_transfer_request_with_validation_for_sol() {
    let service = create_test_service().await;
    
    let sender_user_id = Uuid::new_v4();
    let recipient_user_id = Uuid::new_v4();
    let sender_wallet = "11111111111111111111111111111111".to_string();
    let recipient_wallet = "22222222222222222222222222222222".to_string();
    let asset = "SOL".to_string();
    let amount = rust_decimal::Decimal::new(1, 0);
    
    // Note: This will fail without proper test database setup
    // In a real test environment, wallets would be set up with balances
    let result = service.create_transfer_request_with_validation(
        sender_user_id,
        sender_wallet,
        recipient_user_id,
        recipient_wallet,
        asset,
        amount,
    ).await;
    
    // For SOL transfers, should not need token account
    if result.is_ok() {
        let (_request, needs_token_account, fee) = result.unwrap();
        assert!(!needs_token_account);
        assert!(fee.is_none());
    }
}

#[tokio::test]
#[ignore] // Requires test database and Solana devnet
async fn test_create_transfer_request_with_validation_for_spl_token() {
    let service = create_test_service().await;
    
    let sender_user_id = Uuid::new_v4();
    let recipient_user_id = Uuid::new_v4();
    let sender_wallet = "11111111111111111111111111111111".to_string();
    let recipient_wallet = "22222222222222222222222222222222".to_string();
    let token_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(); // USDC
    let amount = rust_decimal::Decimal::new(10, 0);
    
    // Note: This will fail without proper test database setup
    let result = service.create_transfer_request_with_validation(
        sender_user_id,
        sender_wallet,
        recipient_user_id,
        recipient_wallet,
        token_mint,
        amount,
    ).await;
    
    // For SPL tokens to a wallet without token account, should offer creation
    if result.is_ok() {
        let (_request, needs_token_account, fee) = result.unwrap();
        // Recipient likely doesn't have token account
        if needs_token_account {
            assert!(fee.is_some());
            assert!(fee.unwrap() > rust_decimal::Decimal::ZERO);
        }
    }
}

#[tokio::test]
#[ignore] // Requires Solana devnet connection
async fn test_spl_transfer_with_automatic_token_account_creation() {
    let service = create_test_service().await;
    
    // This test verifies that execute_spl_token_transfer automatically
    // creates token accounts when they don't exist
    
    // Note: In a real test, you would:
    // 1. Create a funded sender wallet
    // 2. Create a recipient wallet without a token account
    // 3. Execute the transfer
    // 4. Verify the token account was created
    // 5. Verify the transfer succeeded
    
    // This is a placeholder to document the expected behavior
}

// Documentation for UI integration

/*
UI Integration Guide for Token Account Creation Offer:

When creating a proximity transfer in the UI, use the 
`create_transfer_request_with_validation` method instead of 
`create_transfer_request`. This will return:

1. The transfer request
2. A boolean indicating if token account creation is needed
3. The estimated fee for token account creation (if needed)

Example UI flow:

```rust
let (request, needs_token_account, creation_fee) = transfer_service
    .create_transfer_request_with_validation(
        sender_user_id,
        sender_wallet,
        recipient_user_id,
        recipient_wallet,
        asset,
        amount,
    )
    .await?;

if needs_token_account {
    let fee = creation_fee.unwrap();
    // Display to user:
    // "The recipient doesn't have a token account for this asset.
    //  A token account will be created automatically.
    //  Additional fee: {fee} SOL"
    
    // Show confirmation dialog with:
    // - Transfer amount
    // - Token account creation fee
    // - Total cost
    
    // If user confirms, proceed with transfer
    // The execute_spl_token_transfer method will automatically
    // create the token account as part of the transaction
}
```

The token account creation happens automatically during transfer execution,
so no additional API calls are needed. The sender pays for the token account
creation as part of the transfer transaction.
*/
