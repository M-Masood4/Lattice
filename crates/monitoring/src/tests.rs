#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorkerPoolConfig, MonitoringEngine};

    #[tokio::test]
    #[ignore] // Requires Redis to be running
    async fn test_worker_pool_creation() {
        let config = WorkerPoolConfig {
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            solana_fallback_url: None,
            redis_url: "redis://localhost:6379".to_string(),
            worker_count: 2,
            whales_per_worker: 100,
            check_interval_seconds: 30,
        };

        let result = MonitoringEngine::new(config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_worker_pool_config() {
        let config = WorkerPoolConfig {
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            solana_fallback_url: Some("https://api.mainnet-beta.solana.com".to_string()),
            redis_url: "redis://localhost:6379".to_string(),
            worker_count: 5,
            whales_per_worker: 100,
            check_interval_seconds: 30,
        };

        assert_eq!(config.worker_count, 5);
        assert_eq!(config.whales_per_worker, 100);
        assert_eq!(config.check_interval_seconds, 30);
    }
}
