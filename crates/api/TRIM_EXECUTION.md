# Agentic Position Trimming - Execution Implementation

## Overview

This document describes the implementation of task 6.3: Trim Execution, which completes the agentic position trimming feature by executing pending trim recommendations.

## Architecture

The trim execution system consists of three main components working together:

1. **TrimConfigService** (Task 6.1) - Manages user trim preferences
2. **PositionEvaluator** (Task 6.2) - Evaluates positions and generates trim recommendations
3. **TrimExecutor** (Task 6.3) - Executes pending trim recommendations

### Data Flow

```
┌─────────────────────┐
│ PositionEvaluator   │
│ (Every 5 minutes)   │
└──────────┬──────────┘
           │
           │ Stores recommendations
           ▼
┌─────────────────────┐
│  pending_trims      │
│  (Database Table)   │
└──────────┬──────────┘
           │
           │ Reads pending trims
           ▼
┌─────────────────────┐
│  TrimExecutor       │
│  (Every 1 minute)   │
└──────────┬──────────┘
           │
           ├─► TradingService (Execute sell)
           ├─► Database (Log execution)
           └─► NotificationService (Notify user)
```

## Implementation Details

### TrimExecutor Service

**Location**: `crates/api/src/trim_executor.rs`

**Key Responsibilities**:
- Process pending trim recommendations from the database
- Execute partial sells via the trading service
- Calculate profit realized from the trim
- Log trim actions with reasoning
- Send notifications to users

**Background Worker**:
- Runs every 1 minute
- Processes all pending trims in the queue
- Respects daily trim limits per user
- Handles failures gracefully (keeps pending for retry)

### Key Methods

#### `execute_trim()`

Executes a single trim recommendation:

1. **Calculate trim amount**: Applies the configured trim percentage to the position
2. **Get current price**: Fetches the latest price for profit calculation
3. **Calculate profit**: Computes profit realized based on entry price
4. **Create recommendation**: Builds a SELL recommendation for the trading service
5. **Execute trade**: Calls the trading service to execute the sell order
6. **Log execution**: Records the trim in the `trim_executions` table
7. **Send notification**: Notifies the user about the trim

**Validates Requirements**: 7.3, 7.4, 7.5, 7.6

#### `log_trim_execution()`

Logs trim execution to the database with all required fields:
- Amount sold
- Price at execution
- Profit realized
- AI confidence level
- Reasoning for the trim
- Transaction hash

**Validates Requirement**: 7.6

#### `send_trim_notification()`

Sends a notification to the user containing:
- Asset trimmed
- Amount sold
- Price and profit realized
- AI confidence and reasoning
- Transaction hash for verification

**Validates Requirement**: 7.5

### Database Schema

#### pending_trims Table

Stores trim recommendations awaiting execution:

```sql
CREATE TABLE pending_trims (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  wallet_id UUID REFERENCES wallets(id),
  token_mint VARCHAR(255) NOT NULL,
  token_symbol VARCHAR(50) NOT NULL,
  amount VARCHAR(255) NOT NULL,
  confidence INTEGER NOT NULL,
  reasoning TEXT NOT NULL,
  suggested_trim_percent DECIMAL(5, 2) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(user_id, token_mint)
);
```

#### trim_executions Table

Records all executed trims for audit and history:

```sql
CREATE TABLE trim_executions (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  position_id UUID NOT NULL,
  asset VARCHAR(50) NOT NULL,
  amount_sold DECIMAL(36, 18) NOT NULL,
  price_usd DECIMAL(18, 8) NOT NULL,
  profit_realized DECIMAL(18, 2) NOT NULL,
  confidence INTEGER NOT NULL,
  reasoning TEXT NOT NULL,
  transaction_hash VARCHAR(255),
  executed_at TIMESTAMP DEFAULT NOW()
);
```

## Integration

### Main Application

The trim executor is initialized and started in `crates/api/src/main.rs`:

```rust
// Initialize trim executor
let trim_executor = Arc::new(TrimExecutor::new(
    db_pool.clone(),
    trim_config_service.clone(),
    trading_service.clone(),
    notification_service.clone(),
));

// Start background worker
let trim_executor_clone = trim_executor.clone();
tokio::spawn(async move {
    trim_executor_clone.start();
});
```

### Dependencies

- **TrimConfigService**: Checks daily limits and user preferences
- **TradingService**: Executes the actual sell orders
- **NotificationService**: Sends notifications to users
- **Database**: Stores pending trims and execution history

## Safety Features

### Daily Limits

The executor respects the user's configured `max_trims_per_day` setting:
- Checks limit before processing each trim
- Skips trims for users who have reached their limit
- Resets daily at midnight

### Retry Logic

Failed trims remain in the pending queue:
- Retried on the next execution cycle (1 minute later)
- Logged with warnings for monitoring
- No automatic removal on failure

### Profit Calculation

Profit is calculated conservatively:
- Uses entry price from first buy trade
- Falls back to zero profit if entry price unavailable
- Logs all calculations for audit

## Testing

### Unit Tests

**Location**: `crates/api/tests/trim_executor_test.rs`

Tests cover:
- Trim amount calculation (percentage of position)
- Profit calculation (entry vs current price)
- Data structure serialization
- Validation logic (confidence threshold, trim percentages)
- Multiple scenarios and edge cases

**Run tests**:
```bash
cargo test --package api --test trim_executor_test
```

### Integration Tests

Integration with other services is tested through:
- Position evaluator tests (recommendation generation)
- Trim config service tests (configuration management)
- End-to-end workflow validation

## Requirements Validation

### Requirement 7.3: Execute partial sells via trading service
✅ Implemented in `execute_trim()` - Creates SELL recommendation and calls trading service

### Requirement 7.4: Calculate profit realized
✅ Implemented in `execute_trim()` - Calculates profit from entry and current price

### Requirement 7.5: Send notifications
✅ Implemented in `send_trim_notification()` - Sends detailed notification to user

### Requirement 7.6: Log trim actions with reasoning
✅ Implemented in `log_trim_execution()` - Logs all details to database

## Monitoring and Observability

### Logging

The executor provides detailed logging at multiple levels:

- **INFO**: Successful executions, background job status
- **DEBUG**: Processing details, skipped trims
- **WARN**: Failed executions, missing data
- **ERROR**: Critical failures, database errors

### Metrics to Monitor

1. **Pending trim queue size**: Number of trims awaiting execution
2. **Execution success rate**: Percentage of successful trim executions
3. **Average execution time**: Time to process each trim
4. **Daily trim count per user**: Track against configured limits
5. **Profit realized**: Total profit locked in via trimming

## Future Enhancements

### Potential Improvements

1. **Price Oracle Integration**: Replace placeholder price fetching with Birdeye service
2. **Advanced Notifications**: Email and push notifications in addition to in-app
3. **Execution Scheduling**: Allow users to schedule trims for specific times
4. **Partial Execution**: Support trimming in multiple smaller chunks
5. **Dry Run Mode**: Test trim execution without actual trades

### Performance Optimizations

1. **Batch Processing**: Process multiple trims in parallel
2. **Smart Scheduling**: Prioritize high-confidence trims
3. **Caching**: Cache prices and user settings for faster execution
4. **Connection Pooling**: Optimize database connection usage

## Troubleshooting

### Common Issues

**Trims not executing**:
- Check if user has reached daily limit
- Verify trading service is operational
- Check database connectivity
- Review logs for specific errors

**Incorrect profit calculations**:
- Verify entry price is recorded in trade history
- Check price fetching from portfolio assets
- Review calculation logic in logs

**Missing notifications**:
- Verify notification service is running
- Check user notification preferences
- Review notification service logs

## API Endpoints (Future)

While not implemented in this task, future API endpoints could include:

- `GET /api/trims/pending` - View pending trims
- `GET /api/trims/history` - View trim execution history
- `POST /api/trims/{id}/cancel` - Cancel a pending trim
- `GET /api/trims/stats` - View trim statistics

## Conclusion

The trim executor completes the agentic position trimming feature by:
1. Processing AI-generated trim recommendations
2. Executing trades through the trading service
3. Calculating and recording profit realized
4. Notifying users of all trim actions

This implementation ensures users can automatically lock in profits based on AI analysis while maintaining full control through configuration and daily limits.
