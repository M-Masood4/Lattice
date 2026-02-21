use redis::{AsyncCommands, Client};
use shared::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Redis store for tracking whale monitoring state
#[derive(Clone)]
pub struct RedisStore {
    client: Arc<Mutex<redis::aio::ConnectionManager>>,
}

impl RedisStore {
    /// Create a new Redis store
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| shared::Error::Redis(format!("Failed to connect to Redis: {}", e)))?;
        
        let connection_manager = client
            .get_connection_manager()
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to get connection manager: {}", e)))?;

        Ok(Self {
            client: Arc::new(Mutex::new(connection_manager)),
        })
    }

    /// Get the last checked transaction signature for a whale
    pub async fn get_last_transaction(&self, whale_address: &str) -> Result<Option<String>> {
        let key = format!("whale:{}:last_tx", whale_address);
        let mut conn = self.client.lock().await;
        
        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to get last transaction: {}", e)))?;
        
        Ok(result)
    }

    /// Set the last checked transaction signature for a whale
    pub async fn set_last_transaction(&self, whale_address: &str, signature: &str) -> Result<()> {
        let key = format!("whale:{}:last_tx", whale_address);
        let mut conn = self.client.lock().await;
        
        conn.set::<_, _, ()>(&key, signature)
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to set last transaction: {}", e)))?;
        
        Ok(())
    }

    /// Get cached whale holdings
    pub async fn get_whale_holdings(&self, whale_address: &str) -> Result<Option<String>> {
        let key = format!("whale:{}:holdings", whale_address);
        let mut conn = self.client.lock().await;
        
        let result: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to get whale holdings: {}", e)))?;
        
        Ok(result)
    }

    /// Set cached whale holdings with 5-minute TTL
    pub async fn set_whale_holdings(&self, whale_address: &str, holdings_json: &str) -> Result<()> {
        let key = format!("whale:{}:holdings", whale_address);
        let mut conn = self.client.lock().await;
        
        conn.set_ex::<_, _, ()>(&key, holdings_json, 300)
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to set whale holdings: {}", e)))?;
        
        Ok(())
    }

    /// Check if a worker is active
    pub async fn is_worker_active(&self, worker_id: usize) -> Result<bool> {
        let mut conn = self.client.lock().await;
        
        let result: bool = conn
            .sismember("monitoring:workers", worker_id)
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to check worker status: {}", e)))?;
        
        Ok(result)
    }

    /// Register a worker as active
    pub async fn register_worker(&self, worker_id: usize) -> Result<()> {
        let mut conn = self.client.lock().await;
        
        conn.sadd::<_, _, ()>("monitoring:workers", worker_id)
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to register worker: {}", e)))?;
        
        Ok(())
    }

    /// Unregister a worker
    pub async fn unregister_worker(&self, worker_id: usize) -> Result<()> {
        let mut conn = self.client.lock().await;
        
        conn.srem::<_, _, ()>("monitoring:workers", worker_id)
            .await
            .map_err(|e| shared::Error::Redis(format!("Failed to unregister worker: {}", e)))?;
        
        Ok(())
    }
}
