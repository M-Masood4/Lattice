use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

pub mod migrations;
pub mod proximity;
pub mod redis_client;

pub use proximity::*;
pub use redis_client::{create_redis_client, create_redis_pool, RedisPool};

pub type DbPool = Pool;

pub async fn create_pool(database_url: &str, max_connections: u32) -> anyhow::Result<DbPool> {
    tracing::info!("Creating database connection pool");
    
    let mut cfg = Config::new();
    cfg.url = Some(database_url.to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.pool = Some(deadpool_postgres::PoolConfig::new(max_connections as usize));
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    
    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> anyhow::Result<()> {
    tracing::info!("Running database migrations");
    
    let client = pool.get().await?;
    
    // Read and execute migration files
    let migrations = vec![
        include_str!("../migrations/20240101000001_create_users_table.sql"),
        include_str!("../migrations/20240101000002_create_wallets_table.sql"),
        include_str!("../migrations/20240101000003_create_portfolio_assets_table.sql"),
        include_str!("../migrations/20240101000004_create_whales_table.sql"),
        include_str!("../migrations/20240101000005_create_user_whale_tracking_table.sql"),
        include_str!("../migrations/20240101000006_create_whale_movements_table.sql"),
        include_str!("../migrations/20240101000007_create_recommendations_table.sql"),
        include_str!("../migrations/20240101000008_create_trade_executions_table.sql"),
        include_str!("../migrations/20240101000009_create_notifications_table.sql"),
        include_str!("../migrations/20240101000010_create_notification_preferences_table.sql"),
        include_str!("../migrations/20240101000011_create_subscriptions_table.sql"),
        include_str!("../migrations/20240101000012_create_user_settings_table.sql"),
        include_str!("../migrations/20240101000013_create_portfolio_snapshots_table.sql"),
        include_str!("../migrations/20240101000014_create_multi_chain_wallets_table.sql"),
        include_str!("../migrations/20240101000015_create_benchmarks_table.sql"),
        include_str!("../migrations/20240101000016_create_conversions_table.sql"),
        include_str!("../migrations/20240101000017_create_staking_positions_table.sql"),
        include_str!("../migrations/20240101000018_create_trim_configs_table.sql"),
        include_str!("../migrations/20240101000019_create_trim_executions_table.sql"),
        include_str!("../migrations/20240101000020_create_voice_commands_table.sql"),
        include_str!("../migrations/20240101000021_create_blockchain_receipts_table.sql"),
        include_str!("../migrations/20240101000022_create_chat_messages_table.sql"),
        include_str!("../migrations/20240101000023_create_p2p_offers_table.sql"),
        include_str!("../migrations/20240101000024_create_p2p_exchanges_table.sql"),
        include_str!("../migrations/20240101000025_create_identity_verifications_table.sql"),
        include_str!("../migrations/20240101000026_create_wallet_verifications_table.sql"),
        include_str!("../migrations/20240101000027_add_user_tag_column.sql"),
        include_str!("../migrations/20240101000029_create_proximity_transfers_table.sql"),
        include_str!("../migrations/20240101000030_create_discovery_sessions_table.sql"),
        include_str!("../migrations/20240101000031_create_peer_blocklist_table.sql"),
        include_str!("../migrations/20240101000032_add_proximity_transfer_to_receipts.sql"),
        include_str!("../migrations/20240101000037_create_mesh_price_cache_table.sql"),
        include_str!("../migrations/20240101000038_create_mesh_seen_messages_table.sql"),
    ];
    
    for (idx, migration) in migrations.iter().enumerate() {
        tracing::debug!("Running migration {}", idx + 1);
        client.batch_execute(migration).await?;
    }
    
    tracing::info!("Database migrations completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with a real database
    async fn test_pool_creation() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/test".to_string());
        
        let pool = create_pool(&database_url, 5).await;
        assert!(pool.is_ok());
    }
}