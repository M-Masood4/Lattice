# Proximity Transfer Database Persistence Layer

This document describes the database persistence layer implemented for the proximity-based P2P transfer feature.

## Overview

The persistence layer provides functions to interact with three main database tables:
- `proximity_transfers` - Stores transfer records
- `discovery_sessions` - Tracks active discovery sessions
- `peer_blocklist` - Manages blocked peers

## Module Structure

All proximity-related database functions are in `crates/database/src/proximity.rs` and are exported from the main database crate.

## Data Structures

### ProximityTransferRecord
Represents a complete transfer record from the database with all fields including timestamps and optional transaction hash.

### DiscoverySessionRecord
Represents a discovery session with start/end times and expiration tracking.

### BlockedPeerRecord
Represents a blocked peer relationship between two users.

### TransferFilter
Provides flexible filtering options for querying transfers:
- Filter by user (sender or recipient)
- Filter by status
- Filter by asset
- Filter by date range
- Pagination support (limit/offset)

## Proximity Transfers Functions

### insert_proximity_transfer
Creates a new transfer record with sender, recipient, asset, amount, and discovery method.

**Returns:** UUID of the created transfer

### update_transfer_status
Updates transfer status with support for:
- Status transitions (Pending → Accepted → Executing → Completed/Failed)
- Transaction hash recording
- Automatic timestamp updates (accepted_at, completed_at)
- Failure reason tracking

### get_transfer_by_id
Retrieves a single transfer by its UUID.

**Returns:** Option<ProximityTransferRecord>

### get_user_proximity_transfers
Queries transfers with flexible filtering options. Supports:
- User-based filtering (sender or recipient)
- Status filtering
- Asset filtering
- Date range filtering
- Pagination

**Returns:** Vec<ProximityTransferRecord>

## Discovery Sessions Functions

### insert_discovery_session
Creates a new discovery session with expiration time and auto-extend flag.

**Returns:** UUID of the created session

### update_session_expiration
Extends an active session by updating its expiration time. Only affects sessions that haven't ended.

### end_discovery_session
Marks a session as ended by setting the ended_at timestamp. Ended sessions cannot be extended.

## Peer Blocklist Functions

### add_blocked_peer
Adds a peer to the user's blocklist. Uses ON CONFLICT DO NOTHING to handle duplicate blocks gracefully.

### remove_blocked_peer
Removes a peer from the user's blocklist.

### get_blocked_peers
Retrieves all blocked peers for a user, ordered by block time (most recent first).

**Returns:** Vec<BlockedPeerRecord>

## Testing

Comprehensive tests are provided in `crates/database/tests/proximity_test.rs`:

- Transfer insertion and retrieval
- Status updates with various scenarios
- Failed transfer handling
- Multi-filter queries
- Discovery session lifecycle
- Peer blocklist operations
- Date range filtering

Tests are marked with `#[ignore]` and require a real database connection to run.

## Usage Example

```rust
use database::{create_pool, insert_proximity_transfer, get_transfer_by_id};
use rust_decimal::Decimal;
use uuid::Uuid;

let pool = create_pool(&database_url, 5).await?;

// Insert a transfer
let transfer_id = insert_proximity_transfer(
    &pool,
    sender_id,
    "sender_wallet",
    recipient_id,
    "recipient_wallet",
    "SOL",
    Decimal::new(150, 2), // 1.50 SOL
    "Pending",
    "WiFi",
).await?;

// Retrieve the transfer
let transfer = get_transfer_by_id(&pool, transfer_id).await?;
```

## Database Schema

The implementation works with the following tables created by migrations:
- `20240101000029_create_proximity_transfers_table.sql`
- `20240101000030_create_discovery_sessions_table.sql`
- `20240101000031_create_peer_blocklist_table.sql`

All tables include appropriate indexes for performance optimization.
