# 🐋 Crypto Trading Platform

> Comprehensive multi-chain cryptocurrency trading platform with AI-powered portfolio management, whale tracking, P2P exchange, and advanced privacy features.

A next-generation crypto trading platform that combines real-time whale tracking, AI-driven position management, multi-chain support, and peer-to-peer exchange capabilities in one unified interface.

## Features

### Core Trading & Portfolio Management
- 🌐 **Multi-Chain Support**: Track and trade across Solana, Ethereum, BSC, and Polygon
- 🔗 **Wallet Connection**: Connect multiple wallets across different blockchains
- 📊 **Real-time Portfolio**: Aggregated portfolio view with live prices via Birdeye API
- 📈 **Advanced Analytics**: Performance metrics, profit/loss tracking, and position distribution
- 💱 **In-App Conversions**: Swap between any supported assets via SideShift and Jupiter
- 🎯 **Price Benchmarks**: Set automated buy/sell triggers at target prices

### AI-Powered Features
- 🤖 **AI Analysis**: Claude or local model analysis of whale movements
- ✂️ **Agentic Trimming**: Automated profit-taking based on AI recommendations
- 🧠 **Local AI Models**: Privacy-focused on-device AI with Llama 2 or Mistral
- 🎤 **Voice Trading**: Execute trades and queries via Intercom voice commands

### Whale Tracking
- 🐋 **Whale Detection**: Automatically identify and track large holders
- 👀 **Movement Monitoring**: Real-time tracking of whale transactions (every 30 seconds)
- 🔔 **Smart Notifications**: Alerts for significant whale movements
- 📉 **Impact Analysis**: Understand how whale activity affects your positions

### P2P Exchange & Privacy
- 🤝 **P2P Trading**: Direct user-to-user trading with escrow protection
- 🔒 **Wallet Freezing**: Emergency freeze capability for security
- 🎭 **Anonymous Tags**: Privacy-preserving user identifiers
- ⏱️ **Temporary Wallets**: Create short-lived wallets for specific activities
- ✅ **Verification System**: KYC and wallet ownership verification

### Blockchain Features
- 📜 **Blockchain Receipts**: Immutable proof of all transactions
- 💬 **On-Chain Chat**: Verified peer-to-peer messaging
- 🔐 **End-to-End Encryption**: Secure message encryption
- 🎫 **Payment Receipts**: Detailed receipts for tax compliance (7-year retention)

### Staking & Yield
- 💰 **Auto-Staking**: Automated staking of idle balances via SideShift
- 📊 **Reward Tracking**: Monitor staking positions and earned rewards
- 🔄 **Auto-Compound**: Optional automatic reward reinvestment

### Security & Compliance
- 🔐 **Discrete Login**: Email and password only, no personal data collection
- 🔑 **2FA Support**: TOTP-based two-factor authentication
- 🛡️ **JWT Authentication**: Secure token-based authentication
- 📋 **Tax Compliance**: Comprehensive receipt system for reporting

## Quick Start

### Using Docker (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd solana-whale-tracker

# Create environment file
cp .env.example .env
# Edit .env with your configuration

# Start all services
docker-compose up -d

# Access the application
open http://localhost:3000
```

### Manual Setup

```bash
# Install dependencies
cargo build --release

# Setup database
createdb whale_tracker
DATABASE_URL=postgresql://localhost/whale_tracker cargo run --bin migrate

# Start Redis
redis-server

# Run the application
cargo run --bin api

# Access the application
open http://localhost:3000
```

## Architecture

The platform consists of:

- **Rust Backend**: High-performance API server with Axum
- **PostgreSQL**: Primary data store for users, portfolios, and whale data
- **Redis**: Caching layer for performance optimization (60s TTL for Birdeye data)
- **Multi-Chain Support**: Solana, Ethereum, BSC, and Polygon integration
- **Birdeye API**: Multi-chain price data and portfolio tracking
- **SideShift API**: Cryptocurrency conversions and staking
- **Intercom API**: Voice command processing
- **Claude API / Local Models**: AI-powered analysis (Llama 2, Mistral support)
- **Stripe**: Payment processing
- **Web UI**: Modern responsive interface with WebSocket real-time updates

## Documentation

- [Deployment Guide](DEPLOYMENT.md) - Complete deployment instructions
- [Migration Guide](MIGRATION.md) - Upgrading from Solana Whale Tracker
- [API Documentation](API_REFERENCE.md) - Complete API endpoint reference
- [Environment Variables](ENV_VARIABLES.md) - Configuration reference
- [Design Document](.kiro/specs/crypto-trading-platform-enhancements/design.md) - System design and architecture
- [Requirements](.kiro/specs/crypto-trading-platform-enhancements/requirements.md) - Functional requirements

## Development

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Redis 7+
- Node.js 18+ (optional, for frontend development)

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test --package api
cargo test --package blockchain

# Run with logging
RUST_LOG=debug cargo test
```

### Project Structure

```
solana-whale-tracker/
├── crates/
│   ├── api/              # REST API and handlers
│   ├── blockchain/       # Solana integration
│   ├── database/         # Database layer
│   ├── monitoring/       # Whale monitoring engine
│   ├── ai-service/       # Claude API integration
│   ├── notification/     # Notification service
│   ├── trading/          # Trading and auto-trader
│   ├── payment/          # Stripe integration
│   └── shared/           # Shared types and utilities
├── frontend/             # Web UI
│   ├── index.html
│   ├── styles.css
│   └── app.js
├── Dockerfile
├── docker-compose.yml
└── DEPLOYMENT.md
```

## Configuration

### Environment Variables

Create a `.env` file with the following variables:

```env
# Database
DATABASE_URL=postgresql://user:password@localhost/whale_tracker

# Redis
REDIS_URL=redis://localhost:6379

# Multi-Chain RPC Endpoints
SOLANA_RPC_URL=https://api.devnet.solana.com
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-key
BSC_RPC_URL=https://bsc-dataseed.binance.org
POLYGON_RPC_URL=https://polygon-rpc.com

# External APIs
BIRDEYE_API_KEY=your_birdeye_api_key
SIDESHIFT_API_KEY=your_sideshift_api_key
INTERCOM_API_KEY=your_intercom_api_key
CLAUDE_API_KEY=your_claude_api_key

# KYC Provider
KYC_PROVIDER_API_KEY=your_kyc_provider_key

# AI Configuration
USE_LOCAL_MODEL=false
LOCAL_MODEL_PATH=/path/to/model.gguf
LOCAL_MODEL_TYPE=mistral-7b

# Feature Flags
ENABLE_VOICE_TRADING=true
ENABLE_P2P_EXCHANGE=true
ENABLE_AGENTIC_TRIMMING=true

# Stripe
STRIPE_SECRET_KEY=your_secret_key

# JWT
JWT_SECRET=your_random_secret
```

See [ENV_VARIABLES.md](ENV_VARIABLES.md) for complete configuration options.

## API Endpoints

### Health Check
```bash
GET /health
```

### Multi-Chain Wallet Management
```bash
POST /api/wallets/connect              # Connect wallet (any blockchain)
GET  /api/wallets/portfolio            # Multi-chain aggregated portfolio
GET  /api/wallets/:address/portfolio   # Specific wallet portfolio
POST /api/wallets/temporary            # Create temporary wallet
POST /api/wallets/:address/freeze      # Freeze wallet
POST /api/wallets/:address/unfreeze    # Unfreeze wallet (requires 2FA)
```

### Benchmarks
```bash
POST /api/benchmarks                   # Create price benchmark
GET  /api/benchmarks                   # List user benchmarks
PUT  /api/benchmarks/:id               # Update benchmark
DELETE /api/benchmarks/:id             # Delete benchmark
GET  /api/benchmarks/active            # Get active benchmarks
```

### Conversions & Swaps
```bash
POST /api/conversions/quote            # Get conversion quote
POST /api/conversions/execute          # Execute conversion
GET  /api/conversions/history          # Conversion history
GET  /api/conversions/:id              # Get conversion details
```

### Staking
```bash
POST /api/staking/enable               # Enable auto-staking
POST /api/staking/disable              # Disable auto-staking
GET  /api/staking/positions            # List staking positions
GET  /api/staking/rewards              # Get staking rewards
```

### Agentic Trimming
```bash
POST /api/trim/config                  # Configure trim settings
GET  /api/trim/config                  # Get trim configuration
GET  /api/trim/history                 # Trim execution history
POST /api/trim/enable                  # Enable agentic trimming
POST /api/trim/disable                 # Disable agentic trimming
```

### Voice Commands
```bash
POST /api/voice/command                # Process voice command
GET  /api/voice/history                # Voice command history
```

### P2P Exchange
```bash
POST /api/p2p/offers                   # Create P2P offer
GET  /api/p2p/offers                   # List active offers
GET  /api/p2p/offers/:id               # Get offer details
DELETE /api/p2p/offers/:id             # Cancel offer
GET  /api/p2p/matches                  # Get matched offers
GET  /api/p2p/exchanges                # Exchange history
```

### Chat
```bash
POST /api/chat/messages                # Send message
GET  /api/chat/messages                # Get chat history
POST /api/chat/messages/:id/verify     # Verify message on-chain
POST /api/chat/messages/:id/report     # Report message
```

### Receipts
```bash
GET  /api/receipts                     # List receipts (with filters)
GET  /api/receipts/:id                 # Get receipt details
GET  /api/receipts/:id/verify          # Verify receipt on blockchain
GET  /api/receipts/:id/download        # Download receipt (PDF)
POST /api/receipts/export              # Export receipts (CSV)
```

### Verification
```bash
POST /api/verification/identity        # Submit identity verification
GET  /api/verification/status          # Check verification status
POST /api/verification/wallet          # Verify wallet ownership
GET  /api/verification/wallets         # List verified wallets
```

### User & Privacy
```bash
POST /api/auth/register                # Register (email, password, wallet only)
POST /api/auth/login                   # Login (returns JWT)
POST /api/auth/2fa/enable              # Enable 2FA
POST /api/auth/2fa/verify              # Verify 2FA code
PUT  /api/users/tag                    # Update user tag
GET  /api/users/profile                # Get user profile
```

### Whale Tracking
```bash
GET  /api/whales/tracked               # List tracked whales
GET  /api/whales/:address/details      # Whale details
POST /api/whales/refresh               # Refresh whale data
```

### Analytics & Dashboard
```bash
GET  /api/analytics/portfolio          # Portfolio performance metrics
GET  /api/analytics/whale-impact       # Whale impact analysis
GET  /api/analytics/dashboard          # Enhanced dashboard data
WS   /api/ws/dashboard                 # WebSocket real-time updates
```

See full API documentation in [API_REFERENCE.md](API_REFERENCE.md).

## Deployment

### Production Deployment

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed deployment instructions including:

- Docker deployment
- AWS deployment (ECS, RDS, ElastiCache)
- DigitalOcean deployment
- Heroku deployment
- Frontend-only deployment (Netlify, Vercel, S3)

### Quick Deploy to Heroku

```bash
heroku create whale-tracker
heroku addons:create heroku-postgresql:hobby-dev
heroku addons:create heroku-redis:hobby-dev
git push heroku main
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Testing

The project includes comprehensive test coverage:

- Unit tests for all core functionality
- Integration tests for API endpoints
- Property-based tests for critical algorithms
- Mock data for frontend development

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --workspace --out Html
```

## Security

- No private keys stored in database
- Read-only wallet access for monitoring
- Separate transaction signing permissions
- AES-256 encryption at rest
- TLS 1.3 for data in transit
- Rate limiting and CORS protection
- Suspicious activity detection

## Performance

- Sub-60-second whale movement detection
- Supports 1,000+ concurrent users
- Tracks 10,000+ whale accounts simultaneously
- Horizontal scaling support
- Redis caching for optimal performance

## License

MIT License - see [LICENSE](LICENSE) file for details

## Support

- Documentation: [DEPLOYMENT.md](DEPLOYMENT.md)
- Issues: GitHub Issues
- Email: support@example.com

## Acknowledgments

- Built with Rust and Axum
- Powered by Solana blockchain
- AI analysis by Anthropic Claude
- Payment processing by Stripe

---

Made with ❤️ for the Solana community

## Features

- **Wallet Connection**: Connect your Solana wallet to track your portfolio
- **Whale Detection**: Automatically identify whale accounts holding the same assets
- **Real-Time Monitoring**: Continuous monitoring of whale transactions
- **AI Analysis**: Claude-powered analysis of whale movements with actionable recommendations
- **Automated Trading**: Optional auto-trading based on AI recommendations
- **Multi-Channel Notifications**: In-app, email, and push notifications
- **Payment Integration**: Stripe-powered subscription management

## Architecture

The project uses a modular Rust workspace architecture:

- `shared`: Common types, models, and configuration
- `database`: PostgreSQL and Redis client setup with migrations
- `blockchain`: Solana blockchain integration
- `monitoring`: Whale activity monitoring engine
- `ai-service`: Claude API integration for movement analysis
- `notification`: Multi-channel notification delivery
- `trading`: Automated trading service
- `api`: REST API server with Axum

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- PostgreSQL 14+
- Redis 7+
- Solana CLI (optional, for testing)

## Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd solana-whale-tracker
   ```

2. **Install dependencies**
   ```bash
   cargo build
   ```

3. **Set up PostgreSQL**
   ```bash
   # Create database
   createdb solana_whale_tracker
   ```

4. **Set up Redis**
   ```bash
   # Start Redis (if not running)
   redis-server
   ```

5. **Configure environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

6. **Run database migrations**
   ```bash
   cargo run --bin api
   # Migrations run automatically on startup
   ```

## Configuration

Edit `.env` file with your settings:

- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `SOLANA_RPC_URL`: Solana RPC endpoint
- `CLAUDE_API_KEY`: Anthropic Claude API key
- `STRIPE_SECRET_KEY`: Stripe API key
- `JWT_SECRET`: Secret for JWT token generation

See `.env.example` for all available options.

## Running

```bash
# Development mode
cargo run --bin api

# Production build
cargo build --release
./target/release/api
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Project Status

This project is currently under development. See `.kiro/specs/solana-whale-tracker/tasks.md` for implementation progress.

**Completed:**
- ✅ Task 1: Project setup and core infrastructure

**In Progress:**
- Task 2: Solana blockchain integration
- Task 3: Wallet service and portfolio management

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]
