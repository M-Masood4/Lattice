mod worker_pool;
mod worker;
mod redis_store;
mod message_queue;

#[cfg(test)]
mod tests;

pub use worker_pool::{WorkerPool, WorkerPoolConfig};
pub use worker::Worker;
pub use redis_store::RedisStore;
pub use message_queue::{MessageQueueClient, WhaleMovementEvent};

use shared::Result;

/// Whale monitoring engine that manages a pool of workers
/// to continuously track whale accounts for transaction activity
pub struct MonitoringEngine {
    worker_pool: WorkerPool,
}

impl MonitoringEngine {
    /// Create a new monitoring engine with the given configuration
    pub async fn new(config: WorkerPoolConfig) -> Result<Self> {
        let worker_pool = WorkerPool::new(config).await?;
        Ok(Self { worker_pool })
    }

    /// Start monitoring whales for a user
    pub async fn start_monitoring(&mut self, user_id: uuid::Uuid, whale_addresses: Vec<String>) -> Result<()> {
        self.worker_pool.assign_whales(user_id, whale_addresses).await
    }

    /// Stop monitoring for a user
    pub async fn stop_monitoring(&mut self, user_id: uuid::Uuid) -> Result<()> {
        self.worker_pool.remove_user_whales(user_id).await
    }

    /// Start the worker pool
    pub async fn run(&mut self) -> Result<()> {
        self.worker_pool.run().await
    }

    /// Shutdown the worker pool gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        self.worker_pool.shutdown().await
    }
}
