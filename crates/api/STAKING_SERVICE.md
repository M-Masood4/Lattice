# Staking Service

## Overview

The Staking Service implements auto-staking functionality for idle cryptocurrency balances using the SideShift API. It detects idle balances, requests user approval, and tracks staking positions with rewards.

**Validates Requirements:** 3.1, 3.3, 3.4

## Features

### 1. Idle Balance Detection (Requirement 3.1)

The service identifies balances eligible for staking based on:
- **Minimum Amount**: Balance must meet or exceed the configured minimum (default: 100 units)
- **Idle Duration**: No trades for the configured duration (default: 24 hours)
- **Not Already Staked**: Balance is not currently in a staking position

```rust
let config = StakingConfig {
    minimum_idle_amount: Decimal::from(100),
    idle_duration_hours: 24,
    auto_compound: false,
};

let idle_balances = staking_service
    .identify_idle_balances(user_id, &config)
    .await?;
```

### 2. User Approval Flow (Requirement 3.3)

Before staking any assets, the system requests user approval:

1. **Create Approval Request**: System fetches staking info from SideShift and creates a request
2. **User Reviews**: User sees APY, lock period, and amount
3. **User Approves/Rejects**: User makes decision within 24 hours
4. **Staking Initiated**: If approved, staking position is created

```rust
// Create approval request
let request = staking_service
    .create_staking_approval_request(user_id, "SOL", amount)
    .await?;

// User approves
let result = staking_service
    .initiate_staking(request.request_id, true)
    .await?;
```

### 3. Position Tracking (Requirement 3.4)

All staking positions are tracked with:
- **Entry Date**: When staking started
- **Amount**: Amount staked
- **Current Rewards**: Accumulated rewards
- **APY**: Annual percentage yield
- **Provider**: Staking provider (SideShift)
- **Auto-compound**: Whether rewards are automatically reinvested

```rust
// Get all positions for a user
let positions = staking_service
    .get_staking_positions(user_id)
    .await?;

// Get specific position
let position = staking_service
    .get_staking_position(position_id)
    .await?;

// Update rewards (typically called by background job)
staking_service
    .update_staking_rewards(position_id, new_rewards)
    .await?;
```

## API Endpoints

### Get Idle Balances
```
GET /api/staking/:user_id/idle-balances
```

Returns list of assets with idle balances eligible for staking.

**Response:**
```json
{
  "success": true,
  "data": [
    ["SOL", "1500.5"],
    ["ETH", "2.3"]
  ]
}
```

### Create Staking Request
```
POST /api/staking/:user_id/:asset/request
```

Creates a staking approval request for the user.

**Request Body:**
```json
{
  "amount": "1000.0"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "request_id": "uuid",
    "user_id": "uuid",
    "asset": "SOL",
    "amount": "1000.0",
    "provider": "SideShift",
    "apy": "5.0",
    "lock_period_days": 30,
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": "2024-01-02T00:00:00Z"
  }
}
```

### Approve Staking Request
```
POST /api/staking/requests/:request_id/approve
```

Approves or rejects a staking request.

**Request Body:**
```json
{
  "approved": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "position_id": "uuid",
    "asset": "SOL",
    "amount": "1000.0",
    "apy": "5.0",
    "started_at": "2024-01-01T00:00:00Z"
  }
}
```

### Get Staking Positions
```
GET /api/staking/:user_id/positions
```

Returns all staking positions for a user.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "asset": "SOL",
      "amount": "1000.0",
      "provider": "SideShift",
      "apy": "5.0",
      "rewards_earned": "10.5",
      "auto_compound": false,
      "started_at": "2024-01-01T00:00:00Z",
      "last_reward_at": "2024-01-15T00:00:00Z"
    }
  ]
}
```

### Get Staking Position
```
GET /api/staking/positions/:position_id
```

Returns a specific staking position.

### Enable Auto-Staking
```
POST /api/staking/:user_id/auto-staking
```

Enables or disables auto-staking for an asset.

**Request Body:**
```json
{
  "asset": "SOL",
  "enabled": true,
  "minimum_idle_amount": "100.0",
  "idle_duration_hours": 24,
  "auto_compound": false
}
```

### Get Auto-Staking Config
```
GET /api/staking/:user_id/:asset/config
```

Returns auto-staking configuration for an asset.

## Database Schema

### staking_positions
```sql
CREATE TABLE staking_positions (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  asset VARCHAR(50) NOT NULL,
  amount DECIMAL(36, 18) NOT NULL,
  provider VARCHAR(50) NOT NULL,
  apy DECIMAL(5, 2),
  rewards_earned DECIMAL(36, 18) DEFAULT 0,
  auto_compound BOOLEAN DEFAULT FALSE,
  started_at TIMESTAMP DEFAULT NOW(),
  last_reward_at TIMESTAMP
);
```

### staking_approval_requests
```sql
CREATE TABLE staking_approval_requests (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  asset VARCHAR(50) NOT NULL,
  amount DECIMAL(36, 18) NOT NULL,
  provider VARCHAR(50) NOT NULL,
  apy DECIMAL(5, 2) NOT NULL,
  lock_period_days INTEGER NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'pending',
  created_at TIMESTAMP DEFAULT NOW(),
  expires_at TIMESTAMP NOT NULL
);
```

### auto_staking_configs
```sql
CREATE TABLE auto_staking_configs (
  user_id UUID REFERENCES users(id),
  asset VARCHAR(50) NOT NULL,
  enabled BOOLEAN DEFAULT FALSE,
  minimum_idle_amount DECIMAL(36, 18) NOT NULL DEFAULT 100,
  idle_duration_hours INTEGER NOT NULL DEFAULT 24,
  auto_compound BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  PRIMARY KEY (user_id, asset)
);
```

## Configuration

The staking service uses the SideShift API for staking operations. Configuration is managed through the `StakingConfig` struct:

```rust
pub struct StakingConfig {
    pub minimum_idle_amount: Decimal,  // Default: 100
    pub idle_duration_hours: u32,      // Default: 24
    pub auto_compound: bool,           // Default: false
}
```

## Error Handling

The service handles various error conditions:

- **Insufficient Balance**: Amount below minimum staking requirement
- **SideShift API Errors**: Logged and user notified (Requirement 3.5)
- **Expired Requests**: Approval requests expire after 24 hours
- **Invalid Status**: Requests must be in 'pending' status to approve

## Integration with SideShift

The service integrates with SideShift API for:
1. **Staking Info**: Fetches APY, minimum amounts, lock periods
2. **Staking Initiation**: Creates staking positions
3. **Reward Tracking**: Monitors and updates rewards

## Future Enhancements

- Background job to automatically process idle balances
- Reward compounding automation
- Multi-provider support (beyond SideShift)
- Unstaking functionality
- Staking analytics and projections

## Testing

Unit tests cover:
- Configuration defaults and validation
- Position creation and serialization
- Reward tracking
- Auto-staking configuration

Run tests:
```bash
cargo test --package api --test staking_service_test
```

## Example Usage

```rust
// Initialize service
let staking_service = StakingService::new(db_pool, sideshift_client);

// Enable auto-staking for SOL
let config = StakingConfig {
    minimum_idle_amount: Decimal::from(500),
    idle_duration_hours: 48,
    auto_compound: true,
};
staking_service.set_auto_staking(user_id, "SOL", true, Some(config)).await?;

// Check for idle balances
let idle = staking_service.identify_idle_balances(user_id, &config).await?;

// Create approval request for first idle balance
if let Some((asset, amount)) = idle.first() {
    let request = staking_service
        .create_staking_approval_request(user_id, asset, *amount)
        .await?;
    
    // User approves
    let result = staking_service
        .initiate_staking(request.request_id, true)
        .await?;
}

// Track positions
let positions = staking_service.get_staking_positions(user_id).await?;
for position in positions {
    println!("Staking {} {} at {}% APY, earned: {}",
        position.amount, position.asset,
        position.apy.unwrap_or_default(),
        position.rewards_earned
    );
}
```
