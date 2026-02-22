# P2P Mesh Network Price Distribution - Deployment Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Redis Setup](#redis-setup)
3. [Database Migration](#database-migration)
4. [Provider Node Setup](#provider-node-setup)
5. [Network Topology Recommendations](#network-topology-recommendations)
6. [Monitoring and Maintenance](#monitoring-and-maintenance)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

**Minimum Requirements (Consumer Node):**
- CPU: 1 core
- RAM: 512 MB
- Storage: 100 MB
- Network: Stable internet connection

**Recommended Requirements (Provider Node):**
- CPU: 2 cores
- RAM: 1 GB
- Storage: 500 MB
- Network: Stable internet connection with low latency

### Software Dependencies

- **Rust**: 1.70 or higher
- **PostgreSQL**: 13 or higher
- **Redis**: 6.0 or higher
- **Operating System**: Linux, macOS, or Windows

### API Keys

For provider nodes, you'll need:
- **Birdeye API Key**: Get from [https://birdeye.so/](https://birdeye.so/)
  - Free tier: 100 requests/day
  - Pro tier: Unlimited requests (recommended for production)

---

## Redis Setup

Redis is required for:
- Distributed coordination between provider nodes
- Seen message deduplication cache
- Price data caching for fast access

### Installation

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install redis-server
sudo systemctl start redis-server
sudo systemctl enable redis-server
```

#### macOS
```bash
brew install redis
brew services start redis
```

#### Docker
```bash
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:7-alpine
```

### Configuration

Edit `/etc/redis/redis.conf` (or create a custom config):

```conf
# Bind to all interfaces (or specific IP)
bind 0.0.0.0

# Set password for security
requirepass your_secure_password_here

# Enable persistence
save 900 1
save 300 10
save 60 10000

# Set max memory and eviction policy
maxmemory 256mb
maxmemory-policy allkeys-lru

# Enable AOF for durability
appendonly yes
appendfsync everysec
```

### Verify Installation

```bash
redis-cli ping
# Should return: PONG

# Test with password
redis-cli -a your_secure_password_here ping
# Should return: PONG
```

### Environment Configuration

Add to your `.env` file:

```bash
REDIS_URL=redis://:your_secure_password_here@localhost:6379
REDIS_POOL_SIZE=10
```

**Requirements:** 5.5, 11.2

---

## Database Migration

The mesh network requires two database tables for persistent storage.

### Migration Files

The migrations are located in `crates/database/migrations/`:

1. **mesh_price_cache** - Stores cached price data
2. **mesh_seen_messages** - Tracks seen message IDs

### Running Migrations

#### Using sqlx-cli

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run --database-url "postgresql://user:password@localhost:5432/whale_tracker"
```

#### Manual Migration

If you prefer to run migrations manually:

```sql
-- Migration: Create mesh_price_cache table
CREATE TABLE IF NOT EXISTS mesh_price_cache (
    asset VARCHAR(255) PRIMARY KEY,
    price VARCHAR(255) NOT NULL,
    blockchain VARCHAR(50) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    source_node_id UUID NOT NULL,
    change_24h VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mesh_price_cache_timestamp 
    ON mesh_price_cache(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_mesh_price_cache_blockchain 
    ON mesh_price_cache(blockchain);

-- Migration: Create mesh_seen_messages table
CREATE TABLE IF NOT EXISTS mesh_seen_messages (
    message_id UUID PRIMARY KEY,
    seen_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mesh_seen_messages_expires 
    ON mesh_seen_messages(expires_at);
```

### Verify Migrations

```bash
# Check tables exist
psql -U user -d whale_tracker -c "\dt mesh_*"

# Should show:
# mesh_price_cache
# mesh_seen_messages
```

### Database Maintenance

Set up a cron job to clean up expired seen messages:

```sql
-- Run daily to clean up expired messages
DELETE FROM mesh_seen_messages 
WHERE expires_at < NOW() - INTERVAL '1 day';
```

**Requirements:** 5.5, 6.1, 6.3

---

## Provider Node Setup

Provider nodes fetch price data from Birdeye API and broadcast to the network.

### Step 1: Get Birdeye API Key

1. Visit [https://birdeye.so/](https://birdeye.so/)
2. Sign up for an account
3. Navigate to API settings
4. Generate an API key
5. Choose appropriate tier:
   - **Free**: 100 requests/day (testing only)
   - **Pro**: Unlimited requests (production)

### Step 2: Configure Environment

Add to your `.env` file:

```bash
# Birdeye API Configuration
BIRDEYE_API_KEY=your_birdeye_api_key_here

# Mesh Network Configuration
MESH_PROVIDER_FETCH_INTERVAL_SECS=30
MESH_COORDINATION_WINDOW_SECS=5
MESH_MESSAGE_TTL=10
```

### Step 3: Enable Provider Mode

#### Via API

```bash
curl -X POST http://localhost:8080/api/mesh/provider/enable \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your_birdeye_api_key_here"}'
```

#### Via UI

1. Navigate to Settings → Mesh Network
2. Toggle "Provider Mode" to ON
3. Enter your Birdeye API key
4. Click "Enable"

### Step 4: Verify Provider Status

```bash
curl http://localhost:8080/api/mesh/provider/status

# Should return:
# {
#   "success": true,
#   "data": {
#     "enabled": true,
#     "node_id": "550e8400-e29b-41d4-a716-446655440000"
#   }
# }
```

### Step 5: Monitor Provider Activity

Check logs for successful fetches:

```bash
tail -f logs/app.log | grep "Provider"

# Should see:
# [INFO] Provider node fetching price data
# [INFO] Provider node broadcast complete: 50 assets to 5 peers
```

### Provider Node Best Practices

1. **API Key Security**
   - Never commit API keys to version control
   - Use environment variables or secrets management
   - Rotate keys periodically

2. **Fetch Interval**
   - Default: 30 seconds (recommended)
   - High-frequency: 10-15 seconds (requires Pro API tier)
   - Low-bandwidth: 60+ seconds

3. **Coordination**
   - Multiple providers automatically coordinate
   - No manual configuration needed
   - 5-second coordination window prevents duplicate fetches

4. **Monitoring**
   - Monitor fetch success rate (should be >95%)
   - Monitor broadcast success rate (should be >90%)
   - Alert on API key expiration or rate limits

**Requirements:** 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5

---

## Network Topology Recommendations

### Small Network (1-10 nodes)

**Recommended Setup:**
- 1-2 provider nodes
- All nodes connected in a mesh
- Min peer connections: 2
- Max peer connections: 5

```
Provider1 ←→ Consumer1 ←→ Consumer2
    ↓            ↓            ↓
Provider2 ←→ Consumer3 ←→ Consumer4
```

**Configuration:**
```bash
MESH_MIN_PEER_CONNECTIONS=2
MESH_MAX_PEER_CONNECTIONS=5
MESH_MESSAGE_TTL=5
```

### Medium Network (10-50 nodes)

**Recommended Setup:**
- 2-3 provider nodes (geographically distributed)
- Hierarchical topology with relay nodes
- Min peer connections: 3
- Max peer connections: 8

```
        Provider1 (US-East)
           /    \
    Relay1      Relay2
     /  \        /  \
  C1   C2      C3   C4
  
        Provider2 (EU-West)
           /    \
    Relay3      Relay4
     /  \        /  \
  C5   C6      C7   C8
```

**Configuration:**
```bash
MESH_MIN_PEER_CONNECTIONS=3
MESH_MAX_PEER_CONNECTIONS=8
MESH_MESSAGE_TTL=7
```

### Large Network (50-100+ nodes)

**Recommended Setup:**
- 3-5 provider nodes (globally distributed)
- Multi-tier relay hierarchy
- Min peer connections: 3
- Max peer connections: 10

```
Provider1 (US)  Provider2 (EU)  Provider3 (APAC)
     |               |                |
  Tier1          Tier1            Tier1
  Relays         Relays           Relays
     |               |                |
  Tier2          Tier2            Tier2
  Relays         Relays           Relays
     |               |                |
 Consumers      Consumers        Consumers
```

**Configuration:**
```bash
MESH_MIN_PEER_CONNECTIONS=3
MESH_MAX_PEER_CONNECTIONS=10
MESH_MESSAGE_TTL=10
```

### Topology Best Practices

1. **Provider Distribution**
   - Distribute providers geographically for redundancy
   - Avoid single points of failure
   - Aim for 2-5 providers depending on network size

2. **Connection Strategy**
   - Prefer peers with lower hop counts to providers
   - Maintain connections on shortest paths
   - Limit max connections to prevent resource exhaustion

3. **TTL Configuration**
   - Small networks: TTL 5-7
   - Medium networks: TTL 7-10
   - Large networks: TTL 10-15
   - Higher TTL = more redundancy but more bandwidth

4. **Peer Discovery**
   - Use existing proximity P2P discovery (BLE/mDNS)
   - Nodes automatically discover and connect to nearby peers
   - No manual peer configuration needed

**Requirements:** 10.1, 10.2, 10.3, 10.5, 13.1, 13.2, 13.3, 13.4, 13.5

---

## Monitoring and Maintenance

### Key Metrics to Monitor

#### Provider Metrics

```bash
# Fetch success rate
mesh_provider_fetch_success_total / mesh_provider_fetch_attempts_total

# Broadcast success rate
mesh_provider_broadcast_success_total / mesh_provider_broadcast_attempts_total

# API latency
mesh_provider_api_latency_seconds
```

#### Network Metrics

```bash
# Active provider count (should be >0)
mesh_network_active_providers

# Connected peer count (should be ≥3)
mesh_network_connected_peers

# Message propagation latency (should be <5s)
mesh_message_propagation_latency_seconds
```

#### Cache Metrics

```bash
# Cache hit rate (should be >90%)
mesh_cache_hits_total / (mesh_cache_hits_total + mesh_cache_misses_total)

# Data freshness (should be <300s)
mesh_cache_data_age_seconds

# Seen messages cache size
mesh_seen_messages_cache_size
```

### Health Checks

#### Provider Health Check

```bash
#!/bin/bash
# check_provider_health.sh

STATUS=$(curl -s http://localhost:8080/api/mesh/provider/status)
ENABLED=$(echo $STATUS | jq -r '.data.enabled')

if [ "$ENABLED" = "true" ]; then
  echo "Provider is healthy"
  exit 0
else
  echo "Provider is not enabled"
  exit 1
fi
```

#### Network Health Check

```bash
#!/bin/bash
# check_network_health.sh

STATUS=$(curl -s http://localhost:8080/api/mesh/network/status)
PROVIDERS=$(echo $STATUS | jq -r '.data.active_providers | length')
PEERS=$(echo $STATUS | jq -r '.data.connected_peers')

if [ "$PROVIDERS" -gt 0 ] && [ "$PEERS" -ge 3 ]; then
  echo "Network is healthy"
  exit 0
else
  echo "Network health issue: providers=$PROVIDERS, peers=$PEERS"
  exit 1
fi
```

### Maintenance Tasks

#### Daily Tasks

1. **Check provider status**
   ```bash
   curl http://localhost:8080/api/mesh/provider/status
   ```

2. **Monitor logs for errors**
   ```bash
   grep -i error logs/app.log | tail -20
   ```

3. **Verify data freshness**
   ```bash
   curl http://localhost:8080/api/mesh/network/status | jq '.data.data_freshness'
   ```

#### Weekly Tasks

1. **Clean up expired seen messages**
   ```sql
   DELETE FROM mesh_seen_messages 
   WHERE expires_at < NOW() - INTERVAL '7 days';
   ```

2. **Review price discrepancy logs**
   ```bash
   grep "price discrepancy" logs/app.log | tail -50
   ```

3. **Check Redis memory usage**
   ```bash
   redis-cli info memory
   ```

#### Monthly Tasks

1. **Rotate API keys** (if required by policy)
2. **Review and optimize configuration**
3. **Update software dependencies**
4. **Backup database and Redis data**

### Alerting

Set up alerts for:

- **Critical**: All providers offline for >10 minutes
- **Critical**: Database connection failures
- **Critical**: Redis connection failures
- **Warning**: Data staleness >1 hour
- **Warning**: Provider fetch failure rate >10%
- **Warning**: Price discrepancy detected
- **Info**: New provider joined network
- **Info**: Provider disconnected

**Requirements:** 2.4, 8.4, 9.2, 9.5, 14.5

---

## Troubleshooting

### Common Issues

#### Issue: Provider mode won't enable

**Symptoms:**
- API returns 401 Unauthorized
- Error: "Invalid API key"

**Diagnosis:**
```bash
# Test API key directly with Birdeye
curl -H "X-API-KEY: your_key_here" \
  https://public-api.birdeye.so/public/price?address=So11111111111111111111111111111111111111112
```

**Solutions:**
1. Verify API key is correct (no extra spaces)
2. Check API key hasn't expired
3. Verify API tier has sufficient quota
4. Check Birdeye API status

**Requirements:** 1.1, 1.3

---

#### Issue: No price updates received

**Symptoms:**
- Network status shows 0 active providers
- Data freshness shows "Stale"
- No WebSocket updates

**Diagnosis:**
```bash
# Check network status
curl http://localhost:8080/api/mesh/network/status

# Check connected peers
curl http://localhost:8080/api/mesh/network/status | jq '.data.connected_peers'

# Check logs
grep "price update" logs/app.log | tail -20
```

**Solutions:**
1. Enable provider mode on at least one node
2. Check peer connections (need ≥1 peer)
3. Verify network connectivity
4. Check firewall rules
5. Restart proximity P2P service

**Requirements:** 9.2, 10.2, 10.5

---

#### Issue: Stale data warnings

**Symptoms:**
- UI shows "Data is X hours old"
- Data freshness shows "HoursAgo(n)"

**Diagnosis:**
```bash
# Check when last update was received
curl http://localhost:8080/api/mesh/network/status | jq '.data.last_update_time'

# Check provider status
curl http://localhost:8080/api/mesh/provider/status
```

**Solutions:**
1. Check if providers are online
2. Enable provider mode if none active
3. Check provider fetch logs for errors
4. Verify Birdeye API is responding
5. Check coordination service (Redis)

**Requirements:** 6.5, 7.1, 9.1, 9.2

---

#### Issue: High message propagation latency

**Symptoms:**
- Updates take >10 seconds to reach all nodes
- Network feels sluggish

**Diagnosis:**
```bash
# Check network topology
curl http://localhost:8080/api/mesh/network/status

# Check peer hop counts
curl http://localhost:8080/api/mesh/network/status | jq '.data.active_providers[].hop_count'

# Check logs for relay delays
grep "relay" logs/app.log | tail -50
```

**Solutions:**
1. Optimize network topology (reduce hop counts)
2. Increase max peer connections
3. Add more relay nodes in strategic locations
4. Check network bandwidth
5. Reduce TTL if network is small

**Requirements:** 13.1, 13.2, 13.3, 15.1

---

#### Issue: Redis connection failures

**Symptoms:**
- Logs show "Redis connection failed"
- Coordination not working
- Seen messages not persisting

**Diagnosis:**
```bash
# Test Redis connection
redis-cli -h localhost -p 6379 -a your_password ping

# Check Redis logs
tail -f /var/log/redis/redis-server.log

# Check Redis memory
redis-cli info memory
```

**Solutions:**
1. Verify Redis is running: `systemctl status redis`
2. Check Redis configuration
3. Verify password in REDIS_URL
4. Check Redis memory limits
5. Restart Redis if needed
6. System falls back to in-memory cache automatically

**Requirements:** 5.5, 11.2

---

#### Issue: Database write failures

**Symptoms:**
- Logs show "Database write failed"
- Cache not persisting across restarts

**Diagnosis:**
```bash
# Test database connection
psql -U user -d whale_tracker -c "SELECT 1"

# Check database logs
tail -f /var/log/postgresql/postgresql-13-main.log

# Check disk space
df -h
```

**Solutions:**
1. Verify PostgreSQL is running
2. Check database connection string
3. Verify database user permissions
4. Check disk space
5. Check database locks
6. System continues with Redis cache

**Requirements:** 6.3

---

#### Issue: Price discrepancy warnings

**Symptoms:**
- UI shows "Price discrepancy detected"
- Logs show providers reporting different prices

**Diagnosis:**
```bash
# Check which providers are reporting discrepancies
grep "price discrepancy" logs/app.log | tail -20

# Get prices from all providers
curl http://localhost:8080/api/mesh/prices
```

**Solutions:**
1. This is often normal (different data sources)
2. Check if one provider is consistently wrong
3. Verify provider API keys are valid
4. Check if providers are using different Birdeye endpoints
5. Adjust MESH_PRICE_DISCREPANCY_THRESHOLD_PERCENT if needed

**Requirements:** 8.2, 8.4

---

### Performance Tuning

#### For High-Frequency Trading

```bash
# Aggressive configuration
MESH_PROVIDER_FETCH_INTERVAL_SECS=10
MESH_COORDINATION_WINDOW_SECS=2
MESH_STALENESS_THRESHOLD_SECS=300
MESH_MESSAGE_TTL=7
```

#### For Low-Bandwidth Networks

```bash
# Conservative configuration
MESH_PROVIDER_FETCH_INTERVAL_SECS=60
MESH_COORDINATION_WINDOW_SECS=10
MESH_MAX_PEER_CONNECTIONS=5
MESH_MESSAGE_TTL=5
```

#### For Large Networks

```bash
# Scalable configuration
MESH_MAX_PEER_CONNECTIONS=10
MESH_MIN_PEER_CONNECTIONS=3
MESH_MESSAGE_TTL=10
MESH_SEEN_MESSAGES_CACHE_SIZE=20000
```

---

## Security Considerations

### API Key Management

1. **Never commit API keys to version control**
   ```bash
   # Add to .gitignore
   echo ".env" >> .gitignore
   ```

2. **Use environment variables**
   ```bash
   export BIRDEYE_API_KEY="your_key_here"
   ```

3. **Rotate keys periodically**
   - Set calendar reminder for quarterly rotation
   - Update all provider nodes simultaneously

### Network Security

1. **Validate all incoming messages**
   - System automatically validates message structure
   - Rejects negative prices, future timestamps
   - Logs validation failures with source node ID

2. **Monitor for malicious nodes**
   ```bash
   # Check for repeated validation failures
   grep "validation failed" logs/app.log | \
     awk '{print $NF}' | sort | uniq -c | sort -rn
   ```

3. **Use secure Redis**
   - Enable password authentication
   - Bind to specific interfaces
   - Use TLS for production

### Data Integrity

1. **Source tracking**
   - All price data includes source_node_id
   - Enables accountability and debugging

2. **Timestamp validation**
   - Rejects future timestamps
   - Prevents time-based attacks

3. **TTL enforcement**
   - Prevents infinite message loops
   - Limits network flooding

**Requirements:** 14.1, 14.2, 14.3, 14.4, 14.5

---

## Backup and Recovery

### Backup Strategy

#### Redis Backup

```bash
# Manual backup
redis-cli SAVE
cp /var/lib/redis/dump.rdb /backup/redis-$(date +%Y%m%d).rdb

# Automated daily backup
0 2 * * * redis-cli SAVE && cp /var/lib/redis/dump.rdb /backup/redis-$(date +\%Y\%m\%d).rdb
```

#### Database Backup

```bash
# Manual backup
pg_dump -U user whale_tracker > /backup/whale_tracker-$(date +%Y%m%d).sql

# Automated daily backup
0 3 * * * pg_dump -U user whale_tracker > /backup/whale_tracker-$(date +\%Y\%m\%d).sql
```

### Recovery Procedures

#### Redis Recovery

```bash
# Stop Redis
systemctl stop redis

# Restore backup
cp /backup/redis-20240101.rdb /var/lib/redis/dump.rdb

# Start Redis
systemctl start redis
```

#### Database Recovery

```bash
# Drop and recreate database
dropdb whale_tracker
createdb whale_tracker

# Restore backup
psql -U user whale_tracker < /backup/whale_tracker-20240101.sql

# Run migrations if needed
sqlx migrate run
```

---

## Scaling Considerations

### Horizontal Scaling

The mesh network scales horizontally by design:

1. **Add more provider nodes** for redundancy
2. **Add more relay nodes** to reduce hop counts
3. **Add more consumer nodes** without limit

### Vertical Scaling

For high-traffic nodes:

1. **Increase Redis memory**
   ```conf
   maxmemory 512mb
   ```

2. **Increase database connections**
   ```bash
   DATABASE_MAX_CONNECTIONS=20
   ```

3. **Increase peer connections**
   ```bash
   MESH_MAX_PEER_CONNECTIONS=15
   ```

### Load Balancing

For provider nodes:

1. **Geographic distribution** reduces latency
2. **Time-based rotation** spreads API load
3. **Automatic coordination** prevents duplicate fetches

---

## Support

For issues not covered in this guide:

1. Check application logs: `logs/app.log`
2. Check system logs: `journalctl -u whale_tracker`
3. Review GitHub issues
4. Contact support team

---

## Appendix: Quick Reference

### Essential Commands

```bash
# Enable provider mode
curl -X POST http://localhost:8080/api/mesh/provider/enable \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your_key"}'

# Check status
curl http://localhost:8080/api/mesh/network/status

# Get prices
curl http://localhost:8080/api/mesh/prices

# Check logs
tail -f logs/app.log | grep mesh

# Test Redis
redis-cli ping

# Test database
psql -U user -d whale_tracker -c "SELECT COUNT(*) FROM mesh_price_cache"
```

### Configuration Quick Reference

| Setting | Default | Range | Use Case |
|---------|---------|-------|----------|
| Fetch Interval | 30s | 10-300s | How often to fetch |
| Coordination Window | 5s | 2-30s | Duplicate prevention |
| Message TTL | 10 | 5-20 | Network size |
| Max Connections | 10 | 5-20 | Resource limits |
| Min Connections | 3 | 2-5 | Reliability |
| Staleness Threshold | 3600s | 300-7200s | Data freshness |

---

**Document Version:** 1.0.0  
**Last Updated:** 2024-01-01  
**Requirements Coverage:** 1.1-15.5
