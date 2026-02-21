# Provider Failover and Recovery Implementation

## Overview

This document describes the implementation of provider failover and recovery functionality for the P2P mesh network price distribution system. The implementation ensures that the system handles provider disconnections gracefully and maintains data availability through caching.

## Requirements Addressed

### Requirement 9.1: Data Persistence After Provider Disconnect
- **Implementation**: The `PriceCache` automatically retains all cached data when providers disconnect
- **Location**: `crates/api/src/price_cache.rs`
- **Behavior**: Cache data persists in memory, Redis, and database regardless of provider status

### Requirement 9.2: No Providers Warning Display
- **Implementation**: `NetworkStatusTracker` tracks when all providers go offline and provides warning information
- **Location**: `crates/api/src/network_status_tracker.rs`
- **Behavior**: 
  - Logs warning when all providers go offline
  - Tracks offline timestamp for duration calculation
  - Provides status information via `NetworkStatus` struct

### Requirement 9.3: Provider Reconnection Recovery
- **Implementation**: `NetworkStatusTracker::on_provider_reconnected()` handles provider recovery
- **Location**: `crates/api/src/network_status_tracker.rs`
- **Behavior**:
  - Updates provider status to active
  - Clears offline tracking when providers come back online
  - Logs recovery with offline duration

### Requirement 9.4: New Provider Auto-Discovery
- **Implementation**: `MeshPriceService::handle_network_status()` detects new providers
- **Location**: `crates/api/src/mesh_price_service.rs`
- **Behavior**:
  - Automatically discovers providers through network status messages
  - Distinguishes between new providers and reconnecting providers
  - Logs auto-discovery events

### Requirement 9.5: Extended Offline Indicator
- **Implementation**: `NetworkStatusTracker` tracks offline duration and provides extended offline flag
- **Location**: `crates/api/src/network_status_tracker.rs`
- **Behavior**:
  - Tracks how long all providers have been offline
  - Sets `extended_offline` flag after 10 minutes
  - Provides `offline_duration_minutes` in network status

## Key Components Modified

### 1. NetworkStatusTracker

**New Fields:**
```rust
all_providers_offline_since: Arc<tokio::sync::RwLock<Option<DateTime<Utc>>>>
```

**New Methods:**
- `is_extended_offline()` - Check if offline for 10+ minutes
- `get_offline_duration()` - Get current offline duration
- `on_provider_reconnected()` - Handle provider coming back online
- `on_provider_disconnected()` - Handle provider going offline

**Modified Methods:**
- `get_status()` - Now tracks offline duration and provides extended offline indicator

### 2. NetworkStatus Struct

**New Fields:**
```rust
pub extended_offline: bool,
pub offline_duration_minutes: Option<i64>,
```

These fields provide UI-ready information about provider offline status.

### 3. MeshPriceService

**Modified Methods:**
- `on_peer_disconnected()` - Now detects provider disconnections and handles failover
- `handle_network_status()` - Now detects new providers and reconnections
- `get_price_data()` - Logs when serving cached data due to no providers
- `get_all_price_data()` - Logs when serving cached data due to no providers

## Data Flow

### Provider Disconnect Flow

```
1. Peer disconnects
   ↓
2. MeshPriceService::on_peer_disconnected()
   ↓
3. Check if peer was a provider
   ↓
4. NetworkStatusTracker::on_provider_disconnected()
   ↓
5. Update provider status to inactive
   ↓
6. Check if all providers are offline
   ↓
7. If yes, record offline timestamp and log warning
   ↓
8. Cached data remains available
```

### Provider Reconnection Flow

```
1. Peer connects
   ↓
2. MeshPriceService::on_peer_connected()
   ↓
3. Send network status to peer
   ↓
4. Receive network status from peer
   ↓
5. MeshPriceService::handle_network_status()
   ↓
6. Detect if peer is a provider
   ↓
7. NetworkStatusTracker::on_provider_reconnected()
   ↓
8. Update provider status to active
   ↓
9. Clear offline timestamp
   ↓
10. Log recovery with offline duration
```

### Cached Data Serving Flow

```
1. Request for price data
   ↓
2. MeshPriceService::get_price_data()
   ↓
3. PriceCache::get()
   ↓
4. Check in-memory cache
   ↓
5. If not found, check Redis
   ↓
6. Return cached data (if available)
   ↓
7. Log if serving cached data due to no providers
```

## Testing

### Unit Tests
Location: `crates/api/tests/mesh_price_service_test.rs`

Tests added:
- `test_provider_disconnect_keeps_cached_data` - Verifies cache persistence
- `test_all_providers_offline_warning` - Verifies warning status
- `test_provider_reconnection` - Verifies reconnection handling
- `test_new_provider_auto_discovery` - Verifies auto-discovery
- `test_extended_offline_indicator` - Verifies 10-minute offline indicator
- `test_cached_data_served_when_no_providers` - Verifies cache fallback

### Integration Tests
Location: `crates/api/tests/provider_failover_test.rs`

Tests added:
- `test_provider_disconnect_preserves_cache` - End-to-end disconnect test
- `test_all_providers_offline_status` - Status tracking test
- `test_provider_reconnection_clears_offline_status` - Reconnection test
- `test_cached_data_fallback_when_offline` - Cache fallback test

## Logging

The implementation includes comprehensive logging:

### Warning Level
- All providers going offline
- Extended offline (10+ minutes)
- Provider disconnections

### Info Level
- Provider reconnections
- Network recovery with offline duration
- Auto-discovery of new providers

### Debug Level
- Serving cached data when no providers online
- Network status updates

## UI Integration

The `NetworkStatus` struct provides all information needed for UI display:

```rust
pub struct NetworkStatus {
    pub active_providers: Vec<ProviderInfo>,
    pub connected_peers: usize,
    pub total_network_size: usize,
    pub last_update_time: Option<DateTime<Utc>>,
    pub data_freshness: DataFreshness,
    pub extended_offline: bool,              // NEW
    pub offline_duration_minutes: Option<i64>, // NEW
}
```

### UI Display Recommendations

1. **No Providers Warning** (Requirement 9.2)
   - Display when `active_providers.is_empty()`
   - Message: "No live data sources - serving cached data"

2. **Extended Offline Indicator** (Requirement 9.5)
   - Display when `extended_offline == true`
   - Message: "Offline for {offline_duration_minutes} minutes"
   - Use prominent styling (e.g., red indicator)

3. **Data Freshness**
   - Use `data_freshness` field to show age of data
   - Show staleness warning for old data

## Error Handling

The implementation handles errors gracefully:

1. **Redis Connection Failures**: Falls back to in-memory cache
2. **Database Write Failures**: Continues with Redis cache
3. **Provider Disconnect**: Maintains cached data, logs warning
4. **All Providers Offline**: Serves cached data, displays warning

## Performance Considerations

1. **Offline Tracking**: Uses RwLock for minimal contention
2. **Status Queries**: Efficient Redis queries with caching
3. **Cache Access**: In-memory cache for fast lookups
4. **Logging**: Appropriate log levels to avoid spam

## Future Enhancements

Potential improvements for future iterations:

1. **Provider Health Scoring**: Track provider reliability over time
2. **Automatic Provider Promotion**: Promote reliable consumers to providers
3. **Cache Expiration Policies**: More sophisticated cache management
4. **Provider Load Balancing**: Distribute load across multiple providers
5. **Offline Mode UI**: Dedicated offline mode with clear indicators

## Conclusion

The provider failover and recovery implementation ensures the mesh network remains resilient to provider disconnections. Cached data is always available, users are informed of network status, and the system automatically recovers when providers reconnect.
