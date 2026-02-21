// Permission handling for proximity transfers
// Manages WiFi and Bluetooth permissions across platforms

use crate::{DiscoveryMethod, ProximityError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Permission status for a specific capability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    /// Permission has been granted
    Granted,
    /// Permission has been denied by the user
    Denied,
    /// Permission has not been requested yet
    NotRequested,
    /// Permission is not applicable on this platform
    NotApplicable,
}

/// Manages permissions for proximity discovery
pub struct PermissionManager {
    wifi_permission: Arc<RwLock<PermissionStatus>>,
    bluetooth_permission: Arc<RwLock<PermissionStatus>>,
}

impl PermissionManager {
    /// Create a new PermissionManager
    pub fn new() -> Self {
        Self {
            wifi_permission: Arc::new(RwLock::new(PermissionStatus::NotRequested)),
            bluetooth_permission: Arc::new(RwLock::new(PermissionStatus::NotRequested)),
        }
    }

    /// Request permission for a specific discovery method
    pub async fn request_permission(&self, method: DiscoveryMethod) -> Result<PermissionStatus> {
        match method {
            DiscoveryMethod::WiFi => self.request_wifi_permission().await,
            DiscoveryMethod::Bluetooth => self.request_bluetooth_permission().await,
        }
    }

    /// Request WiFi/network permission
    async fn request_wifi_permission(&self) -> Result<PermissionStatus> {
        info!("Requesting WiFi permission");

        // Check current status
        let current_status = *self.wifi_permission.read().await;
        if current_status != PermissionStatus::NotRequested {
            debug!("WiFi permission already requested: {:?}", current_status);
            return Ok(current_status);
        }

        // Platform-specific permission request
        let status = self.platform_request_wifi_permission().await?;

        // Update stored status
        *self.wifi_permission.write().await = status;

        info!("WiFi permission status: {:?}", status);
        Ok(status)
    }

    /// Request Bluetooth permission
    async fn request_bluetooth_permission(&self) -> Result<PermissionStatus> {
        info!("Requesting Bluetooth permission");

        // Check current status
        let current_status = *self.bluetooth_permission.read().await;
        if current_status != PermissionStatus::NotRequested {
            debug!("Bluetooth permission already requested: {:?}", current_status);
            return Ok(current_status);
        }

        // Platform-specific permission request
        let status = self.platform_request_bluetooth_permission().await?;

        // Update stored status
        *self.bluetooth_permission.write().await = status;

        info!("Bluetooth permission status: {:?}", status);
        Ok(status)
    }

    /// Check if permission is granted for a discovery method
    pub async fn check_permission(&self, method: DiscoveryMethod) -> Result<PermissionStatus> {
        match method {
            DiscoveryMethod::WiFi => Ok(*self.wifi_permission.read().await),
            DiscoveryMethod::Bluetooth => Ok(*self.bluetooth_permission.read().await),
        }
    }

    /// Verify that permission is granted before starting discovery
    pub async fn verify_permission(&self, method: DiscoveryMethod) -> Result<()> {
        let status = self.check_permission(method).await?;

        match status {
            PermissionStatus::Granted => Ok(()),
            PermissionStatus::Denied => {
                Err(ProximityError::PermissionDenied(format!(
                    "{} permission was denied. Please enable it in your device settings.",
                    method
                )))
            }
            PermissionStatus::NotRequested => {
                Err(ProximityError::PermissionDenied(format!(
                    "{} permission has not been requested. Please request permission first.",
                    method
                )))
            }
            PermissionStatus::NotApplicable => {
                Err(ProximityError::PermissionDenied(format!(
                    "{} is not available on this platform.",
                    method
                )))
            }
        }
    }

    /// Handle permission denial gracefully
    pub async fn handle_permission_denial(&self, method: DiscoveryMethod) -> String {
        let message = match method {
            DiscoveryMethod::WiFi => {
                "WiFi discovery requires network access permission. \
                 Please enable WiFi/Network permissions in your device settings to use this feature."
            }
            DiscoveryMethod::Bluetooth => {
                "Bluetooth discovery requires Bluetooth permission. \
                 Please enable Bluetooth permissions in your device settings to use this feature."
            }
        };

        warn!("Permission denied for {}: {}", method, message);
        message.to_string()
    }

    /// Get a link to device settings for permission management
    pub fn get_settings_link(&self) -> String {
        #[cfg(target_os = "ios")]
        {
            "app-settings:".to_string()
        }

        #[cfg(target_os = "android")]
        {
            "android.settings.APPLICATION_DETAILS_SETTINGS".to_string()
        }

        #[cfg(target_arch = "wasm32")]
        {
            "Browser settings".to_string()
        }

        #[cfg(not(any(target_os = "ios", target_os = "android", target_arch = "wasm32")))]
        {
            "System settings".to_string()
        }
    }

    /// Platform-specific WiFi permission request
    #[cfg(target_arch = "wasm32")]
    async fn platform_request_wifi_permission(&self) -> Result<PermissionStatus> {
        // Web platform: WiFi discovery is limited
        // WebRTC doesn't require explicit permission for local network discovery
        // but browser may show permission prompts
        debug!("Web platform: WiFi permission not explicitly required");
        Ok(PermissionStatus::Granted)
    }

    #[cfg(target_os = "ios")]
    async fn platform_request_wifi_permission(&self) -> Result<PermissionStatus> {
        // iOS: Request local network permission
        // In a real implementation, this would use iOS APIs:
        // - NEHotspotConfiguration for WiFi
        // - Local Network permission (iOS 14+)
        debug!("iOS: Requesting local network permission");
        
        // Simulate permission request
        // In production, this would show iOS permission dialog
        Ok(PermissionStatus::Granted)
    }

    #[cfg(target_os = "android")]
    async fn platform_request_wifi_permission(&self) -> Result<PermissionStatus> {
        // Android: Request ACCESS_WIFI_STATE and ACCESS_NETWORK_STATE
        // In a real implementation, this would use Android APIs:
        // - ActivityCompat.requestPermissions()
        // - Manifest.permission.ACCESS_WIFI_STATE
        // - Manifest.permission.ACCESS_NETWORK_STATE
        debug!("Android: Requesting WiFi state permission");
        
        // Simulate permission request
        // In production, this would show Android permission dialog
        Ok(PermissionStatus::Granted)
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android")))]
    async fn platform_request_wifi_permission(&self) -> Result<PermissionStatus> {
        // Desktop platforms: Usually no explicit permission needed
        debug!("Desktop platform: WiFi permission not required");
        Ok(PermissionStatus::Granted)
    }

    /// Platform-specific Bluetooth permission request
    #[cfg(target_arch = "wasm32")]
    async fn platform_request_bluetooth_permission(&self) -> Result<PermissionStatus> {
        // Web platform: Bluetooth not available
        debug!("Web platform: Bluetooth not supported");
        Ok(PermissionStatus::NotApplicable)
    }

    #[cfg(target_os = "ios")]
    async fn platform_request_bluetooth_permission(&self) -> Result<PermissionStatus> {
        // iOS: Request Bluetooth permission
        // In a real implementation, this would use iOS APIs:
        // - CoreBluetooth framework
        // - CBCentralManager authorization
        debug!("iOS: Requesting Bluetooth permission");
        
        // Simulate permission request
        // In production, this would show iOS permission dialog
        Ok(PermissionStatus::Granted)
    }

    #[cfg(target_os = "android")]
    async fn platform_request_bluetooth_permission(&self) -> Result<PermissionStatus> {
        // Android: Request Bluetooth permissions
        // In a real implementation, this would use Android APIs:
        // - Manifest.permission.BLUETOOTH
        // - Manifest.permission.BLUETOOTH_ADMIN
        // - Manifest.permission.BLUETOOTH_SCAN (Android 12+)
        // - Manifest.permission.BLUETOOTH_CONNECT (Android 12+)
        debug!("Android: Requesting Bluetooth permission");
        
        // Simulate permission request
        // In production, this would show Android permission dialog
        Ok(PermissionStatus::Granted)
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android")))]
    async fn platform_request_bluetooth_permission(&self) -> Result<PermissionStatus> {
        // Desktop platforms: Usually no explicit permission needed
        debug!("Desktop platform: Bluetooth permission not required");
        Ok(PermissionStatus::Granted)
    }

    /// Reset permission status (for testing)
    #[cfg(test)]
    pub async fn reset_permissions(&self) {
        *self.wifi_permission.write().await = PermissionStatus::NotRequested;
        *self.bluetooth_permission.write().await = PermissionStatus::NotRequested;
    }

    /// Manually set permission status (for testing)
    pub async fn set_permission(&self, method: DiscoveryMethod, status: PermissionStatus) {
        match method {
            DiscoveryMethod::WiFi => {
                *self.wifi_permission.write().await = status;
            }
            DiscoveryMethod::Bluetooth => {
                *self.bluetooth_permission.write().await = status;
            }
        }
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_manager_creation() {
        let manager = PermissionManager::new();
        
        let wifi_status = manager.check_permission(DiscoveryMethod::WiFi).await.unwrap();
        let bluetooth_status = manager.check_permission(DiscoveryMethod::Bluetooth).await.unwrap();
        
        assert_eq!(wifi_status, PermissionStatus::NotRequested);
        assert_eq!(bluetooth_status, PermissionStatus::NotRequested);
    }

    #[tokio::test]
    async fn test_request_wifi_permission() {
        let manager = PermissionManager::new();
        
        let status = manager.request_permission(DiscoveryMethod::WiFi).await.unwrap();
        
        // On most platforms, WiFi permission should be granted
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(status, PermissionStatus::Granted);
    }

    #[tokio::test]
    async fn test_request_bluetooth_permission() {
        let manager = PermissionManager::new();
        
        let status = manager.request_permission(DiscoveryMethod::Bluetooth).await.unwrap();
        
        // On web, Bluetooth is not applicable
        #[cfg(target_arch = "wasm32")]
        assert_eq!(status, PermissionStatus::NotApplicable);
        
        // On native platforms, should be granted
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(status, PermissionStatus::Granted);
    }

    #[tokio::test]
    async fn test_verify_permission_granted() {
        let manager = PermissionManager::new();
        
        // Request permission first
        manager.request_permission(DiscoveryMethod::WiFi).await.unwrap();
        
        // Verify should succeed
        let result = manager.verify_permission(DiscoveryMethod::WiFi).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_permission_denied() {
        let manager = PermissionManager::new();
        
        // Manually set permission to denied
        manager.set_permission(DiscoveryMethod::WiFi, PermissionStatus::Denied).await;
        
        // Verify should fail
        let result = manager.verify_permission(DiscoveryMethod::WiFi).await;
        assert!(result.is_err());
        
        if let Err(ProximityError::PermissionDenied(msg)) = result {
            assert!(msg.contains("denied"));
        } else {
            panic!("Expected PermissionDenied error");
        }
    }

    #[tokio::test]
    async fn test_verify_permission_not_requested() {
        let manager = PermissionManager::new();
        
        // Don't request permission
        let result = manager.verify_permission(DiscoveryMethod::WiFi).await;
        assert!(result.is_err());
        
        if let Err(ProximityError::PermissionDenied(msg)) = result {
            assert!(msg.contains("not been requested"));
        } else {
            panic!("Expected PermissionDenied error");
        }
    }

    #[tokio::test]
    async fn test_handle_permission_denial() {
        let manager = PermissionManager::new();
        
        let wifi_message = manager.handle_permission_denial(DiscoveryMethod::WiFi).await;
        assert!(wifi_message.contains("WiFi"));
        assert!(wifi_message.contains("settings"));
        
        let bluetooth_message = manager.handle_permission_denial(DiscoveryMethod::Bluetooth).await;
        assert!(bluetooth_message.contains("Bluetooth"));
        assert!(bluetooth_message.contains("settings"));
    }

    #[tokio::test]
    async fn test_get_settings_link() {
        let manager = PermissionManager::new();
        let link = manager.get_settings_link();
        
        // Should return a non-empty string
        assert!(!link.is_empty());
    }

    #[tokio::test]
    async fn test_permission_caching() {
        let manager = PermissionManager::new();
        
        // Request permission once
        let status1 = manager.request_permission(DiscoveryMethod::WiFi).await.unwrap();
        
        // Request again - should return cached status without re-requesting
        let status2 = manager.request_permission(DiscoveryMethod::WiFi).await.unwrap();
        
        assert_eq!(status1, status2);
    }
}
