# Local Deployment with Public Access

## Status: ✅ LIVE

Your application is now running locally and exposed to the internet via localtunnel.

## Access URLs

### For You (Local)
- **Frontend**: http://localhost:8080
- **API**: http://localhost:3000
- **Health Check**: http://localhost:3000/health

### For Your Peer (Public Internet)
- **Public URL**: https://33d95d77b2e10638-193-1-64-12.serveousercontent.com
- **Frontend**: https://33d95d77b2e10638-193-1-64-12.serveousercontent.com

**Note**: This URL is live and accessible immediately - no warning pages!

## P2P Mesh Network Testing

### Your Information
- **Local IP**: 10.73.98.240
- **Network**: tcd.ie WiFi
- **Subnet**: 255.255.0.0 (same as peer)

### Your Peer's Information  
- **IP**: 10.73.126.198
- **Subnet**: 255.255.0.0
- **Gateway**: 10.73.0.1

### To Find Your Local IP
```bash
ifconfig | grep "inet " | grep -v 127.0.0.1
```

### Testing P2P Connection

Both you and your peer should:

1. **Access the application**:
   - You: http://localhost:8080
   - Peer: https://33d95d77b2e10638-193-1-64-12.serveousercontent.com

2. **Enable P2P Mesh Provider**:
   - Toggle the "Enable as Price Provider" switch in the UI
   - This makes your node broadcast price data to peers

3. **Check Network Status**:
   - Look at the "Mesh Network Status" section
   - You should see:
     - Active Providers: 2 (when both are online)
     - Connected Peers: 1 (each sees the other)
     - Messages Received: increasing count

4. **Verify Price Distribution**:
   - Prices should show green "Fresh" indicators
   - Last update timestamps should be recent
   - Both nodes should see similar prices

## Running Services

- ✅ PostgreSQL (port 5432)
- ✅ Redis (port 6379)
- ✅ API Server (port 3000)
- ✅ Frontend Server (port 8080)
- ✅ Public Tunnel (https://33d95d77b2e10638-193-1-64-12.serveousercontent.com)

## Stopping the Tunnel

When you're done testing:
```bash
# Stop the serveo tunnel
pkill -f "ssh -R"
```

## Troubleshooting

### Tunnel Not Working
If the public URL doesn't work:
1. Check tunnel status: `ps aux | grep "ssh -R"`
2. Restart tunnel: `pkill -f "ssh -R" && ssh -R 80:localhost:8080 serveo.net`

### P2P Not Connecting
1. Verify both nodes are on same WiFi (tcd.ie)
2. Check firewall settings aren't blocking WebSocket connections
3. Look at browser console for connection errors
4. Verify "Enable as Price Provider" is toggled ON

### API Not Responding
1. Check API is running: `ps aux | grep api`
2. Check logs: `tail -f logs/api.log` (if logging to file)
3. Restart API: `./start.sh`

## Notes

- The tunnel URL (https://33d95d77b2e10638-193-1-64-12.serveousercontent.com) is temporary and will change if you restart the tunnel
- Both users need to be on the same WiFi network for P2P mesh to work optimally
- WebSocket connections are used for real-time P2P communication
- The mesh network uses gossip protocol to distribute price data between peers
