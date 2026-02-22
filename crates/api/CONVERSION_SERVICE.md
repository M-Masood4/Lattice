# Conversion Service

## Overview

The Conversion Service orchestrates cryptocurrency swaps using SideShift as the primary provider with Jupiter as a fallback for Solana tokens. All conversions are recorded in the database for audit and history tracking.

## Features

- **Quote Generation**: Get real-time conversion quotes with detailed fee breakdown
- **Multi-Provider Support**: Uses SideShift as primary, Jupiter as fallback for Solana tokens
- **Fee Transparency**: Displays network fees, platform fees, and provider fees separately
- **Database Recording**: All conversions are recorded with complete details
- **Conversion History**: Users can view their past conversions

## Requirements Implemented

- **Requirement 6.1**: Support direct swaps between any two supported assets
- **Requirement 6.2**: Display exchange rate, fees, and estimated output
- **Requirement 6.3**: Use SideShift API as primary conversion provider
- **Requirement 6.4**: Fall back to Jupiter Aggregator for Solana tokens when SideShift is unavailable
- **Requirement 6.5**: Execute swaps within 30 seconds of user confirmation
- **Requirement 6.6**: Record all conversions with timestamp, amounts, exchange rate, and fees

## Architecture

```
┌─────────────────┐
│  API Handler    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ ConversionService│
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌─────────┐ ┌──────────┐
│SideShift│ │ Jupiter  │
│ Client  │ │(Fallback)│
└─────────┘ └──────────┘
```

## API Endpoints

### 1. Get Conversion Quote

**Endpoint**: `POST /api/conversions/quote`

**Request Body**:
```json
{
  "from_asset": "SOL",
  "to_asset": "USDC",
  "amount": "1.5",
  "amount_type": "from"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "quote_id": "abc123",
    "from_asset": "SOL",
    "to_asset": "USDC",
    "from_amount": "1.5",
    "to_amount": "150.25",
    "exchange_rate": "100.16",
    "network_fee": "0.05",
    "platform_fee": "0.10",
    "provider_fee": "0.15",
    "total_fees": "0.30",
    "provider": "SideShift",
    "expires_at": "2024-01-15T10:30:00Z"
  }
}
```

### 2. Execute Conversion

**Endpoint**: `POST /api/conversions/:user_id/execute`

**Request Body**:
```json
{
  "quote_id": "abc123",
  "from_asset": "SOL",
  "to_asset": "USDC",
  "from_amount": "1.5",
  "to_amount": "150.25",
  "exchange_rate": "100.16",
  "network_fee": "0.05",
  "platform_fee": "0.10",
  "provider_fee": "0.15",
  "total_fees": "0.30",
  "provider": "SideShift",
  "expires_at": "2024-01-15T10:30:00Z",
  "settle_address": "user_wallet_address",
  "refund_address": "refund_wallet_address"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "order_id": "order_xyz",
    "deposit_address": "deposit_address",
    "deposit_amount": "1.5",
    "settle_amount": "150.25",
    "status": "Pending",
    "transaction_hash": null,
    "provider": "SideShift"
  }
}
```

### 3. Get Conversion History

**Endpoint**: `GET /api/conversions/:user_id/history`

**Response**:
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "user_id": "user_uuid",
      "from_asset": "SOL",
      "to_asset": "USDC",
      "from_amount": "1.5",
      "to_amount": "150.25",
      "exchange_rate": "100.16",
      "network_fee": "0.05",
      "platform_fee": "0.10",
      "provider_fee": "0.15",
      "provider": "SIDESHIFT",
      "transaction_hash": "tx_hash",
      "status": "completed",
      "created_at": "2024-01-15T10:00:00Z",
      "completed_at": "2024-01-15T10:05:00Z"
    }
  ]
}
```

## Usage Example

```rust
use api::{ConversionService, SideShiftClient, AmountType};
use rust_decimal::Decimal;
use std::sync::Arc;

// Initialize service
let sideshift_client = Arc::new(SideShiftClient::new(None));
let conversion_service = ConversionService::new(db_pool, sideshift_client);

// Get a quote
let quote = conversion_service
    .get_quote("SOL", "USDC", Decimal::new(15, 1), AmountType::From)
    .await?;

println!("Exchange rate: {}", quote.exchange_rate);
println!("Total fees: {}", quote.total_fees);

// Execute conversion
let result = conversion_service
    .execute_conversion(
        user_id,
        quote,
        "settle_wallet_address",
        Some("refund_wallet_address"),
    )
    .await?;

println!("Order ID: {}", result.order_id);
println!("Deposit to: {}", result.deposit_address);
```

## Provider Fallback Logic

The service implements intelligent fallback logic:

1. **Primary**: Always try SideShift first for all conversions
2. **Fallback**: If SideShift fails AND both assets are Solana tokens, fall back to Jupiter
3. **Error**: If no fallback is available, return error to user

### Solana Token Detection

The following tokens are recognized as Solana tokens for Jupiter fallback:
- SOL
- USDC
- USDT
- RAY
- SRM
- BONK
- JUP
- ORCA

## Database Schema

```sql
CREATE TABLE conversions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  from_asset VARCHAR(50) NOT NULL,
  to_asset VARCHAR(50) NOT NULL,
  from_amount DECIMAL(36, 18) NOT NULL,
  to_amount DECIMAL(36, 18) NOT NULL,
  exchange_rate DECIMAL(18, 8) NOT NULL,
  network_fee DECIMAL(18, 8),
  platform_fee DECIMAL(18, 8),
  provider_fee DECIMAL(18, 8),
  provider VARCHAR(50) NOT NULL, -- SIDESHIFT or JUPITER
  transaction_hash VARCHAR(255),
  status VARCHAR(20) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW(),
  completed_at TIMESTAMP
);
```

## Configuration

Add to your `.env` file:

```bash
# Optional: SideShift affiliate ID for revenue sharing
SIDESHIFT_AFFILIATE_ID=your_affiliate_id
```

## Error Handling

The service handles various error scenarios:

- **Quote Expired**: Returns error if quote has expired before execution
- **SideShift Unavailable**: Falls back to Jupiter for Solana tokens
- **Invalid Amount**: Validates amount format and returns clear error
- **Database Errors**: Logs errors and returns user-friendly messages

## Testing

Run the conversion service tests:

```bash
cargo test --package api --test conversion_service_test
```

## Future Enhancements

- **Jupiter Integration**: Complete Jupiter API integration for Solana token fallback
- **More Providers**: Add additional conversion providers for redundancy
- **Rate Limiting**: Implement rate limiting per user
- **Quote Caching**: Cache quotes briefly to reduce API calls
- **Webhook Support**: Add webhook notifications for conversion status updates

## Notes

- Jupiter integration is currently a placeholder and needs to be implemented
- All conversions are recorded in the database for audit purposes
- Fee breakdown provides transparency to users
- The service supports both "exact input" and "exact output" modes via `AmountType`
