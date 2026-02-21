use api::PrivacyService;
use database::{create_pool, run_migrations};
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
async fn test_create_temporary_wallet() {
    let db = setup_test_db().await;
    let service = PrivacyService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    let result = service
        .create_temporary_wallet(
            user_id,
            "Ethereum".to_string(),
            Some("Trading Wallet".to_string()),
            24,
        )
        .await;
    
    assert!(result.is_ok(), "Failed to create temporary wallet");
    let wallet = result.unwrap();
    
    assert_eq!(wallet.user_id, user_id);
    assert_eq!(wallet.blockchain, "Ethereum");
    assert!(wallet.expires_at.is_some());
}

#[tokio::test]
#[ignore] // Requires database
async fn test_wallet_limit_enforcement() {
    let db = setup_test_db().await;
    let service = PrivacyService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    // Create 10 temporary wallets (the limit)
    for i in 0..10 {
        let result = service
            .create_temporary_wallet(
                user_id,
                "Ethereum".to_string(),
                Some(format!("Wallet {}", i)),
                24,
            )
            .await;
        assert!(result.is_ok(), "Failed to create wallet {}", i);
    }
    
    // 11th wallet should fail
    let result = service
        .create_temporary_wallet(
            user_id,
            "Ethereum".to_string(),
            Some("Wallet 11".to_string()),
            24,
        )
        .await;
    
    assert!(result.is_err(), "Should have failed due to wallet limit");
}

#[tokio::test]
#[ignore] // Requires database
async fn test_freeze_and_unfreeze_wallet() {
    let db = setup_test_db().await;
    let service = PrivacyService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    let wallet = service
        .create_temporary_wallet(
            user_id,
            "Solana".to_string(),
            None,
            24,
        )
        .await
        .expect("Failed to create wallet");
    
    // Freeze the wallet
    let result = service.freeze_wallet(wallet.id, user_id).await;
    assert!(result.is_ok(), "Failed to freeze wallet");
    
    // Check if frozen
    let is_frozen = service
        .is_wallet_frozen(wallet.id)
        .await
        .expect("Failed to check frozen status");
    assert!(is_frozen);
    
    // Unfreeze the wallet
    let result = service.unfreeze_wallet(wallet.id, user_id).await;
    assert!(result.is_ok(), "Failed to unfreeze wallet");
    
    // Check if unfrozen
    let is_frozen = service
        .is_wallet_frozen(wallet.id)
        .await
        .expect("Failed to check frozen status");
    assert!(!is_frozen);
}

#[test]
fn test_generate_user_tag() {
    let tag = PrivacyService::generate_user_tag();
    
    assert!(tag.starts_with("Trader_"));
    assert_eq!(tag.len(), 13); // "Trader_" (7) + 6 random chars
    
    // Generate multiple tags to ensure they're different
    let tag2 = PrivacyService::generate_user_tag();
    // They should be different (with very high probability)
    assert_ne!(tag, tag2);
}

#[test]
fn test_user_tag_format() {
    for _ in 0..10 {
        let tag = PrivacyService::generate_user_tag();
        
        // Check format
        assert!(tag.starts_with("Trader_"));
        
        // Check that random part contains only alphanumeric
        let random_part = &tag[7..];
        assert_eq!(random_part.len(), 6);
        assert!(random_part.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
