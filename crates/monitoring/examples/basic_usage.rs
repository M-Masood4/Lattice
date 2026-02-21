/// Example demonstrating basic usage of the monitoring engine
/// 
/// This example shows how to:
/// 1. Create a monitoring engine with worker pool configuration
/// 2. Assign whales to monitor for a user
/// 3. Start the monitoring loop
/// 4. Gracefully shutdown
///
/// Note: This requires Redis to be running on localhost:6379

use monitoring::{MonitoringEngine, WorkerPoolConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Configure the worker pool
    // - 10 workers (1 worker per 100 whales as per requirements)
    // - Each worker checks whales every 30 seconds
    let config = WorkerPoolConfig {
        solana_rpc_url: "https://api.devnet.solana.com".to_string(),
        solana_fallback_url: Some("https://api.mainnet-beta.solana.com".to_string()),
        redis_url: "redis://localhost:6379".to_string(),
        worker_count: 10,
        whales_per_worker: 100,
        check_interval_seconds: 30,
    };

    // Create the monitoring engine
    println!("Creating monitoring engine...");
    let mut engine = MonitoringEngine::new(config).await?;

    // Example: Assign whales for a user to monitor
    let user_id = Uuid::new_v4();
    let whale_addresses = vec![
        "11111111111111111111111111111111".to_string(), // Example whale address
        "22222222222222222222222222222222".to_string(), // Example whale address
    ];

    println!("Assigning {} whales for user {}", whale_addresses.len(), user_id);
    engine.start_monitoring(user_id, whale_addresses).await?;

    // Start the monitoring loop
    println!("Starting monitoring engine...");
    engine.run().await?;

    // In a real application, you would:
    // 1. Keep the engine running in the background
    // 2. Add/remove whales dynamically as users connect/disconnect
    // 3. Handle shutdown signals gracefully

    // For this example, we'll just run for a short time
    println!("Monitoring engine running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal (Ctrl+C)
    tokio::signal::ctrl_c().await?;

    // Gracefully shutdown
    println!("Shutting down monitoring engine...");
    engine.shutdown().await?;

    println!("Monitoring engine stopped.");
    Ok(())
}
