# Platform-Specific Adaptations Implementation

This document describes the platform-specific adaptations implemented for the proximity-based P2P transfer feature.

## Overview

Task 17 implements platform-specific functionality to ensure the proximity transfer feature works across web, mobile, and desktop platforms. The implementation includes:

1. Platform abstraction layer for connections
2. Permission handling for WiFi and Bluetooth
3. Background/foreground lifecycle management

## Components

### 1. Platform Abstraction Layer (`platform.rs`)

**Purpose**: Provides a unified interface for platform-specific connection implementations.

**Key Features**:
- `PlatformConnection` trait: Defines common connection operations
- `PlatformConnectionFactory` trait: Creates platform-specific connections
- WebRTC implementation for web platform (WASM)
- Native TCP socket implementation for mobile/desktop

**Platform Detection**:
```rust
#[cfg(target_arch = "wasm32")]
// WebRTC for web

#[cfg(not(target_arch = "wasm32"))]
// Native sockets for mobile/desktop
```

**Usage**:
```rust
let factory = get_default_factory();
let connection = factory.create_connection(peer_id).await?;
connection.connect(&peer_id).await?;
connection.send(&message).await?;
```

**Validates Requirements**: 8.1, 8.3, 8.4

### 2. Permission Management (`permissions.rs`)

**Purpose**: Handles platform-specific permission requests for WiFi and Bluetooth discovery.

**Key Features**:
- Request permissions before starting discovery
- Check permission status
- Verify permissions are granted
- Handle permission denial gracefully
- Provide user-friendly error messages
- Platform-specific permission implementations

**Permission States**:
- `Granted`: Permission has been granted
- `Denied`: Permission has been denied by the user
- `NotRequested`: Permission has not been requested yet
- `NotApplicable`: Permission is not applicable on this platform

**Platform-Specific Behavior**:
- **Web**: WiFi granted by default, Bluetooth not applicable
- **iOS**: Requests local network and Bluetooth permissions
- **Android**: Requests WiFi state and Bluetooth permissions
- **Desktop**: Permissions granted by default (no explicit permission needed)

**Usage**:
```rust
let permission_manager = PermissionManager::new();

// Request permission
let status = permission_manager.request_permission(DiscoveryMethod::WiFi).await?;

// Verify before starting discovery
permission_manager.verify_permission(DiscoveryMethod::WiFi).await?;

// Handle denial
if status == PermissionStatus::Denied {
    let message = permission_manager.handle_permission_denial(DiscoveryMethod::WiFi).await;
    let settings_link = permission_manager.get_settings_link();
}
```

**Validates Requirements**: 11.1, 11.3, 11.5

### 3. Lifecycle Management (`lifecycle.rs`)

**Purpose**: Manages application lifecycle transitions and automatic discovery management.

**Key Features**:
- Track application state (foreground/background)
- Save discovery state when backgrounding
- Restore discovery state when returning to foreground
- Automatic timeout after 5 minutes in background
- User preference for restoration

**Application States**:
- `Foreground`: Application is active
- `Background`: Application is in the background

**Background Timeout**:
- Default: 5 minutes
- Configurable via `LifecycleManager::with_timeout(minutes)`
- Discovery automatically disabled after timeout

**Usage**:
```rust
let lifecycle_manager = LifecycleManager::new();

// Enable restoration preference
lifecycle_manager.set_restore_on_foreground(true).await;

// App goes to background
lifecycle_manager.on_background(true, Some(DiscoveryMethod::WiFi)).await?;

// Check if discovery should be disabled
if lifecycle_manager.should_disable_discovery().await {
    // Stop discovery
}

// App returns to foreground
if let Some(method) = lifecycle_manager.on_foreground().await? {
    // Restore discovery with the saved method
}
```

**Validates Requirements**: 16.4, 16.5

## Integration

All three components work together to provide a complete platform-specific solution:

```rust
// Complete flow
let permission_manager = PermissionManager::new();
let lifecycle_manager = LifecycleManager::new();
let factory = get_default_factory();

// 1. Request permission
let status = permission_manager.request_permission(DiscoveryMethod::WiFi).await?;
permission_manager.verify_permission(DiscoveryMethod::WiFi).await?;

// 2. Start discovery (using platform-specific connections)
let connection = factory.create_connection(peer_id).await?;

// 3. Handle backgrounding
lifecycle_manager.set_restore_on_foreground(true).await;
lifecycle_manager.on_background(true, Some(DiscoveryMethod::WiFi)).await?;

// 4. Handle foregrounding
if let Some(method) = lifecycle_manager.on_foreground().await? {
    // Restore discovery
    permission_manager.verify_permission(method).await?;
}
```

## Testing

### Unit Tests
- Platform abstraction: 2 tests
- Permission management: 9 tests
- Lifecycle management: 12 tests

### Integration Tests
- Platform integration: 11 tests covering complete flows

**Total**: 34 tests, all passing

## Platform Support Matrix

| Feature | Web (WASM) | iOS | Android | Desktop |
|---------|-----------|-----|---------|---------|
| WiFi Discovery | ✓ (Limited) | ✓ | ✓ | ✓ |
| Bluetooth Discovery | ✗ | ✓ | ✓ | ✓ |
| WebRTC Connections | ✓ | ✗ | ✗ | ✗ |
| Native Sockets | ✗ | ✓ | ✓ | ✓ |
| Permission Requests | Auto | Manual | Manual | Auto |
| Background Timeout | N/A | ✓ | ✓ | N/A |

## Error Handling

All components provide comprehensive error handling:

- **Permission Denied**: User-friendly messages with settings links
- **Platform Not Supported**: Graceful degradation
- **Background Timeout**: Automatic cleanup
- **Connection Failures**: Retry logic with exponential backoff

## Future Enhancements

1. **WebRTC Signaling**: Implement actual WebRTC signaling server integration
2. **Native Socket Addressing**: Implement peer address resolution from discovery info
3. **iOS/Android Native APIs**: Replace simulated permission requests with actual platform APIs
4. **Battery Optimization**: Add battery level monitoring and adaptive timeouts
5. **Network Quality Monitoring**: Implement real-time network quality assessment

## References

- Design Document: `.kiro/specs/proximity-p2p-transfers/design.md`
- Requirements: `.kiro/specs/proximity-p2p-transfers/requirements.md`
- Tasks: `.kiro/specs/proximity-p2p-transfers/tasks.md`
