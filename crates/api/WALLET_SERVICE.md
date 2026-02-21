# Wallet Service Implementation

## Overview

The Wallet Service provides functionality for connecting Solana wallets, validating addresses, retrieving portfolio data, and persisting wallet connections to the database.

## Implementation Status

**Task 3.1: Create wallet connection and validation logic** ✅ COMPLETE

This implementation validates:
- **Requirement 1.1**: Retrieve portfolio from Solana blockchain for valid wallet addresses
- **Requirement 1.2**: Return descriptive error messages for invalid wallet addresses
- **Requirement 1.3**: Persist wallet connections in PostgreSQL for future monitoring
- **Requirement 1.4**: Display all cryptocurrency assets and their quantities
- **Requirement 1.5**: Refresh portfolio data from blockchain

## Architecture

```
WalletService
├── SolanaClient (blockchain interaction)
└── DbPool (PostgreSQL persistence)
```

## Key Methods

### `connect_wallet(wallet_address: &str, user_id: Uuid) -> Result<Portfolio>`

Connects a user's Solana wallet and retrieves their portfolio.

**Flow:**
1. Validates wallet address format using Solana SDK
2. Retrieves wallet balance from blockchain (SOL + SPL tokens)
3. Stores wallet connection in database
4. Persists portfolio assets in database
5. Returns Portfolio with all assets

**Validates:** Requirements 1.1, 1.2, 1.3

### `get_portfolio(wallet_address: &str) -> Result<Portfolio>`

Retrieves portfolio holdings from the database.

**Flow:**
1. Validates wallet address format
2. Queries wallet from database
3. Retrieves all portfolio assets
4. Calculates total USD value
5. Returns Portfolio

**Validates:** Requirement 1.4

### `validate_wallet_address(address: &str) -> Result<()>`

Validates Solana wallet address format.

**Flow:**
1. Uses Solana SDK to parse address
2. Returns Ok if valid, Error if invalid

**Validates:** Requirement 1.2

### `refresh_portfolio(wallet_address: &str) -> Result<Portfolio>`

Refreshes portfolio data from the blockchain.

**Flow:**
1. Validates wallet address
2. Retrieves fresh data from blockchain
3. Updates last_synced timestamp
4. Updates portfolio assets in database
5. Returns updated Portfolio

**Validates:** Requirement 1.5

## Database Schema

### Wallets Table
```sql
CREATE TABLE wallets (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    address VARCHAR(44) UNIQUE NOT NULL,
    connected_at TIMESTAMP,
    last_synced TIMESTAMP
);
```

### Portfolio Assets Table
```sql
CREATE TABLE portfolio_assets (
    id UUID PRIMARY KEY,
    wallet_id UUID REFERENCES wallets(id),
    token_mint VARCHAR(44) NOT NULL,
    token_symbol VARCHAR(20) NOT NULL,
    amount DECIMAL(36, 18) NOT NULL,
    value_usd DECIMAL(18, 2),
    updated_at TIMESTAMP
);
```

## Error Handling

The service handles the following error cases:

1. **Invalid Wallet Address** (`Error::InvalidWalletAddress`)
   - Malformed address format
   - Empty address
   - Non-base58 characters

2. **Wallet Not Found** (`Error::WalletNotFound`)
   - Wallet not in database
   - User attempting to access non-connected wallet

3. **Solana RPC Errors** (`Error::SolanaRpc`)
   - Network failures
   - RPC rate limiting
   - Blockchain unavailability
   - Handled with retry logic and circuit breakers

4. **Database Errors** (`Error::Database`)
   - Connection failures
   - Query errors
   - Constraint violations

## Usage Example

```rust
use api::WalletService;
use blockchain::SolanaClient;
use database::create_pool;

// Setup
let solana_client = SolanaClient::new(rpc_url, None);
let db_pool = create_pool(&database_url, 5).await?;
let wallet_service = WalletService::new(solana_client, db_pool);

// Connect wallet
let portfolio = wallet_service
    .connect_wallet("YourWalletAddress...", user_id)
    .await?;

println!("Connected wallet with {} assets", portfolio.assets.len());

// Get portfolio
let portfolio = wallet_service
    .get_portfolio("YourWalletAddress...")
    .await?;

// Refresh portfolio
let updated_portfolio = wallet_service
    .refresh_portfolio("YourWalletAddress...")
    .await?;
```

## Testing

### Unit Tests
Located in `src/wallet_service.rs`:
- Wallet service creation
- Address validation logic

### Integration Tests
Located in `tests/wallet_service_test.rs`:
- `test_wallet_validation`: Validates address format checking
- `test_wallet_connection_persistence`: Validates wallet persistence
- `test_get_nonexistent_wallet`: Validates error handling

Run with:
```bash
# Requires DATABASE_URL and SOLANA_RPC_URL
cargo test -p api -- --ignored
```

### Demo
Located in `examples/wallet_demo.rs`:
```bash
DATABASE_URL=postgresql://... cargo run -p api --example wallet_demo
```

## Dependencies

- `blockchain`: Solana client wrapper with retry logic
- `database`: PostgreSQL connection pool
- `shared`: Common models and error types
- `solana-sdk`: Wallet address validation
- `rust_decimal`: Precise decimal arithmetic for token amounts
- `uuid`: User and wallet identifiers
- `chrono`: Timestamp handling

## Future Enhancements

1. **Price Feed Integration**: Add USD value calculation for tokens
2. **Token Metadata**: Fetch token symbols and logos from registry
3. **Portfolio Caching**: Cache portfolio data in Redis for faster retrieval
4. **Batch Operations**: Support connecting multiple wallets at once
5. **Historical Data**: Track portfolio value over time
6. **NFT Support**: Include NFT holdings in portfolio

## Notes

- SOL balance is stored with mint address `So11111111111111111111111111111111111111112`
- Token amounts are stored as DECIMAL(36, 18) for precision
- USD values are currently NULL (awaiting price feed integration)
- Token symbols are placeholder format `TOKEN_{first_8_chars}` until metadata service is added
- The service uses upsert logic (INSERT ... ON CONFLICT) for idempotent operations
