use api::analytics::{
    AnalyticsService, MultiChainPortfolio, PerformanceMetrics, PositionDistribution,
};
use chrono::Utc;
use database::DbPool;
use uuid::Uuid;

// Helper to create test database pool
async fn create_test_pool() -> DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/whale_tracker_test".to_string());
    
    database::create_pool(&database_url, 2)
        .await
        .expect("Failed to create test database pool")
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_multi_chain_portfolio() {
    let pool = create_test_pool().await;
    let service = AnalyticsService::new(pool.clone());
    
    // Create test user
    let client = pool.get().await.unwrap();
    let user_row = client
        .query_one(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
            &[&"test@example.com", &"hash"],
        )
        .await
        .unwrap();
    let user_id: Uuid = user_row.get(0);
    
    // Create multi-chain wallets
    let wallet1_row = client
        .query_one(
            "INSERT INTO multi_chain_wallets (user_id, address, blockchain, is_active) 
             VALUES ($1, $2, $3, true) RETURNING id",
            &[&user_id, &"solana_address_1", &"Solana"],
        )
        .await
        .unwrap();
    let wallet1_id: Uuid = wallet1_row.get(0);
    
    let wallet2_row = client
        .query_one(
            "INSERT INTO multi_chain_wallets (user_id, address, blockchain, is_active) 
             VALUES ($1, $2, $3, true) RETURNING id",
            &[&user_id, &"eth_address_1", &"Ethereum"],
        )
        .await
        .unwrap();
    let wallet2_id: Uuid = wallet2_row.get(0);
    
    // Add portfolio assets
    client
        .execute(
            "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd) 
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &wallet1_id,
                &"SOL",
                &"SOL",
                &rust_decimal::Decimal::from(100),
                &rust_decimal::Decimal::from(5000),
            ],
        )
        .await
        .unwrap();
    
    client
        .execute(
            "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd) 
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &wallet2_id,
                &"ETH",
                &"ETH",
                &rust_decimal::Decimal::from(2),
                &rust_decimal::Decimal::from(6000),
            ],
        )
        .await
        .unwrap();
    
    // Test multi-chain portfolio aggregation
    let result = service.get_multi_chain_portfolio(user_id).await;
    assert!(result.is_ok());
    
    let portfolio = result.unwrap();
    assert_eq!(portfolio.user_id, user_id);
    assert_eq!(portfolio.total_value_usd, 11000.0);
    assert_eq!(portfolio.chains.len(), 2);
    
    // Verify Solana chain
    let solana_chain = portfolio.chains.iter().find(|c| c.blockchain == "Solana").unwrap();
    assert_eq!(solana_chain.value_usd, 5000.0);
    assert_eq!(solana_chain.assets.len(), 1);
    assert_eq!(solana_chain.assets[0].token_symbol, "SOL");
    
    // Verify Ethereum chain
    let eth_chain = portfolio.chains.iter().find(|c| c.blockchain == "Ethereum").unwrap();
    assert_eq!(eth_chain.value_usd, 6000.0);
    assert_eq!(eth_chain.assets.len(), 1);
    assert_eq!(eth_chain.assets[0].token_symbol, "ETH");
    
    // Cleanup
    client.execute("DELETE FROM portfolio_assets WHERE wallet_id = $1 OR wallet_id = $2", &[&wallet1_id, &wallet2_id]).await.unwrap();
    client.execute("DELETE FROM multi_chain_wallets WHERE id = $1 OR id = $2", &[&wallet1_id, &wallet2_id]).await.unwrap();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_performance_metrics() {
    let pool = create_test_pool().await;
    let service = AnalyticsService::new(pool.clone());
    
    // Create test user
    let client = pool.get().await.unwrap();
    let user_row = client
        .query_one(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
            &[&"test2@example.com", &"hash"],
        )
        .await
        .unwrap();
    let user_id: Uuid = user_row.get(0);
    
    // Create wallet
    let wallet_row = client
        .query_one(
            "INSERT INTO multi_chain_wallets (user_id, address, blockchain, is_active) 
             VALUES ($1, $2, $3, true) RETURNING id",
            &[&user_id, &"test_address", &"Solana"],
        )
        .await
        .unwrap();
    let wallet_id: Uuid = wallet_row.get(0);
    
    // Add portfolio assets
    client
        .execute(
            "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd) 
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &wallet_id,
                &"SOL",
                &"SOL",
                &rust_decimal::Decimal::from(100),
                &rust_decimal::Decimal::from(10000),
            ],
        )
        .await
        .unwrap();
    
    // Create historical snapshots
    client
        .execute(
            "INSERT INTO portfolio_snapshots (wallet_id, total_value_usd, snapshot_time) 
             VALUES ($1, $2, NOW() - INTERVAL '24 hours')",
            &[&wallet_id, &rust_decimal::Decimal::from(9000)],
        )
        .await
        .unwrap();
    
    client
        .execute(
            "INSERT INTO portfolio_snapshots (wallet_id, total_value_usd, snapshot_time) 
             VALUES ($1, $2, NOW() - INTERVAL '7 days')",
            &[&wallet_id, &rust_decimal::Decimal::from(8000)],
        )
        .await
        .unwrap();
    
    // Test performance metrics
    let result = service.get_performance_metrics(user_id).await;
    assert!(result.is_ok());
    
    let metrics = result.unwrap();
    assert_eq!(metrics.user_id, user_id);
    assert_eq!(metrics.current_value_usd, 10000.0);
    
    // 24h change: 10000 - 9000 = 1000 (11.11%)
    assert!((metrics.change_24h_usd - 1000.0).abs() < 0.01);
    assert!((metrics.change_24h_percent - 11.11).abs() < 0.1);
    
    // 7d change: 10000 - 8000 = 2000 (25%)
    assert!((metrics.change_7d_usd - 2000.0).abs() < 0.01);
    assert!((metrics.change_7d_percent - 25.0).abs() < 0.1);
    
    // Cleanup
    client.execute("DELETE FROM portfolio_snapshots WHERE wallet_id = $1", &[&wallet_id]).await.unwrap();
    client.execute("DELETE FROM portfolio_assets WHERE wallet_id = $1", &[&wallet_id]).await.unwrap();
    client.execute("DELETE FROM multi_chain_wallets WHERE id = $1", &[&wallet_id]).await.unwrap();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_position_distribution() {
    let pool = create_test_pool().await;
    let service = AnalyticsService::new(pool.clone());
    
    // Create test user
    let client = pool.get().await.unwrap();
    let user_row = client
        .query_one(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
            &[&"test3@example.com", &"hash"],
        )
        .await
        .unwrap();
    let user_id: Uuid = user_row.get(0);
    
    // Create wallets on different chains
    let wallet1_row = client
        .query_one(
            "INSERT INTO multi_chain_wallets (user_id, address, blockchain, is_active) 
             VALUES ($1, $2, $3, true) RETURNING id",
            &[&user_id, &"sol_addr", &"Solana"],
        )
        .await
        .unwrap();
    let wallet1_id: Uuid = wallet1_row.get(0);
    
    let wallet2_row = client
        .query_one(
            "INSERT INTO multi_chain_wallets (user_id, address, blockchain, is_active) 
             VALUES ($1, $2, $3, true) RETURNING id",
            &[&user_id, &"eth_addr", &"Ethereum"],
        )
        .await
        .unwrap();
    let wallet2_id: Uuid = wallet2_row.get(0);
    
    let wallet3_row = client
        .query_one(
            "INSERT INTO multi_chain_wallets (user_id, address, blockchain, is_active) 
             VALUES ($1, $2, $3, true) RETURNING id",
            &[&user_id, &"bsc_addr", &"BSC"],
        )
        .await
        .unwrap();
    let wallet3_id: Uuid = wallet3_row.get(0);
    
    // Add assets with different values
    client
        .execute(
            "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd) 
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &wallet1_id,
                &"SOL",
                &"SOL",
                &rust_decimal::Decimal::from(100),
                &rust_decimal::Decimal::from(5000), // 50% of total
            ],
        )
        .await
        .unwrap();
    
    client
        .execute(
            "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd) 
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &wallet2_id,
                &"ETH",
                &"ETH",
                &rust_decimal::Decimal::from(1),
                &rust_decimal::Decimal::from(3000), // 30% of total
            ],
        )
        .await
        .unwrap();
    
    client
        .execute(
            "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd) 
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &wallet3_id,
                &"BNB",
                &"BNB",
                &rust_decimal::Decimal::from(5),
                &rust_decimal::Decimal::from(2000), // 20% of total
            ],
        )
        .await
        .unwrap();
    
    // Test position distribution
    let result = service.get_position_distribution(user_id).await;
    assert!(result.is_ok());
    
    let distribution = result.unwrap();
    assert_eq!(distribution.user_id, user_id);
    assert_eq!(distribution.total_value_usd, 10000.0);
    assert_eq!(distribution.by_blockchain.len(), 3);
    
    // Verify blockchain distribution (sorted by value descending)
    assert_eq!(distribution.by_blockchain[0].blockchain, "Solana");
    assert_eq!(distribution.by_blockchain[0].value_usd, 5000.0);
    assert!((distribution.by_blockchain[0].percentage - 50.0).abs() < 0.01);
    
    assert_eq!(distribution.by_blockchain[1].blockchain, "Ethereum");
    assert_eq!(distribution.by_blockchain[1].value_usd, 3000.0);
    assert!((distribution.by_blockchain[1].percentage - 30.0).abs() < 0.01);
    
    assert_eq!(distribution.by_blockchain[2].blockchain, "BSC");
    assert_eq!(distribution.by_blockchain[2].value_usd, 2000.0);
    assert!((distribution.by_blockchain[2].percentage - 20.0).abs() < 0.01);
    
    // Cleanup
    client.execute("DELETE FROM portfolio_assets WHERE wallet_id IN ($1, $2, $3)", &[&wallet1_id, &wallet2_id, &wallet3_id]).await.unwrap();
    client.execute("DELETE FROM multi_chain_wallets WHERE id IN ($1, $2, $3)", &[&wallet1_id, &wallet2_id, &wallet3_id]).await.unwrap();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.unwrap();
}

#[test]
fn test_multi_chain_portfolio_structure() {
    // Test that the data structures serialize correctly
    let portfolio = MultiChainPortfolio {
        user_id: Uuid::new_v4(),
        total_value_usd: 10000.0,
        chains: vec![],
        timestamp: Utc::now(),
    };
    
    let json = serde_json::to_string(&portfolio).unwrap();
    assert!(json.contains("user_id"));
    assert!(json.contains("total_value_usd"));
    assert!(json.contains("chains"));
    assert!(json.contains("timestamp"));
}

#[test]
fn test_performance_metrics_structure() {
    let metrics = PerformanceMetrics {
        user_id: Uuid::new_v4(),
        current_value_usd: 10000.0,
        change_24h_usd: 500.0,
        change_24h_percent: 5.0,
        change_7d_usd: 1000.0,
        change_7d_percent: 10.0,
        all_time_profit_loss_usd: 2000.0,
        all_time_profit_loss_percent: 20.0,
        positions: vec![],
        timestamp: Utc::now(),
    };
    
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("current_value_usd"));
    assert!(json.contains("change_24h_usd"));
    assert!(json.contains("change_7d_usd"));
    assert!(json.contains("all_time_profit_loss_usd"));
}

#[test]
fn test_position_distribution_structure() {
    let distribution = PositionDistribution {
        user_id: Uuid::new_v4(),
        total_value_usd: 10000.0,
        by_blockchain: vec![],
        by_asset_type: vec![],
        timestamp: Utc::now(),
    };
    
    let json = serde_json::to_string(&distribution).unwrap();
    assert!(json.contains("total_value_usd"));
    assert!(json.contains("by_blockchain"));
    assert!(json.contains("by_asset_type"));
}

#[test]
fn test_percentage_calculations() {
    // Test percentage calculation edge cases
    let total: f64 = 10000.0;
    let part: f64 = 3000.0;
    let percentage = (part / total) * 100.0;
    assert!((percentage - 30.0).abs() < 0.01);
    
    // Test zero total
    let zero_total: f64 = 0.0;
    let percentage_zero = if zero_total > 0.0 {
        (part / zero_total) * 100.0
    } else {
        0.0
    };
    assert_eq!(percentage_zero, 0.0);
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_active_benchmarks() {
    let pool = create_test_pool().await;
    let service = AnalyticsService::new(pool.clone());
    
    // Create test user
    let client = pool.get().await.unwrap();
    let user_row = client
        .query_one(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
            &[&"test4@example.com", &"hash"],
        )
        .await
        .unwrap();
    let user_id: Uuid = user_row.get(0);
    
    // Create active benchmarks
    client
        .execute(
            "INSERT INTO benchmarks (user_id, asset_symbol, trigger_type, target_price, action, is_active) 
             VALUES ($1, $2, $3, $4, $5, true)",
            &[
                &user_id,
                &"SOL",
                &"above",
                &rust_decimal::Decimal::from(200),
                &"notify",
            ],
        )
        .await
        .unwrap();
    
    client
        .execute(
            "INSERT INTO benchmarks (user_id, asset_symbol, trigger_type, target_price, action, is_active) 
             VALUES ($1, $2, $3, $4, $5, true)",
            &[
                &user_id,
                &"ETH",
                &"below",
                &rust_decimal::Decimal::from(3000),
                &"execute",
            ],
        )
        .await
        .unwrap();
    
    // Test active benchmarks
    let result = service.get_active_benchmarks(user_id).await;
    assert!(result.is_ok());
    
    let benchmarks = result.unwrap();
    assert_eq!(benchmarks.len(), 2);
    
    // Verify SOL benchmark
    let sol_benchmark = benchmarks.iter().find(|b| b.asset_symbol == "SOL").unwrap();
    assert_eq!(sol_benchmark.trigger_type, "above");
    assert_eq!(sol_benchmark.target_price, 200.0);
    assert_eq!(sol_benchmark.action, "notify");
    
    // Verify ETH benchmark
    let eth_benchmark = benchmarks.iter().find(|b| b.asset_symbol == "ETH").unwrap();
    assert_eq!(eth_benchmark.trigger_type, "below");
    assert_eq!(eth_benchmark.target_price, 3000.0);
    assert_eq!(eth_benchmark.action, "execute");
    
    // Cleanup
    client.execute("DELETE FROM benchmarks WHERE user_id = $1", &[&user_id]).await.unwrap();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires database
async fn test_get_recent_ai_actions() {
    let pool = create_test_pool().await;
    let service = AnalyticsService::new(pool.clone());
    
    // Create test user
    let client = pool.get().await.unwrap();
    let user_row = client
        .query_one(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id",
            &[&"test5@example.com", &"hash"],
        )
        .await
        .unwrap();
    let user_id: Uuid = user_row.get(0);
    
    // Create trim execution
    client
        .execute(
            "INSERT INTO trim_executions (user_id, asset_symbol, amount_trimmed, profit_realized, reasoning, executed_at) 
             VALUES ($1, $2, $3, $4, $5, NOW())",
            &[
                &user_id,
                &"SOL",
                &rust_decimal::Decimal::from(10),
                &rust_decimal::Decimal::from(500),
                &"Profit target reached",
            ],
        )
        .await
        .unwrap();
    
    // Create recommendation
    client
        .execute(
            "INSERT INTO recommendations (user_id, asset_symbol, action, confidence, reasoning, created_at) 
             VALUES ($1, $2, $3, $4, $5, NOW())",
            &[
                &user_id,
                &"ETH",
                &"buy",
                &rust_decimal::Decimal::from(85),
                &"Strong uptrend detected",
            ],
        )
        .await
        .unwrap();
    
    // Test recent AI actions
    let result = service.get_recent_ai_actions(user_id, 10).await;
    assert!(result.is_ok());
    
    let actions = result.unwrap();
    assert_eq!(actions.user_id, user_id);
    assert!(actions.actions.len() >= 2);
    
    // Verify trim action exists
    let trim_action = actions.actions.iter().find(|a| a.action_type == "trim");
    assert!(trim_action.is_some());
    let trim = trim_action.unwrap();
    assert_eq!(trim.asset_symbol, "SOL");
    assert!(trim.description.contains("Trimmed"));
    assert!(trim.result.is_some());
    
    // Verify recommendation exists
    let rec_action = actions.actions.iter().find(|a| a.action_type == "recommendation");
    assert!(rec_action.is_some());
    let rec = rec_action.unwrap();
    assert_eq!(rec.asset_symbol, "ETH");
    assert!(rec.description.contains("buy"));
    
    // Cleanup
    client.execute("DELETE FROM trim_executions WHERE user_id = $1", &[&user_id]).await.unwrap();
    client.execute("DELETE FROM recommendations WHERE user_id = $1", &[&user_id]).await.unwrap();
    client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await.unwrap();
}

#[test]
fn test_active_benchmark_structure() {
    use api::analytics::ActiveBenchmark;
    
    let benchmark = ActiveBenchmark {
        id: Uuid::new_v4(),
        asset_symbol: "SOL".to_string(),
        trigger_type: "above".to_string(),
        target_price: 200.0,
        current_price: 190.0,
        distance_percent: 5.26,
        action: "notify".to_string(),
        created_at: Utc::now(),
    };
    
    let json = serde_json::to_string(&benchmark).unwrap();
    assert!(json.contains("asset_symbol"));
    assert!(json.contains("trigger_type"));
    assert!(json.contains("target_price"));
    assert!(json.contains("current_price"));
    assert!(json.contains("distance_percent"));
}

#[test]
fn test_recent_ai_actions_structure() {
    use api::analytics::{RecentAIActions, AIAction};
    
    let actions = RecentAIActions {
        user_id: Uuid::new_v4(),
        actions: vec![
            AIAction {
                action_type: "trim".to_string(),
                asset_symbol: "SOL".to_string(),
                description: "Trimmed 10 SOL".to_string(),
                timestamp: Utc::now(),
                result: Some("Profit: $500".to_string()),
            },
        ],
        timestamp: Utc::now(),
    };
    
    let json = serde_json::to_string(&actions).unwrap();
    assert!(json.contains("user_id"));
    assert!(json.contains("actions"));
    assert!(json.contains("action_type"));
    assert!(json.contains("trim"));
}
