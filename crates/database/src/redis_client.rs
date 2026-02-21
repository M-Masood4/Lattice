use redis::{aio::ConnectionManager, Client};

pub type RedisPool = ConnectionManager;

pub async fn create_redis_client(redis_url: &str) -> anyhow::Result<Client> {
    tracing::info!("Creating Redis client");
    Ok(Client::open(redis_url)?)
}

pub async fn create_redis_pool(client: Client) -> anyhow::Result<RedisPool> {
    tracing::info!("Creating Redis connection manager");
    Ok(ConnectionManager::new(client).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run with a real Redis instance
    async fn test_redis_connection() {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let client = create_redis_client(&redis_url).await;
        assert!(client.is_ok());
        
        if let Ok(client) = client {
            let pool = create_redis_pool(client).await;
            assert!(pool.is_ok());
        }
    }
}
