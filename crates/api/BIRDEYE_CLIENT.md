# Birdeye API Client Module

## Overview

The Birdeye client module provides HTTP client functionality for integrating with the Birdeye API to fetch multi-chain cryptocurrency portfolio data and real-time price information. This module supports Solana, Ethereum, Binance Smart Chain (BSC), and Polygon blockchains.

## Features

- **Multi-chain Support**: Fetch portfolio data across Solana, Ethereum, BSC, and Polygon
- **API Authentication**: Secure API key-based authentication
- **Caching Layer**: Redis-based caching with configurable TTL (60s for portfolios, 10s for prices)
- **Retry Logic**: Exponential backoff retry mechanism (up to 3 attempts) for API failures
- **Type Safety**: Comprehensive Rust types for all API requests and responses
- **Error Handling**: Robust error handling with detailed context

## Architecture

```
┌─────────────────┐
│  BirdeyeService │
└────────┬────────┘
         │
         ├──► HTTP Client (reqwest)
         │    └──► Birdeye API
         │
         ├──► Redis Cache
         │    ├──► Portfolio Cache (60s TTL)
         │    └──► Price Cache (10s TTL)
         │
         └──► Retry Logic
              └──► Exponential Backoff
```

## Configuration

### Environment Variables

Add the following to your `.env` file:

```bash
BIRDEYE_API_KEY=your_birdeye_api_key_here
```

Get your API key from: https://birdeye.so/

### Shared Config

The Birdeye configuration is automatically loaded via the shared config module:

```rust
pub struct BirdeyeConfig {
    pub api_key: String,
}
```

## Usage

### Initialization

```rust
use api::BirdeyeService;
use redis::aio::ConnectionManager;

let redis = create_redis_connection().await?;
let api_key = config.birdeye.api_key.clone();

let birdeye_service = BirdeyeService::new(api_key, redis);
```

### Fetch Multi-Chain Portfolio

```rust
use api::birdeye_service::{Blockchain, WalletAddress};

let wallets = vec![
    WalletAddress {
        blockchain: Blockchain::Solana,
        address: "YourSolanaWalletAddress".to_string(),
    },
    WalletAddress {
        blockchain: Blockchain::Ethereum,
        address: "0xYourEthereumAddress".to_string(),
    },
];

let portfolio = birdeye_service
    .get_multi_chain_portfolio(wallets)
    .await?;

println!("Total Portfolio Value: ${}", portfolio.total_value_usd);
for (chain, assets) in portfolio.positions_by_chain {
    println!("Chain: {}", chain);
    for asset in assets {
        println!("  {} ({}): {} @ ${}", 
            asset.name, 
            asset.symbol, 
            asset.balance, 
            asset.price_usd
        );
    }
}
```

### Fetch Asset Price

```rust
use api::birdeye_service::Blockchain;

let sol_address = "So11111111111111111111111111111111111111112";
let price_data = birdeye_service
    .get_asset_price(&Blockchain::Solana, sol_address)
    .await?;

println!("SOL Price: ${}", price_data.price_usd);
```

## Data Types

### Blockchain

Supported blockchain networks:

```rust
pub enum Blockchain {
    Solana,
    Ethereum,
    BinanceSmartChain,
    Polygon,
}
```

### WalletAddress

Represents a wallet on a specific blockchain:

```rust
pub struct WalletAddress {
    pub blockchain: Blockchain,
    pub address: String,
}
```

### Asset

Represents a cryptocurrency asset in a portfolio:

```rust
pub struct Asset {
    pub symbol: String,           // e.g., "SOL"
    pub name: String,             // e.g., "Solana"
    pub address: String,          // Token contract address
    pub blockchain: Blockchain,   // Which chain this asset is on
    pub balance: Decimal,         // Amount held
    pub price_usd: Decimal,       // Current price in USD
    pub value_usd: Decimal,       // Total value (balance * price)
}
```

### MultiChainPortfolio

Aggregated portfolio across multiple blockchains:

```rust
pub struct MultiChainPortfolio {
    pub total_value_usd: Decimal,
    pub positions_by_chain: HashMap<String, Vec<Asset>>,
    pub last_updated: DateTime<Utc>,
}
```

### PriceData

Real-time price information for an asset:

```rust
pub struct PriceData {
    pub price_usd: Decimal,
    pub price_change_24h: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub last_updated: DateTime<Utc>,
}
```

## Caching Strategy

### Portfolio Cache

- **Key Format**: `birdeye:portfolio:{wallet_address}:{blockchain}`
- **TTL**: 60 seconds
- **Purpose**: Reduce API calls for frequently accessed portfolios

### Price Cache

- **Key Format**: `birdeye:price:{blockchain}:{token_address}`
- **TTL**: 10 seconds
- **Purpose**: Provide near real-time prices while minimizing API load

## Error Handling

The service implements comprehensive error handling:

### Retry Logic

- **Max Attempts**: 3
- **Backoff Strategy**: Exponential (100ms, 200ms, 400ms)
- **Retryable Errors**: Network failures, API timeouts, 5xx responses

### Error Types

```rust
// API request failures
Err(anyhow!("Failed to send request to Birdeye API"))

// API error responses
Err(anyhow!("Birdeye API returned error status: {}", status))

// Invalid responses
Err(anyhow!("Birdeye API returned success=false"))

// Missing data
Err(anyhow!("Birdeye API returned no data"))
```

## API Endpoints

### Portfolio Endpoint

```
GET https://public-api.birdeye.so/v1/wallet/token_list?wallet={address}
Headers:
  X-API-KEY: {api_key}
  x-chain: {blockchain}
```

### Price Endpoint

```
GET https://public-api.birdeye.so/defi/price?address={token_address}
Headers:
  X-API-KEY: {api_key}
  x-chain: {blockchain}
```

## Testing

### Unit Tests

Run the unit tests (no external dependencies required):

```bash
cargo test --package api --test birdeye_service_test
```

### Integration Tests

Run integration tests (requires Redis and Birdeye API key):

```bash
export BIRDEYE_API_KEY=your_key_here
cargo test --package api --test birdeye_service_test -- --ignored
```

### Test Coverage

- ✅ Blockchain enum conversion
- ✅ Data structure serialization/deserialization
- ✅ Portfolio aggregation logic
- ✅ Multi-chain portfolio handling
- ✅ Empty portfolio edge cases
- ✅ Asset value calculations
- ⚠️ API integration (requires live API key)
- ⚠️ Cache functionality (requires Redis)
- ⚠️ Retry logic (requires API failures)

## Performance Considerations

### Timeouts

- **HTTP Client Timeout**: 30 seconds
- **Total Retry Time**: Up to ~70 seconds (3 attempts with backoff)

### Rate Limiting

The Birdeye API has rate limits. The caching layer helps minimize API calls:

- Portfolio data cached for 60 seconds
- Price data cached for 10 seconds

### Concurrent Requests

The service processes wallet addresses sequentially to avoid overwhelming the API. For better performance with many wallets, consider:

1. Batching requests
2. Parallel processing with rate limiting
3. Longer cache TTLs for less critical data

## Requirements Validation

This implementation satisfies the following requirements from the spec:

- ✅ **Requirement 1.1**: Fetch position data via Birdeye API for connected wallets
- ✅ **Requirement 1.2**: Support Solana, Ethereum, BSC, and Polygon blockchains
- ✅ **Requirement 1.3**: Normalize Birdeye data into internal Portfolio format
- ✅ **Requirement 1.4**: Retry up to 3 times with exponential backoff on API failures
- ✅ **Requirement 1.5**: Cache Birdeye responses in Redis with 60-second TTL

## Future Enhancements

Potential improvements for future iterations:

1. **Batch API Calls**: Support for fetching multiple wallets in a single request
2. **WebSocket Support**: Real-time price updates via WebSocket connections
3. **Historical Data**: Fetch historical price and portfolio data
4. **Advanced Caching**: Implement cache warming and predictive prefetching
5. **Metrics**: Add Prometheus metrics for API performance monitoring
6. **Circuit Breaker**: Implement circuit breaker pattern for API failures

## Troubleshooting

### Common Issues

**Issue**: `Failed to send request to Birdeye API`
- **Solution**: Check network connectivity and API endpoint availability

**Issue**: `Birdeye API returned error status: 401`
- **Solution**: Verify your API key is correct and active

**Issue**: `Birdeye API returned error status: 429`
- **Solution**: Rate limit exceeded. Increase cache TTL or reduce request frequency

**Issue**: `Failed to connect to Redis`
- **Solution**: Ensure Redis is running and accessible at the configured URL

## References

- [Birdeye API Documentation](https://docs.birdeye.so/)
- [Birdeye Dashboard](https://birdeye.so/)
- [Requirements Document](../../.kiro/specs/crypto-trading-platform-enhancements/requirements.md)
- [Design Document](../../.kiro/specs/crypto-trading-platform-enhancements/design.md)
