# API Reference

Complete API documentation for the Crypto Trading Platform.

## Base URL

```
http://localhost:3000/api
```

## Authentication

Most endpoints require JWT authentication. Include the token in the Authorization header:

```
Authorization: Bearer <your_jwt_token>
```

## Response Format

All responses follow this structure:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": { ... }
  }
}
```

---

## Authentication Endpoints

### Register User

```http
POST /api/auth/register
```

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user_id": "uuid",
    "email": "user@example.com",
    "user_tag": "Trader_A7X9K2",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Login

```http
POST /api/auth/login
```

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_at": "2024-01-02T00:00:00Z",
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "user_tag": "Trader_A7X9K2"
    }
  }
}
```

### Enable 2FA

```http
POST /api/auth/2fa/enable
```

**Response:**
```json
{
  "success": true,
  "data": {
    "qr_code": "data:image/png;base64,...",
    "secret": "JBSWY3DPEHPK3PXP"
  }
}
```

---

## Multi-Chain Wallet Endpoints

### Connect Wallet

```http
POST /api/wallets/connect
```

**Request Body:**
```json
{
  "blockchain": "ethereum",
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "is_primary": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "wallet_id": "uuid",
    "blockchain": "ethereum",
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "is_primary": true,
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Get Multi-Chain Portfolio

```http
GET /api/wallets/portfolio
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_value_usd": 125000.50,
    "last_updated": "2024-01-01T00:00:00Z",
    "positions_by_chain": {
      "solana": [
        {
          "asset": "SOL",
          "amount": 100.5,
          "value_usd": 10050.00,
          "price_usd": 100.00
        }
      ],
      "ethereum": [
        {
          "asset": "ETH",
          "amount": 50.0,
          "value_usd": 115000.00,
          "price_usd": 2300.00
        }
      ]
    }
  }
}
```

### Create Temporary Wallet

```http
POST /api/wallets/temporary
```

**Request Body:**
```json
{
  "blockchain": "solana",
  "tag": "trading-bot",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "wallet_id": "uuid",
    "address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
    "tag": "trading-bot",
    "expires_at": "2024-12-31T23:59:59Z"
  }
}
```

### Freeze Wallet

```http
POST /api/wallets/:address/freeze
```

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "is_frozen": true,
    "frozen_at": "2024-01-01T00:00:00Z"
  }
}
```

---

## Benchmark Endpoints

### Create Benchmark

```http
POST /api/benchmarks
```

**Request Body:**
```json
{
  "asset": "SOL",
  "blockchain": "solana",
  "target_price": 150.00,
  "trigger_type": "ABOVE",
  "action_type": "EXECUTE",
  "trade_action": "SELL",
  "trade_amount": 10.0
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "asset": "SOL",
    "target_price": 150.00,
    "trigger_type": "ABOVE",
    "action_type": "EXECUTE",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### List Benchmarks

```http
GET /api/benchmarks?active=true
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "asset": "SOL",
      "target_price": 150.00,
      "current_price": 100.00,
      "distance_percent": 50.0,
      "trigger_type": "ABOVE",
      "is_active": true
    }
  ]
}
```

---

## Conversion Endpoints

### Get Conversion Quote

```http
POST /api/conversions/quote
```

**Request Body:**
```json
{
  "from_asset": "SOL",
  "to_asset": "USDC",
  "from_amount": 10.0
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "from_asset": "SOL",
    "to_asset": "USDC",
    "from_amount": 10.0,
    "to_amount": 995.50,
    "exchange_rate": 100.00,
    "network_fee": 0.50,
    "platform_fee": 2.00,
    "provider_fee": 2.00,
    "total_fees": 4.50,
    "provider": "SIDESHIFT",
    "expires_at": "2024-01-01T00:05:00Z"
  }
}
```

### Execute Conversion

```http
POST /api/conversions/execute
```

**Request Body:**
```json
{
  "quote_id": "uuid",
  "from_asset": "SOL",
  "to_asset": "USDC",
  "from_amount": 10.0
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "conversion_id": "uuid",
    "status": "PENDING",
    "transaction_hash": "0x...",
    "estimated_completion": "2024-01-01T00:01:00Z"
  }
}
```

---

## Staking Endpoints

### Enable Auto-Staking

```http
POST /api/staking/enable
```

**Request Body:**
```json
{
  "asset": "SOL",
  "minimum_idle_amount": 10.0,
  "idle_duration_hours": 24,
  "auto_compound": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "asset": "SOL",
    "enabled": true,
    "config": {
      "minimum_idle_amount": 10.0,
      "idle_duration_hours": 24,
      "auto_compound": true
    }
  }
}
```

### List Staking Positions

```http
GET /api/staking/positions
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "asset": "SOL",
      "amount": 100.0,
      "apy": 5.5,
      "rewards_earned": 2.75,
      "started_at": "2024-01-01T00:00:00Z",
      "last_reward_at": "2024-01-15T00:00:00Z"
    }
  ]
}
```

---

## Agentic Trimming Endpoints

### Configure Trim Settings

```http
POST /api/trim/config
```

**Request Body:**
```json
{
  "enabled": true,
  "minimum_profit_percent": 20.0,
  "trim_percent": 25.0,
  "max_trims_per_day": 3
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "enabled": true,
    "minimum_profit_percent": 20.0,
    "trim_percent": 25.0,
    "max_trims_per_day": 3,
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

### Get Trim History

```http
GET /api/trim/history?limit=10
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "asset": "SOL",
      "amount_sold": 25.0,
      "price_usd": 120.00,
      "profit_realized": 500.00,
      "confidence": 87,
      "reasoning": "Strong resistance at $120, volume declining",
      "executed_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

---

## Voice Command Endpoints

### Process Voice Command

```http
POST /api/voice/command
Content-Type: multipart/form-data
```

**Request Body:**
```
audio: <audio file>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "command_id": "uuid",
    "transcribed_text": "Buy 100 dollars of Solana",
    "command_type": "PLACE_ORDER",
    "parameters": {
      "asset": "SOL",
      "action": "BUY",
      "amount_usd": 100.0
    },
    "requires_confirmation": true
  }
}
```

---

## P2P Exchange Endpoints

### Create P2P Offer

```http
POST /api/p2p/offers
```

**Request Body:**
```json
{
  "offer_type": "SELL",
  "from_asset": "SOL",
  "to_asset": "USDC",
  "from_amount": 10.0,
  "price": 100.00
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "offer_id": "uuid",
    "offer_type": "SELL",
    "from_asset": "SOL",
    "to_asset": "USDC",
    "from_amount": 10.0,
    "to_amount": 1000.00,
    "price": 100.00,
    "status": "ACTIVE",
    "escrow_tx_hash": "0x...",
    "expires_at": "2024-01-02T00:00:00Z"
  }
}
```

### List Active Offers

```http
GET /api/p2p/offers?asset=SOL&type=SELL
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "offer_id": "uuid",
      "user_tag": "Trader_X7Y9K2",
      "offer_type": "SELL",
      "from_asset": "SOL",
      "to_asset": "USDC",
      "from_amount": 10.0,
      "price": 100.00,
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

---

## Chat Endpoints

### Send Message

```http
POST /api/chat/messages
```

**Request Body:**
```json
{
  "to_user_id": "uuid",
  "content": "Hello, interested in your P2P offer",
  "verify_on_chain": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "message_id": "uuid",
    "from_user_tag": "Trader_A7X9K2",
    "to_user_tag": "Trader_B8Z1L3",
    "content": "Hello, interested in your P2P offer",
    "encrypted": true,
    "blockchain_hash": "0x...",
    "verification_status": "PENDING",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Get Chat History

```http
GET /api/chat/messages?with_user_id=uuid&limit=50
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "message_id": "uuid",
      "from_user_tag": "Trader_A7X9K2",
      "content": "Hello, interested in your P2P offer",
      "verification_status": "CONFIRMED",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

---

## Receipt Endpoints

### List Receipts

```http
GET /api/receipts?type=CONVERSION&start_date=2024-01-01&end_date=2024-01-31
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "receipt_id": "uuid",
      "type": "CONVERSION",
      "amount": 1000.00,
      "currency": "USDC",
      "timestamp": "2024-01-15T00:00:00Z",
      "blockchain": "solana",
      "transaction_hash": "0x...",
      "verification_status": "CONFIRMED"
    }
  ]
}
```

### Download Receipt

```http
GET /api/receipts/:id/download
```

Returns PDF file.

### Export Receipts

```http
POST /api/receipts/export
```

**Request Body:**
```json
{
  "format": "CSV",
  "start_date": "2024-01-01",
  "end_date": "2024-12-31"
}
```

Returns CSV file.

---

## Verification Endpoints

### Submit Identity Verification

```http
POST /api/verification/identity
Content-Type: multipart/form-data
```

**Request Body:**
```
document_front: <file>
document_back: <file>
selfie: <file>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "verification_id": "uuid",
    "status": "PENDING",
    "submitted_at": "2024-01-01T00:00:00Z",
    "estimated_processing_time": "24 hours"
  }
}
```

### Verify Wallet Ownership

```http
POST /api/verification/wallet
```

**Request Body:**
```json
{
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "blockchain": "ethereum",
  "signature": "0x..."
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "blockchain": "ethereum",
    "verified": true,
    "verified_at": "2024-01-01T00:00:00Z"
  }
}
```

---

## Analytics & Dashboard Endpoints

### Get Dashboard Data

```http
GET /api/analytics/dashboard
```

**Response:**
```json
{
  "success": true,
  "data": {
    "portfolio": {
      "total_value_usd": 125000.50,
      "change_24h_percent": 5.2,
      "change_7d_percent": 12.5,
      "all_time_pnl": 25000.50
    },
    "position_distribution": {
      "by_blockchain": {
        "solana": 40.0,
        "ethereum": 50.0,
        "polygon": 10.0
      },
      "by_asset_type": {
        "native": 60.0,
        "stablecoins": 30.0,
        "defi": 10.0
      }
    },
    "active_benchmarks": [
      {
        "asset": "SOL",
        "target_price": 150.00,
        "current_price": 100.00,
        "distance_percent": 50.0
      }
    ],
    "recent_ai_actions": [
      {
        "type": "TRIM",
        "asset": "SOL",
        "action": "Sold 25% at $120",
        "timestamp": "2024-01-01T00:00:00Z"
      }
    ]
  }
}
```

### WebSocket Real-Time Updates

```
WS /api/ws/dashboard
```

**Message Format:**
```json
{
  "type": "PRICE_UPDATE",
  "data": {
    "asset": "SOL",
    "price": 100.50,
    "change_percent": 0.5
  }
}
```

---

## Error Codes

| Code | Description |
|------|-------------|
| `INVALID_CREDENTIALS` | Email or password incorrect |
| `INSUFFICIENT_BALANCE` | Not enough funds for operation |
| `WALLET_FROZEN` | Wallet is frozen, cannot perform operation |
| `BENCHMARK_VALIDATION_FAILED` | Invalid benchmark parameters |
| `QUOTE_EXPIRED` | Conversion quote has expired |
| `OFFER_EXPIRED` | P2P offer has expired |
| `VERIFICATION_REQUIRED` | Operation requires identity verification |
| `TEMPORARY_WALLET_LIMIT_EXCEEDED` | Maximum 10 temporary wallets allowed |
| `BIRDEYE_API_UNAVAILABLE` | Birdeye API is temporarily unavailable |
| `SIDESHIFT_API_ERROR` | SideShift API error |
| `INTERCOM_API_UNAVAILABLE` | Voice service unavailable |

---

## Rate Limits

- Authentication endpoints: 5 requests per minute
- Voice commands: 10 requests per minute
- Other endpoints: 100 requests per minute

Rate limit headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

---

## Pagination

List endpoints support pagination:

```http
GET /api/receipts?page=1&limit=50
```

Response includes pagination metadata:
```json
{
  "success": true,
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 250,
    "total_pages": 5
  }
}
```
