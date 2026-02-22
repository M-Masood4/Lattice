//! Network connectivity monitoring for auto-settlement
//!
//! This module provides a cross-platform network connectivity monitor that tracks
//! online/offline status and notifies registered callbacks when connectivity changes.
//!
//! Platform-specific implementations can be added via the `PlatformMonitor` trait.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Network connectivity monitor
///
/// Monitors network connectivity status and invokes callbacks when status changes.
/// Uses atomic bool for thread-safe status checks and supports multiple callbacks.
pub struct NetworkMonitor {
    is_online: Arc<AtomicBool>,
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(bool) + Send + Sync>>>>,
}

impl NetworkMonitor {
    /// Create a new network monitor
    ///
    /// Initializes with online status set to true (optimistic default).
    /// Call `start()` to begin monitoring connectivity.
    pub fn new() -> Self {
        Self {
            is_online: Arc::new(AtomicBool::new(true)),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start monitoring network connectivity
    ///
    /// Spawns a background thread that periodically checks connectivity.
    /// In production, this should be replaced with platform-specific APIs:
    /// - iOS: NWPathMonitor
    /// - Android: ConnectivityManager with NetworkCallback
    ///
    /// Current implementation uses a simple polling approach for cross-platform compatibility.
    pub fn start(&self) {
        let is_online = Arc::clone(&self.is_online);
        let callbacks = Arc::clone(&self.callbacks);

        thread::spawn(move || {
            let mut previous_status = is_online.load(Ordering::Relaxed);

            loop {
                // Check connectivity (simplified implementation)
                // In production, use platform-specific APIs
                let current_status = check_connectivity();

                // Update status if changed
                if current_status != previous_status {
                    is_online.store(current_status, Ordering::Relaxed);

                    // Invoke all registered callbacks
                    if let Ok(callbacks_guard) = callbacks.lock() {
                        for callback in callbacks_guard.iter() {
                            callback(current_status);
                        }
                    }

                    previous_status = current_status;
                }

                // Poll every 5 seconds
                thread::sleep(Duration::from_secs(5));
            }
        });
    }

    /// Check if currently online
    ///
    /// Returns the current connectivity status.
    /// This is a non-blocking operation using atomic load.
    pub fn is_online(&self) -> bool {
        self.is_online.load(Ordering::Relaxed)
    }

    /// Register callback for connectivity changes
    ///
    /// The callback will be invoked whenever connectivity status changes.
    /// Callback receives a boolean: true for online, false for offline.
    ///
    /// # Arguments
    /// * `callback` - Function to call on connectivity changes
    pub fn on_connectivity_change<F>(&mut self, callback: F)
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks_guard) = self.callbacks.lock() {
            callbacks_guard.push(Box::new(callback));
        }
    }

    /// Set online status (for testing)
    ///
    /// This method is primarily for testing purposes to simulate
    /// connectivity changes without waiting for actual network events.
    #[cfg(test)]
    pub fn set_online(&self, online: bool) {
        self.is_online.store(online, Ordering::Relaxed);
    }
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Check network connectivity
///
/// Simplified implementation that attempts to resolve a well-known hostname.
/// In production, this should be replaced with platform-specific APIs.
///
/// Returns true if online, false if offline.
fn check_connectivity() -> bool {
    // Simple connectivity check using DNS resolution
    // This is a basic implementation - platform-specific code should use:
    // - iOS: NWPathMonitor for real-time path updates
    // - Android: ConnectivityManager.NetworkCallback for network state changes
    use std::net::ToSocketAddrs;

    // Try to resolve a well-known hostname
    "www.google.com:80"
        .to_socket_addrs()
        .map(|mut addrs| addrs.next().is_some())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_creates_monitor_with_default_online_status() {
        let monitor = NetworkMonitor::new();
        // Default status should be online (optimistic)
        assert!(monitor.is_online());
    }

    #[test]
    fn test_is_online_returns_current_status() {
        let monitor = NetworkMonitor::new();
        
        // Initially online
        assert!(monitor.is_online());
        
        // Manually set to offline for testing
        monitor.is_online.store(false, Ordering::Relaxed);
        assert!(!monitor.is_online());
        
        // Set back to online
        monitor.is_online.store(true, Ordering::Relaxed);
        assert!(monitor.is_online());
    }

    #[test]
    fn test_callback_invocation_on_status_change() {
        let mut monitor = NetworkMonitor::new();
        
        // Track callback invocations
        let callback_invoked = Arc::new(AtomicBool::new(false));
        let callback_status = Arc::new(AtomicBool::new(true));
        
        let callback_invoked_clone = Arc::clone(&callback_invoked);
        let callback_status_clone = Arc::clone(&callback_status);
        
        // Register callback
        monitor.on_connectivity_change(move |status| {
            callback_invoked_clone.store(true, Ordering::Relaxed);
            callback_status_clone.store(status, Ordering::Relaxed);
        });
        
        // Simulate status change by manually invoking callbacks
        monitor.is_online.store(false, Ordering::Relaxed);
        
        // Manually trigger callbacks (simulating what start() does)
        if let Ok(callbacks_guard) = monitor.callbacks.lock() {
            for callback in callbacks_guard.iter() {
                callback(false);
            }
        }
        
        // Verify callback was invoked with correct status
        assert!(callback_invoked.load(Ordering::Relaxed));
        assert!(!callback_status.load(Ordering::Relaxed));
    }

    #[test]
    fn test_multiple_callbacks_registration() {
        let mut monitor = NetworkMonitor::new();
        
        let callback1_invoked = Arc::new(AtomicBool::new(false));
        let callback2_invoked = Arc::new(AtomicBool::new(false));
        
        let callback1_clone = Arc::clone(&callback1_invoked);
        let callback2_clone = Arc::clone(&callback2_invoked);
        
        // Register multiple callbacks
        monitor.on_connectivity_change(move |_| {
            callback1_clone.store(true, Ordering::Relaxed);
        });
        
        monitor.on_connectivity_change(move |_| {
            callback2_clone.store(true, Ordering::Relaxed);
        });
        
        // Trigger callbacks
        if let Ok(callbacks_guard) = monitor.callbacks.lock() {
            for callback in callbacks_guard.iter() {
                callback(true);
            }
        }
        
        // Verify both callbacks were invoked
        assert!(callback1_invoked.load(Ordering::Relaxed));
        assert!(callback2_invoked.load(Ordering::Relaxed));
    }

    #[test]
    fn test_connectivity_change_detection() {
        let monitor = NetworkMonitor::new();
        
        // Test that we can detect status changes
        let initial_status = monitor.is_online();
        
        // Change status
        monitor.is_online.store(!initial_status, Ordering::Relaxed);
        let new_status = monitor.is_online();
        
        // Verify status changed
        assert_ne!(initial_status, new_status);
    }

    #[test]
    fn test_start_spawns_monitoring_thread() {
        let monitor = NetworkMonitor::new();
        
        // Start monitoring (spawns background thread)
        monitor.start();
        
        // Give the thread a moment to start
        thread::sleep(Duration::from_millis(100));
        
        // The monitor should still be accessible and functional
        let _status = monitor.is_online();
        
        // Test passes if no panic occurs
    }

    #[test]
    fn test_default_implementation() {
        let monitor = NetworkMonitor::default();
        assert!(monitor.is_online());
    }
}
