# Environment Variables Reference

Complete reference for all environment variables used in the Crypto Trading Platform.

## Required Variables

### Database Configuration

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@localhost:5432/whale_tracker` | Yes |
| `DATABASE_MAX_CONNECTIONS` | Maximum database connection pool size | `20` | No (default: 20) |

### Redis Configuration

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` | Yes |
| `REDIS_MAX_CONNECTIONS` | Maximum Redis connection pool size | `10` | No (default: 10) |

### Multi-Chain RPC Endpoints

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `SOLANA_RPC_URL` | Solana RPC endpoint | `https://api.mainnet-beta.solana.com` | Yes |
| `SOLANA_FALLBACK_RPC_URL` | Fallback Solana RPC | `https://api.devnet.solana.com` | No |
| `ETHEREUM_RPC_URL` | Ethereum RPC endpoint | `https://eth-mainnet.g.alchemy.com/v2/your-key` | Yes |
| `BSC_RPC_URL` | Binance Smart Chain RPC | `https://bsc-dataseed.binance.org` | Yes |
| `POLYGON_RPC_URL` | Polygon RPC endpoint | `https://polygon-rpc.com` | Yes |

### External API Keys

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `BIRDEYE_API_KEY` | Birdeye API key for multi-chain data | `your_birdeye_api_key` | Yes |
| `SIDESHIFT_API_KEY` | SideShift API key for conversions | `your_sideshift_api_key` | Yes |
| `INTERCOM_API_KEY` | Intercom API key for voice commands | `your_intercom_api_key` | No* |
| `CLAUDE_API_KEY` | Anthropic Claude API key | `sk-ant-api03-...` | Yes** |
| `KYC_PROVIDER_API_KEY` | KYC provider API key (e.g., Onfido) | `your_kyc_api_key` | No* |

\* Required only if feature is enabled  
\** Required if not using local AI model

### AI Configuration

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `USE_LOCAL_MODEL` | Use local AI model instead of Claude | `true` or `false` | No (default: false) |
| `LOCAL_MODEL_PATH` | Path to local model file | `/models/mistral-7b.gguf` | No*** |
| `LOCAL_MODEL_TYPE` | Type of local model | `mistral-7b`, `llama-2-7b`, `llama-2-13b` | No*** |
| `LOCAL_MODEL_GPU` | Enable GPU acceleration | `true` or `false` | No (default: false) |
| `LOCAL_MODEL_THREADS` | Number of CPU threads for inference | `4` | No (default: 4) |
| `LOCAL_MODEL_TIMEOUT_SECS` | Inference timeout in seconds | `10` | No (default: 10) |

\*** Required if `USE_LOCAL_MODEL=true`

### Authentication & Security

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `JWT_SECRET` | Secret key for JWT token signing | `your_random_secret_key` | Yes |
| `JWT_EXPIRATION_HOURS` | JWT token expiration time | `24` | No (default: 24) |
| `PASSWORD_MIN_LENGTH` | Minimum password length | `8` | No (default: 8) |
| `REQUIRE_2FA` | Require 2FA for all users | `true` or `false` | No (default: false) |

### Stripe Payment Processing

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `STRIPE_SECRET_KEY` | Stripe secret API key | `sk_test_...` | Yes |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret | `whsec_...` | Yes |
| `STRIPE_PUBLISHABLE_KEY` | Stripe publishable key | `pk_test_...` | No |

### Server Configuration

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `SERVER_HOST` | Server bind address | `0.0.0.0` | No (default: 0.0.0.0) |
| `SERVER_PORT` | Server port | `3000` | No (default: 3000) |
| `CORS_ALLOWED_ORIGINS` | Comma-separated allowed origins | `http://localhost:3000,https://app.example.com` | No (default: *) |
| `MAX_REQUEST_SIZE_MB` | Maximum request body size | `10` | No (default: 10) |

## Feature Flags

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `ENABLE_VOICE_TRADING` | Enable voice command features | `true` or `false` | No (default: true) |
| `ENABLE_P2P_EXCHANGE` | Enable P2P trading | `true` or `false` | No (default: true) |
| `ENABLE_AGENTIC_TRIMMING` | Enable AI-driven position trimming | `true` or `false` | No (default: true) |
| `ENABLE_BLOCKCHAIN_RECEIPTS` | Enable blockchain receipt creation | `true` or `false` | No (default: true) |
| `ENABLE_CHAT_VERIFICATION` | Enable on-chain chat verification | `true` or `false` | No (default: true) |
| `ENABLE_AUTO_STAKING` | Enable automatic staking | `true` or `false` | No (default: true) |

## Service Configuration

### Birdeye Service

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `BIRDEYE_CACHE_TTL_SECS` | Cache TTL for Birdeye responses | `60` | No (default: 60) |
| `BIRDEYE_MAX_RETRIES` | Maximum retry attempts | `3` | No (default: 3) |
| `BIRDEYE_TIMEOUT_SECS` | Request timeout | `10` | No (default: 10) |

### SideShift Service

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `SIDESHIFT_QUOTE_TTL_SECS` | Quote validity duration | `60` | No (default: 60) |
| `SIDESHIFT_MAX_RETRIES` | Maximum retry attempts | `3` | No (default: 3) |
| `SIDESHIFT_TIMEOUT_SECS` | Request timeout | `30` | No (default: 30) |

### Benchmark Service

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `BENCHMARK_CHECK_INTERVAL_SECS` | Price check interval | `10` | No (default: 10) |
| `BENCHMARK_TRIGGER_TIMEOUT_SECS` | Maximum time to trigger action | `60` | No (default: 60) |

### Agentic Trimming

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `TRIM_EVALUATION_INTERVAL_SECS` | Position evaluation interval | `300` | No (default: 300) |
| `TRIM_MIN_CONFIDENCE` | Minimum AI confidence to trim | `85` | No (default: 85) |
| `TRIM_DEFAULT_PERCENT` | Default trim percentage | `25.0` | No (default: 25.0) |
| `TRIM_MAX_PER_DAY` | Maximum trims per day per position | `3` | No (default: 3) |

### P2P Exchange

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `P2P_OFFER_EXPIRATION_HOURS` | Offer expiration time | `24` | No (default: 24) |
| `P2P_PLATFORM_FEE_PERCENT` | Platform fee percentage | `0.5` | No (default: 0.5) |
| `P2P_MATCH_PRICE_TOLERANCE_PERCENT` | Price matching tolerance | `1.0` | No (default: 1.0) |
| `P2P_ESCROW_TIMEOUT_SECS` | Escrow timeout | `3600` | No (default: 3600) |

### Voice Commands

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `VOICE_RATE_LIMIT_PER_MINUTE` | Max commands per minute | `10` | No (default: 10) |
| `VOICE_CONFIRMATION_REQUIRED` | Require confirmation for trades | `true` or `false` | No (default: true) |
| `VOICE_TIMEOUT_SECS` | Voice processing timeout | `30` | No (default: 30) |

### Receipts

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `RECEIPT_RETENTION_YEARS` | Receipt retention period | `7` | No (default: 7) |
| `RECEIPT_BLOCKCHAIN_RETRIES` | Blockchain submission retries | `3` | No (default: 3) |
| `RECEIPT_PREFERRED_BLOCKCHAIN` | Preferred blockchain for receipts | `solana`, `ethereum`, `polygon` | No (default: solana) |

### Wallet Management

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `MAX_TEMPORARY_WALLETS` | Max temporary wallets per user | `10` | No (default: 10) |
| `TEMPORARY_WALLET_DEFAULT_EXPIRATION_DAYS` | Default expiration | `30` | No (default: 30) |
| `WALLET_FREEZE_REQUIRES_2FA` | Require 2FA to unfreeze | `true` or `false` | No (default: true) |

## Logging & Monitoring

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `RUST_LOG` | Logging level | `info`, `debug`, `trace` | No (default: info) |
| `LOG_FORMAT` | Log output format | `json` or `pretty` | No (default: pretty) |
| `SENTRY_DSN` | Sentry error tracking DSN | `https://...@sentry.io/...` | No |
| `DATADOG_API_KEY` | Datadog monitoring API key | `your_datadog_key` | No |
| `METRICS_ENABLED` | Enable Prometheus metrics | `true` or `false` | No (default: false) |
| `METRICS_PORT` | Metrics endpoint port | `9090` | No (default: 9090) |

## Email Configuration (Optional)

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `SMTP_HOST` | SMTP server hostname | `smtp.gmail.com` | No |
| `SMTP_PORT` | SMTP server port | `587` | No |
| `SMTP_USERNAME` | SMTP username | `your_email@gmail.com` | No |
| `SMTP_PASSWORD` | SMTP password | `your_app_password` | No |
| `SMTP_FROM_EMAIL` | From email address | `noreply@example.com` | No |

## Development & Testing

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `ENVIRONMENT` | Environment name | `development`, `staging`, `production` | No (default: development) |
| `MOCK_EXTERNAL_APIS` | Use mock APIs for testing | `true` or `false` | No (default: false) |
| `SKIP_MIGRATIONS` | Skip database migrations on startup | `true` or `false` | No (default: false) |
| `SEED_DATABASE` | Seed database with test data | `true` or `false` | No (default: false) |

## Example .env File

```env
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/whale_tracker
DATABASE_MAX_CONNECTIONS=20

# Redis
REDIS_URL=redis://localhost:6379

# Multi-Chain RPC
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
BSC_RPC_URL=https://bsc-dataseed.binance.org
POLYGON_RPC_URL=https://polygon-rpc.com

# External APIs
BIRDEYE_API_KEY=your_birdeye_api_key
SIDESHIFT_API_KEY=your_sideshift_api_key
INTERCOM_API_KEY=your_intercom_api_key
CLAUDE_API_KEY=sk-ant-api03-your_key
KYC_PROVIDER_API_KEY=your_kyc_key

# AI Configuration
USE_LOCAL_MODEL=false
# LOCAL_MODEL_PATH=/models/mistral-7b-instruct.gguf
# LOCAL_MODEL_TYPE=mistral-7b

# Authentication
JWT_SECRET=your_random_secret_key_change_in_production
JWT_EXPIRATION_HOURS=24

# Stripe
STRIPE_SECRET_KEY=sk_test_your_key
STRIPE_WEBHOOK_SECRET=whsec_your_secret

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Feature Flags
ENABLE_VOICE_TRADING=true
ENABLE_P2P_EXCHANGE=true
ENABLE_AGENTIC_TRIMMING=true

# Logging
RUST_LOG=info,whale_tracker=debug
LOG_FORMAT=pretty

# Environment
ENVIRONMENT=development
```

## Security Best Practices

1. **Never commit `.env` files to version control**
2. **Use strong, randomly generated secrets** for `JWT_SECRET`
3. **Rotate API keys regularly**
4. **Use environment-specific keys** (test keys for development, production keys for production)
5. **Restrict database user permissions** to minimum required
6. **Use read-only RPC endpoints** where possible
7. **Enable 2FA** for production deployments
8. **Monitor API key usage** for unusual activity

## Generating Secrets

```bash
# Generate JWT secret
openssl rand -base64 32

# Generate random password
openssl rand -base64 24
```

## Environment-Specific Configuration

### Development
- Use test API keys
- Enable `MOCK_EXTERNAL_APIS=true` for faster testing
- Set `RUST_LOG=debug` for detailed logs
- Use `SEED_DATABASE=true` for test data

### Staging
- Use production-like configuration
- Enable all features
- Use separate database and Redis instances
- Monitor performance metrics

### Production
- Use production API keys
- Set `ENVIRONMENT=production`
- Enable `REQUIRE_2FA=true`
- Configure monitoring (Sentry, Datadog)
- Use managed database services
- Enable SSL/TLS
- Set appropriate rate limits
- Configure backup strategies
