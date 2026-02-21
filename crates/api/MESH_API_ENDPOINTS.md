# Mesh Price Service API Endpoints

This document describes the REST API endpoints for the P2P mesh network price distribution system.

## Provider Management Endpoints

### Enable Provider Mode
**POST** `/api/mesh/provider/enable`

Enable provider mode with API key validation.

**Request Body:**
```json
{
  "api_key": "your-birdeye-api-key"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "enabled": true,
    "node_id": "uuid-of-node"
  }
}
```

**Status Codes:**
- `200 OK` - Provider mode enabled successfully
- `401 Unauthorized` - Invalid API key
- `500 Internal Server Error` - Server error

**Requirements:** 1.1, 1.2

---

### Disable Provider Mode
**POST** `/api/mesh/provider/disable`

Disable provider mode and stop fetching price data.

**Response:**
```json
{
  "success": true,
  "data": {
    "enabled": false,
    "node_id": "uuid-of-node"
  }
}
```

**Status Codes:**
- `200 OK` - Provider mode disabled successfully
- `500 Internal Server Error` - Server error

**Requirements:** 1.2, 1.5

---

### Get Provider Status
**GET** `/api/mesh/provider/status`

Get current provider mode status.

**Response:**
```json
{
  "success": true,
  "data": {
    "enabled": true,
    "node_id": "uuid-of-node"
  }
}
```

**Status Codes:**
- `200 OK` - Status retrieved successfully

**Requirements:** 1.4, 1.5

---

## Price Data Access Endpoints

### Get All Cached Prices
**GET** `/api/mesh/prices`

Retrieve all cached price data from the mesh network.

**Response:**
```json
{
  "success": true,
  "data": {
    "SOL": {
      "asset": "SOL",
      "price": "123.45",
      "timestamp": "2024-01-01T12:00:00Z",
      "source_node_id": "uuid-of-source",
      "blockchain": "Solana"
    },
    "ETH": {
      "asset": "ETH",
      "price": "2345.67",
      "timestamp": "2024-01-01T12:00:00Z",
      "source_node_id": "uuid-of-source",
      "blockchain": "Ethereum"
    }
  }
}
```

**Status Codes:**
- `200 OK` - Prices retrieved successfully
- `500 Internal Server Error` - Server error

**Requirements:** 6.4

---

### Get Price for Specific Asset
**GET** `/api/mesh/prices/:asset`

Retrieve cached price data for a specific asset.

**Path Parameters:**
- `asset` - Asset symbol (e.g., "SOL", "ETH")

**Response:**
```json
{
  "success": true,
  "data": {
    "asset": "SOL",
    "price": "123.45",
    "timestamp": "2024-01-01T12:00:00Z",
    "source_node_id": "uuid-of-source",
    "blockchain": "Solana"
  }
}
```

**Status Codes:**
- `200 OK` - Price retrieved successfully
- `404 Not Found` - Asset not found in cache
- `500 Internal Server Error` - Server error

**Requirements:** 6.4

---

### Get Network Status
**GET** `/api/mesh/network/status`

Get current mesh network status and topology information.

**Response:**
```json
{
  "success": true,
  "data": {
    "active_providers": [
      {
        "node_id": "uuid-of-provider",
        "last_seen": "2024-01-01T12:00:00Z",
        "hop_count": 1
      }
    ],
    "connected_peers": 5,
    "total_network_size": 10,
    "last_update_time": "2024-01-01T12:00:00Z",
    "data_freshness": "JustNow"
  }
}
```

**Status Codes:**
- `200 OK` - Network status retrieved successfully
- `500 Internal Server Error` - Server error

**Requirements:** 8.5

---

## Error Response Format

All endpoints return errors in the following format:

```json
{
  "success": false,
  "data": null,
  "error": "Error message describing what went wrong"
}
```

## Integration Notes

1. **AppState Integration**: The mesh price service has been added to the application state and is available to all handlers.

2. **Service Initialization**: The mesh price service is initialized in `main.rs` with:
   - Birdeye service for API access
   - Peer connection manager for P2P communication
   - Redis connection for caching
   - Database pool for persistence
   - WebSocket service for real-time updates

3. **Route Registration**: All routes are registered in `routes.rs` under the `/api/mesh/` prefix.

4. **Dependencies**: The service depends on:
   - `proximity::PeerConnectionManager` for P2P networking
   - `BirdeyeService` for price data fetching
   - Redis for distributed coordination and caching
   - PostgreSQL for persistent storage
   - WebSocket service for pushing updates to clients

## Testing

To test the endpoints:

1. **Enable provider mode:**
   ```bash
   curl -X POST http://localhost:8080/api/mesh/provider/enable \
     -H "Content-Type: application/json" \
     -d '{"api_key": "your-api-key"}'
   ```

2. **Get provider status:**
   ```bash
   curl http://localhost:8080/api/mesh/provider/status
   ```

3. **Get all prices:**
   ```bash
   curl http://localhost:8080/api/mesh/prices
   ```

4. **Get specific asset price:**
   ```bash
   curl http://localhost:8080/api/mesh/prices/SOL
   ```

5. **Get network status:**
   ```bash
   curl http://localhost:8080/api/mesh/network/status
   ```

6. **Disable provider mode:**
   ```bash
   curl -X POST http://localhost:8080/api/mesh/provider/disable
   ```
