use api::{P2PService, OfferType};
use database::{create_pool, run_migrations};
use rust_decimal::Decimal;
use uuid::Uuid;

async fn setup_test_db() -> database::DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/whale_tracker_test".to_string());
    
    let pool = create_pool(&database_url, 5).await.expect("Failed to create pool");
    let _ = run_migrations(&pool).await;
    
    pool
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

#[tokio::test]
#[ignore] // Requires database
async fn test_create_p2p_offer() {
    let db = setup_test_db().await;
    let service = P2PService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    let result = service
        .create_offer(
            user_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(10, 0),
            Decimal::new(1000, 0),
            Decimal::new(100, 0),
            false,
        )
        .await;
    
    assert!(result.is_ok(), "Failed to create offer");
    let offer = result.unwrap();
    
    assert_eq!(offer.user_id, user_id);
    assert_eq!(offer.from_asset, "SOL");
    assert_eq!(offer.to_asset, "USDC");
    assert_eq!(offer.is_proximity_offer, false);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_active_offers() {
    let db = setup_test_db().await;
    let service = P2PService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    // Create an offer
    service
        .create_offer(
            user_id,
            OfferType::Buy,
            "USDC".to_string(),
            "SOL".to_string(),
            Decimal::new(1000, 0),
            Decimal::new(10, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create offer");
    
    // Get active offers
    let offers = service
        .get_active_offers(Some("USDC".to_string()), Some("SOL".to_string()), 10)
        .await;
    
    assert!(offers.is_ok());
    let offers = offers.unwrap();
    assert!(offers.len() > 0);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_cancel_offer() {
    let db = setup_test_db().await;
    let service = P2PService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    let offer = service
        .create_offer(
            user_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(5, 0),
            Decimal::new(500, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create offer");
    
    // Cancel the offer
    let result = service.cancel_offer(offer.id, user_id).await;
    assert!(result.is_ok());
}

#[test]
fn test_offer_type_serialization() {
    assert_eq!(OfferType::Buy.as_str(), "BUY");
    assert_eq!(OfferType::Sell.as_str(), "SELL");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_create_proximity_offer() {
    let db = setup_test_db().await;
    let service = P2PService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    let result = service
        .create_offer(
            user_id,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(10, 0),
            Decimal::new(1000, 0),
            Decimal::new(100, 0),
            true,
        )
        .await;
    
    assert!(result.is_ok(), "Failed to create proximity offer");
    let offer = result.unwrap();
    
    assert_eq!(offer.is_proximity_offer, true);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_proximity_offers() {
    let db = setup_test_db().await;
    let service = P2PService::new(db.clone());
    let user1 = create_test_user(&db).await;
    let user2 = create_test_user(&db).await;
    
    // Create a proximity offer from user1
    service
        .create_offer(
            user1,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(10, 0),
            Decimal::new(1000, 0),
            Decimal::new(100, 0),
            true,
        )
        .await
        .expect("Failed to create proximity offer");
    
    // Create a regular offer from user2
    service
        .create_offer(
            user2,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(5, 0),
            Decimal::new(500, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create regular offer");
    
    // Get proximity offers for discovered peers (only user1)
    let offers = service
        .get_proximity_offers(vec![user1], None, None, 10)
        .await;
    
    assert!(offers.is_ok());
    let offers = offers.unwrap();
    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].user_id, user1);
    assert_eq!(offers[0].is_proximity_offer, true);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_proximity_offer_priority() {
    let db = setup_test_db().await;
    let service = P2PService::new(db.clone());
    let proximity_user = create_test_user(&db).await;
    let regular_user = create_test_user(&db).await;
    
    // Create a regular offer first (older timestamp)
    service
        .create_offer(
            regular_user,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(5, 0),
            Decimal::new(500, 0),
            Decimal::new(100, 0),
            false,
        )
        .await
        .expect("Failed to create regular offer");
    
    // Wait a bit to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Create a proximity offer second (newer timestamp)
    service
        .create_offer(
            proximity_user,
            OfferType::Sell,
            "SOL".to_string(),
            "USDC".to_string(),
            Decimal::new(10, 0),
            Decimal::new(1000, 0),
            Decimal::new(100, 0),
            true,
        )
        .await
        .expect("Failed to create proximity offer");
    
    // Get offers with proximity priority
    let offers = service
        .get_offers_with_proximity_priority(
            vec![proximity_user],
            Some("SOL".to_string()),
            Some("USDC".to_string()),
            10,
        )
        .await;
    
    assert!(offers.is_ok());
    let offers = offers.unwrap();
    assert!(offers.len() >= 2);
    
    // Proximity offer should be first despite being newer
    assert_eq!(offers[0].user_id, proximity_user);
    assert_eq!(offers[0].is_proximity_offer, true);
    
    // Regular offer should be second
    assert_eq!(offers[1].user_id, regular_user);
    assert_eq!(offers[1].is_proximity_offer, false);
}
