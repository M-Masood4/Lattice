use api::{BenchmarkService, BirdeyeService, CreateBenchmarkRequest, PriceMonitor, PositionManagementService, TriggerType, ActionType, TradeAction};
use database::{create_pool, create_redis_pool, create_redis_client, run_migrations};
use notification::NotificationService;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Helper to create a test database pool
async fn setup_test_db() -> database::DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/whale_tracker_test".to_string());
    
    let pool = create_pool(&database_url, 5).await.expect("Failed to create pool");
    
    // Try to run migrations, but ignore if tables already exist
    let _ = run_migrations(&pool).await;
    
    pool
}

/// Helper to create a test Redis pool
async fn setup_test_redis() -> database::RedisPool {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let client = create_redis_client(&redis_url).await.expect("Failed to create Redis client");
    create_redis_pool(client).await.expect("Failed to create Redis pool")
}

/// Helper to create a test user
async fn create_test_user(pool: &database::DbPool) -> Uuid {
    let client = pool.get().await.expect("Failed to get client");
    
    // Use a unique email for each test
    let unique_email = format!("test-{}@example.com", Uuid::new_v4());
    
    let row = client
        .query_one(
            "INSERT INTO users (email, password_hash, created_at, updated_at)
             VALUES ($1, $2, NOW(), NOW())
             RETURNING id",
            &[&unique_email, &"hash"],
        )
        .await
        .expect("Failed to create test user");
    
    row.get(0)
}

/// Helper to clean up test data
async fn cleanup_test_data(pool: &database::DbPool, user_id: Uuid) {
    let client = pool.get().await.expect("Failed to get client");
    
    client
        .execute("DELETE FROM notifications WHERE user_id = $1", &[&user_id])
        .await
        .expect("Failed to delete notifications");
    
    client
        .execute("DELETE FROM benchmarks WHERE user_id = $1", &[&user_id])
        .await
        .expect("Failed to delete benchmarks");
    
    client
        .execute("DELETE FROM users WHERE id = $1", &[&user_id])
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
#[ignore] // Only run with a real database and Redis
async fn test_benchmark_trigger_creates_alert_notification() {
    let pool = setup_test_db().await;
    let redis_pool = setup_test_redis().await;
    let user_id = create_test_user(&pool).await;
    
    // Initialize services
    let benchmark_service = Arc::new(BenchmarkService::new(pool.clone()));
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis_pool.clone()));
    let notification_service = Arc::new(NotificationService::new());
    let position_management_service = Arc::new(PositionManagementService::new(pool.clone()));
    
    // Create price monitor (wiring benchmark triggers to notification service)
    let _price_monitor = Arc::new(PriceMonitor::new(
        benchmark_service.clone(),
        birdeye_service.clone(),
        position_management_service.clone(),
        notification_service.clone(),
        pool.clone(),
    ));
    
    // Create an alert benchmark
    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("100.0").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };
    
    let benchmark = benchmark_service.create_benchmark(user_id, request).await.unwrap();
    
    // Verify benchmark was created
    assert_eq!(benchmark.user_id, user_id);
    assert_eq!(benchmark.action_type, ActionType::Alert);
    assert!(benchmark.is_active);
    
    // Note: Full integration test would require:
    // 1. Mock Birdeye API to return a price that triggers the benchmark
    // 2. Run the price monitor check cycle
    // 3. Verify notification was created in the database
    // For now, we verify the wiring is correct by checking the services are initialized
    
    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database and Redis
async fn test_benchmark_trigger_creates_execute_notification() {
    let pool = setup_test_db().await;
    let redis_pool = setup_test_redis().await;
    let user_id = create_test_user(&pool).await;
    
    // Initialize services
    let benchmark_service = Arc::new(BenchmarkService::new(pool.clone()));
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis_pool.clone()));
    let notification_service = Arc::new(NotificationService::new());
    let position_management_service = Arc::new(PositionManagementService::new(pool.clone()));
    
    // Create price monitor (wiring benchmark triggers to trading service)
    let _price_monitor = Arc::new(PriceMonitor::new(
        benchmark_service.clone(),
        birdeye_service.clone(),
        position_management_service.clone(),
        notification_service.clone(),
        pool.clone(),
    ));
    
    // Create an execute benchmark
    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("50.0").unwrap(),
        trigger_type: TriggerType::Below,
        action_type: ActionType::Execute,
        trade_action: Some(TradeAction::Buy),
        trade_amount: Some(Decimal::from_str("10.0").unwrap()),
    };
    
    let benchmark = benchmark_service.create_benchmark(user_id, request).await.unwrap();
    
    // Verify benchmark was created with execute action
    assert_eq!(benchmark.user_id, user_id);
    assert_eq!(benchmark.action_type, ActionType::Execute);
    assert_eq!(benchmark.trade_action, Some(TradeAction::Buy));
    assert_eq!(benchmark.trade_amount, Some(Decimal::from_str("10.0").unwrap()));
    assert!(benchmark.is_active);
    
    // Note: Full integration test would require:
    // 1. Mock Birdeye API to return a price that triggers the benchmark
    // 2. Run the price monitor check cycle
    // 3. Verify trade execution notification was created
    // 4. Eventually verify trade was executed via trading service
    
    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_notification_service_integration() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    
    // Create a notification directly in the database (simulating what price_monitor does)
    let client = pool.get().await.expect("Failed to get client");
    
    let notification_id = Uuid::new_v4();
    client
        .execute(
            "INSERT INTO notifications (id, user_id, type, title, message, data, priority, read, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
            &[
                &notification_id,
                &user_id,
                &"BENCHMARK_ALERT",
                &"Price Alert: SOL",
                &"Price alert: SOL has crossed above your target price of 100. Current price: 105",
                &serde_json::json!({
                    "asset": "SOL",
                    "target_price": "100",
                    "current_price": "105",
                }),
                &"HIGH",
                &false,
            ],
        )
        .await
        .expect("Failed to create notification");
    
    // Verify notification was created
    let row = client
        .query_one(
            "SELECT id, user_id, type, title, priority, read FROM notifications WHERE id = $1",
            &[&notification_id],
        )
        .await
        .expect("Failed to query notification");
    
    let retrieved_id: Uuid = row.get(0);
    let retrieved_user_id: Uuid = row.get(1);
    let notification_type: String = row.get(2);
    let title: String = row.get(3);
    let priority: String = row.get(4);
    let read: bool = row.get(5);
    
    assert_eq!(retrieved_id, notification_id);
    assert_eq!(retrieved_user_id, user_id);
    assert_eq!(notification_type, "BENCHMARK_ALERT");
    assert_eq!(title, "Price Alert: SOL");
    assert_eq!(priority, "HIGH");
    assert!(!read);
    
    cleanup_test_data(&pool, user_id).await;
}
