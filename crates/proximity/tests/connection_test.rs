use proximity::{
    ConnectionQuality, ConnectionType, PeerConnectionManager, PeerMessage, ProximityError,
    RetryConfig,
};
use uuid::Uuid;

#[tokio::test]
async fn test_establish_connection_success() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    let result = manager.establish_connection(peer_id.clone()).await;
    assert!(result.is_ok());

    let connection = result.unwrap();
    assert_eq!(connection.peer_id, peer_id);
    assert!(matches!(
        connection.connection_type,
        ConnectionType::TcpSocket | ConnectionType::WebRTC
    ));
}

#[tokio::test]
async fn test_establish_connection_idempotent() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // First connection
    let result1 = manager.establish_connection(peer_id.clone()).await;
    assert!(result1.is_ok());

    // Second connection should return existing connection
    let result2 = manager.establish_connection(peer_id.clone()).await;
    assert!(result2.is_ok());

    // Verify it's the same peer
    assert_eq!(result1.unwrap().peer_id, result2.unwrap().peer_id);
}

#[tokio::test]
async fn test_send_message_to_connected_peer() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection first
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Send message
    let message = PeerMessage::Ping;
    let result = manager.send_message(peer_id, message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_message_to_nonexistent_peer() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Try to send message without establishing connection
    let message = PeerMessage::Ping;
    let result = manager.send_message(peer_id.clone(), message).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::PeerNotFound(_)));
}

#[tokio::test]
async fn test_close_connection() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Verify connection exists
    assert!(manager.has_connection(&peer_id).await);

    // Close connection
    let result = manager.close_connection(peer_id.clone()).await;
    assert!(result.is_ok());

    // Verify connection is closed
    assert!(!manager.has_connection(&peer_id).await);
}

#[tokio::test]
async fn test_close_nonexistent_connection() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    let result = manager.close_connection(peer_id).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::PeerNotFound(_)));
}

#[tokio::test]
async fn test_measure_quality() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Measure quality
    let result = manager.measure_quality(peer_id).await;
    assert!(result.is_ok());

    let quality = result.unwrap();
    assert!(quality.latency_ms >= 0);
    assert!(quality.packet_loss_percent >= 0.0);
}

#[tokio::test]
async fn test_measure_quality_nonexistent_peer() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    let result = manager.measure_quality(peer_id).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProximityError::PeerNotFound(_)));
}

#[tokio::test]
async fn test_update_signal_strength() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Update signal strength
    let signal_strength = -60i8;
    let result = manager.update_signal_strength(peer_id.clone(), signal_strength).await;
    assert!(result.is_ok());

    // Verify signal strength was updated
    let quality = manager.get_connection(&peer_id).await;
    assert!(quality.is_some());
    assert_eq!(quality.unwrap().signal_strength, Some(signal_strength));
}

#[tokio::test]
async fn test_calculate_quality_score() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Calculate quality score
    let result = manager.calculate_quality_score(&peer_id).await;
    assert!(result.is_ok());

    let score = result.unwrap();
    assert!(score <= 100);
}

#[tokio::test]
async fn test_quality_score_with_poor_latency() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Simulate poor latency by measuring quality multiple times
    // (In real implementation, this would be set by actual network measurements)
    manager.measure_quality(peer_id.clone()).await.unwrap();

    let score = manager.calculate_quality_score(&peer_id).await.unwrap();
    // Score should be reasonable (not 0, not 100)
    assert!(score > 0 && score <= 100);
}

#[tokio::test]
async fn test_is_quality_poor() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Check if quality is poor
    let result = manager.is_quality_poor(&peer_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_active_connections() {
    let manager = PeerConnectionManager::new();

    // Initially no connections
    let connections = manager.get_active_connections().await;
    assert_eq!(connections.len(), 0);

    // Establish multiple connections
    let peer1 = format!("peer_{}", Uuid::new_v4());
    let peer2 = format!("peer_{}", Uuid::new_v4());

    manager.establish_connection(peer1.clone()).await.unwrap();
    manager.establish_connection(peer2.clone()).await.unwrap();

    // Should have 2 connections
    let connections = manager.get_active_connections().await;
    assert_eq!(connections.len(), 2);
    assert!(connections.contains(&peer1));
    assert!(connections.contains(&peer2));
}

#[tokio::test]
async fn test_has_connection() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Initially no connection
    assert!(!manager.has_connection(&peer_id).await);

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Now should have connection
    assert!(manager.has_connection(&peer_id).await);
}

#[tokio::test]
async fn test_get_connection() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Initially no connection
    assert!(manager.get_connection(&peer_id).await.is_none());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Now should have connection info
    let quality = manager.get_connection(&peer_id).await;
    assert!(quality.is_some());
}

#[tokio::test]
async fn test_custom_retry_config() {
    let retry_config = RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 50,
        max_backoff_ms: 2000,
    };

    let manager = PeerConnectionManager::with_retry_config(retry_config);
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Should still be able to establish connection
    let result = manager.establish_connection(peer_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_measure_quality_with_ping() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Measure quality with ping/pong
    let result = manager.measure_quality_with_ping(peer_id).await;
    assert!(result.is_ok());

    let quality = result.unwrap();
    assert!(quality.latency_ms > 0); // Should have measured some latency
}

#[tokio::test]
async fn test_send_all_message_types() {
    let manager = PeerConnectionManager::new();
    let peer_id = format!("peer_{}", Uuid::new_v4());

    // Establish connection
    manager.establish_connection(peer_id.clone()).await.unwrap();

    // Test different message types
    let messages = vec![
        PeerMessage::Ping,
        PeerMessage::Pong,
        PeerMessage::Challenge {
            nonce: vec![1, 2, 3, 4],
        },
        PeerMessage::ChallengeResponse {
            signature: vec![5, 6, 7, 8],
            public_key: vec![9, 10, 11, 12],
        },
    ];

    for message in messages {
        let result = manager.send_message(peer_id.clone(), message).await;
        assert!(result.is_ok());
    }
}
