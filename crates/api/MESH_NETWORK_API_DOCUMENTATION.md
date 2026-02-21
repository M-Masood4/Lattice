# P2P Mesh Network Price Distribution - Complete API Documentation

## Table of Contents
1. [REST API Endpoints](#rest-api-endpoints)
2. [WebSocket API](#websocket-api)
3. [Configuration Options](#configuration-options)
4. [Error Codes and Handling](#error-codes-and-handling)
5. [Data Models](#data-models)

---

## REST API Endpoints

### Provider Management

#### Enable Provider Mode
**POST** `/api/mesh/provider/enable`

Enable provider mode with API key validation. The node will start fetching price data from Birdeye API and broadcasting to the mesh network.

**Request Body:**
```json
{
  "api_key": "your-birdeye-api-key"
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "enabled": true,
    "node_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Error Responses:**
- `401 Unauthorized` - Invalid API key
  ```json
  {
    "success": false,
    "data": null,
    "error": "Invalid API key: authentication failed with Birdeye API"
  }
  ```
- `500 Internal Server Error` - Server error
  ```json
  {
    "success": false,
    "data": null,
    "error": "Failed to enable provider mode: internal error"
  }
  ```

**Requirements:** 1.1, 1.2, 1.3

---

#### Disable Provider Mode
**POST** `/api/mesh/provider/disable`

Disable provider mode and stop fetching price data. The node will continue to relay messages from other providers.

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "enabled": false,
    "node_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Error Responses:**
- `500 Internal Server Error` - Server error

**Requirements:** 1.2, 1.5

---

#### Get Provider Status
**GET** `/api/mesh/provider/status`

Get current provider mode status for this node.

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "enabled": true,
    "node_id": "550e8400-e29b-41d4-a716-446655440000",
    "last_fetch_time": "2024-01-01T12:00:00Z",
    "fetch_count": 42
  }
}
```

**Requirements:** 1.4, 1.5

---

### Price Data Access

#### Get All Cached Prices
**GET** `/api/mesh/prices`

Retrieve all cached price data from the mesh network.

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "SOL": {
      "asset": "SOL",
      "price": "123.45",
      "timestamp": "2024-01-01T12:00:00Z",
      "source_node_id": "550e8400-e29b-41d4-a716-446655440000",
      "blockchain": "Solana",
      "change_24h": "5.2"
    },
    "ETH": {
      "asset": "ETH",
      "price": "2345.67",
      "timestamp": "2024-01-01T12:00:00Z",
      "source_node_id": "550e8400-e29b-41d4-a716-446655440001",
      "blockchain": "Ethereum",
      "change_24h": "-2.1"
    }
  }
}
```

**Error Responses:**
- `500 Internal Server Error` - Server error

**Requirements:** 6.4

---

#### Get Price for Specific Asset
**GET** `/api/mesh/prices/:asset`

Retrieve cached price data for a specific asset.

**Path Parameters:**
- `asset` (string, required) - Asset symbol (e.g., "SOL", "ETH", "BTC")

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "asset": "SOL",
    "price": "123.45",
    "timestamp": "2024-01-01T12:00:00Z",
    "source_node_id": "550e8400-e29b-41d4-a716-446655440000",
    "blockchain": "Solana",
    "change_24h": "5.2"
  }
}
```

**Error Responses:**
- `404 Not Found` - Asset not found in cache
  ```json
  {
    "success": false,
    "data": null,
    "error": "Asset 'XYZ' not found in cache"
  }
  ```
- `500 Internal Server Error` - Server error

**Requirements:** 6.4

---

#### Get Network Status
**GET** `/api/mesh/network/status`

Get current mesh network status and topology information.

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "active_providers": [
      {
        "node_id": "550e8400-e29b-41d4-a716-446655440000",
        "last_seen": "2024-01-01T12:00:00Z",
        "hop_count": 1
      },
      {
        "node_id": "550e8400-e29b-41d4-a716-446655440001",
        "last_seen": "2024-01-01T12:00:05Z",
        "hop_count": 2
      }
    ],
    "connected_peers": 5,
    "total_network_size": 10,
    "last_update_time": "2024-01-01T12:00:00Z",
    "data_freshness": "JustNow"
  }
}
```

**Data Freshness Values:**
- `"JustNow"` - Data less than 1 minute old
- `"MinutesAgo(n)"` - Data n minutes old (1-60 minutes)
- `"HoursAgo(n)"` - Data n hours old (>1 hour)
- `"Stale"` - Data older than staleness threshold

**Error Responses:**
- `500 Internal Server Error` - Server error

**Requirements:** 8.5, 9.2

---

## WebSocket API

### Connection

Connect to the WebSocket endpoint to receive real-time price updates:

```
ws://localhost:8080/ws
```

### Message Types

#### 1. Initial Data Message

Sent immediately upon WebSocket connection with all cached price data.

```json
{
  "type": "PriceMeshInitial",
  "data": {
    "SOL": {
      "asset": "SOL",
      "price": "123.45",
      "timestamp": "2024-01-01T12:00:00Z",
      "source_node_id": "550e8400-e29b-41d4-a716-446655440000",
      "blockchain": "Solana",
      "change_24h": "5.2"
    },
    "ETH": {
      "asset": "ETH",
      "price": "2345.67",
      "timestamp": "2024-01-01T12:00:00Z",
      "source_node_id": "550e8400-e29b-41d4-a716-446655440001",
      "blockchain": "Ethereum",
      "change_24h": "-2.1"
    }
  }
}
```

**Requirements:** 12.2

---

#### 2. Price Update Message

Sent when new price data is received from the mesh network. Only includes changed assets (delta updates).

```json
{
  "type": "PriceMeshUpdate",
  "data": {
    "SOL": {
      "asset": "SOL",
      "price": "124.50",
      "timestamp": "2024-01-01T12:00:30Z",
      "source_node_id": "550e8400-e29b-41d4-a716-446655440000",
      "blockchain": "Solana",
      "change_24h": "6.1"
    }
  }
}
```

**Requirements:** 12.1, 12.4

---

#### 3. Network Status Update Message

Sent when network topology or provider status changes.

```json
{
  "type": "NetworkStatusUpdate",
  "data": {
    "active_providers": 2,
    "connected_peers": 5,
    "total_network_size": 10,
    "last_update_time": "2024-01-01T12:00:30Z",
    "data_freshness": "JustNow"
  }
}
```

**Requirements:** 8.5, 9.2

---

#### 4. Warning Messages

Sent when network conditions require user attention.

**No Providers Warning:**
```json
{
  "type": "MeshWarning",
  "data": {
    "severity": "warning",
    "message": "No live data sources available",
    "code": "NO_PROVIDERS"
  }
}
```

**Stale Data Warning:**
```json
{
  "type": "MeshWarning",
  "data": {
    "severity": "warning",
    "message": "Price data is older than 1 hour",
    "code": "STALE_DATA",
    "age_seconds": 3720
  }
}
```

**Price Discrepancy Warning:**
```json
{
  "type": "MeshWarning",
  "data": {
    "severity": "warning",
    "message": "Price discrepancy detected for SOL",
    "code": "PRICE_DISCREPANCY",
    "asset": "SOL",
    "discrepancy_percent": 6.5
  }
}
```

**Requirements:** 6.5, 8.2, 9.2, 9.5

---

### WebSocket Client Example

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('Connected to mesh network');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'PriceMeshInitial':
      console.log('Initial prices:', message.data);
      break;
    case 'PriceMeshUpdate':
      console.log('Price update:', message.data);
      break;
    case 'NetworkStatusUpdate':
      console.log('Network status:', message.data);
      break;
    case 'MeshWarning':
      console.warn('Warning:', message.data.message);
      break;
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected from mesh network');
  // Implement reconnection with exponential backoff
  setTimeout(() => reconnect(), 1000);
};
```

**Requirements:** 12.3

---

## Configuration Options

All configuration options can be set via environment variables. See `.env.example` for a complete list.

### Provider Node Configuration

| Variable | Default | Description | Requirement |
|----------|---------|-------------|-------------|
| `MESH_PROVIDER_FETCH_INTERVAL_SECS` | `30` | How often provider nodes fetch price data from Birdeye API | 2.1 |
| `MESH_COORDINATION_WINDOW_SECS` | `5` | Time window to coordinate fetches between multiple providers | 2.5 |
| `MESH_MESSAGE_TTL` | `10` | Initial TTL value for price update messages | 3.2 |

### Cache Configuration

| Variable | Default | Description | Requirement |
|----------|---------|-------------|-------------|
| `MESH_SEEN_MESSAGES_CACHE_SIZE` | `10000` | Maximum entries in the seen messages cache | 5.4 |
| `MESH_SEEN_MESSAGES_EXPIRATION_SECS` | `300` | Expiration time for seen messages (5 minutes) | 5.2 |
| `MESH_STALENESS_THRESHOLD_SECS` | `3600` | Threshold for marking data as stale (1 hour) | 6.5, 7.1 |

### Network Configuration

| Variable | Default | Description | Requirement |
|----------|---------|-------------|-------------|
| `MESH_MAX_PEER_CONNECTIONS` | `10` | Maximum number of peer connections per node | 13.5 |
| `MESH_MIN_PEER_CONNECTIONS` | `3` | Minimum peer connections to maintain | 10.5 |
| `MESH_OFFLINE_INDICATOR_THRESHOLD_SECS` | `600` | Time before showing offline indicator (10 minutes) | 9.5 |

### Data Quality Configuration

| Variable | Default | Description | Requirement |
|----------|---------|-------------|-------------|
| `MESH_PRICE_DISCREPANCY_THRESHOLD_PERCENT` | `5.0` | Percentage threshold for price discrepancy warnings | 8.2 |

### Example Configuration

```bash
# Aggressive fetching for high-frequency trading
MESH_PROVIDER_FETCH_INTERVAL_SECS=10
MESH_COORDINATION_WINDOW_SECS=2
MESH_STALENESS_THRESHOLD_SECS=300

# Conservative configuration for low-bandwidth networks
MESH_PROVIDER_FETCH_INTERVAL_SECS=60
MESH_MESSAGE_TTL=5
MESH_MAX_PEER_CONNECTIONS=5
```

---

## Error Codes and Handling

### HTTP Error Codes

| Code | Description | Common Causes | Resolution |
|------|-------------|---------------|------------|
| `400` | Bad Request | Invalid request body, missing required fields | Check request format and required fields |
| `401` | Unauthorized | Invalid API key | Verify Birdeye API key is correct |
| `404` | Not Found | Asset not in cache | Wait for provider to fetch data or check asset symbol |
| `500` | Internal Server Error | Database error, Redis error, service crash | Check logs, verify services are running |
| `503` | Service Unavailable | Service starting up or shutting down | Wait and retry with exponential backoff |

### Application Error Codes

#### Provider Errors

| Code | Message | Cause | Resolution |
|------|---------|-------|------------|
| `PROVIDER_API_KEY_INVALID` | Invalid API key | API key validation failed | Verify API key with Birdeye |
| `PROVIDER_FETCH_FAILED` | Failed to fetch price data | API timeout, network error | Check network, API status |
| `PROVIDER_BROADCAST_FAILED` | Failed to broadcast update | No connected peers | Wait for peer connections |

**Requirements:** 1.1, 1.3, 2.3, 2.4, 3.4

#### Network Errors

| Code | Message | Cause | Resolution |
|------|---------|-------|------------|
| `NO_PROVIDERS` | No live data sources | All providers offline | Enable provider mode or wait |
| `PEER_CONNECTION_FAILED` | Failed to connect to peer | Network issue, peer offline | Automatic retry with backoff |
| `NETWORK_PARTITION` | Network partition detected | Split network | Wait for network healing |

**Requirements:** 9.2, 10.2

#### Cache Errors

| Code | Message | Cause | Resolution |
|------|---------|-------|------------|
| `CACHE_MISS` | Asset not found in cache | No data received yet | Wait for provider fetch |
| `CACHE_STALE` | Cached data is stale | No updates for >1 hour | Check provider status |
| `REDIS_CONNECTION_FAILED` | Redis connection failed | Redis down | Falls back to in-memory cache |

**Requirements:** 6.4, 6.5

#### Validation Errors

| Code | Message | Cause | Resolution |
|------|---------|-------|------------|
| `INVALID_PRICE_UPDATE` | Invalid price update | Negative price, future timestamp | Check source node, may be malicious |
| `MISSING_REQUIRED_FIELD` | Required field missing | Malformed message | Reject message, log source |
| `INVALID_SOURCE_NODE` | Invalid source node ID | Missing or malformed node ID | Reject message, security concern |

**Requirements:** 14.1, 14.2, 14.3, 14.4, 14.5

### Error Handling Best Practices

#### 1. Retry Logic

For transient errors (network timeouts, temporary service unavailability):

```javascript
async function fetchWithRetry(url, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch(url);
      if (response.ok) return response;
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await sleep(Math.pow(2, i) * 1000); // Exponential backoff
    }
  }
}
```

**Requirements:** 2.3, 12.3

#### 2. Graceful Degradation

When providers are offline, continue serving cached data:

```javascript
async function getPrices() {
  try {
    const response = await fetch('/api/mesh/prices');
    const data = await response.json();
    
    if (!data.success) {
      // Fall back to local cache
      return getLocalCache();
    }
    
    return data.data;
  } catch (error) {
    // Network error, use local cache
    return getLocalCache();
  }
}
```

**Requirements:** 6.4, 9.1

#### 3. User Notifications

Display appropriate warnings based on error codes:

```javascript
function handleMeshWarning(warning) {
  switch (warning.code) {
    case 'NO_PROVIDERS':
      showWarning('No live price data sources. Using cached data.');
      break;
    case 'STALE_DATA':
      showWarning(`Price data is ${warning.age_seconds / 3600} hours old.`);
      break;
    case 'PRICE_DISCREPANCY':
      showWarning(`Price discrepancy detected for ${warning.asset}.`);
      break;
  }
}
```

**Requirements:** 6.5, 8.2, 9.2, 9.5

---

## Data Models

### PriceUpdate

```typescript
interface PriceUpdate {
  message_id: string;           // UUID
  source_node_id: string;        // UUID
  timestamp: string;             // ISO 8601 datetime
  prices: Record<string, PriceData>;
  ttl: number;                   // 0-10
}
```

### PriceData

```typescript
interface PriceData {
  asset: string;                 // Asset symbol (e.g., "SOL")
  price: string;                 // Decimal string
  blockchain: string;            // Blockchain name
  change_24h?: string;           // Optional 24h change percentage
}
```

### CachedPriceData

```typescript
interface CachedPriceData {
  asset: string;
  price: string;
  timestamp: string;             // ISO 8601 datetime
  source_node_id: string;        // UUID
  blockchain: string;
  change_24h?: string;
}
```

### NetworkStatus

```typescript
interface NetworkStatus {
  active_providers: ProviderInfo[];
  connected_peers: number;
  total_network_size: number;
  last_update_time: string | null;  // ISO 8601 datetime
  data_freshness: DataFreshness;
}
```

### ProviderInfo

```typescript
interface ProviderInfo {
  node_id: string;               // UUID
  last_seen: string;             // ISO 8601 datetime
  hop_count: number;             // Distance in hops
}
```

### DataFreshness

```typescript
type DataFreshness = 
  | "JustNow"
  | { MinutesAgo: number }
  | { HoursAgo: number }
  | "Stale";
```

---

## Rate Limiting

The mesh network implements intelligent rate limiting to prevent API abuse:

1. **Provider Coordination**: Multiple providers coordinate to avoid duplicate API calls within a 5-second window
2. **Fetch Interval**: Providers fetch data every 30 seconds by default
3. **Exponential Backoff**: Failed API calls retry with exponential backoff (1s, 2s, 4s)
4. **Maximum Retries**: Up to 3 retry attempts before giving up

**Requirements:** 2.3, 2.5, 11.2, 11.3, 11.4, 11.5

---

## Security Considerations

1. **API Key Protection**: API keys are never exposed in responses or logs
2. **Message Validation**: All incoming price updates are validated before processing
3. **Source Tracking**: All price data includes source node ID for accountability
4. **Malicious Node Detection**: Repeated validation failures are logged with source node ID
5. **TTL Enforcement**: Messages with TTL=0 are not relayed to prevent spam

**Requirements:** 14.1, 14.2, 14.3, 14.4, 14.5

---

## Performance Characteristics

- **Message Propagation**: Updates reach all nodes within 5 seconds in a 100-node network
- **Cache Lookup**: O(1) performance for price lookups
- **Memory Usage**: <50MB per node for price data and message tracking
- **Throughput**: 100+ messages per second per node
- **WebSocket Connections**: Supports 100+ concurrent connections per node

**Requirements:** 15.1, 15.2, 15.3, 15.4, 15.5

---

## Support and Troubleshooting

### Common Issues

**Issue**: Provider mode won't enable
- **Cause**: Invalid API key
- **Solution**: Verify API key with Birdeye, check for typos

**Issue**: No price updates received
- **Cause**: No providers online, network partition
- **Solution**: Enable provider mode or wait for providers to come online

**Issue**: Stale data warnings
- **Cause**: Providers offline for >1 hour
- **Solution**: Check provider status, enable provider mode

**Issue**: WebSocket disconnects frequently
- **Cause**: Network instability, server restarts
- **Solution**: Implement reconnection with exponential backoff

### Monitoring

Monitor these metrics for healthy operation:

- Active provider count (should be >0)
- Connected peer count (should be ≥3)
- Data freshness (should be <5 minutes)
- Message propagation latency (should be <5 seconds)
- Cache hit rate (should be >90%)

---

## Version History

- **v1.0.0** - Initial release with core mesh network functionality
- Requirements coverage: 1.1-15.5

