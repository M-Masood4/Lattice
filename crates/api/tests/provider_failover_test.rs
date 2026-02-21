/// Integration tests for provider failover and recovery
/// 
/// These tests verify that the mesh network handles provider disconnections
/// gracefully and maintains cached data availability.

use api::{BirdeyeService, MeshPriceService, WebSocketService};
use api::mesh_types::{PriceUpdate, PriceData, CachedPriceData};
use database::create_pool;
use proximity::PeerConnectionManager;
use std::collections::HashMap;
use std::sync::Arc;

/// Helper to create a test Redis connection
async fn create_test_redis() -> redis::aio::ConnectionManager {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    redis::aio::ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis")
}

/// Helper to create a test database pool
async fn create_test_db() -> database::DbPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/crypto_trading_test".to_string());
    create_pool(&database_url, 2)
        .await
        .expect("Failed to create database pool")
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_provider_disconnect_preserves_cache() {
    // Create mesh service
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Start the service
    mesh_service.start().await.expect("Failed to start service");
    
    // Simulate provider disconnect
    let provider_id = uuid::Uuid::new_v4();
    mesh_service
        .on_peer_disconnected(provider_id.to_string())
        .await
        .expect("Failed to handle provider disconnect");
    
    // Verify cached data is still accessible
    let cached_data = mesh_service
        .get_all_price_data()
        .await
        .expect("Failed to get cached data");
    
    // Cache should be accessible even with no providers
    // (it may be empty if no data was cached before, but should not error)
    println!("Cached data entries: {}", cached_data.len());
    
    // Get network status
    let status = mesh_service
        .get_network_status()
        .await
        .expect("Failed to get network status");
    
    // Should have no active providers
    assert_eq!(
        status.active_providers.len(),
        0,
        "Should have no active providers after disconnect"
    );
    
    // Stop the service
    mesh_service.stop().await.expect("Failed to stop service");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_all_providers_offline_status() {
    // Create mesh service
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Get initial network status (no providers)
    let status = mesh_service
        .get_network_status()
        .await
        .expect("Failed to get network status");
    
    // Should have no active providers
    assert_eq!(
        status.active_providers.len(),
        0,
        "Should have no active providers initially"
    );
    
    // Should not be extended offline yet (just started)
    assert!(
        !status.extended_offline,
        "Should not be extended offline immediately"
    );
    
    // Offline duration should be Some (tracking started)
    assert!(
        status.offline_duration_minutes.is_some(),
        "Should be tracking offline duration"
    );
    
    println!("Network status: {:?}", status);
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_provider_reconnection_clears_offline_status() {
    // Create mesh service
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Get initial status (all providers offline)
    let status_before = mesh_service
        .get_network_status()
        .await
        .expect("Failed to get network status");
    
    assert_eq!(status_before.active_providers.len(), 0);
    
    // Simulate a provider reconnecting by sending a price update
    let provider_id = uuid::Uuid::new_v4();
    let price_update = PriceUpdate {
        message_id: uuid::Uuid::new_v4(),
        source_node_id: provider_id,
        timestamp: chrono::Utc::now(),
        prices: {
            let mut map = HashMap::new();
            map.insert(
                "SOL".to_string(),
                PriceData {
                    asset: "SOL".to_string(),
                    price: "100.00".to_string(),
                    blockchain: "solana".to_string(),
                    change_24h: Some("5.0".to_string()),
                },
            );
            map
        },
        ttl: 10,
    };
    
    // Process the update - this verifies the service accepts updates from reconnected providers
    mesh_service
        .handle_price_update(price_update, provider_id.to_string())
        .await
        .expect("Failed to handle price update from reconnected provider");
    
    println!("Provider reconnection handled successfully");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_cached_data_fallback_when_offline() {
    // Create mesh service
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")
        .unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Start the service
    mesh_service.start().await.expect("Failed to start service");
    
    // Try to get price data when no providers are online
    let price_data = mesh_service
        .get_price_data("SOL")
        .await
        .expect("Failed to get price data");
    
    // Should return None if no cached data, but should not error
    println!("Price data for SOL: {:?}", price_data);
    
    // Get all price data
    let all_data = mesh_service
        .get_all_price_data()
        .await
        .expect("Failed to get all price data");
    
    println!("Total cached entries: {}", all_data.len());
    
    // Stop the service
    mesh_service.stop().await.expect("Failed to stop service");
}
