// Application lifecycle management for proximity transfers
// Handles background/foreground transitions and automatic discovery management

use crate::{DiscoveryMethod, Result};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Application state for lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Application is in the foreground and active
    Foreground,
    /// Application is in the background
    Background,
}

/// Discovery state before backgrounding
#[derive(Debug, Clone)]
struct DiscoveryState {
    was_active: bool,
    method: Option<DiscoveryMethod>,
    backgrounded_at: DateTime<Utc>,
}

/// Manages application lifecycle and discovery state
pub struct LifecycleManager {
    app_state: Arc<RwLock<AppState>>,
    discovery_state: Arc<RwLock<Option<DiscoveryState>>>,
    background_timeout_minutes: i64,
    restore_on_foreground: Arc<RwLock<bool>>,
}

impl LifecycleManager {
    /// Create a new LifecycleManager with default 5-minute background timeout
    pub fn new() -> Self {
        Self {
            app_state: Arc::new(RwLock::new(AppState::Foreground)),
            discovery_state: Arc::new(RwLock::new(None)),
            background_timeout_minutes: 5,
            restore_on_foreground: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new LifecycleManager with custom background timeout
    pub fn with_timeout(timeout_minutes: i64) -> Self {
        Self {
            app_state: Arc::new(RwLock::new(AppState::Foreground)),
            discovery_state: Arc::new(RwLock::new(None)),
            background_timeout_minutes: timeout_minutes,
            restore_on_foreground: Arc::new(RwLock::new(false)),
        }
    }

    /// Get the current application state
    pub async fn get_state(&self) -> AppState {
        *self.app_state.read().await
    }

    /// Set whether to restore discovery state when returning to foreground
    pub async fn set_restore_on_foreground(&self, restore: bool) {
        *self.restore_on_foreground.write().await = restore;
        debug!("Restore on foreground set to: {}", restore);
    }

    /// Get whether discovery should be restored on foreground
    pub async fn should_restore_on_foreground(&self) -> bool {
        *self.restore_on_foreground.read().await
    }

    /// Handle application moving to background
    /// 
    /// **Validates: Requirements 16.4**
    pub async fn on_background(&self, discovery_active: bool, method: Option<DiscoveryMethod>) -> Result<()> {
        info!("Application moving to background");

        // Update app state
        *self.app_state.write().await = AppState::Background;

        // Save discovery state if active
        if discovery_active {
            let state = DiscoveryState {
                was_active: true,
                method,
                backgrounded_at: Utc::now(),
            };
            *self.discovery_state.write().await = Some(state);
            debug!("Saved discovery state: {:?}", method);
        }

        Ok(())
    }

    /// Handle application returning to foreground
    /// 
    /// **Validates: Requirements 16.5**
    pub async fn on_foreground(&self) -> Result<Option<DiscoveryMethod>> {
        info!("Application returning to foreground");

        // Update app state
        *self.app_state.write().await = AppState::Foreground;

        // Check if we should restore discovery
        let should_restore = *self.restore_on_foreground.read().await;
        if !should_restore {
            debug!("Discovery restoration disabled by user preference");
            *self.discovery_state.write().await = None;
            return Ok(None);
        }

        // Check saved discovery state
        let discovery_state = self.discovery_state.read().await.clone();
        if let Some(state) = discovery_state {
            if state.was_active {
                debug!("Discovery was active before backgrounding, checking timeout");
                
                // Check if background timeout has been exceeded
                let now = Utc::now();
                let background_duration = now.signed_duration_since(state.backgrounded_at);
                
                if background_duration > Duration::minutes(self.background_timeout_minutes) {
                    warn!(
                        "Background timeout exceeded ({} minutes), not restoring discovery",
                        self.background_timeout_minutes
                    );
                    *self.discovery_state.write().await = None;
                    return Ok(None);
                }

                info!("Restoring discovery state: {:?}", state.method);
                *self.discovery_state.write().await = None;
                return Ok(state.method);
            }
        }

        debug!("No discovery state to restore");
        Ok(None)
    }

    /// Check if discovery should be disabled due to background timeout
    /// 
    /// **Validates: Requirements 16.4**
    pub async fn should_disable_discovery(&self) -> bool {
        let app_state = *self.app_state.read().await;
        if app_state != AppState::Background {
            return false;
        }

        let discovery_state = self.discovery_state.read().await;
        if let Some(state) = discovery_state.as_ref() {
            let now = Utc::now();
            let background_duration = now.signed_duration_since(state.backgrounded_at);
            
            if background_duration > Duration::minutes(self.background_timeout_minutes) {
                debug!(
                    "Background timeout exceeded: {} minutes > {} minutes",
                    background_duration.num_minutes(),
                    self.background_timeout_minutes
                );
                return true;
            }
        }

        false
    }

    /// Get time remaining before background timeout
    pub async fn time_until_timeout(&self) -> Option<Duration> {
        let app_state = *self.app_state.read().await;
        if app_state != AppState::Background {
            return None;
        }

        let discovery_state = self.discovery_state.read().await;
        if let Some(state) = discovery_state.as_ref() {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(state.backgrounded_at);
            let timeout = Duration::minutes(self.background_timeout_minutes);
            
            if elapsed < timeout {
                return Some(timeout - elapsed);
            }
        }

        None
    }

    /// Clear saved discovery state
    pub async fn clear_state(&self) {
        *self.discovery_state.write().await = None;
        debug!("Cleared discovery state");
    }

    /// Check if app is in background
    pub async fn is_background(&self) -> bool {
        *self.app_state.read().await == AppState::Background
    }

    /// Check if app is in foreground
    pub async fn is_foreground(&self) -> bool {
        *self.app_state.read().await == AppState::Foreground
    }

    /// Get background timeout in minutes
    pub fn get_timeout_minutes(&self) -> i64 {
        self.background_timeout_minutes
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
        let manager = LifecycleManager::new();
        
        assert_eq!(manager.get_state().await, AppState::Foreground);
        assert!(!manager.should_restore_on_foreground().await);
        assert_eq!(manager.get_timeout_minutes(), 5);
    }

    #[tokio::test]
    async fn test_custom_timeout() {
        let manager = LifecycleManager::with_timeout(10);
        
        assert_eq!(manager.get_timeout_minutes(), 10);
    }

    #[tokio::test]
    async fn test_background_transition() {
        let manager = LifecycleManager::new();
        
        // Move to background with active discovery
        manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
        
        assert_eq!(manager.get_state().await, AppState::Background);
        assert!(manager.is_background().await);
        assert!(!manager.is_foreground().await);
    }

    #[tokio::test]
    async fn test_foreground_transition_without_restore() {
        let manager = LifecycleManager::new();
        
        // Move to background with active discovery
        manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
        
        // Move to foreground without restore preference
        let restored_method = manager.on_foreground().await.unwrap();
        
        assert_eq!(manager.get_state().await, AppState::Foreground);
        assert!(restored_method.is_none());
    }

    #[tokio::test]
    async fn test_foreground_transition_with_restore() {
        let manager = LifecycleManager::new();
        
        // Enable restore on foreground
        manager.set_restore_on_foreground(true).await;
        
        // Move to background with active discovery
        manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
        
        // Move to foreground immediately (within timeout)
        let restored_method = manager.on_foreground().await.unwrap();
        
        assert_eq!(manager.get_state().await, AppState::Foreground);
        assert_eq!(restored_method, Some(DiscoveryMethod::WiFi));
    }

    #[tokio::test]
    async fn test_background_timeout() {
        // Use a very short timeout for testing (1 second = 0.0167 minutes)
        let manager = LifecycleManager::with_timeout(0);
        manager.set_restore_on_foreground(true).await;
        
        // Move to background with active discovery
        manager.on_background(true, Some(DiscoveryMethod::Bluetooth)).await.unwrap();
        
        // Wait for timeout to exceed
        sleep(TokioDuration::from_millis(100)).await;
        
        // Check if discovery should be disabled
        assert!(manager.should_disable_discovery().await);
        
        // Move to foreground after timeout
        let restored_method = manager.on_foreground().await.unwrap();
        
        // Should not restore due to timeout
        assert!(restored_method.is_none());
    }

    #[tokio::test]
    async fn test_background_without_active_discovery() {
        let manager = LifecycleManager::new();
        manager.set_restore_on_foreground(true).await;
        
        // Move to background without active discovery
        manager.on_background(false, None).await.unwrap();
        
        // Move to foreground
        let restored_method = manager.on_foreground().await.unwrap();
        
        // Should not restore anything
        assert!(restored_method.is_none());
    }

    #[tokio::test]
    async fn test_time_until_timeout() {
        let manager = LifecycleManager::with_timeout(5);
        
        // No timeout in foreground
        assert!(manager.time_until_timeout().await.is_none());
        
        // Move to background
        manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
        
        // Should have time remaining
        let remaining = manager.time_until_timeout().await;
        assert!(remaining.is_some());
        
        let remaining_minutes = remaining.unwrap().num_minutes();
        assert!(remaining_minutes <= 5);
        assert!(remaining_minutes >= 4); // Allow for small timing variations
    }

    #[tokio::test]
    async fn test_clear_state() {
        let manager = LifecycleManager::new();
        manager.set_restore_on_foreground(true).await;
        
        // Move to background with active discovery
        manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
        
        // Clear state
        manager.clear_state().await;
        
        // Move to foreground
        let restored_method = manager.on_foreground().await.unwrap();
        
        // Should not restore due to cleared state
        assert!(restored_method.is_none());
    }

    #[tokio::test]
    async fn test_should_disable_discovery_in_foreground() {
        let manager = LifecycleManager::new();
        
        // Should not disable in foreground
        assert!(!manager.should_disable_discovery().await);
    }

    #[tokio::test]
    async fn test_multiple_background_foreground_cycles() {
        let manager = LifecycleManager::new();
        manager.set_restore_on_foreground(true).await;
        
        // First cycle
        manager.on_background(true, Some(DiscoveryMethod::WiFi)).await.unwrap();
        let method1 = manager.on_foreground().await.unwrap();
        assert_eq!(method1, Some(DiscoveryMethod::WiFi));
        
        // Second cycle with different method
        manager.on_background(true, Some(DiscoveryMethod::Bluetooth)).await.unwrap();
        let method2 = manager.on_foreground().await.unwrap();
        assert_eq!(method2, Some(DiscoveryMethod::Bluetooth));
        
        // Third cycle without active discovery
        manager.on_background(false, None).await.unwrap();
        let method3 = manager.on_foreground().await.unwrap();
        assert!(method3.is_none());
    }
}
