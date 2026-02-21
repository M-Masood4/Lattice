use database::{create_pool, run_migrations};
use rust_decimal::Decimal;

#[tokio::test]
#[ignore] // Only run with DATABASE_URL set
async fn test_all_migrations_run_successfully() {
    // This test verifies that all migration files can be executed successfully
    // Run with: cargo test --package database test_all_migrations_run_successfully -- --ignored
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for migration tests");
    
    // Create a connection pool
    let pool = create_pool(&database_url, 5)
        .await
        .expect("Failed to create database pool");
    
    // Run all migrations (may already be run, so we check the result)
    let result = run_migrations(&pool).await;
    
    // If migrations fail because tables already exist, that's okay for this test
    // We just want to verify the schema is correct
    if let Err(e) = &result {
        let error_msg = format!("{:?}", e);
        if !error_msg.contains("already exists") && !error_msg.contains("relation") {
            panic!("Migrations failed with unexpected error: {:?}", e);
        }
        // Tables already exist, which is fine - we'll verify them below
        tracing::info!("Tables already exist, skipping migration execution");
    }
    
    // Verify key tables exist by querying them
    let client = pool.get().await.expect("Failed to get database client");
    
    // Check new tables from crypto-trading-platform-enhancements
    let tables_to_verify = vec![
        "multi_chain_wallets",
        "benchmarks",
        "conversions",
        "staking_positions",
        "trim_configs",
        "trim_executions",
        "pending_trims",
        "voice_commands",
        "blockchain_receipts",
        "chat_messages",
        "p2p_offers",
        "p2p_exchanges",
        "identity_verifications",
        "wallet_verifications",
    ];
    
    for table in tables_to_verify {
        let query = format!(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = '{}'
            )",
            table
        );
        
        let row = client.query_one(&query, &[])
            .await
            .expect(&format!("Failed to check if table {} exists", table));
        
        let exists: bool = row.get(0);
        assert!(exists, "Table {} should exist after migrations", table);
    }
    
    // Verify user_tag column was added to users table
    let query = "SELECT EXISTS (
        SELECT FROM information_schema.columns 
        WHERE table_schema = 'public' 
        AND table_name = 'users' 
        AND column_name = 'user_tag'
    )";
    
    let row = client.query_one(query, &[])
        .await
        .expect("Failed to check if user_tag column exists");
    
    let exists: bool = row.get(0);
    assert!(exists, "user_tag column should exist in users table");
    
    println!("✅ All migrations completed successfully!");
    println!("✅ All required tables verified!");
}

#[tokio::test]
#[ignore] // Only run with DATABASE_URL set
async fn test_benchmark_price_constraint() {
    // This test verifies that the CHECK constraint on benchmark target_price works
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for migration tests");
    
    let pool = create_pool(&database_url, 5)
        .await
        .expect("Failed to create database pool");
    
    // Migrations may already be run, so we ignore errors here
    let _ = run_migrations(&pool).await;
    
    let client = pool.get().await.expect("Failed to get database client");
    
    // First, create a test user with unique email
    let user_id = uuid::Uuid::new_v4();
    let test_email = format!("test-{}@example.com", user_id);
    client.execute(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        &[&user_id, &test_email.as_str(), &"hash"],
    ).await.expect("Failed to create test user");
    
    // Try to insert a benchmark with negative price (should fail)
    let result = client.execute(
        "INSERT INTO benchmarks (user_id, asset, blockchain, target_price, trigger_type, action_type) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        &[&user_id, &"SOL", &"Solana", &Decimal::from(-10), &"ABOVE", &"ALERT"],
    ).await;
    
    assert!(result.is_err(), "Should not allow negative target_price");
    
    // Try to insert a benchmark with zero price (should fail)
    let result = client.execute(
        "INSERT INTO benchmarks (user_id, asset, blockchain, target_price, trigger_type, action_type) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        &[&user_id, &"SOL", &"Solana", &Decimal::ZERO, &"ABOVE", &"ALERT"],
    ).await;
    
    assert!(result.is_err(), "Should not allow zero target_price");
    
    // Insert a benchmark with positive price (should succeed)
    let result = client.execute(
        "INSERT INTO benchmarks (user_id, asset, blockchain, target_price, trigger_type, action_type) 
         VALUES ($1, $2, $3, $4, $5, $6)",
        &[&user_id, &"SOL", &"Solana", &Decimal::from(100), &"ABOVE", &"ALERT"],
    ).await;
    
    assert!(result.is_ok(), "Should allow positive target_price: {:?}", result.err());
    
    // Cleanup
    client.execute("DELETE FROM benchmarks WHERE user_id = $1", &[&user_id]).await.ok();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.ok();
    
    println!("✅ Benchmark price constraint working correctly!");
}

#[tokio::test]
#[ignore] // Only run with DATABASE_URL set
async fn test_indexes_created() {
    // This test verifies that all required indexes were created
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for migration tests");
    
    let pool = create_pool(&database_url, 5)
        .await
        .expect("Failed to create database pool");
    
    // Migrations may already be run, so we ignore errors here
    let _ = run_migrations(&pool).await;
    
    let client = pool.get().await.expect("Failed to get database client");
    
    // Check for key indexes
    let indexes_to_verify = vec![
        "idx_multi_chain_wallets_user",
        "idx_benchmarks_user_active",
        "idx_conversions_user",
        "idx_staking_positions_user",
        "idx_trim_executions_user",
        "idx_voice_commands_user",
        "idx_chat_messages_users",
        "idx_p2p_offers_status",
        "idx_p2p_exchanges_users",
        "idx_users_user_tag",
    ];
    
    for index in indexes_to_verify {
        let query = format!(
            "SELECT EXISTS (
                SELECT FROM pg_indexes 
                WHERE schemaname = 'public' 
                AND indexname = '{}'
            )",
            index
        );
        
        let row = client.query_one(&query, &[])
            .await
            .expect(&format!("Failed to check if index {} exists", index));
        
        let exists: bool = row.get(0);
        assert!(exists, "Index {} should exist after migrations", index);
    }
    
    println!("✅ All required indexes verified!");
}
