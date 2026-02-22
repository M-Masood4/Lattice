# P2P Mesh Network Setup Guide

## Current Status
✅ Backend running on `10.73.98.240:3000`
✅ Mesh price service started
✅ Provider mode enabled with Birdeye API key
✅ Frontend auto-detects API URL

## For You (Host - 10.73.98.240)

### 1. Access Your Dashboard
Open: `http://10.73.98.240:8080`

The frontend will automatically connect to your local API at `10.73.98.240:3000`

### 2. Enable Provider Mode (Already Done!)
Provider mode is already enabled with your Birdeye API key.

Check status:
```bash
curl http://10.73.98.240:3000/api/mesh/provider/status
```

### 3. Check Network Status
```bash
curl http://10.73.98.240:3000/api/mesh/network/status
```

## For Your Peer (10.73.126.198)

### 1. Access the Dashboard
Open: `http://10.73.98.240:8080`

The frontend will automatically detect it's accessing from a remote host and connect to `10.73.98.240:3000`

### 2. View Mesh Network Status
Once on the dashboard, they can:
- See the mesh network status
- View price data being distributed
- See you as an active provider node

## Testing the Mesh Network

### 1. Check Provider Status
```bash
# From either machine
curl http://10.73.98.240:3000/api/mesh/provider/status
```

Expected response:
```json
{
  "success": true,
  "data": {
    "enabled": true,
    "node_id": "b4da25d3-9f6e-4ea3-8189-86524598c7bf"
  }
}
```

### 2. Check Network Status
```bash
curl http://10.73.98.240:3000/api/mesh/network/status
```

Expected response:
```json
{
  "success": true,
  "data": {
    "active_providers": [
      {
        "node_id": "b4da25d3-9f6e-4ea3-8189-86524598c7bf",
        "last_seen": "2026-02-22T00:13:07Z"
      }
    ],
    "connected_peers": 0,
    "data_freshness": "Fresh",
    "total_network_size": 1
  }
}
```

### 3. Check Price Data
```bash
# Get all prices
curl http://10.73.98.240:3000/api/mesh/prices

# Get specific asset price
curl http://10.73.98.240:3000/api/mesh/prices/SOL
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Your Machine (Provider)                  │
│                      10.73.98.240                           │
│                                                             │
│  ┌──────────────┐         ┌─────────────────┐             │
│  │   Backend    │◄────────┤  Birdeye API    │             │
│  │   :3000      │         │  (Price Data)   │             │
│  └──────┬───────┘         └─────────────────┘             │
│         │                                                   │
│         │ Mesh Network                                     │
│         │ (P2P Distribution)                               │
│         │                                                   │
│  ┌──────▼───────┐                                          │
│  │   Frontend   │                                          │
│  │   :8080      │                                          │
│  └──────────────┘                                          │
└─────────────────────────────────────────────────────────────┘
                    │
                    │ WiFi (tcd.ie)
                    │
┌─────────────────────────────────────────────────────────────┐
│                    Peer Machine (Consumer)                   │
│                      10.73.126.198                          │
│                                                             │
│  Browser ──► http://10.73.98.240:8080                      │
│              (Auto-connects to 10.73.98.240:3000)          │
│                                                             │
│  Receives:                                                  │
│  - Real-time price updates                                 │
│  - Network status                                          │
│  - Provider information                                    │
└─────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### Frontend Not Connecting
1. Check the browser console for errors
2. Verify API_BASE_URL is set correctly:
   ```javascript
   console.log(API_BASE_URL)
   ```
3. Should show: `http://10.73.98.240:3000`

### No Price Data
1. Check provider status is enabled
2. Check Birdeye API key is valid
3. Check logs: `tail -f logs/api.log`

### Peer Can't Connect
1. Verify both machines are on same WiFi (tcd.ie)
2. Check firewall isn't blocking port 3000 or 8080
3. Test connectivity: `ping 10.73.98.240`

## API Endpoints

### Provider Management
- `POST /api/mesh/provider/enable` - Enable provider mode
- `POST /api/mesh/provider/disable` - Disable provider mode  
- `GET /api/mesh/provider/status` - Get provider status

### Price Data
- `GET /api/mesh/prices` - Get all cached prices
- `GET /api/mesh/prices/:asset` - Get specific asset price

### Network Status
- `GET /api/mesh/network/status` - Get network topology and status

## Next Steps

1. ✅ Backend is running with mesh service
2. ✅ Provider mode is enabled
3. ✅ Frontend auto-detects API URL
4. 🔄 Have your peer access `http://10.73.98.240:8080`
5. 🔄 Both of you should see the mesh network status

## Notes

- The mesh network uses P2P gossip protocol for price distribution
- Provider nodes fetch from Birdeye API every 30 seconds
- Consumer nodes receive updates via the mesh network
- All data is cached for offline resilience
- WebSocket support for real-time updates (coming soon)
