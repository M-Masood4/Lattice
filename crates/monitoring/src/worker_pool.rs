use crate::redis_store::RedisStore;
use crate::worker::Worker;
use crate::message_queue::MessageQueueClient;
use blockchain::SolanaClient;
use database::DbPool;
use shared::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{info, warn};
use uuid::Uuid;

/// Configuration for the worker pool
#[derive(Clone)]
pub struct WorkerPoolConfig {
    pub solana_rpc_url: String,
    pub solana_fallback_url: Option<String>,
    pub redis_url: String,
    pub worker_count: usize,
    pub whales_per_worker: usize,
    pub check_interval_seconds: u64,
}

/// Worker pool that manages multiple workers for parallel whale monitoring
pub struct WorkerPool {
    config: WorkerPoolConfig,
    workers: Vec<Arc<RwLock<Worker>>>,
    worker_handles: Arc<RwLock<Vec<JoinHandle<Result<()>>>>>,
    #[allow(dead_code)]
    solana_client: Arc<SolanaClient>,
    #[allow(dead_code)]
    redis_store: RedisStore,
    db_pool: Option<DbPool>,
    message_queue: Option<Arc<MessageQueueClient>>,
    // Track which whales are assigned to which workers
    whale_assignments: Arc<RwLock<HashMap<String, usize>>>,
    // Track which users are monitoring which whales
    user_whales: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
    initialized: bool,
}

impl WorkerPool {
    /// Create a new worker pool (async constructor)
    pub async fn new(config: WorkerPoolConfig) -> Result<Self> {
        info!(
            "Creating worker pool with {} workers, {} whales per worker",
            config.worker_count, config.whales_per_worker
        );

        // Create Solana client
        let solana_client = Arc::new(SolanaClient::new(
            config.solana_rpc_url.clone(),
            config.solana_fallback_url.clone(),
        ));

        // Initialize Redis store
        let redis_store = RedisStore::new(&config.redis_url).await?;

        // Create workers
        let mut workers = Vec::new();
        for i in 0..config.worker_count {
            let worker = Worker::new(
                i,
                solana_client.clone(),
                redis_store.clone(),
                config.check_interval_seconds,
            );
            workers.push(Arc::new(RwLock::new(worker)));
        }

        info!("Worker pool initialized with {} workers", workers.len());

        Ok(Self {
            config: config.clone(),
            workers,
            worker_handles: Arc::new(RwLock::new(Vec::new())),
            solana_client,
            redis_store,
            db_pool: None,
            message_queue: None,
            whale_assignments: Arc::new(RwLock::new(HashMap::new())),
            user_whales: Arc::new(RwLock::new(HashMap::new())),
            initialized: true,
        })
    }

    /// Set the database pool for all workers
    pub async fn set_db_pool(&mut self, pool: DbPool) {
        self.db_pool = Some(pool.clone());
        
        // Set the pool for all workers
        for worker in &self.workers {
            let mut worker_guard = worker.write().await;
            worker_guard.set_db_pool(pool.clone());
        }
        
        info!("Database pool configured for all workers");
    }

    /// Set the message queue client for all workers
    /// 
    /// **Validates: Requirements 3.2**
    pub async fn set_message_queue(&mut self, mq_client: Arc<MessageQueueClient>) {
        self.message_queue = Some(mq_client.clone());
        
        // Set the message queue for all workers
        for worker in &self.workers {
            let mut worker_guard = worker.write().await;
            worker_guard.set_message_queue(mq_client.clone());
        }
        
        info!("Message queue configured for all workers");
    }

    /// Assign whales to a user for monitoring
    pub async fn assign_whales(&mut self, user_id: Uuid, whale_addresses: Vec<String>) -> Result<()> {
        info!(
            "Assigning {} whales for user {}",
            whale_addresses.len(),
            user_id
        );

        // Store user-whale mapping
        let mut user_whales = self.user_whales.write().await;
        user_whales.insert(user_id, whale_addresses.clone());
        drop(user_whales);

        // Distribute whales across workers using round-robin
        let mut assignments = self.whale_assignments.write().await;
        
        for (idx, whale_address) in whale_addresses.iter().enumerate() {
            // Skip if already assigned
            if assignments.contains_key(whale_address) {
                continue;
            }

            // Find worker with least whales (simple load balancing)
            let worker_idx = idx % self.workers.len();
            
            // Check if worker has capacity
            let worker = &self.workers[worker_idx];
            let current_count = worker.read().await.whale_count().await;
            
            if current_count >= self.config.whales_per_worker {
                warn!(
                    "Worker {} at capacity ({}/{}), whale {} may not be monitored",
                    worker_idx,
                    current_count,
                    self.config.whales_per_worker,
                    whale_address
                );
                continue;
            }

            // Assign whale to worker
            worker.read().await.assign_whales(vec![whale_address.clone()]).await?;
            assignments.insert(whale_address.clone(), worker_idx);
        }

        info!(
            "Assigned {} whales across {} workers",
            assignments.len(),
            self.workers.len()
        );

        Ok(())
    }

    /// Remove whales for a user
    pub async fn remove_user_whales(&mut self, user_id: Uuid) -> Result<()> {
        info!("Removing whales for user {}", user_id);

        // Get user's whales
        let mut user_whales = self.user_whales.write().await;
        let whale_addresses = match user_whales.remove(&user_id) {
            Some(whales) => whales,
            None => {
                info!("No whales found for user {}", user_id);
                return Ok(());
            }
        };
        drop(user_whales);

        // Remove from assignments and workers
        let mut assignments = self.whale_assignments.write().await;
        
        for whale_address in &whale_addresses {
            if let Some(worker_idx) = assignments.remove(whale_address) {
                if let Some(worker) = self.workers.get(worker_idx) {
                    worker.read().await.remove_whales(std::slice::from_ref(whale_address)).await?;
                }
            }
        }

        info!("Removed {} whales for user {}", whale_addresses.len(), user_id);
        Ok(())
    }

    /// Start all workers
    pub async fn run(&mut self) -> Result<()> {
        if !self.initialized {
            return Err(Error::Internal("Worker pool not initialized".to_string()));
        }

        info!("Starting worker pool with {} workers", self.workers.len());

        let mut handles = self.worker_handles.write().await;
        
        for worker in &self.workers {
            let worker_clone = worker.clone();
            let handle = tokio::spawn(async move {
                let worker_guard = worker_clone.read().await;
                worker_guard.run().await
            });
            handles.push(handle);
        }

        info!("All workers started");
        Ok(())
    }

    /// Shutdown all workers gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down worker pool");

        // Signal all workers to shutdown
        for worker in &self.workers {
            worker.read().await.shutdown().await;
        }

        // Wait for all workers to finish
        let mut handles = self.worker_handles.write().await;
        while let Some(handle) = handles.pop() {
            if let Err(e) = handle.await {
                warn!("Worker task join error: {}", e);
            }
        }

        info!("Worker pool shutdown complete");
        Ok(())
    }

    /// Get statistics about the worker pool
    pub async fn get_stats(&self) -> WorkerPoolStats {
        let assignments = self.whale_assignments.read().await;
        let user_whales = self.user_whales.read().await;

        let mut worker_loads = Vec::new();
        for worker in &self.workers {
            worker_loads.push(worker.read().await.whale_count().await);
        }

        WorkerPoolStats {
            total_workers: self.workers.len(),
            total_whales: assignments.len(),
            total_users: user_whales.len(),
            worker_loads,
        }
    }
}

/// Statistics about the worker pool
#[derive(Debug, Clone)]
pub struct WorkerPoolStats {
    pub total_workers: usize,
    pub total_whales: usize,
    pub total_users: usize,
    pub worker_loads: Vec<usize>,
}
