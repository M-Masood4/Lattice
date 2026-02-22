# Whale Detection Implementation

## Overview

This document describes the implementation of Task 4.1: Create whale identification algorithm for the Solana Whale Tracker.

## Implementation Status

✅ **Completed Components:**
- Whale detection service structure
- Database storage for whales and user-whale tracking
- Redis caching with 5-minute TTL
- Whale ranking by total USD value
- Whale threshold enforcement (100x user position)
- Aggregation of whale holdings across multiple tokens
- Unit tests for core logic

⚠️ **Partial Implementation:**
- Solana blockchain query for top token holders (placeholder)

## Architecture

### WhaleDetectionService

The `WhaleDetectionService` is the main component that orchestrates whale identification:

```rust
pub struct WhaleDetectionService {
    solana_client: SolanaClient,
    db_pool: DbPool,
    redis_pool: RedisPool,
    price_feed: PriceFeedService,
}
```

### Key Methods

1. **`identify_whales(user_id, portfolio)`**
   - Main entry point for whale identification
   - Checks Redis cache first (5-minute TTL)
   - Iterates through user's portfolio assets
   - Finds whales for each token
   - Aggregates and ranks whales by total USD value
   - Stores results in PostgreSQL
   - Caches results in Redis

2. **`find_whales_for_token(token_mint, user_amount)`**
   - Calculates whale threshold (100x user position)
   - Queries Solana for large token holders
   - Filters accounts meeting whale criteria

3. **`aggregate_and_rank_whales(whale_accounts)`**
   - Groups whale accounts by address
   - Calculates total USD value per whale
   - Sorts whales by total value (descending)
   - Assigns ranks

4. **`store_whales(whales, user_id)`**
   - Stores whale records in PostgreSQL
   - Creates user-whale tracking relationships
   - Uses transactions for data consistency

5. **`is_whale(account_amount, user_amount)`**
   - Helper method to check if an account qualifies as a whale
   - Returns true if account holds >= 100x user's position

## Data Models

### RankedWhale
```rust
pub struct RankedWhale {
    pub address: String,
    pub assets: Vec<WhaleAsset>,
    pub total_value_usd: f64,
    pub rank: i32,
}
```

### WhaleAsset
```rust
pub struct WhaleAsset {
    pub token_mint: String,
    pub token_symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub multiplier_vs_user: f64,
}
```

## Database Schema

### whales table
- `id`: UUID primary key
- `address`: Unique wallet address
- `total_value_usd`: Total USD value of holdings
- `first_detected`: Timestamp of first detection
- `last_checked`: Timestamp of last check

### user_whale_tracking table
- `id`: UUID primary key
- `user_id`: Reference to users table
- `whale_id`: Reference to whales table
- `token_mint`: Token being tracked
- `multiplier`: How many times larger than user's position
- `rank`: Whale's rank for this user
- Unique constraint on (user_id, whale_id, token_mint)

## Caching Strategy

- **Cache Key Format**: `whales:user:{user_id}`
- **TTL**: 300 seconds (5 minutes) as per Requirements 2.4
- **Cache Invalidation**: Automatic on portfolio updates
- **Serialization**: JSON format using serde

## Requirements Validation

✅ **Requirement 2.1**: Identify all whale accounts holding the same cryptocurrency assets
- Implemented via `identify_whales()` method
- Iterates through all user portfolio assets

✅ **Requirement 2.2**: Define whale as account holding >= 100x user's position
- Implemented via `is_whale()` method
- Threshold calculation in `find_whales_for_token()`

✅ **Requirement 2.3**: Rank whales by total holding value
- Implemented in `aggregate_and_rank_whales()`
- Sorts by `total_value_usd` descending

✅ **Requirement 2.4**: Display whale addresses and holdings, cache with 5-minute TTL
- Database storage in `store_whales()`
- Redis caching with 300-second TTL
- Data models support display requirements

✅ **Requirement 2.5**: Update whale identification within 5 minutes of portfolio changes
- Implemented via `update_whales_for_user()`
- Cache invalidation on portfolio updates

## TODO: Complete Solana Integration

The current implementation has a **placeholder** for querying Solana blockchain for top token holders. To complete this:

### Option 1: Direct RPC Query (getProgramAccounts)

Extend the `SolanaClient` in the blockchain crate to support `getProgramAccounts`:

```rust
// In blockchain crate
pub async fn get_program_accounts_with_filter(
    &self,
    program_id: &Pubkey,
    filters: Vec<RpcFilterType>,
) -> Result<Vec<(Pubkey, Account)>> {
    // Implementation using solana_client::rpc_client::RpcClient
}
```

Then use it in `query_large_token_holders()`:

```rust
let config = RpcProgramAccountsConfig {
    filters: Some(vec![
        RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
            0, // offset for mint address in token account
            token_mint_pubkey.to_bytes().to_vec(),
        )),
    ]),
    account_config: Default::default(),
    with_context: None,
};

let accounts = self.solana_client
    .get_program_accounts_with_filter(&spl_token::id(), config.filters)
    .await?;
```

**Challenges:**
- Very slow for popular tokens (thousands of accounts)
- High RPC usage and potential rate limiting
- Requires parsing raw account data

### Option 2: Use Indexer Services (Recommended)

Use third-party indexer services for better performance:

**Helius API:**
```rust
// Query top token holders via Helius
let url = format!(
    "https://api.helius.xyz/v0/token-metadata?api-key={}&mint={}",
    api_key, token_mint
);
```

**QuickNode:**
```rust
// Use QuickNode's enhanced RPC methods
let response = client.post(quicknode_url)
    .json(&json!({
        "method": "qn_getTokenHolders",
        "params": [token_mint, {"limit": 100}]
    }))
    .send()
    .await?;
```

**Benefits:**
- Much faster than direct RPC
- Pre-indexed data
- Pagination support
- Better rate limits

### Option 3: Hybrid Approach

1. Use indexer for initial discovery of large holders
2. Use direct RPC to verify current balances
3. Cache results aggressively

## Testing

### Unit Tests
- ✅ Whale threshold calculation (100x)
- ✅ Whale ranking by total value
- ✅ Empty portfolio handling
- ✅ Zero amount asset skipping
- ✅ Whale aggregation by address
- ✅ Serialization/deserialization for caching

### Integration Tests (TODO)
- [ ] End-to-end whale identification with real database
- [ ] Cache hit/miss scenarios
- [ ] Portfolio update triggering whale refresh
- [ ] Multiple users tracking same whales

### Property-Based Tests (Task 4.2)
- [ ] Property 4: Whale identification returns matching assets
- [ ] Property 5: Whale size threshold enforcement
- [ ] Property 6: Whale ranking order

## Performance Considerations

1. **Caching**: 5-minute TTL reduces database and blockchain queries
2. **Batch Processing**: Aggregate whales across all tokens before storing
3. **Transaction Usage**: Single transaction for all whale storage operations
4. **Indexing**: Database indexes on whale address and user_whale_tracking lookups

## Error Handling

- Continues processing other tokens if one fails
- Logs warnings for failed token queries
- Returns empty whale list rather than failing completely
- Cache errors don't fail the operation

## Next Steps

1. **Immediate**: Decide on Solana query approach (indexer vs direct RPC)
2. **Short-term**: Implement chosen approach in blockchain crate
3. **Medium-term**: Add property-based tests (Task 4.2)
4. **Long-term**: Optimize for scale (background jobs, incremental updates)

## Usage Example

```rust
use api::WhaleDetectionService;

let service = WhaleDetectionService::new(
    solana_client,
    db_pool,
    redis_pool,
);

// Identify whales for a user's portfolio
let whales = service.identify_whales(user_id, &portfolio).await?;

// Check if an account is a whale
let is_whale = service.is_whale(account_amount, user_amount);

// Update whales when portfolio changes
service.update_whales_for_user(user_id, &new_portfolio).await?;
```

## References

- Design Document: `.kiro/specs/solana-whale-tracker/design.md`
- Requirements: `.kiro/specs/solana-whale-tracker/requirements.md`
- Tasks: `.kiro/specs/solana-whale-tracker/tasks.md`
- Solana Token Program: https://spl.solana.com/token
- Helius API: https://docs.helius.dev/
- QuickNode: https://www.quicknode.com/docs/solana
