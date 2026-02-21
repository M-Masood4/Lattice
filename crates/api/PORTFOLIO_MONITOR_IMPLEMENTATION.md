# Portfolio Monitor Implementation

## Overview

This document describes the implementation of Task 4.3: "Implement whale list updates when portfolio changes".

## Implementation Details

### Background Job Service

The `PortfolioMonitor` service (`crates/api/src/portfolio_monitor.rs`) implements a background job that:

1. **Periodically checks all connected wallets** for portfolio changes (default: every 5 minutes per Requirement 2.5)
2. **Detects portfolio composition changes** by comparing stored token mints with current portfolio
3. **Triggers whale list updates** when portfolio composition changes
4. **Updates user-whale tracking relationships** via the `WhaleDetectionService`

### Key Components

#### PortfolioMonitor Struct

```rust
pub struct PortfolioMonitor {
    wallet_service: Arc<WalletService>,
    whale_detection_service: Arc<WhaleDetectionService>,
    db_pool: DbPool,
    check_interval: Duration,
}
```

#### Main Methods

1. **`start()`** - Spawns a Tokio background task that runs the monitoring loop
2. **`check_and_update_portfolios()`** - Iterates through all wallets and checks for changes
3. **`has_portfolio_changed()`** - Detects if token composition has changed
4. **`trigger_whale_update()`** - Manually triggers a whale list update for a specific user

### Integration Points

#### Service Initialization (main.rs)

The portfolio monitor is initialized and started in `main.rs`:

```rust
let portfolio_monitor = Arc::new(PortfolioMonitor::new(
    wallet_service.clone(),
    whale_detection_service.clone(),
    db_pool.clone(),
    None, // Use default 5-minute interval
));

let _monitor_handle = portfolio_monitor.start();
```

#### Whale Detection Service

The existing `WhaleDetectionService::update_whales_for_user()` method is used to:
- Invalidate the whale cache for the user
- Re-identify whales with the new portfolio composition
- Update the `user_whale_tracking` table in the database

### Architecture Changes

To support shared ownership of services, the following changes were made:

1. **SolanaClient wrapped in Arc** - Since `SolanaClient` doesn't implement `Clone`, it's now wrapped in `Arc<SolanaClient>`
2. **Service constructors updated** - Both `WalletService` and `WhaleDetectionService` now accept `Arc<SolanaClient>`
3. **Services wrapped in Arc** - Services are wrapped in `Arc` for sharing between the main application and background job

### Error Handling

The implementation follows the design's error resilience requirements:

- **Continues on individual failures** - If one wallet fails to refresh, the monitor continues with other wallets
- **Logs errors** - All errors are logged with appropriate severity levels
- **Non-blocking** - The background job runs independently and doesn't block the main application

### Performance Considerations

1. **Configurable check interval** - Default 5 minutes per Requirement 2.5, but configurable
2. **Cache invalidation** - Only invalidates cache when changes are detected
3. **Efficient change detection** - Compares token mint sets rather than full portfolio data
4. **Database ordering** - Queries wallets ordered by `last_synced` to prioritize stale data

## Validation

**Validates: Requirements 2.5**

> WHEN the User's Portfolio composition changes, THE System SHALL update the Whale identification within 5 minutes

The implementation satisfies this requirement by:
- Running the background job every 5 minutes (configurable)
- Detecting portfolio composition changes (new tokens or removed tokens)
- Automatically triggering whale list updates when changes are detected
- Updating the `user_whale_tracking` table with new whale relationships

## Testing

Unit tests are provided for:
- Portfolio monitor creation
- Portfolio change detection (requires database - marked as `#[ignore]`)

Integration tests will be added in Task 15.3 to validate the end-to-end flow.

## Future Enhancements

1. **Event-driven updates** - Instead of polling, trigger updates on wallet transaction events
2. **Batch processing** - Process multiple wallets in parallel for better performance
3. **Metrics** - Add monitoring metrics for update frequency and success rates
4. **Manual refresh API** - Expose `trigger_whale_update()` via REST API for user-initiated refreshes
