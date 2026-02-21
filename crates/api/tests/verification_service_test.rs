use api::{VerificationService, VerificationLevel};
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
async fn test_verify_wallet() {
    let db = setup_test_db().await;
    let service = VerificationService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    let result = service
        .verify_wallet(
            user_id,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0".to_string(),
            "Ethereum".to_string(),
            "Sign this message to verify ownership".to_string(),
            "0xabcdef...".to_string(),
        )
        .await;
    
    assert!(result.is_ok(), "Failed to verify wallet");
    let verification = result.unwrap();
    
    assert_eq!(verification.user_id, user_id);
    assert!(verification.verified);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_is_wallet_verified() {
    let db = setup_test_db().await;
    let service = VerificationService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    let wallet_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
    
    // Initially not verified
    let is_verified = service
        .is_wallet_verified(user_id, wallet_address, "Ethereum")
        .await
        .expect("Failed to check verification");
    
    assert!(!is_verified);
    
    // Verify the wallet
    service
        .verify_wallet(
            user_id,
            wallet_address.to_string(),
            "Ethereum".to_string(),
            "Challenge".to_string(),
            "Signature".to_string(),
        )
        .await
        .expect("Failed to verify wallet");
    
    // Now should be verified
    let is_verified = service
        .is_wallet_verified(user_id, wallet_address, "Ethereum")
        .await
        .expect("Failed to check verification");
    
    assert!(is_verified);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_verification_level() {
    let db = setup_test_db().await;
    let service = VerificationService::new(db.clone());
    let user_id = create_test_user(&db).await;
    
    // Initially no verification
    let level = service
        .get_verification_level(user_id)
        .await
        .expect("Failed to get verification level");
    
    assert_eq!(level as i32, VerificationLevel::None as i32);
}

#[test]
fn test_verification_level_enum() {
    assert_eq!(VerificationLevel::None as i32, 0);
    assert_eq!(VerificationLevel::Basic as i32, 1);
    assert_eq!(VerificationLevel::Advanced as i32, 2);
}
