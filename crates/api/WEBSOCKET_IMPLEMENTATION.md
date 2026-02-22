# WebSocket Real-Time Dashboard Updates

## Overview

The WebSocket service provides real-time updates for the dashboard, pushing notifications about price changes, trades, trims, benchmark triggers, portfolio updates, and conversions to connected clients.

## Implementation

### WebSocket Endpoint

**URL**: `ws://localhost:8080/ws/dashboard` (or `wss://` for production)

### Connection

Clients connect to the WebSocket endpoint and receive JSON-formatted updates in real-time.

```javascript
// Example client connection
const ws = new WebSocket('ws://localhost:8080/ws/dashboard');

ws.onopen = () => {
    console.log('Connected to dashboard updates');
};

ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    console.log('Received update:', update);
    
    // Handle different update types
    switch (update.type) {
        case 'price_update':
            updatePriceDisplay(update);
            break;
        case 'trade_executed':
            showTradeNotification(update);
            break;
        case 'trim_executed':
            showTrimNotification(update);
            break;
        case 'benchmark_triggered':
            showBenchmarkAlert(update);
            break;
        case 'portfolio_update':
            updatePortfolioValue(update);
            break;
        case 'conversion_completed':
            showConversionNotification(update);
            break;
    }
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = () => {
    console.log('Disconnected from dashboard updates');
    // Implement reconnection logic here
};
```

## Update Types

### 1. Price Update

Sent when asset prices change.

```json
{
    "type": "price_update",
    "asset": "SOL",
    "blockchain": "Solana",
    "price": "100.50",
    "change_24h": "5.2",
    "timestamp": 1234567890
}
```

### 2. Trade Executed

Sent when a trade is executed.

```json
{
    "type": "trade_executed",
    "trade_id": "uuid",
    "asset": "ETH",
    "action": "BUY",
    "amount": "1.5",
    "price": "2000.00",
    "timestamp": 1234567890
}
```

### 3. Trim Executed

Sent when an agentic trim is executed.

```json
{
    "type": "trim_executed",
    "trim_id": "uuid",
    "asset": "BTC",
    "amount_sold": "0.25",
    "profit_realized": "5000.00",
    "reasoning": "Market conditions favorable for profit taking",
    "timestamp": 1234567890
}
```

### 4. Benchmark Triggered

Sent when a price benchmark is triggered.

```json
{
    "type": "benchmark_triggered",
    "benchmark_id": "uuid",
    "asset": "SOL",
    "target_price": "150.00",
    "current_price": "151.50",
    "action": "ALERT",
    "timestamp": 1234567890
}
```

### 5. Portfolio Update

Sent when portfolio value changes significantly.

```json
{
    "type": "portfolio_update",
    "total_value_usd": "50000.00",
    "change_24h": "3.5",
    "timestamp": 1234567890
}
```

### 6. Conversion Completed

Sent when a cryptocurrency conversion completes.

```json
{
    "type": "conversion_completed",
    "conversion_id": "uuid",
    "from_asset": "USDC",
    "to_asset": "SOL",
    "from_amount": "1000.00",
    "to_amount": "10.5",
    "timestamp": 1234567890
}
```

## Broadcasting Updates

Services can broadcast updates to all connected clients using the WebSocketService:

```rust
// In a service or handler
let ws_service = &state.websocket_service;

// Broadcast a price update
ws_service.broadcast_price_update(
    "SOL".to_string(),
    "Solana".to_string(),
    "100.50".to_string(),
    "5.2".to_string(),
);

// Broadcast a trade execution
ws_service.broadcast_trade_executed(
    trade_id,
    "ETH".to_string(),
    "BUY".to_string(),
    "1.5".to_string(),
    "2000.00".to_string(),
);

// Broadcast a trim execution
ws_service.broadcast_trim_executed(
    trim_id,
    "BTC".to_string(),
    "0.25".to_string(),
    "5000.00".to_string(),
    "Market conditions favorable".to_string(),
);
```

## Integration Points

The WebSocket service should be integrated with:

1. **Price Monitor**: Broadcast price updates when prices change
2. **Trading Service**: Broadcast trade executions
3. **Trim Executor**: Broadcast trim executions
4. **Benchmark Service**: Broadcast benchmark triggers
5. **Portfolio Monitor**: Broadcast portfolio value updates
6. **Conversion Service**: Broadcast conversion completions

## Architecture

- Uses Tokio broadcast channels for efficient message distribution
- Supports multiple concurrent WebSocket connections
- Automatic cleanup when clients disconnect
- Non-blocking broadcasts (won't slow down services if no clients connected)
- Buffer capacity of 100 messages to handle bursts

## Performance Considerations

- WebSocket connections are lightweight
- Broadcasts are non-blocking
- Late subscribers only receive new updates (no history replay)
- Automatic reconnection should be implemented on the client side

## Requirements Satisfied

This implementation satisfies **Requirement 10.6**:
- "THE Dashboard SHALL update metrics in real-time with WebSocket connections"

The WebSocket endpoint provides real-time updates for:
- Price changes
- Trade executions
- Agentic trims
- Benchmark triggers
- Portfolio value changes
- Conversion completions
