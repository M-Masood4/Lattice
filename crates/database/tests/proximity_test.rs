use chrono::{Duration, Utc};
use database::{
    add_blocked_peer, create_pool, end_discovery_session, get_blocked_peers, get_transfer_by_id,
    get_user_proximity_transfers, insert_discovery_session, insert_proximity_transfer,
    remove_blocked_peer, run_migrations, update_session_expiration, update_transfer_status,
    TransferFilter,
};
use rust_decimal::Decimal;
use uuid::Uuid;

async fn setup_test_db() -> database::DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/test".to_string());

    let pool = create_pool(&database_url, 5).await.unwrap();
    run_migrations(&pool).await.unwrap();
    pool
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_insert_and_get_proximity_transfer() {
    let pool = setup_test_db().await;

    let sender_id = Uuid::new_v4();
    let recipient_id = Uuid::new_v4();

    // Insert a transfer
    let transfer_id = insert_proximity_transfer(
        &pool,
        sender_id,
        "SenderWallet123",
        recipient_id,
        "RecipientWallet456",
        "SOL",
        Decimal::new(150, 2), // 1.50 SOL
        "Pending",
        "WiFi",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    // Retrieve the transfer
    let transfer = get_transfer_by_id(&pool, transfer_id).await.unwrap();
    assert!(transfer.is_some());

    let transfer = transfer.unwrap();
    assert_eq!(transfer.sender_user_id, sender_id);
    assert_eq!(transfer.recipient_user_id, recipient_id);
    assert_eq!(transfer.asset, "SOL");
    assert_eq!(transfer.amount, Decimal::new(150, 2));
    assert_eq!(transfer.status, "Pending");
    assert_eq!(transfer.discovery_method, "WiFi");
}

#[tokio::test]
#[ignore]
async fn test_update_transfer_status() {
    let pool = setup_test_db().await;

    let sender_id = Uuid::new_v4();
    let recipient_id = Uuid::new_v4();

    // Insert a transfer
    let transfer_id = insert_proximity_transfer(
        &pool,
        sender_id,
        "SenderWallet123",
        recipient_id,
        "RecipientWallet456",
        "SOL",
        Decimal::new(100, 2),
        "Pending",
        "Bluetooth",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    // Update to Accepted
    update_transfer_status(&pool, transfer_id, "Accepted", None, None)
        .await
        .unwrap();

    let transfer = get_transfer_by_id(&pool, transfer_id).await.unwrap().unwrap();
    assert_eq!(transfer.status, "Accepted");
    assert!(transfer.accepted_at.is_some());

    // Update to Completed with transaction hash
    update_transfer_status(
        &pool,
        transfer_id,
        "Completed",
        Some("tx_hash_12345"),
        None,
    )
    .await
    .unwrap();

    let transfer = get_transfer_by_id(&pool, transfer_id).await.unwrap().unwrap();
    assert_eq!(transfer.status, "Completed");
    assert_eq!(transfer.transaction_hash, Some("tx_hash_12345".to_string()));
    assert!(transfer.completed_at.is_some());
}

#[tokio::test]
#[ignore]
async fn test_update_transfer_status_failed() {
    let pool = setup_test_db().await;

    let sender_id = Uuid::new_v4();
    let recipient_id = Uuid::new_v4();

    let transfer_id = insert_proximity_transfer(
        &pool,
        sender_id,
        "SenderWallet123",
        recipient_id,
        "RecipientWallet456",
        "SOL",
        Decimal::new(100, 2),
        "Executing",
        "WiFi",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    // Update to Failed with reason
    update_transfer_status(
        &pool,
        transfer_id,
        "Failed",
        None,
        Some("Insufficient balance"),
    )
    .await
    .unwrap();

    let transfer = get_transfer_by_id(&pool, transfer_id).await.unwrap().unwrap();
    assert_eq!(transfer.status, "Failed");
    assert_eq!(
        transfer.failed_reason,
        Some("Insufficient balance".to_string())
    );
}

#[tokio::test]
#[ignore]
async fn test_get_user_proximity_transfers_with_filters() {
    let pool = setup_test_db().await;

    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();
    let user3_id = Uuid::new_v4();

    // Insert multiple transfers
    insert_proximity_transfer(
        &pool,
        user1_id,
        "Wallet1",
        user2_id,
        "Wallet2",
        "SOL",
        Decimal::new(100, 2),
        "Completed",
        "WiFi",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    insert_proximity_transfer(
        &pool,
        user1_id,
        "Wallet1",
        user3_id,
        "Wallet3",
        "USDC",
        Decimal::new(5000, 2),
        "Pending",
        "Bluetooth",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    insert_proximity_transfer(
        &pool,
        user2_id,
        "Wallet2",
        user1_id,
        "Wallet1",
        "SOL",
        Decimal::new(200, 2),
        "Completed",
        "WiFi",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    // Filter by user
    let filter = TransferFilter {
        user_id: Some(user1_id),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 3); // user1 is sender or recipient in 3 transfers

    // Filter by status
    let filter = TransferFilter {
        user_id: Some(user1_id),
        status: Some("Completed".to_string()),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 2);

    // Filter by asset
    let filter = TransferFilter {
        user_id: Some(user1_id),
        asset: Some("USDC".to_string()),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].asset, "USDC");

    // Filter with limit
    let filter = TransferFilter {
        user_id: Some(user1_id),
        limit: Some(2),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_discovery_session_lifecycle() {
    let pool = setup_test_db().await;

    let user_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::minutes(30);

    // Insert a session
    let session_id = insert_discovery_session(&pool, user_id, "WiFi", expires_at, false)
        .await
        .unwrap();

    assert!(!session_id.is_nil());

    // Extend the session
    let new_expires_at = expires_at + Duration::minutes(15);
    update_session_expiration(&pool, session_id, new_expires_at)
        .await
        .unwrap();

    // End the session
    end_discovery_session(&pool, session_id).await.unwrap();

    // Verify we can't extend an ended session (should not error, just no-op)
    let result = update_session_expiration(&pool, session_id, Utc::now() + Duration::minutes(30))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_peer_blocklist() {
    let pool = setup_test_db().await;

    let user_id = Uuid::new_v4();
    let blocked_user1 = Uuid::new_v4();
    let blocked_user2 = Uuid::new_v4();

    // Initially no blocked peers
    let blocked = get_blocked_peers(&pool, user_id).await.unwrap();
    assert_eq!(blocked.len(), 0);

    // Add blocked peers
    add_blocked_peer(&pool, user_id, blocked_user1)
        .await
        .unwrap();
    add_blocked_peer(&pool, user_id, blocked_user2)
        .await
        .unwrap();

    // Verify blocked peers
    let blocked = get_blocked_peers(&pool, user_id).await.unwrap();
    assert_eq!(blocked.len(), 2);

    // Adding the same peer again should not error (ON CONFLICT DO NOTHING)
    add_blocked_peer(&pool, user_id, blocked_user1)
        .await
        .unwrap();
    let blocked = get_blocked_peers(&pool, user_id).await.unwrap();
    assert_eq!(blocked.len(), 2);

    // Remove a blocked peer
    remove_blocked_peer(&pool, user_id, blocked_user1)
        .await
        .unwrap();
    let blocked = get_blocked_peers(&pool, user_id).await.unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].blocked_user_id, blocked_user2);

    // Remove the other blocked peer
    remove_blocked_peer(&pool, user_id, blocked_user2)
        .await
        .unwrap();
    let blocked = get_blocked_peers(&pool, user_id).await.unwrap();
    assert_eq!(blocked.len(), 0);
}

#[tokio::test]
#[ignore]
async fn test_transfer_filter_date_range() {
    let pool = setup_test_db().await;

    let user_id = Uuid::new_v4();
    let other_user = Uuid::new_v4();

    // Insert transfers
    insert_proximity_transfer(
        &pool,
        user_id,
        "Wallet1",
        other_user,
        "Wallet2",
        "SOL",
        Decimal::new(100, 2),
        "Completed",
        "WiFi",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    // Wait a bit to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let middle_time = Utc::now();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    insert_proximity_transfer(
        &pool,
        user_id,
        "Wallet1",
        other_user,
        "Wallet2",
        "SOL",
        Decimal::new(200, 2),
        "Completed",
        "WiFi",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    // Filter by date range - should get only the second transfer
    let filter = TransferFilter {
        user_id: Some(user_id),
        from_date: Some(middle_time),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].amount, Decimal::new(200, 2));
}

#[tokio::test]
#[ignore]
async fn test_transfer_filter_by_transaction_type() {
    let pool = setup_test_db().await;

    let user_id = Uuid::new_v4();
    let other_user = Uuid::new_v4();

    // Insert a direct transfer
    insert_proximity_transfer(
        &pool,
        user_id,
        "Wallet1",
        other_user,
        "Wallet2",
        "SOL",
        Decimal::new(100, 2),
        "Completed",
        "WiFi",
        "DIRECT_TRANSFER",
    )
    .await
    .unwrap();

    // Insert a P2P exchange
    insert_proximity_transfer(
        &pool,
        user_id,
        "Wallet1",
        other_user,
        "Wallet2",
        "USDC",
        Decimal::new(500, 2),
        "Completed",
        "WiFi",
        "P2P_EXCHANGE",
    )
    .await
    .unwrap();

    // Filter by DIRECT_TRANSFER
    let filter = TransferFilter {
        user_id: Some(user_id),
        transaction_type: Some("DIRECT_TRANSFER".to_string()),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].transaction_type, "DIRECT_TRANSFER");
    assert_eq!(transfers[0].asset, "SOL");

    // Filter by P2P_EXCHANGE
    let filter = TransferFilter {
        user_id: Some(user_id),
        transaction_type: Some("P2P_EXCHANGE".to_string()),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].transaction_type, "P2P_EXCHANGE");
    assert_eq!(transfers[0].asset, "USDC");

    // No filter - should get both
    let filter = TransferFilter {
        user_id: Some(user_id),
        ..Default::default()
    };
    let transfers = get_user_proximity_transfers(&pool, filter).await.unwrap();
    assert_eq!(transfers.len(), 2);
}
