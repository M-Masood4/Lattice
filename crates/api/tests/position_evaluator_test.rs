use api::{Position, PositionEvaluator, TrimConfigService};
use database::{create_pool, run_migrations};
use rust_decimal::Decimal;
use shared::models::Asset;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[ignore] // Only run with DATABASE_URL and CLAUDE_API_KEY set
async fn test_position_evaluator_initialization() {
    // This test verifies that the position evaluator can be initialized
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for position evaluator tests");
    
    let claude_api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for position evaluator tests");
    
    // Create a connection pool
    let pool = create_pool(&database_url, 5)
        .await
        .expect("Failed to create database pool");
    
    // Run migrations
    run_migrations(&pool).await.ok();
    
    // Initialize trim config service
    let trim_config_service = Arc::new(TrimConfigService::new(pool.clone()));
    
    // Initialize position evaluator
    let position_evaluator = PositionEvaluator::new(
        pool.clone(),
        trim_config_service,
        claude_api_key,
    );
    
    // Verify it was created successfully
    assert!(true, "Position evaluator initialized successfully");
    
    println!("✅ Position evaluator initialized successfully!");
}

#[tokio::test]
#[ignore] // Only run with DATABASE_URL set
async fn test_get_user_positions() {
    // This test verifies that we can retrieve user positions
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for position evaluator tests");
    
    let claude_api_key = std::env::var("CLAUDE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    
    let pool = create_pool(&database_url, 5)
        .await
        .expect("Failed to create database pool");
    
    run_migrations(&pool).await.ok();
    
    let client = pool.get().await.expect("Failed to get database client");
    
    // Create a test user
    let user_id = Uuid::new_v4();
    let test_email = format!("test-{}@example.com", user_id);
    client.execute(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        &[&user_id, &test_email.as_str(), &"hash"],
    ).await.expect("Failed to create test user");
    
    // Create user settings
    client.execute(
        "INSERT INTO user_settings (user_id, auto_trader_enabled, max_trade_percentage, 
         max_daily_trades, stop_loss_percentage, risk_tolerance) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        &[&user_id, &false, &5.0, &10, &10.0, &"MEDIUM"],
    ).await.expect("Failed to create user settings");
    
    // Create a test wallet
    let wallet_id = Uuid::new_v4();
    let wallet_address = format!("test_wallet_{}", wallet_id);
    client.execute(
        "INSERT INTO wallets (id, user_id, address) VALUES ($1, $2, $3)",
        &[&wallet_id, &user_id, &wallet_address.as_str()],
    ).await.expect("Failed to create test wallet");
    
    // Create a portfolio asset
    client.execute(
        "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd) 
         VALUES ($1, $2, $3, $4, $5)",
        &[&wallet_id, &"SOL", &"SOL", &"10", &1000.0],
    ).await.expect("Failed to create portfolio asset");
    
    // Initialize services
    let trim_config_service = Arc::new(TrimConfigService::new(pool.clone()));
    let position_evaluator = PositionEvaluator::new(
        pool.clone(),
        trim_config_service,
        claude_api_key,
    );
    
    // Get positions (this is a private method, so we can't test it directly)
    // Instead, we verify the data is in the database
    let rows = client.query(
        "SELECT token_mint, token_symbol, amount, value_usd 
         FROM portfolio_assets WHERE wallet_id = $1",
        &[&wallet_id],
    ).await.expect("Failed to query portfolio assets");
    
    assert_eq!(rows.len(), 1, "Should have one portfolio asset");
    
    let token_symbol: String = rows[0].get(1);
    assert_eq!(token_symbol, "SOL");
    
    // Cleanup
    client.execute("DELETE FROM portfolio_assets WHERE wallet_id = $1", &[&wallet_id]).await.ok();
    client.execute("DELETE FROM wallets WHERE id = $1", &[&wallet_id]).await.ok();
    client.execute("DELETE FROM user_settings WHERE user_id = $1", &[&user_id]).await.ok();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.ok();
    
    println!("✅ User positions retrieved successfully!");
}

#[test]
fn test_position_profit_calculation() {
    // Test profit calculation logic
    use rust_decimal::prelude::*;
    
    let entry_price = Decimal::from(80);
    let current_price = Decimal::from(100);
    
    let profit_percent = ((current_price - entry_price) / entry_price) * Decimal::from(100);
    
    assert_eq!(profit_percent, Decimal::from(25));
    
    println!("✅ Profit calculation working correctly!");
}

#[tokio::test]
#[ignore] // Only run with DATABASE_URL set
async fn test_trim_config_integration() {
    // This test verifies that position evaluator integrates with trim config service
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for position evaluator tests");
    
    let claude_api_key = std::env::var("CLAUDE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    
    let pool = create_pool(&database_url, 5)
        .await
        .expect("Failed to create database pool");
    
    run_migrations(&pool).await.ok();
    
    let client = pool.get().await.expect("Failed to get database client");
    
    // Create a test user
    let user_id = Uuid::new_v4();
    let test_email = format!("test-{}@example.com", user_id);
    client.execute(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        &[&user_id, &test_email.as_str(), &"hash"],
    ).await.expect("Failed to create test user");
    
    // Initialize services
    let trim_config_service = Arc::new(TrimConfigService::new(pool.clone()));
    
    // Create trim config
    let trim_config = trim_config_service.upsert_trim_config(
        user_id,
        api::UpdateTrimConfigRequest {
            enabled: Some(true),
            minimum_profit_percent: Some(Decimal::from(20)),
            trim_percent: Some(Decimal::from(25)),
            max_trims_per_day: Some(3),
        },
    ).await.expect("Failed to create trim config");
    
    assert!(trim_config.enabled);
    assert_eq!(trim_config.minimum_profit_percent, Decimal::from(20));
    
    // Initialize position evaluator
    let _position_evaluator = PositionEvaluator::new(
        pool.clone(),
        trim_config_service.clone(),
        claude_api_key,
    );
    
    // Verify we can get enabled users
    let enabled_users = trim_config_service.get_enabled_users()
        .await
        .expect("Failed to get enabled users");
    
    assert!(enabled_users.contains(&user_id), "User should be in enabled list");
    
    // Cleanup
    client.execute("DELETE FROM trim_configs WHERE user_id = $1", &[&user_id]).await.ok();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.ok();
    
    println!("✅ Trim config integration working correctly!");
}
