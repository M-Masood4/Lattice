# Mobile Hotspot Setup - P2P Mesh Network Testing

## ✅ Current Status
- **Backend Server IP:** 10.203.223.111
- **Backend Port:** 3000
- **Frontend Port:** 8080
- **Network:** Mobile Hotspot (bypasses university WiFi restrictions)
- **Status:** All endpoints responding correctly

## 📱 For Your Peer to Connect

### Step 1: Connect to the Same Hotspot
Make sure your peer is connected to the same mobile hotspot as you.

### Step 2: Access the Mesh Test Page
Your peer should open their browser and navigate to:

```
http://10.203.223.111:8080/mesh-test.html
```

### Step 3: Verify Connectivity
The page should automatically:
- Load the mesh network status
- Display cached price data (SOL, ETH, BTC)
- Show network information
- Auto-refresh every 10 seconds

### Alternative: Network Diagnostic Tool
If there are any issues, use the diagnostic tool:

```
http://10.203.223.111:8080/network-diagnostic.html
```

This will run comprehensive connectivity tests and show detailed error messages.

## 🔧 API Endpoints Available

All endpoints are accessible at `http://10.203.223.111:3000`

### Health Check
```bash
curl http://10.203.223.111:3000/health
```

### Mesh Network Status
```bash
curl http://10.203.223.111:3000/api/mesh/network/status
```

### Mesh Prices (All Assets)
```bash
curl http://10.203.223.111:3000/api/mesh/prices
```

### Mesh Price (Specific Asset)
```bash
curl http://10.203.223.111:3000/api/mesh/prices/SOL
curl http://10.203.223.111:3000/api/mesh/prices/ETH
curl http://10.203.223.111:3000/api/mesh/prices/BTC
```

## 📊 Current Cached Price Data

The system has the following cached prices:
- **SOL:** $105.75 (Solana)
- **ETH:** $2,000.00 (Ethereum)
- **BTC:** $45,000.00 (Bitcoin)

## 🔄 Provider Mode

Provider mode is currently **disabled**. To enable it and fetch live prices from Birdeye:

```bash
curl -X POST http://10.203.223.111:3000/api/mesh/provider/enable \
  -H "Content-Type: application/json" \
  -d '{"api_key": "fb5da84450bf4d49963bb14c8ee845e9"}'
```

Once enabled, the provider will:
- Fetch fresh price data from Birdeye API every 30 seconds
- Broadcast updates to all connected peers
- Update the cached data automatically

## 🌐 P2P Mesh Network Testing

### What to Test:
1. **Data Access:** Both devices should see the same cached price data
2. **Network Status:** Check active providers and connected peers
3. **Real-time Updates:** When provider mode is enabled, both should see updates
4. **Peer Discovery:** The mesh network should detect connected peers

### Expected Behavior:
- **Without Provider Mode:** Both devices serve cached data
- **With Provider Mode:** Provider fetches fresh data and broadcasts to peers
- **Auto-refresh:** Frontend updates every 10 seconds

## 🐛 Troubleshooting

### If the peer can't connect:
1. Verify both devices are on the same hotspot
2. Check the IP address hasn't changed: `ifconfig | grep "inet "`
3. Ensure backend is running: `ps aux | grep api`
4. Test locally first: `curl http://10.203.223.111:3000/health`

### If data isn't updating:
1. Check provider mode status: `curl http://10.203.223.111:3000/api/mesh/provider/status`
2. Enable provider mode (see command above)
3. Check backend logs for errors

### If frontend doesn't load:
1. Verify frontend server is running on port 8080
2. Try accessing directly: `http://10.203.223.111:8080/`
3. Check browser console for errors

## 📝 Notes

- The mobile hotspot removes WiFi client isolation that was blocking connections on tcd.ie
- All devices on the hotspot can communicate directly with each other
- The backend is listening on all interfaces (*:3000) so it's accessible from any device on the network
- Price data is cached in Redis and persists even when provider mode is disabled

## 🎯 Next Steps

1. Have your peer access `http://10.203.223.111:8080/mesh-test.html`
2. Verify they can see the cached price data
3. Enable provider mode to test live data fetching
4. Monitor the network status to see peer connections
5. Test the P2P mesh network functionality

---

**Last Updated:** Connected to mobile hotspot at 10.203.223.111
**Backend Status:** Running and responding in ~3ms
**Cached Data:** SOL, ETH, BTC prices available
