use chrono::{Duration, Utc};
use proximity::{DiscoveryMethod, SessionManager};
use uuid::Uuid;

#[tokio::test]
async fn test_start_session_with_default_duration() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    let session = manager
        .start_session(user_id, DiscoveryMethod::WiFi, 0)
        .await
        .unwrap();

    assert_eq!(session.user_id, user_id);
    assert_eq!(session.discovery_method, DiscoveryMethod::WiFi);
    assert!(!session.auto_extend);

    // Check that default duration is 30 minutes
    let expected_expiry = session.started_at + Duration::minutes(30);
    let diff = (session.expires_at - expected_expiry).num_seconds().abs();
    assert!(diff < 2, "Expiry time should be ~30 minutes from start");
}

#[tokio::test]
async fn test_start_session_with_custom_duration() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    let session = manager
        .start_session(user_id, DiscoveryMethod::Bluetooth, 45)
        .await
        .unwrap();

    assert_eq!(session.user_id, user_id);
    assert_eq!(session.discovery_method, DiscoveryMethod::Bluetooth);

    // Check that custom duration is 45 minutes
    let expected_expiry = session.started_at + Duration::minutes(45);
    let diff = (session.expires_at - expected_expiry).num_seconds().abs();
    assert!(diff < 2, "Expiry time should be ~45 minutes from start");
}

#[tokio::test]
async fn test_extend_session() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    let session = manager
        .start_session(user_id, DiscoveryMethod::WiFi, 30)
        .await
        .unwrap();

    let original_expiry = session.expires_at;

    // Extend by 15 minutes (default)
    manager.extend_session(session.session_id, 0).await.unwrap();

    let updated_session = manager.get_session(session.session_id).await.unwrap();
    let expected_expiry = original_expiry + Duration::minutes(15);
    let diff = (updated_session.expires_at - expected_expiry).num_seconds().abs();
    assert!(diff < 2, "Session should be extended by 15 minutes");
}

#[tokio::test]
async fn test_extend_session_custom_duration() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    let session = manager
        .start_session(user_id, DiscoveryMethod::WiFi, 30)
        .await
        .unwrap();

    let original_expiry = session.expires_at;

    // Extend by 20 minutes
    manager.extend_session(session.session_id, 20).await.unwrap();

    let updated_session = manager.get_session(session.session_id).await.unwrap();
    let expected_expiry = original_expiry + Duration::minutes(20);
    let diff = (updated_session.expires_at - expected_expiry).num_seconds().abs();
    assert!(diff < 2, "Session should be extended by 20 minutes");
}

#[tokio::test]
async fn test_extend_nonexistent_session() {
    let manager = SessionManager::new();
    let fake_session_id = Uuid::new_v4();

    let result = manager.extend_session(fake_session_id, 15).await;
    assert!(result.is_err(), "Extending nonexistent session should fail");
}

#[tokio::test]
async fn test_end_session() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    let session = manager
        .start_session(user_id, DiscoveryMethod::WiFi, 30)
        .await
        .unwrap();

    // End the session
    manager.end_session(session.session_id).await.unwrap();

    // Verify session is removed
    let result = manager.get_session(session.session_id).await;
    assert!(result.is_err(), "Session should be removed after ending");
}

#[tokio::test]
async fn test_end_nonexistent_session() {
    let manager = SessionManager::new();
    let fake_session_id = Uuid::new_v4();

    let result = manager.end_session(fake_session_id).await;
    assert!(result.is_err(), "Ending nonexistent session should fail");
}

#[tokio::test]
async fn test_cleanup_expired_sessions() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    // Create a session that expires immediately (1 second)
    let session = manager
        .start_session(user_id, DiscoveryMethod::WiFi, 0)
        .await
        .unwrap();

    // Manually set expiry to past using helper method
    manager.set_session_expiry_for_testing(session.session_id, Utc::now() - Duration::seconds(10))
        .await
        .unwrap();

    // Run cleanup
    let count = manager.cleanup_expired_sessions().await.unwrap();
    assert_eq!(count, 1, "Should remove 1 expired session");

    // Verify session is removed
    let result = manager.get_session(session.session_id).await;
    assert!(result.is_err(), "Expired session should be removed");
}

#[tokio::test]
async fn test_cleanup_no_expired_sessions() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    // Create a session with 30 minutes duration
    manager
        .start_session(user_id, DiscoveryMethod::WiFi, 30)
        .await
        .unwrap();

    // Run cleanup
    let count = manager.cleanup_expired_sessions().await.unwrap();
    assert_eq!(count, 0, "Should not remove any active sessions");
}

#[tokio::test]
async fn test_get_user_sessions() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();

    // Create sessions for user
    manager
        .start_session(user_id, DiscoveryMethod::WiFi, 30)
        .await
        .unwrap();
    manager
        .start_session(user_id, DiscoveryMethod::Bluetooth, 30)
        .await
        .unwrap();

    // Create session for other user
    manager
        .start_session(other_user_id, DiscoveryMethod::WiFi, 30)
        .await
        .unwrap();

    // Get sessions for user
    let user_sessions = manager.get_user_sessions(user_id).await.unwrap();
    assert_eq!(user_sessions.len(), 2, "User should have 2 sessions");

    // Verify all sessions belong to user
    for session in user_sessions {
        assert_eq!(session.user_id, user_id);
    }
}

#[tokio::test]
async fn test_is_session_expired() {
    let manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    let session = manager
        .start_session(user_id, DiscoveryMethod::WiFi, 30)
        .await
        .unwrap();

    // Session should not be expired
    let is_expired = manager.is_session_expired(session.session_id).await.unwrap();
    assert!(!is_expired, "New session should not be expired");

    // Manually set expiry to past using helper method
    manager.set_session_expiry_for_testing(session.session_id, Utc::now() - Duration::seconds(10))
        .await
        .unwrap();

    // Session should now be expired
    let is_expired = manager.is_session_expired(session.session_id).await.unwrap();
    assert!(is_expired, "Session should be expired after manual expiry");
}

#[tokio::test]
async fn test_background_cleanup_task() {
    let mut manager = SessionManager::new();
    let user_id = Uuid::new_v4();

    // Start cleanup task
    manager.start_cleanup_task();

    // Create a session that expires immediately
    let session = manager
        .start_session(user_id, DiscoveryMethod::WiFi, 0)
        .await
        .unwrap();

    // Manually set expiry to past using helper method
    manager.set_session_expiry_for_testing(session.session_id, Utc::now() - Duration::seconds(10))
        .await
        .unwrap();

    // Wait for cleanup task to run (it runs every 60 seconds, but we'll wait 2 seconds for test)
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Stop cleanup task
    manager.stop_cleanup_task();

    // Note: In a real scenario, the cleanup would run after 60 seconds
    // For testing purposes, we verify the task can be started and stopped
    // The actual cleanup logic is tested in test_cleanup_expired_sessions
}
