use api::{ChatService, P2PService, OfferType, OfferStatus, ReceiptService};
use blockchain::MultiChainClient;
use database::{create_pool, run_migrations};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_db() -> database::DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/whale_tracker_test".to_string());
    
    let pool = create_pool(&database_url, 5).await.expect("Failed to create pool");
    let _ = run_migrations(&pool).await;
    
    pool
}

async fn create_test_services() -> (P2PService, ChatService, database::DbPool) {
    let db_pool = setup_test_db().await;
    let blockchain_client = Arc::new(MultiChainClient::new());
    let receipt_service = Arc::new(ReceiptService::new(db_pool.clone(), blockchain_client));
    let chat_service = ChatService::new(db_pool.clone(), receipt_service);
    let p2p_service = P2PService::new(db_pool.clone());
    
    (p2p_service, chat_service, db_pool)
}

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

/// Task 15.1: Test complete offer acceptance flow
/// Requirements: 1.2, 1.3, 1.4, 1.5, 3.1, 3.4
#[tokio::test]
#[ignore] // Requires database
async fn test_complete_offer_acceptance_flow() {
    let (p2p_service, chat_service, db) = create_test_services().await;
    
    // Create two test users
    let creator_id = create_test_user(&db).await;
    let acceptor_id = create_test_user(&db).await;
    
    // Step 1: Create offer via API (simulated)
    let offer = p2p_service
        .create_offer(
            creator_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(10, 0),
            Decimal::new(1000, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create offer");
    
    assert_eq!(offer.status, OfferStatus::Active);
    assert_eq!(offer.acceptor_id, None);
    assert_eq!(offer.accepted_at, None);
    assert_eq!(offer.conversation_id, None);
    
    // Step 2: Accept offer via API (simulated)
    let accepted_offer = p2p_service
        .accept_offer(offer.id, acceptor_id, &chat_service)
        .await
        .expect("Failed to accept offer");
    
    // Step 3: Verify offer status updated (Requirement 1.2)
    assert_eq!(accepted_offer.status, OfferStatus::Matched);
    
    // Step 4: Verify acceptor_id recorded (Requirement 1.3)
    assert_eq!(accepted_offer.acceptor_id, Some(acceptor_id));
    
    // Step 5: Verify accepted_at timestamp recorded (Requirement 1.5)
    assert!(accepted_offer.accepted_at.is_some());
    
    // Step 6: Verify exchange record created (Requirement 1.4)
    let client = db.get().await.expect("Failed to get db connection");
    let exchange_rows = client
        .query(
            "SELECT buyer_id, seller_id, offer_id FROM p2p_exchanges WHERE offer_id = $1",
            &[&offer.id],
        )
        .await
        .expect("Failed to query exchange records");
    
    assert_eq!(exchange_rows.len(), 1, "Exchange record should be created");
    let buyer_id: Uuid = exchange_rows[0].get("buyer_id");
    let seller_id: Uuid = exchange_rows[0].get("seller_id");
    let exchange_offer_id: Uuid = exchange_rows[0].get("offer_id");
    
    // For a SELL offer, creator is seller and acceptor is buyer
    assert_eq!(seller_id, creator_id);
    assert_eq!(buyer_id, acceptor_id);
    assert_eq!(exchange_offer_id, offer.id);
    
    // Step 7: Verify chat conversation created (Requirement 3.1)
    assert!(accepted_offer.conversation_id.is_some(), "Conversation should be created");
    let conversation_id = accepted_offer.conversation_id.unwrap();
    
    let conversation_rows = client
        .query(
            "SELECT id, offer_id FROM chat_conversations WHERE id = $1",
            &[&conversation_id],
        )
        .await
        .expect("Failed to query conversation");
    
    assert_eq!(conversation_rows.len(), 1, "Conversation should exist");
    let stored_offer_id: Uuid = conversation_rows[0].get("offer_id");
    assert_eq!(stored_offer_id, offer.id);
    
    // Verify both participants added to conversation
    let participant_rows = client
        .query(
            "SELECT user_id FROM chat_participants WHERE conversation_id = $1 ORDER BY user_id",
            &[&conversation_id],
        )
        .await
        .expect("Failed to query participants");
    
    assert_eq!(participant_rows.len(), 2, "Both users should be participants");
    let participant_ids: Vec<Uuid> = participant_rows.iter().map(|r| r.get("user_id")).collect();
    assert!(participant_ids.contains(&creator_id));
    assert!(participant_ids.contains(&acceptor_id));
    
    // Step 8: Verify notification sent (Requirement 3.4)
    let notification_rows = client
        .query(
            "SELECT content FROM chat_messages WHERE from_user_id = $1 AND to_user_id = $2",
            &[&acceptor_id, &creator_id],
        )
        .await
        .expect("Failed to query notifications");
    
    assert!(notification_rows.len() > 0, "Notification should be sent");
    let notification_content: String = notification_rows[0].get("content");
    assert!(notification_content.contains("accepted"), "Notification should mention acceptance");
}

/// Task 15.1: Test offer acceptance with BUY offer type
#[tokio::test]
#[ignore] // Requires database
async fn test_buy_offer_acceptance_flow() {
    let (p2p_service, chat_service, db) = create_test_services().await;
    
    let creator_id = create_test_user(&db).await;
    let acceptor_id = create_test_user(&db).await;
    
    // Create a BUY offer
    let offer = p2p_service
        .create_offer(
            creator_id,
            OfferType::Buy,
            "USDC".to_string(),
            "SOL".to_string(),
            Decimal::new(1000, 0),
            Decimal::new(10, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create buy offer");
    
    // Accept the offer
    let accepted_offer = p2p_service
        .accept_offer(offer.id, acceptor_id, &chat_service)
        .await
        .expect("Failed to accept buy offer");
    
    assert_eq!(accepted_offer.status, OfferStatus::Matched);
    
    // Verify exchange record has correct buyer/seller
    let client = db.get().await.expect("Failed to get db connection");
    let exchange_rows = client
        .query(
            "SELECT buyer_id, seller_id FROM p2p_exchanges WHERE offer_id = $1",
            &[&offer.id],
        )
        .await
        .expect("Failed to query exchange records");
    
    assert_eq!(exchange_rows.len(), 1);
    let buyer_id: Uuid = exchange_rows[0].get("buyer_id");
    let seller_id: Uuid = exchange_rows[0].get("seller_id");
    
    // For a BUY offer, creator is buyer and acceptor is seller
    assert_eq!(buyer_id, creator_id);
    assert_eq!(seller_id, acceptor_id);
}

/// Task 15.1: Test proximity offer acceptance
#[tokio::test]
#[ignore] // Requires database
async fn test_proximity_offer_acceptance() {
    let (p2p_service, chat_service, _db) = create_test_services().await;
    
    let creator_id = create_test_user(&_db).await;
    let acceptor_id = create_test_user(&_db).await;
    
    // Create a proximity offer
    let offer = p2p_service
        .create_offer(
            creator_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(5, 0),
            Decimal::new(500, 0),
            Decimal::new(100, 0),
            true, // proximity offer
        )
        .await
        .expect("Failed to create proximity offer");
    
    assert!(offer.is_proximity_offer);
    
    // Accept the proximity offer
    let accepted_offer = p2p_service
        .accept_offer(offer.id, acceptor_id, &chat_service)
        .await
        .expect("Failed to accept proximity offer");
    
    assert_eq!(accepted_offer.status, OfferStatus::Matched);
    assert_eq!(accepted_offer.acceptor_id, Some(acceptor_id));
    assert!(accepted_offer.conversation_id.is_some());
}

/// Task 15.1: Test get_offer returns complete information
#[tokio::test]
#[ignore] // Requires database
async fn test_get_offer_after_acceptance() {
    let (p2p_service, chat_service, _db) = create_test_services().await;
    
    let creator_id = create_test_user(&_db).await;
    let acceptor_id = create_test_user(&_db).await;
    
    // Create and accept offer
    let offer = p2p_service
        .create_offer(
            creator_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(10, 0),
            Decimal::new(1000, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create offer");
    
    p2p_service
        .accept_offer(offer.id, acceptor_id, &chat_service)
        .await
        .expect("Failed to accept offer");
    
    // Retrieve the offer
    let retrieved_offer = p2p_service
        .get_offer(offer.id)
        .await
        .expect("Failed to get offer");
    
    // Verify all fields are present
    assert_eq!(retrieved_offer.status, OfferStatus::Matched);
    assert_eq!(retrieved_offer.acceptor_id, Some(acceptor_id));
    assert!(retrieved_offer.accepted_at.is_some());
    assert!(retrieved_offer.conversation_id.is_some());
}

/// Task 15.1: Test marketplace excludes accepted offers
#[tokio::test]
#[ignore] // Requires database
async fn test_marketplace_excludes_accepted_offers() {
    let (p2p_service, chat_service, _db) = create_test_services().await;
    
    let creator_id = create_test_user(&_db).await;
    let acceptor_id = create_test_user(&_db).await;
    let viewer_id = create_test_user(&_db).await;
    
    // Create two offers
    let offer1 = p2p_service
        .create_offer(
            creator_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(10, 0),
            Decimal::new(1000, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create offer 1");
    
    let offer2 = p2p_service
        .create_offer(
            creator_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(5, 0),
            Decimal::new(500, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create offer 2");
    
    // Accept offer1
    p2p_service
        .accept_offer(offer1.id, acceptor_id, &chat_service)
        .await
        .expect("Failed to accept offer");
    
    // Get marketplace offers for viewer
    let marketplace_offers = p2p_service
        .get_marketplace_offers(viewer_id, None, None, 100)
        .await
        .expect("Failed to get marketplace offers");
    
    // Marketplace should only show offer2 (active), not offer1 (matched)
    let offer_ids: Vec<Uuid> = marketplace_offers.iter().map(|o| o.id).collect();
    assert!(!offer_ids.contains(&offer1.id), "Accepted offer should not be in marketplace");
    assert!(offer_ids.contains(&offer2.id), "Active offer should be in marketplace");
}
