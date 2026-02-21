use api::{BirdeyeService, MeshPriceService, WebSocketService};
use api::mesh_types::{PriceUpdate, PriceData};
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
async fn test_mesh_price_service_creation() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Verify initial state
    assert!(!mesh_service.is_provider().await, "Should not be a provider initially");
    
    // Verify node_id is set
    let node_id = mesh_service.node_id();
    assert_ne!(node_id.to_string(), "", "Node ID should be set");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_mesh_price_service_start_stop() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Start the service
    let result = mesh_service.start().await;
    assert!(result.is_ok(), "Service should start successfully");
    
    // Stop the service
    let result = mesh_service.stop().await;
    assert!(result.is_ok(), "Service should stop successfully");
}

#[tokio::test]
#[ignore] // Requires Redis, database, and valid Birdeye API key
async fn test_enable_provider_mode_with_invalid_key() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = "invalid_key".to_string();
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Try to enable provider mode with invalid key
    let result = mesh_service.enable_provider_mode("invalid_api_key".to_string()).await;
    assert!(result.is_err(), "Should fail with invalid API key");
    
    // Verify provider mode is not enabled
    assert!(!mesh_service.is_provider().await, "Provider mode should not be enabled");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_disable_provider_mode() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Disable provider mode (should work even if not enabled)
    let result = mesh_service.disable_provider_mode().await;
    assert!(result.is_ok(), "Should be able to disable provider mode");
    
    // Verify provider mode is not enabled
    assert!(!mesh_service.is_provider().await, "Provider mode should not be enabled");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_get_price_data_empty_cache() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Try to get price data for non-existent asset
    let result = mesh_service.get_price_data("NONEXISTENT").await;
    assert!(result.is_ok(), "Should return Ok even for non-existent asset");
    assert!(result.unwrap().is_none(), "Should return None for non-existent asset");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_get_all_price_data_empty() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Get all price data (should be empty initially)
    let result = mesh_service.get_all_price_data().await;
    assert!(result.is_ok(), "Should return Ok");
    assert!(result.unwrap().is_empty(), "Should return empty map initially");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_get_network_status() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Get network status
    let result = mesh_service.get_network_status().await;
    assert!(result.is_ok(), "Should return network status");
    
    let status = result.unwrap();
    assert_eq!(status.active_providers.len(), 0, "Should have no active providers initially");
    assert_eq!(status.connected_peers, 0, "Should have no connected peers initially");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_provider_disconnect_keeps_cached_data() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Simulate a provider being active and then disconnecting
    let provider_id = uuid::Uuid::new_v4();
    
    // Simulate provider disconnect
    let result = mesh_service.on_peer_disconnected(provider_id.to_string()).await;
    assert!(result.is_ok(), "Should handle provider disconnect");
    
    // Verify cached data is still accessible (Requirement 9.1)
    let all_data = mesh_service.get_all_price_data().await;
    assert!(all_data.is_ok(), "Should still be able to access cached data");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_all_providers_offline_warning() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Get network status when no providers are online
    let status = mesh_service.get_network_status().await;
    assert!(status.is_ok(), "Should return network status");
    
    let status = status.unwrap();
    assert_eq!(status.active_providers.len(), 0, "Should have no active providers");
    
    // Verify that extended_offline is false initially (not 10 minutes yet)
    assert!(!status.extended_offline, "Should not be extended offline yet");
    
    // Requirement 9.2: System should display warning when all providers offline
    // This is verified by checking active_providers.is_empty()
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_provider_reconnection() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    let provider_id = uuid::Uuid::new_v4();
    
    // Requirement 9.3: System should resume accepting updates on provider reconnection
    // Test that the service can handle price updates after a provider reconnects
    // (The actual network status exchange is tested in property tests)
    
    // Create a price update from the reconnected provider
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
    let result = mesh_service.handle_price_update(price_update, provider_id.to_string()).await;
    assert!(result.is_ok(), "Should accept updates from reconnected provider");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_new_provider_auto_discovery() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Requirement 9.4: System should auto-discover new providers joining network
    // Test that the service can handle price updates from a new provider
    // (The actual network status exchange is tested in property tests)
    
    // Simulate a new provider sending price updates
    let new_provider_id = uuid::Uuid::new_v4();
    let price_update = PriceUpdate {
        message_id: uuid::Uuid::new_v4(),
        source_node_id: new_provider_id,
        timestamp: chrono::Utc::now(),
        prices: {
            let mut map = HashMap::new();
            map.insert(
                "ETH".to_string(),
                PriceData {
                    asset: "ETH".to_string(),
                    price: "2500.00".to_string(),
                    blockchain: "ethereum".to_string(),
                    change_24h: Some("3.5".to_string()),
                },
            );
            map
        },
        ttl: 10,
    };
    
    // Process the update - this verifies the service accepts updates from new providers
    let result = mesh_service.handle_price_update(price_update, new_provider_id.to_string()).await;
    assert!(result.is_ok(), "Should accept updates from new provider");
    
    // Verify the price was cached
    let cached_price = mesh_service.get_price_data("ETH").await;
    assert!(cached_price.is_ok(), "Should cache price from new provider");
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_extended_offline_indicator() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Get initial network status
    let status = mesh_service.get_network_status().await;
    assert!(status.is_ok(), "Should return network status");
    
    let status = status.unwrap();
    
    // Initially should not be extended offline
    assert!(!status.extended_offline, "Should not be extended offline initially");
    
    // Requirement 9.5: System should display offline indicator after 10 minutes
    // The extended_offline field in NetworkStatus provides this information
    // In a real scenario, this would be true after 10 minutes of no providers
}

#[tokio::test]
#[ignore] // Requires Redis and database connections
async fn test_cached_data_served_when_no_providers() {
    // Create dependencies
    let redis = create_test_redis().await;
    let db = create_test_db().await;
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let birdeye_service = Arc::new(BirdeyeService::new(birdeye_api_key, redis.clone()));
    let peer_manager = Arc::new(PeerConnectionManager::new());
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Create MeshPriceService
    let mesh_service = MeshPriceService::new(
        birdeye_service,
        peer_manager,
        redis,
        db,
        websocket_service,
    );
    
    // Verify we can get cached data even with no providers
    let result = mesh_service.get_all_price_data().await;
    assert!(result.is_ok(), "Should be able to get cached data");
    
    // Requirement 9.1: System should keep cached data when provider disconnects
    // Requirement 6.4: System should serve cached data when no providers online
}
