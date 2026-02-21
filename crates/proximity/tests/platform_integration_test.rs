// Integration tests for platform-specific adaptations
// Tests platform abstraction, permissions, and lifecycle management

use proximity::{
    AppState, DiscoveryMethod, LifecycleManager, PermissionManager, PermissionStatus,
    get_default_factory,
};

#[tokio::test]
async fn test_platform_factory_creation() {
    let factory = get_default_factory();
    
    // Factory should be created successfully
    assert!(!factory.platform_name().is_empty());
    
    // Should be able to create a connection
    let peer_id = "test-peer".to_string();
    let connection = factory.create_connection(peer_id.clone()).await;
    assert!(connection.is_ok());
}

#[tokio::test]
async fn test_permission_before_discovery() {
    let permission_manager = PermissionManager::new();
    
    // Check initial state
    let wifi_status = permission_manager.check_permission(DiscoveryMethod::WiFi).await.unwrap();
    assert_eq!(wifi_status, PermissionStatus::NotRequested);
    
    // Request permission
    let status = permission_manager.request_permission(DiscoveryMethod::WiFi).await.unwrap();
    
    // On most platforms, should be granted
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(status, PermissionStatus::Granted);
    
    // Verify permission
    let verify_result = permission_manager.verify_permission(DiscoveryMethod::WiFi).await;
    assert!(verify_result.is_ok());
}

#[tokio::test]
async fn test_lifecycle_with_discovery() {
    let lifecycle_manager = LifecycleManager::new();
    
    // Start in foreground
    assert_eq!(lifecycle_manager.get_state().await, AppState::Foreground);
    
    // Enable restore preference
    lifecycle_manager.set_restore_on_foreground(true).await;
    
    // Simulate discovery active, then move to background
    lifecycle_manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
    assert_eq!(lifecycle_manager.get_state().await, AppState::Background);
    
    // Return to foreground immediately (within timeout)
    let restored_method = lifecycle_manager.on_foreground().await.unwrap();
    assert_eq!(restored_method, Some(DiscoveryMethod::WiFi));
    assert_eq!(lifecycle_manager.get_state().await, AppState::Foreground);
}

#[tokio::test]
async fn test_permission_denial_handling() {
    let permission_manager = PermissionManager::new();
    
    // Manually set permission to denied (simulating user denial)
    permission_manager.set_permission(DiscoveryMethod::Bluetooth, PermissionStatus::Denied).await;
    
    // Verify should fail
    let result = permission_manager.verify_permission(DiscoveryMethod::Bluetooth).await;
    assert!(result.is_err());
    
    // Get user-friendly message
    let message = permission_manager.handle_permission_denial(DiscoveryMethod::Bluetooth).await;
    assert!(message.contains("Bluetooth"));
    assert!(message.contains("settings"));
    
    // Get settings link
    let link = permission_manager.get_settings_link();
    assert!(!link.is_empty());
}

#[tokio::test]
async fn test_background_timeout_disables_discovery() {
    // Use very short timeout for testing
    let lifecycle_manager = LifecycleManager::with_timeout(0);
    lifecycle_manager.set_restore_on_foreground(true).await;
    
    // Move to background with active discovery
    lifecycle_manager.on_background(true, Some(DiscoveryMethod::Bluetooth)).await.unwrap();
    
    // Wait a bit to exceed timeout
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Should indicate discovery should be disabled
    assert!(lifecycle_manager.should_disable_discovery().await);
    
    // Return to foreground after timeout
    let restored_method = lifecycle_manager.on_foreground().await.unwrap();
    
    // Should not restore due to timeout
    assert!(restored_method.is_none());
}

#[tokio::test]
async fn test_complete_discovery_flow_with_permissions_and_lifecycle() {
    let permission_manager = PermissionManager::new();
    let lifecycle_manager = LifecycleManager::new();
    
    // Step 1: Request permission
    let permission_status = permission_manager.request_permission(DiscoveryMethod::WiFi).await.unwrap();
    
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(permission_status, PermissionStatus::Granted);
    
    // Step 2: Verify permission before starting discovery
    let verify_result = permission_manager.verify_permission(DiscoveryMethod::WiFi).await;
    assert!(verify_result.is_ok());
    
    // Step 3: Start discovery (simulated)
    assert_eq!(lifecycle_manager.get_state().await, AppState::Foreground);
    
    // Step 4: Enable restore preference
    lifecycle_manager.set_restore_on_foreground(true).await;
    
    // Step 5: App goes to background
    lifecycle_manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
    assert_eq!(lifecycle_manager.get_state().await, AppState::Background);
    
    // Step 6: App returns to foreground within timeout
    let restored_method = lifecycle_manager.on_foreground().await.unwrap();
    assert_eq!(restored_method, Some(DiscoveryMethod::WiFi));
    
    // Step 7: Verify permission still valid
    let verify_again = permission_manager.verify_permission(DiscoveryMethod::WiFi).await;
    assert!(verify_again.is_ok());
}

#[tokio::test]
async fn test_platform_connection_with_different_methods() {
    let factory = get_default_factory();
    
    // Test WiFi peer connection
    let wifi_peer = "wifi-peer-123".to_string();
    let wifi_connection = factory.create_connection(wifi_peer.clone()).await;
    assert!(wifi_connection.is_ok());
    
    // Test Bluetooth peer connection
    let ble_peer = "ble-peer-456".to_string();
    let ble_connection = factory.create_connection(ble_peer.clone()).await;
    assert!(ble_connection.is_ok());
}

#[tokio::test]
async fn test_permission_not_requested_error() {
    let permission_manager = PermissionManager::new();
    
    // Try to verify without requesting
    let result = permission_manager.verify_permission(DiscoveryMethod::WiFi).await;
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not been requested"));
}

#[tokio::test]
async fn test_lifecycle_without_restore_preference() {
    let lifecycle_manager = LifecycleManager::new();
    
    // Don't set restore preference (defaults to false)
    assert!(!lifecycle_manager.should_restore_on_foreground().await);
    
    // Move to background with active discovery
    lifecycle_manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
    
    // Return to foreground
    let restored_method = lifecycle_manager.on_foreground().await.unwrap();
    
    // Should not restore because preference is false
    assert!(restored_method.is_none());
}

#[tokio::test]
async fn test_multiple_permission_requests() {
    let permission_manager = PermissionManager::new();
    
    // Request WiFi permission
    let wifi_status = permission_manager.request_permission(DiscoveryMethod::WiFi).await.unwrap();
    
    // Request Bluetooth permission
    let bluetooth_status = permission_manager.request_permission(DiscoveryMethod::Bluetooth).await.unwrap();
    
    // Both should have valid statuses
    assert_ne!(wifi_status, PermissionStatus::NotRequested);
    assert_ne!(bluetooth_status, PermissionStatus::NotRequested);
    
    // On web, Bluetooth should be not applicable
    #[cfg(target_arch = "wasm32")]
    assert_eq!(bluetooth_status, PermissionStatus::NotApplicable);
}

#[tokio::test]
async fn test_lifecycle_clear_state() {
    let lifecycle_manager = LifecycleManager::new();
    lifecycle_manager.set_restore_on_foreground(true).await;
    
    // Move to background with active discovery
    lifecycle_manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
    
    // Clear state
    lifecycle_manager.clear_state().await;
    
    // Return to foreground
    let restored_method = lifecycle_manager.on_foreground().await.unwrap();
    
    // Should not restore because state was cleared
    assert!(restored_method.is_none());
}
