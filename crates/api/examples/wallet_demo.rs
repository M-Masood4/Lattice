/// Example demonstrating wallet service usage
/// 
/// Run with: cargo run --example wallet_demo
/// 
/// Requires:
/// - DATABASE_URL environment variable
/// - REDIS_URL environment variable (optional, defaults to localhost)
/// - SOLANA_RPC_URL environment variable (optional, defaults to devnet)

use api::WalletService;
use blockchain::SolanaClient;
use database::{create_pool, create_redis_client, create_redis_pool, run_migrations};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Solana Whale Tracker - Wallet Service Demo ===\n");

    // Setup Solana client
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| {
            println!("Using default Solana devnet RPC");
            "https://api.devnet.solana.com".to_string()
        });
    
    println!("Connecting to Solana RPC: {}", rpc_url);
    let solana_client = Arc::new(SolanaClient::new(rpc_url, None));

    // Setup database
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    println!("Connecting to database...");
    let db_pool = create_pool(&db_url, 5).await?;
    
    println!("Running database migrations...");
    run_migrations(&db_pool).await?;

    // Setup Redis
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| {
            println!("Using default Redis URL");
            "redis://localhost:6379".to_string()
        });
    
    println!("Connecting to Redis: {}", redis_url);
    let redis_client = create_redis_client(&redis_url).await?;
    let redis_pool = create_redis_pool(redis_client).await?;

    // Create wallet service
    let wallet_service = WalletService::new(solana_client, db_pool.clone(), redis_pool);

    println!("\n--- Test 1: Validate Wallet Addresses ---");
    
    // Valid address
    let valid_address = "11111111111111111111111111111111";
    match wallet_service.validate_wallet_address(valid_address) {
        Ok(_) => println!("✓ Valid address: {}", valid_address),
        Err(e) => println!("✗ Validation failed: {}", e),
    }

    // Invalid address
    let invalid_address = "not_a_valid_address";
    match wallet_service.validate_wallet_address(invalid_address) {
        Ok(_) => println!("✗ Should have failed: {}", invalid_address),
        Err(e) => println!("✓ Correctly rejected invalid address: {}", e),
    }

    println!("\n--- Test 2: Connect Wallet ---");
    
    // Create a test user
    let client = db_pool.get().await?;
    let user_id = Uuid::new_v4();
    let test_email = format!("demo_{}@example.com", user_id);
    
    client
        .execute(
            "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
            &[&user_id, &test_email, &"demo_hash"],
        )
        .await?;
    
    println!("Created test user: {}", user_id);

    // Try to connect a wallet
    // Note: This uses the system program address which always exists
    let wallet_address = "11111111111111111111111111111111";
    
    println!("Attempting to connect wallet: {}", wallet_address);
    match wallet_service.connect_wallet(wallet_address, user_id).await {
        Ok(portfolio) => {
            println!("✓ Wallet connected successfully!");
            println!("  Address: {}", portfolio.wallet_address);
            println!("  Assets: {}", portfolio.assets.len());
            println!("  Total Value: ${:.2}", portfolio.total_value_usd);
            
            for asset in &portfolio.assets {
                println!("    - {} {}", asset.amount, asset.token_symbol);
            }
        }
        Err(e) => {
            println!("✗ Failed to connect wallet: {}", e);
            println!("  (This is expected if RPC is rate-limited or unavailable)");
        }
    }

    println!("\n--- Test 3: Retrieve Portfolio ---");
    
    match wallet_service.get_portfolio(wallet_address).await {
        Ok(portfolio) => {
            println!("✓ Portfolio retrieved from database!");
            println!("  Assets: {}", portfolio.assets.len());
        }
        Err(e) => {
            println!("✗ Failed to retrieve portfolio: {}", e);
        }
    }

    // Cleanup
    println!("\n--- Cleanup ---");
    client
        .execute("DELETE FROM users WHERE id = $1", &[&user_id])
        .await?;
    println!("✓ Test user deleted");

    println!("\n=== Demo Complete ===");
    Ok(())
}
