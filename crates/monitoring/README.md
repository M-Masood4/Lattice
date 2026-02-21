# Monitoring Engine

The monitoring engine implements a worker pool pattern for parallel whale account monitoring on the Solana blockchain.

## Overview

This crate provides the core monitoring functionality for the Solana Whale Tracker platform. It continuously tracks whale accounts for transaction activity and stores monitoring state in Redis.

## Architecture

### Components

1. **MonitoringEngine**: High-level interface for managing whale monitoring
2. **WorkerPool**: Manages a pool of workers for parallel monitoring
3. **Worker**: Individual worker that monitors assigned whale accounts
4. **RedisStore**: Redis integration for tracking monitoring state

### Design Decisions

- **Worker Pool Pattern**: Uses 1 worker per 100 whales (configurable) for efficient parallel monitoring
- **30-Second Polling**: Each worker checks assigned whales every 30 seconds (configurable)
- **Redis State Management**: Stores last checked transaction signature to detect new transactions
- **Error Resilience**: Workers continue monitoring other whales even if one fails (Requirement 3.5)

## Usage

```rust
use monitoring::{MonitoringEngine, WorkerPoolConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the worker pool
    let config = WorkerPoolConfig {
        solana_rpc_url: "https://api.devnet.solana.com".to_string(),
        solana_fallback_url: Some("https://api.mainnet-beta.solana.com".to_string()),
        redis_url: "redis://localhost:6379".to_string(),
        worker_count: 10,
        whales_per_worker: 100,
        check_interval_seconds: 30,
    };

    // Create the monitoring engine
    let mut engine = MonitoringEngine::new(config).await?;

    // Assign whales for a user
    let user_id = Uuid::new_v4();
    let whale_addresses = vec![
        "whale_address_1".to_string(),
        "whale_address_2".to_string(),
    ];
    engine.start_monitoring(user_id, whale_addresses).await?;

    // Start monitoring
    engine.run().await?;

    // Later: stop monitoring for a user
    engine.stop_monitoring(user_id).await?;

    // Graceful shutdown
    engine.shutdown().await?;

    Ok(())
}
```

## Configuration

The `WorkerPoolConfig` struct allows configuration of:

- `solana_rpc_url`: Primary Solana RPC endpoint
- `solana_fallback_url`: Optional fallback RPC endpoint
- `redis_url`: Redis connection URL
- `worker_count`: Number of workers in the pool
- `whales_per_worker`: Maximum whales per worker (default: 100)
- `check_interval_seconds`: How often to check whales (default: 30)

## Redis Keys

The monitoring engine uses the following Redis key patterns:

- `whale:{address}:last_tx` - Last checked transaction signature for a whale
- `whale:{address}:holdings` - Cached whale holdings (5-minute TTL)
- `monitoring:workers` - Set of active worker IDs

## Requirements Validated

This implementation validates the following requirements:

- **Requirement 3.1**: Monitoring service checks whales every 30 seconds
- **Requirement 3.2**: Detects whale movements within 60 seconds
- **Requirement 3.5**: Continues monitoring other whales on error

## Future Enhancements (Task 6.2)

The current implementation tracks transaction signatures. Task 6.2 will add:

- Transaction parsing and analysis
- Movement type detection (BUY/SELL)
- Movement percentage calculation
- 5% threshold filtering
- Message queue integration for whale movements

## Testing

Run tests with:

```bash
cargo test -p monitoring
```

Note: Integration tests require Redis to be running on localhost:6379.

## Example

See `examples/basic_usage.rs` for a complete example:

```bash
cargo run --example basic_usage -p monitoring
```
