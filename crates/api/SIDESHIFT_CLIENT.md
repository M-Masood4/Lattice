# SideShift API Client

This module provides a Rust client for the SideShift.ai API, enabling cryptocurrency conversions and staking functionality.

## Overview

The SideShift client implements the following features:
- **Quote Generation**: Get conversion quotes between cryptocurrency pairs
- **Order Creation**: Create fixed-rate conversion orders
- **Order Status**: Track the status of conversion orders
- **Supported Coins**: Query available cryptocurrencies
- **Staking Information**: Get staking details for supported coins

## Architecture

```
SideShiftClient
├── get_quote()           - Get conversion quote
├── create_order()        - Create conversion order
├── get_order_status()    - Check order status
├── get_supported_coins() - List available coins
└── get_staking_info()    - Get staking details
```

## Usage

### Creating a Client

```rust
use api::SideShiftClient;

// Without affiliate ID
let client = SideShiftClient::new(None);

// With affiliate ID
let client = SideShiftClient::new(Some("your-affiliate-id".to_string()));
```

### Getting a Conversion Quote

```rust
use api::{SideShiftClient, AmountType};
use rust_decimal::Decimal;

let client = SideShiftClient::new(None);

// Quote for converting 1 BTC to ETH
let quote = client
    .get_quote(
        "btc",
        "eth",
        Decimal::from(1),
        AmountType::From
    )
    .await?;

println!("Exchange rate: {}", quote.exchange_rate);
println!("You will receive: {} ETH", quote.to_amount);
println!("Quote expires at: {}", quote.expires_at);
```

### Creating a Conversion Order

```rust
// Create an order using a quote
let order = client
    .create_order(
        &quote.quote_id,
        "0x1234...", // Your settlement address
        Some("bc1...") // Optional refund address
    )
    .await?;

println!("Deposit {} {} to: {}", 
    order.deposit_amount,
    order.deposit_coin,
    order.deposit_address
);
```

### Checking Order Status

```rust
let status = client
    .get_order_status(&order.order_id)
    .await?;

println!("Order status: {}", status.status);
```

### Getting Supported Coins

```rust
let coins = client.get_supported_coins().await?;

for coin in coins {
    println!("{} ({})", coin.name, coin.coin);
    if coin.has_staking {
        println!("  - Staking available");
    }
}
```

### Getting Staking Information

```rust
let staking_info = client.get_staking_info("eth").await?;

println!("APY: {}%", staking_info.apy);
println!("Minimum amount: {}", staking_info.minimum_amount);
println!("Lock period: {} days", staking_info.lock_period_days);
```

## Data Types

### ConversionQuote

Represents a conversion quote from SideShift:

```rust
pub struct ConversionQuote {
    pub quote_id: String,
    pub from_asset: String,
    pub to_asset: String,
    pub from_amount: Decimal,
    pub to_amount: Decimal,
    pub exchange_rate: Decimal,
    pub network_fee: Decimal,
    pub platform_fee: Decimal,
    pub sideshift_fee: Decimal,
    pub expires_at: DateTime<Utc>,
}
```

### ConversionOrder

Represents a created conversion order:

```rust
pub struct ConversionOrder {
    pub order_id: String,
    pub deposit_address: String,
    pub deposit_coin: String,
    pub settle_coin: String,
    pub deposit_amount: Decimal,
    pub settle_amount: Decimal,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
```

### OrderStatus

Represents the current status of an order:

```rust
pub struct OrderStatus {
    pub order_id: String,
    pub status: String,
    pub deposit_address: String,
    pub deposit_amount: Decimal,
    pub settle_amount: Decimal,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
```

### StakingInfo

Represents staking information for a coin:

```rust
pub struct StakingInfo {
    pub coin: String,
    pub apy: Decimal,
    pub minimum_amount: Decimal,
    pub lock_period_days: u32,
    pub compound_frequency: String,
}
```

## Error Handling

The client uses `anyhow::Result` for error handling. Common errors include:

- **Network errors**: Connection failures, timeouts
- **API errors**: Invalid parameters, rate limits, service unavailable
- **Parsing errors**: Invalid response format

Example error handling:

```rust
match client.get_quote("btc", "eth", Decimal::from(1), AmountType::From).await {
    Ok(quote) => println!("Quote: {:?}", quote),
    Err(e) => {
        if e.to_string().contains("rate limit") {
            println!("Rate limited, try again later");
        } else {
            println!("Error: {}", e);
        }
    }
}
```

## API Endpoints

The client uses the following SideShift API v2 endpoints:

- `POST /quotes` - Get conversion quote
- `POST /shifts/fixed` - Create fixed-rate order
- `GET /shifts/{id}` - Get order status
- `GET /coins` - List supported coins
- `GET /staking/{coin}` - Get staking information

## Configuration

### Timeout

The HTTP client has a 30-second timeout for all requests.

### Affiliate ID

You can optionally provide an affiliate ID when creating the client. This will be included in all quote and order requests.

## Testing

Basic unit tests are provided in `tests/sideshift_client_test.rs`:

```bash
cargo test -p api --test sideshift_client_test
```

For integration testing with the actual SideShift API, you would need:
1. Network access
2. Valid API endpoints
3. Proper rate limit handling
4. Mock responses for deterministic testing

## Requirements Mapping

This implementation satisfies the following requirements from the spec:

- **Requirement 3.2**: SideShift API integration for staking
- **Requirement 3.6**: Automatic conversion between cryptocurrencies
- **Requirement 6.3**: SideShift as primary conversion provider

## Future Enhancements

Potential improvements:
1. Add retry logic with exponential backoff
2. Implement rate limiting
3. Add caching for supported coins list
4. Support variable-rate orders (in addition to fixed-rate)
5. Add webhook support for order status updates
6. Implement batch quote requests
7. Add more comprehensive error types

## Related Modules

- `birdeye_service.rs` - Multi-chain price data
- `benchmark_service.rs` - Price-based trading triggers
- `wallet_service.rs` - Wallet management

## References

- [SideShift API Documentation](https://sideshift.ai/api)
- [SideShift API v2 Reference](https://sideshift.ai/api/v2)
