#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis_store::RedisStore;
    use blockchain::SolanaClient;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_worker_creation() {
        let solana_client = Arc::new(SolanaClient::new(
            "https://api.devnet.solana.com".to_string(),
            None,
        ));

        // Note: This test requires Redis to be running
        // Skip if Redis is not available
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let redis_store = match RedisStore::new(&redis_url).await {
            Ok(store) => store,
            Err(_) => {
                println!("Skipping test - Redis not available");
                return;
            }
        };

        let worker = Worker::new(0, solana_client, redis_store, 30);

        assert_eq!(worker.whale_count().await, 0);
    }

    #[tokio::test]
    async fn test_worker_whale_assignment() {
        let solana_client = Arc::new(SolanaClient::new(
            "https://api.devnet.solana.com".to_string(),
            None,
        ));

        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let redis_store = match RedisStore::new(&redis_url).await {
            Ok(store) => store,
            Err(_) => {
                println!("Skipping test - Redis not available");
                return;
            }
        };

        let worker = Worker::new(0, solana_client, redis_store, 30);

        // Assign some whale addresses
        let whales = vec![
            "11111111111111111111111111111111".to_string(),
            "11111111111111111111111111111112".to_string(),
        ];

        worker.assign_whales(whales.clone()).await.unwrap();
        assert_eq!(worker.whale_count().await, 2);

        // Remove one whale
        worker.remove_whales(&[whales[0].clone()]).await.unwrap();
        assert_eq!(worker.whale_count().await, 1);
    }

    #[test]
    fn test_whale_movement_data_structure() {
        let movement = WhaleMovementData {
            whale_address: "11111111111111111111111111111111".to_string(),
            transaction_signature: "5j7s6NiJS3JAkvgkoc18WVAsiSaci2pxB2A6ueCJP4tprA2TFg9wSyTLeYouxPBJEMzJinENTkpA52YStRW5Dia7".to_string(),
            movement_type: "SELL".to_string(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: "1000000".to_string(),
            percent_of_position: Some(10.5),
        };

        assert_eq!(movement.movement_type, "SELL");
        assert_eq!(movement.percent_of_position, Some(10.5));
    }
}
