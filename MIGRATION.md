# Migration Guide: Solana Whale Tracker → Crypto Trading Platform

This guide helps existing Solana Whale Tracker users migrate to the enhanced Crypto Trading Platform with multi-chain support and new features.

## Overview

The Crypto Trading Platform is a major enhancement that adds:
- Multi-chain support (Ethereum, BSC, Polygon)
- In-app conversions and swaps
- P2P exchange
- Voice trading
- Agentic position trimming
- Blockchain receipts
- Enhanced privacy features

**All existing Solana whale tracking features remain fully functional.**

## Breaking Changes

### None for Core Features

The migration is designed to be **backward compatible**. Your existing:
- Solana wallets continue to work
- Whale tracking remains active
- AI recommendations continue
- Notifications keep working
- Subscriptions are preserved

### New Requirements

1. **Additional Environment Variables**: New API keys required for enhanced features
2. **Database Migrations**: Automatic schema updates on first startup
3. **User Tags**: All users automatically get anonymous tags generated

## Migration Steps

### Step 1: Backup Your Data

```bash
# Backup PostgreSQL database
pg_dump -h localhost -U your_user whale_tracker > backup_$(date +%Y%m%d).sql

# Backup Redis (optional, cache data)
redis-cli SAVE
cp /var/lib/redis/dump.rdb backup_redis_$(date +%Y%m%d).rdb
```

### Step 2: Update Environment Variables

Add new required variables to your `.env` file:

```env
# New Multi-Chain RPC Endpoints
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
BSC_RPC_URL=https://bsc-dataseed.binance.org
POLYGON_RPC_URL=https://polygon-rpc.com

# New External APIs
BIRDEYE_API_KEY=your_birdeye_api_key
SIDESHIFT_API_KEY=your_sideshift_api_key

# Optional: Voice Trading
INTERCOM_API_KEY=your_intercom_api_key

# Optional: KYC Verification
KYC_PROVIDER_API_KEY=your_kyc_provider_key

# Feature Flags (all default to true)
ENABLE_VOICE_TRADING=true
ENABLE_P2P_EXCHANGE=true
ENABLE_AGENTIC_TRIMMING=true
```

**Minimum Required for Basic Multi-Chain Support:**
- `ETHEREUM_RPC_URL`
- `BSC_RPC_URL`
- `POLYGON_RPC_URL`
- `BIRDEYE_API_KEY`
- `SIDESHIFT_API_KEY`

### Step 3: Update Application Code

```bash
# Pull latest code
git pull origin main

# Rebuild application
cargo build --release

# Or with Docker
docker-compose pull
docker-compose build
```

### Step 4: Run Database Migrations

Migrations run automatically on startup, but you can run them manually:

```bash
# Manual migration
DATABASE_URL=your_database_url cargo run --bin migrate

# Or let the application run them
cargo run --bin api
```

**Migrations will add:**
- `multi_chain_wallets` table
- `benchmarks` table
- `conversions` table
- `staking_positions` table
- `trim_configs` and `trim_executions` tables
- `voice_commands` table
- `blockchain_receipts` table
- `chat_messages` table
- `p2p_offers` and `p2p_exchanges` tables
- `identity_verifications` and `wallet_verifications` tables
- `user_tag` column to `users` table

### Step 5: Restart Application

```bash
# With Docker
docker-compose down
docker-compose up -d

# Manual
systemctl restart whale-tracker
# or
./target/release/api
```

### Step 6: Verify Migration

```bash
# Check health endpoint
curl http://localhost:3000/health

# Expected response includes new services
{
  "status": "healthy",
  "services": {
    "database": "healthy",
    "redis": "healthy",
    "solana_rpc": "healthy",
    "ethereum_rpc": "healthy",
    "birdeye_api": "healthy",
    "sideshift_api": "healthy"
  }
}
```

## Feature-by-Feature Migration

### Multi-Chain Wallets

**Before:** Only Solana wallets supported

**After:** Connect wallets from Ethereum, BSC, Polygon

**Action Required:**
1. Navigate to Wallet Settings
2. Click "Add Wallet"
3. Select blockchain (Ethereum, BSC, or Polygon)
4. Connect wallet via MetaMask or WalletConnect

**API Changes:**
```bash
# Old: Solana-only
POST /api/wallets/connect
{
  "address": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"
}

# New: Multi-chain
POST /api/wallets/connect
{
  "blockchain": "ethereum",
  "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
}
```

### Portfolio View

**Before:** Solana assets only

**After:** Aggregated view across all chains

**Action Required:** None - automatic

**What Changes:**
- Portfolio now shows positions grouped by blockchain
- Total value aggregates all chains
- Prices fetched from Birdeye API (more accurate)

### Whale Tracking

**Before:** Solana whales only

**After:** Whales tracked across all connected chains

**Action Required:** None - automatic

**What Changes:**
- Whale detection works on all blockchains
- Whale movements tracked per chain
- AI analysis considers cross-chain activity

### AI Recommendations

**Before:** Claude API only

**After:** Claude API or local models (Llama 2, Mistral)

**Action Required (Optional):**
1. Download local model (e.g., Mistral-7B-Instruct)
2. Set environment variables:
```env
USE_LOCAL_MODEL=true
LOCAL_MODEL_PATH=/path/to/mistral-7b-instruct.gguf
LOCAL_MODEL_TYPE=mistral-7b
```

**Benefits:**
- Lower API costs
- Better privacy (no data sent to Claude)
- Faster inference (with GPU)

### Notifications

**Before:** In-app and email

**After:** Same, plus voice responses (if enabled)

**Action Required:** None - existing notifications continue working

## New Features Setup

### 1. Price Benchmarks

Set automated buy/sell triggers:

```bash
POST /api/benchmarks
{
  "asset": "SOL",
  "target_price": 150.00,
  "trigger_type": "ABOVE",
  "action_type": "ALERT"
}
```

### 2. In-App Conversions

Swap between any assets:

```bash
POST /api/conversions/quote
{
  "from_asset": "SOL",
  "to_asset": "USDC",
  "from_amount": 10.0
}
```

### 3. Auto-Staking

Enable automatic staking of idle balances:

```bash
POST /api/staking/enable
{
  "asset": "SOL",
  "minimum_idle_amount": 10.0,
  "idle_duration_hours": 24
}
```

### 4. Agentic Trimming

Enable AI-driven profit-taking:

```bash
POST /api/trim/config
{
  "enabled": true,
  "minimum_profit_percent": 20.0,
  "trim_percent": 25.0
}
```

### 5. P2P Exchange

Create offers to trade with other users:

```bash
POST /api/p2p/offers
{
  "offer_type": "SELL",
  "from_asset": "SOL",
  "to_asset": "USDC",
  "from_amount": 10.0,
  "price": 100.00
}
```

### 6. Voice Trading (Optional)

Requires `INTERCOM_API_KEY`:

1. Enable in settings
2. Grant microphone permissions
3. Click voice button
4. Say command: "Buy 100 dollars of Solana"

### 7. Privacy Features

**Anonymous User Tags:**
- Automatically generated on first login
- Format: `Trader_A7X9K2`
- Customize in settings

**Temporary Wallets:**
```bash
POST /api/wallets/temporary
{
  "blockchain": "solana",
  "tag": "trading-bot",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Wallet Freezing:**
```bash
POST /api/wallets/:address/freeze
```

## Data Migration

### User Data

**Automatically Migrated:**
- User accounts
- Email and password hashes
- Wallet addresses (converted to multi-chain format)
- Subscription status
- Notification preferences

**New Fields Added:**
- `user_tag` (auto-generated)
- `show_email_publicly` (default: false)

### Wallet Data

**Automatically Migrated:**
- Existing Solana wallets moved to `multi_chain_wallets` table
- `blockchain` field set to "solana"
- All wallet balances preserved

### Portfolio Data

**Automatically Migrated:**
- Existing positions preserved
- Historical data maintained
- Performance metrics recalculated with multi-chain support

### Whale Tracking Data

**Automatically Migrated:**
- Tracked whales preserved
- Movement history maintained
- Blockchain field added (set to "solana")

## API Changes

### Deprecated Endpoints

None - all existing endpoints remain functional

### New Endpoints

See [API_REFERENCE.md](API_REFERENCE.md) for complete list:
- `/api/benchmarks/*` - Price benchmarks
- `/api/conversions/*` - Asset swaps
- `/api/staking/*` - Staking management
- `/api/trim/*` - Agentic trimming
- `/api/voice/*` - Voice commands
- `/api/p2p/*` - P2P exchange
- `/api/chat/*` - On-chain chat
- `/api/receipts/*` - Blockchain receipts
- `/api/verification/*` - Identity verification

### Modified Endpoints

**Portfolio Endpoint:**
```bash
# Old response (Solana only)
GET /api/wallets/portfolio
{
  "total_value_usd": 10000,
  "positions": [...]
}

# New response (multi-chain)
GET /api/wallets/portfolio
{
  "total_value_usd": 10000,
  "positions_by_chain": {
    "solana": [...],
    "ethereum": [...]
  }
}
```

## Performance Considerations

### Redis Cache

**Before:** Cached Solana RPC responses

**After:** Caches Birdeye API responses (60s TTL)

**Impact:** Faster portfolio loads, reduced API costs

**Action:** Increase Redis memory if needed:
```bash
# In redis.conf
maxmemory 512mb
maxmemory-policy allkeys-lru
```

### Database Size

**Before:** ~100MB for typical user

**After:** ~150MB (additional tables for new features)

**Action:** Monitor disk space, consider increasing storage

### API Rate Limits

**New External APIs:**
- Birdeye: 100 requests/minute (free tier)
- SideShift: 60 requests/minute
- Intercom: 10 requests/minute

**Action:** Monitor usage, upgrade plans if needed

## Rollback Procedure

If you need to rollback:

### Step 1: Stop Application

```bash
docker-compose down
# or
systemctl stop whale-tracker
```

### Step 2: Restore Database

```bash
# Drop current database
dropdb whale_tracker

# Restore from backup
createdb whale_tracker
psql whale_tracker < backup_20240101.sql
```

### Step 3: Restore Code

```bash
git checkout <previous_version_tag>
cargo build --release
```

### Step 4: Restore Environment

```bash
# Use old .env file
cp .env.backup .env
```

### Step 5: Restart

```bash
docker-compose up -d
# or
./target/release/api
```

## Troubleshooting

### Migration Fails

**Error:** `relation "multi_chain_wallets" already exists`

**Solution:** Migrations already ran, safe to ignore

---

**Error:** `BIRDEYE_API_KEY not set`

**Solution:** Add required API key to `.env`

---

**Error:** `Failed to connect to Ethereum RPC`

**Solution:** Check `ETHEREUM_RPC_URL` is valid and accessible

### Features Not Working

**Voice commands not responding:**
- Check `INTERCOM_API_KEY` is set
- Verify `ENABLE_VOICE_TRADING=true`
- Check microphone permissions

**P2P offers not matching:**
- Verify both users have compatible offers
- Check price tolerance settings
- Ensure sufficient escrow balance

**Conversions failing:**
- Check `SIDESHIFT_API_KEY` is valid
- Verify sufficient balance
- Check quote hasn't expired (60s TTL)

### Performance Issues

**Slow portfolio loading:**
- Check Redis is running
- Verify Birdeye API is responding
- Increase `BIRDEYE_CACHE_TTL_SECS`

**High memory usage:**
- Reduce `DATABASE_MAX_CONNECTIONS`
- Disable unused features via feature flags
- Consider not using local AI model (uses 4-8GB RAM)

## Getting Help

- **Documentation:** [README.md](README.md), [DEPLOYMENT.md](DEPLOYMENT.md)
- **API Reference:** [API_REFERENCE.md](API_REFERENCE.md)
- **GitHub Issues:** Report bugs and request features
- **Community:** Join our Discord/Telegram

## Post-Migration Checklist

- [ ] Database backup created
- [ ] New environment variables added
- [ ] Application updated and rebuilt
- [ ] Database migrations completed
- [ ] Application restarted successfully
- [ ] Health check passes
- [ ] Existing Solana wallets visible
- [ ] Whale tracking still active
- [ ] Notifications working
- [ ] New multi-chain wallets can be added
- [ ] Portfolio shows aggregated view
- [ ] User tag generated and visible
- [ ] New features accessible (benchmarks, conversions, etc.)

## Next Steps

1. **Connect additional wallets** from Ethereum, BSC, Polygon
2. **Set up price benchmarks** for automated alerts
3. **Enable agentic trimming** for automated profit-taking
4. **Explore P2P exchange** for direct trading
5. **Try voice commands** (if enabled)
6. **Configure privacy settings** (user tag, temporary wallets)
7. **Review blockchain receipts** for tax compliance

Welcome to the enhanced Crypto Trading Platform! 🚀
