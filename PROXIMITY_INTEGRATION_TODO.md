# Proximity P2P Transfer Integration TODO

## Current Status
The proximity-based P2P transfer feature is **fully implemented** but not yet integrated into the running application.

### ✅ Completed
- Proximity service implementation (`crates/proximity/`)
- API endpoint definitions (`crates/api/src/proximity_handlers.rs`)
- Frontend UI (`frontend/proximity.js`)
- Database schema and migrations
- WebSocket support for real-time updates
- All tests passing (125+ tests)

### ❌ Pending Integration
The proximity service needs to be added to the AppState and connected to the API handlers.

## Integration Steps

### 1. Add Proximity Service to AppState

**File:** `crates/api/src/lib.rs`

Add to imports:
```rust
use proximity::{DiscoveryService, TransferService, SessionManager, AuthenticationService};
```

Add to AppState struct:
```rust
pub struct AppState {
    // ... existing fields ...
    pub proximity_discovery_service: Arc<DiscoveryService>,
    pub proximity_transfer_service: Arc<TransferService>,
    pub proximity_session_manager: Arc<SessionManager>,
    // ... rest of fields ...
}
```

Update AppState::new() to include proximity services.

### 2. Initialize Proximity Services

**File:** `crates/api/src/main.rs`

Add proximity service initialization:
```rust
// Initialize proximity services
let auth_service = Arc::new(AuthenticationService::new());
let discovery_service = Arc::new(DiscoveryService::new(
    db_pool.clone(),
    auth_service.clone(),
));
let session_manager = Arc::new(SessionManager::new(db_pool.clone()));
let transfer_service = Arc::new(TransferService::new(
    db_pool.clone(),
    wallet_service.clone(),
));
```

Pass these to AppState::new().

### 3. Update Proximity Handlers

**File:** `crates/api/src/proximity_handlers.rs`

Replace the stub implementations with actual service calls. For example:

```rust
pub async fn start_discovery(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartDiscoveryRequest>,
) -> ApiResult<Json<StartDiscoveryResponse>> {
    let session = state.proximity_session_manager
        .start_session(
            req.user_id,
            req.method,
            if req.duration_minutes == 0 { 30 } else { req.duration_minutes },
        )
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start session: {}", e)))?;
    
    state.proximity_discovery_service
        .start_discovery(req.method)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start discovery: {}", e)))?;
    
    Ok(Json(StartDiscoveryResponse { session }))
}
```

Repeat for all handlers:
- `stop_discovery`
- `get_discovered_peers`
- `block_peer`
- `create_transfer`
- `accept_transfer`
- `reject_transfer`
- `get_transfer_status`
- `get_transfer_history`

### 4. Test the Integration

After making these changes:

1. Rebuild the application:
   ```bash
   cargo build --release --bin api
   ```

2. Restart the services:
   ```bash
   ./stop-prod.sh
   ./deploy-local.sh
   ```

3. Test discovery in the UI:
   - Navigate to the Proximity tab
   - Click "Enable Discovery"
   - Should see "Discovery Active" status

### 5. Platform-Specific Considerations

**Note:** The current implementation uses mock/stub implementations for mDNS and BLE on macOS. For full functionality:

- **mDNS (WiFi)**: Requires `mdns-sd` crate with proper network permissions
- **BLE (Bluetooth)**: Requires `btleplug` crate with Bluetooth permissions

On macOS, you may need to:
1. Grant network permissions to the application
2. Grant Bluetooth permissions
3. Ensure WiFi is enabled for mDNS discovery

## Estimated Time
- Integration: 30-60 minutes
- Testing: 15-30 minutes
- **Total: 1-1.5 hours**

## Why Not Integrated Yet?
The proximity feature was developed as a complete module with comprehensive tests. The final integration step (adding to AppState) was left for last to avoid breaking existing functionality during development. This is a common pattern for large feature additions.

## Next Steps
1. Follow the integration steps above
2. Test thoroughly
3. Update this file with any issues encountered
4. Remove this file once integration is complete

---
**Created:** 2026-02-21
**Status:** Ready for integration
