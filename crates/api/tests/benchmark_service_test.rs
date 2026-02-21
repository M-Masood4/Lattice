use api::{BenchmarkService, CreateBenchmarkRequest, UpdateBenchmarkRequest, TriggerType, ActionType, TradeAction};
use database::{create_pool, run_migrations};
use rust_decimal::Decimal;
use std::str::FromStr;
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
        .execute("DELETE FROM benchmarks WHERE user_id = $1", &[&user_id])
        .await
        .expect("Failed to delete benchmarks");
    
    client
        .execute("DELETE FROM users WHERE id = $1", &[&user_id])
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_create_benchmark_with_positive_price() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("100.50").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };

    let result = service.create_benchmark(user_id, request).await;
    assert!(result.is_ok());

    let benchmark = result.unwrap();
    assert_eq!(benchmark.user_id, user_id);
    assert_eq!(benchmark.asset, "SOL");
    assert_eq!(benchmark.target_price, Decimal::from_str("100.50").unwrap());
    assert_eq!(benchmark.trigger_type, TriggerType::Above);
    assert_eq!(benchmark.action_type, ActionType::Alert);
    assert!(benchmark.is_active);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_create_benchmark_rejects_zero_price() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::ZERO,
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };

    let result = service.create_benchmark(user_id, request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("positive"));

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_create_benchmark_rejects_negative_price() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("-10.0").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };

    let result = service.create_benchmark(user_id, request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("positive"));

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_create_execute_benchmark_with_trade_details() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("100.0").unwrap(),
        trigger_type: TriggerType::Below,
        action_type: ActionType::Execute,
        trade_action: Some(TradeAction::Buy),
        trade_amount: Some(Decimal::from_str("10.0").unwrap()),
    };

    let result = service.create_benchmark(user_id, request).await;
    assert!(result.is_ok());

    let benchmark = result.unwrap();
    assert_eq!(benchmark.action_type, ActionType::Execute);
    assert_eq!(benchmark.trade_action, Some(TradeAction::Buy));
    assert_eq!(benchmark.trade_amount, Some(Decimal::from_str("10.0").unwrap()));

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_create_execute_benchmark_requires_trade_action() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("100.0").unwrap(),
        trigger_type: TriggerType::Below,
        action_type: ActionType::Execute,
        trade_action: None, // Missing trade action
        trade_amount: Some(Decimal::from_str("10.0").unwrap()),
    };

    let result = service.create_benchmark(user_id, request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("trade_action"));

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_get_benchmark() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    // Create a benchmark
    let request = CreateBenchmarkRequest {
        asset: "ETH".to_string(),
        blockchain: "Ethereum".to_string(),
        target_price: Decimal::from_str("2000.0").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };

    let created = service.create_benchmark(user_id, request).await.unwrap();

    // Retrieve it
    let retrieved = service.get_benchmark(created.id, user_id).await.unwrap();

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.asset, "ETH");
    assert_eq!(retrieved.target_price, Decimal::from_str("2000.0").unwrap());

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_get_user_benchmarks() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    // Create multiple benchmarks
    for i in 1..=3 {
        let request = CreateBenchmarkRequest {
            asset: format!("TOKEN{}", i),
            blockchain: "Solana".to_string(),
            target_price: Decimal::from_str(&format!("{}.0", i * 100)).unwrap(),
            trigger_type: TriggerType::Above,
            action_type: ActionType::Alert,
            trade_action: None,
            trade_amount: None,
        };
        service.create_benchmark(user_id, request).await.unwrap();
    }

    // Retrieve all
    let benchmarks = service.get_user_benchmarks(user_id).await.unwrap();
    assert_eq!(benchmarks.len(), 3);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_get_active_benchmarks_for_asset() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    // Use a unique asset name to avoid conflicts with other tests
    let unique_asset = format!("SOL-{}", Uuid::new_v4());

    // Create benchmarks for unique asset
    for i in 1..=2 {
        let request = CreateBenchmarkRequest {
            asset: unique_asset.clone(),
            blockchain: "Solana".to_string(),
            target_price: Decimal::from_str(&format!("{}.0", i * 50)).unwrap(),
            trigger_type: TriggerType::Above,
            action_type: ActionType::Alert,
            trade_action: None,
            trade_amount: None,
        };
        service.create_benchmark(user_id, request).await.unwrap();
    }

    // Create benchmark for different asset (should not be included)
    let request = CreateBenchmarkRequest {
        asset: format!("ETH-{}", Uuid::new_v4()),
        blockchain: "Ethereum".to_string(),
        target_price: Decimal::from_str("2000.0").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };
    service.create_benchmark(user_id, request).await.unwrap();

    // Retrieve benchmarks for unique asset
    let sol_benchmarks = service.get_active_benchmarks_for_asset(&unique_asset).await.unwrap();
    assert_eq!(sol_benchmarks.len(), 2);
    assert!(sol_benchmarks.iter().all(|b| b.asset == unique_asset));

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_update_benchmark() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    // Create a benchmark
    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("100.0").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };

    let created = service.create_benchmark(user_id, request).await.unwrap();

    // Update it
    let update_request = UpdateBenchmarkRequest {
        target_price: Some(Decimal::from_str("150.0").unwrap()),
        trigger_type: None,
        action_type: None,
        trade_action: None,
        trade_amount: None,
        is_active: Some(false),
    };

    let updated = service.update_benchmark(created.id, user_id, update_request).await.unwrap();

    assert_eq!(updated.target_price, Decimal::from_str("150.0").unwrap());
    assert!(!updated.is_active);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_delete_benchmark() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    // Create a benchmark
    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("100.0").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };

    let created = service.create_benchmark(user_id, request).await.unwrap();

    // Delete it
    let result = service.delete_benchmark(created.id, user_id).await;
    assert!(result.is_ok());

    // Verify it's gone
    let get_result = service.get_benchmark(created.id, user_id).await;
    assert!(get_result.is_err());

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_mark_triggered() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    // Create a benchmark
    let request = CreateBenchmarkRequest {
        asset: "SOL".to_string(),
        blockchain: "Solana".to_string(),
        target_price: Decimal::from_str("100.0").unwrap(),
        trigger_type: TriggerType::Above,
        action_type: ActionType::Alert,
        trade_action: None,
        trade_amount: None,
    };

    let created = service.create_benchmark(user_id, request).await.unwrap();
    assert!(created.triggered_at.is_none());

    // Mark as triggered
    service.mark_triggered(created.id, true).await.unwrap();

    // Verify it's marked
    let updated = service.get_benchmark(created.id, user_id).await.unwrap();
    assert!(updated.triggered_at.is_some());
    assert!(!updated.is_active); // Should be disabled

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
#[ignore] // Only run with a real database
async fn test_multiple_benchmarks_per_asset() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool).await;
    let service = BenchmarkService::new(pool.clone());

    // Use a unique asset name to avoid conflicts with other tests
    let unique_asset = format!("SOL-{}", Uuid::new_v4());

    // Create multiple benchmarks for the same asset
    let prices = vec!["50.0", "100.0", "150.0"];
    
    for price in prices {
        let request = CreateBenchmarkRequest {
            asset: unique_asset.clone(),
            blockchain: "Solana".to_string(),
            target_price: Decimal::from_str(price).unwrap(),
            trigger_type: TriggerType::Above,
            action_type: ActionType::Alert,
            trade_action: None,
            trade_amount: None,
        };
        service.create_benchmark(user_id, request).await.unwrap();
    }

    // Verify all were created
    let benchmarks = service.get_active_benchmarks_for_asset(&unique_asset).await.unwrap();
    assert_eq!(benchmarks.len(), 3);

    cleanup_test_data(&pool, user_id).await;
}
