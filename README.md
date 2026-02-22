# 🐋 Crypto Trading Platform

> Comprehensive multi-chain cryptocurrency trading platform with AI-powered portfolio management, whale tracking, P2P exchange, proximity transfers, stealth payments, and BLE mesh networking.

A next-generation crypto trading platform that combines real-time account tracking off network, AI-driven position management, multi-chain support, peer-to-peer exchange, privacy-preserving stealth addresses, offline BLE mesh communication, and proximity-based transfers in one unified interface.

## Features

### Core Trading & Portfolio Management
- **Multi-Chain Support**: Track and trade across Solana, Ethereum, BSC, and Polygon
- **Wallet Connection**: Connect multiple wallets across different blockchains
- **Real-time Portfolio**: Aggregated portfolio view with live prices
- **Advanced Analytics**: Metrics, profit/loss tracking, and extensive position information
- **In-App Conversions**: Swap between any supported assets via SideShift and Jupiter
- **Price Benchmarks**: Set automated buy/sell triggers at target prices

### AI-Powered Features
- **Agentic Trimming**: Automated profit-taking based on AI recommendations
- **Local AI Models**: Privacy-focused on-device AI with Llama 2 or Mistral
- **Voice Trading**: Execute trades and queries via Intercom voice commands

### Movement Tracking
- **Whale Detection**: Automatically identify and track large holders in the closed network
- **Movement Monitoring**: Real-time tracking of whale transactions
- **Smart Notifications**: Alerts for significant whale movements

### P2P Exchange & Privacy
- **P2P Trading**: Direct user-to-user trading with escrow protection
- **Wallet Freezing**: Emergency freeze capability for security
- **Anonymous Tags**: Privacy-preserving user identifiers
- **Temporary Wallets**: Create short-lived wallets for specific activities
- **Verification System**: KYC and wallet ownership verification

### Blockchain Features
- **Blockchain Receipts**: Immutable proof of all transactions
- **On-Chain Chat**: Verified peer-to-peer messaging for during transactions and also to all users in range
- **End-to-End Encryption**: Secure message encryption
- **Payment Receipts**: Detailed receipts for tax compliance, fixed retention

### Staking & Yield
- **Auto-Staking**: Automated staking of idle balances via SideShift
- **Reward Tracking**: Monitor staking positions and earned rewards

### Security & Compliance
- **Discrete Login**: Email and password only, no personal data collection
- **2FA Support**: TOTP-based two-factor authentication
- **JWT Authentication**: Secure token-based authentication
- **Tax Compliance**: Comprehensive receipt system for reporting



```bash
cargo test --workspace

cargo test --package api
cargo test --package blockchain

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

## License

MIT License - see [LICENSE](LICENSE) file for details


## Acknowledgments

- Built with Rust and Axum
- Powered by Solana blockchain


## Architecture

The project uses a modular Rust workspace architecture:

- `database`: PostgreSQL and Redis client setup with migrations
- `blockchain`: Solana blockchain integration
- `notification`: Multi-channel notification delivery
- `trading`: Automated trading service
- `api`: REST API server with Axum


## 📋 Technical Architecture Summary

### Workspace Structure

This is a Rust workspace project with 12 specialized crates:

#### Core Infrastructure Crates

**`crates/api`** - Main REST API server and HTTP handlers
- Axum-based web server with JWT authentication
- 40+ service modules for all platform features
- WebSocket support for real-time updates
- Rate limiting, CORS, and security middleware
- Comprehensive error handling and monitoring

**`crates/database`** - PostgreSQL database layer
- Connection pooling with deadpool-postgres
- Database migrations and schema management
- Models for users, wallets, portfolios, transactions, receipts
- Proximity session and transfer persistence
- P2P exchange and chat message storage

**`crates/blockchain`** - Multi-chain blockchain integration
- Solana client with RPC interaction
- EVM client for Ethereum, BSC, Polygon
- Circuit breaker pattern for RPC resilience
- Retry logic with exponential backoff
- Multi-chain abstraction layer

**`crates/shared`** - Common types and utilities
- Shared data models and types
- Configuration management
- Utility functions used across crates

#### Feature-Specific Crates

**`crates/monitoring`** - Whale activity monitoring engine
- Worker pool for parallel whale tracking
- Redis-based state management
- Continuous transaction monitoring 
- Whale movement event publishing

**`crates/ai-service`** - AI analysis integration
- Claude API consumer for whale movement analysis
- AWS SQS message queue consumer
- AI-powered trading recommendations
- Support for local models (Llama 2, Mistral)

**`crates/notification`** - Multi-channel notifications
- In-app notifications
- Email notifications
- Push notifications
- Webhook integrations

**`crates/trading`** - Automated trading service
- Auto-trading based on AI recommendations
- Order execution via DEX aggregators
- Position management and tracking

**`crates/payment`** - Payment processing
- Stripe integration for subscriptions
- Payment method management
- Billing and invoice generation

#### Privacy & P2P Crates

**`crates/proximity`** - Proximity-based P2P transfers
- BLE (Bluetooth Low Energy) discovery and connection
- mDNS (Multicast DNS) for local network discovery
- Peer authentication with challenge-response
- Secure transfer protocol with encryption
- Session management and lifecycle
- QR code generation for connection sharing
- Platform-specific adaptations (iOS, Android)
- Permission management for BLE/location access
- Receipt generation for proximity transfers

**`crates/stealth`** - Stealth address implementation
- EIP-5564 adapted for Solana blockchain
- Privacy-preserving payment addresses
- Stealth address generation and scanning
- Hybrid post-quantum cryptography support
- Payment queue for offline transactions
- Network monitoring for incoming payments
- Wallet manager for stealth operations
- QR code encoding/decoding for addresses
- Platform-specific secure storage (iOS Keychain, Android Keystore)

**`crates/ble-mesh`** - BLE mesh networking
- Offline P2P communication via Bluetooth mesh
- Packet routing with multi-hop support
- Store-and-forward for offline message delivery
- Integration with stealth payment requests
- Mesh network topology management
- Adaptive routing based on signal strength

### Key Services & Components

#### API Services (crates/api/src/)

**Core Trading Services:**
- `wallet_service.rs` - Multi-chain wallet management
- `portfolio_cache.rs` - Portfolio data caching with Redis
- `whale_detection.rs` - Whale account identification
- `portfolio_monitor.rs` - Real-time portfolio tracking
- `analytics.rs` - Performance metrics and analytics
- `birdeye_service.rs` - Multi-chain price data integration
- `benchmark_service.rs` - Price alert and trigger management
- `price_monitor.rs` - Continuous price monitoring
- `position_management_service.rs` - Manual/automatic position modes
- `position_evaluator.rs` - AI-powered position analysis
- `token_metadata_service.rs` - Token information and metadata

**Conversion & Staking:**
- `sideshift_client.rs` - SideShift API integration
- `conversion_service.rs` - Cross-chain asset swaps
- `staking_service.rs` - Auto-staking and reward tracking
- `trim_config_service.rs` - Agentic trimming configuration
- `trim_executor.rs` - Automated profit-taking execution

**P2P & Social:**
- `p2p_service.rs` - Peer-to-peer exchange offers and matching
- `chat_service.rs` - On-chain verified messaging
- `verification_service.rs` - KYC and wallet verification
- `privacy_service.rs` - Temporary wallets and privacy features

**Receipts & Compliance:**
- `receipt_service.rs` - Blockchain receipt generation
- `payment_receipt_service.rs` - Detailed payment receipts (7-year retention)
- `cross_chain_transaction_service.rs` - Normalized multi-chain transactions

**Proximity Integration:**
- `proximity_service.rs` - Proximity transfer orchestration
- `proximity_handlers.rs` - HTTP endpoints for proximity features
- `proximity_websocket.rs` - Real-time proximity events
- `proximity_receipt_integration.rs` - Receipt generation for transfers

**Mesh Network Services:**
- `mesh_price_service.rs` - Decentralized price distribution
- `mesh_metrics.rs` - Mesh network health monitoring
- `coordination_service.rs` - Node coordination
- `gossip_protocol.rs` - Price gossip propagation
- `provider_node.rs` - Price provider node implementation
- `network_status_tracker.rs` - Network health tracking
- `price_update_validator.rs` - Price data validation
- `message_tracker.rs` - Message deduplication
- `price_cache.rs` - Distributed price caching

**Infrastructure:**
- `websocket_service.rs` - Real-time dashboard updates
- `monitoring.rs` - Metrics collection and health checks
- `auth.rs` - JWT authentication and 2FA
- `security.rs` - Security utilities and encryption
- `rate_limit.rs` - API rate limiting
- `error.rs` - Centralized error handling
- `logging.rs` - Structured logging with tracing

### Frontend Architecture

**Single-Page Application (SPA):**
- `frontend/index.html` - Main dashboard with multi-view navigation
- `frontend/app.js` - Core application logic and API integration
- `frontend/styles.css` - Responsive UI styling
- `frontend/proximity.js` - Proximity transfer UI
- `frontend/stealth.js` - Stealth payment UI
- Real-time WebSocket updates
- Multi-chain wallet connection
- Interactive charts and analytics

### Database Schema

**Core Tables:**
- `users` - User accounts with email/password authentication
- `wallets` - Multi-chain wallet addresses
- `portfolio_assets` - Asset holdings per wallet
- `whale_accounts` - Tracked whale addresses
- `whale_movements` - Historical whale transactions
- `benchmarks` - Price alerts and triggers
- `conversions` - Conversion history
- `staking_positions` - Active staking positions
- `trim_configs` - Agentic trimming settings
- `trim_executions` - Trim execution history

**P2P & Social:**
- `p2p_offers` - Active P2P exchange offers
- `p2p_exchanges` - Completed exchanges
- `chat_messages` - On-chain verified messages
- `verifications` - KYC verification records
- `wallet_verifications` - Wallet ownership proofs
- `temporary_wallets` - Time-limited wallet addresses

**Receipts & Compliance:**
- `receipts` - Blockchain transaction receipts
- `payment_receipts` - Detailed payment records (7-year retention)

**Proximity:**
- `proximity_sessions` - Active proximity sessions
- `proximity_transfers` - Transfer history
- `proximity_peers` - Discovered peers

**Stealth:**
- `stealth_addresses` - Generated stealth addresses
- `stealth_payments` - Stealth payment queue
- `stealth_scans` - Scanning history

### External Integrations

**Blockchain RPCs:**
- Solana RPC (mainnet/devnet)
- Ethereum RPC (Alchemy, Infura)
- BSC RPC (Binance)
- Polygon RPC

**APIs:**
- **Birdeye API** - Multi-chain price data and portfolio tracking
- **SideShift API** - Cryptocurrency conversions and staking
- **Claude API** - AI-powered whale movement analysis
- **Intercom API** - Voice command processing
- **Stripe API** - Payment processing and subscriptions
- **KYC Provider API** - Identity verification

**Infrastructure:**
- **PostgreSQL 14+** - Primary data store
- **Redis 7+** - Caching layer (60s TTL for price data)
- **AWS SQS** - Message queue for whale events and AI processing

### Security Features

**Authentication & Authorization:**
- JWT token-based authentication
- TOTP-based 2FA support
- Argon2 password hashing
- Discrete login (email/password only, no PII)

**Data Protection:**
- AES-256 encryption at rest
- TLS 1.3 for data in transit
- No private keys stored in database
- Read-only wallet access for monitoring
- Zeroize for sensitive data in memory

**API Security:**
- Rate limiting per endpoint
- CORS protection
- Request validation
- Suspicious activity detection
- Circuit breaker for external APIs

**Privacy Features:**
- Stealth addresses for anonymous payments
- Temporary wallets for one-time use
- Anonymous user tags
- End-to-end encrypted messaging
- Wallet freeze capability

### Performance Characteristics

**Scalability:**
- Supports 1,000+ concurrent users
- Tracks 10,000+ whale accounts simultaneously
- Horizontal scaling support
- Connection pooling for database and Redis
- Async/await throughout (Tokio runtime)

**Monitoring:**
- Sub-60-second whale movement detection
- Real-time WebSocket updates
- Redis caching for optimal performance
- Metrics collection and health checks
- Alert management for critical events

**Reliability:**
- Circuit breaker pattern for RPC calls
- Retry logic with exponential backoff
- Graceful degradation on service failures
- Database connection pooling
- Worker pool for parallel processing

### Testing Infrastructure

**Test Coverage:**
- Unit tests for all core functionality
- Integration tests for API endpoints
- Property-based tests for critical algorithms
- Mock data for frontend development
- Platform-specific tests (iOS, Android)

**Test Files:**
- `crates/api/tests/` - 30+ integration test files
- `crates/blockchain/tests/` - Blockchain client tests
- `crates/proximity/tests/` - Proximity feature tests
- `crates/stealth/tests/` - Stealth address tests
- `crates/database/tests/` - Database migration tests

### Development Tools

**Build System:**
- Cargo workspace with 12 crates
- Shared dependencies via workspace.dependencies
- Docker support with multi-stage builds
- Docker Compose for local development

**Configuration:**
- Environment-based configuration (.env)
- Feature flags for optional features
- Platform-specific compilation (iOS, Android, Web)

**Deployment:**
- Dockerfile for containerized deployment
- Procfile for Heroku deployment
- PID management for process control
- Health check endpoints

### Feature Flags

**Configurable Features:**
- `ENABLE_VOICE_TRADING` - Voice command support
- `ENABLE_P2P_EXCHANGE` - P2P trading features
- `ENABLE_AGENTIC_TRIMMING` - AI-powered profit-taking
- `USE_LOCAL_MODEL` - Local AI models vs. Claude API

**Platform Features:**
- iOS Keychain integration for secure storage
- Android Keystore integration
- BLE support for proximity transfers
- mDNS for local network discovery

### API Endpoint Categories

**Authentication:** `/api/auth/*`
**Wallets:** `/api/wallets/*`
**Portfolio:** `/api/analytics/*`
**Whale Tracking:** `/api/whales/*`
**Benchmarks:** `/api/benchmarks/*`
**Conversions:** `/api/conversions/*`
**Staking:** `/api/staking/*`
**Trimming:** `/api/trim/*`
**P2P Exchange:** `/api/p2p/*`
**Chat:** `/api/chat/*`
**Receipts:** `/api/receipts/*`
**Verification:** `/api/verification/*`
**Proximity:** `/api/proximity/*`
**Stealth:** `/api/stealth/*`
**Mesh Network:** `/api/mesh/*`
**WebSocket:** `/api/ws/*`

### Technology Stack Summary

**Backend:**
- Rust 1.75+ with Tokio async runtime
- Axum web framework
- PostgreSQL with deadpool connection pooling
- Redis for caching and pub/sub
- AWS SQS for message queuing

**Frontend:**
- Vanilla JavaScript (ES6+)
- WebSocket for real-time updates
- Responsive CSS with mobile support
- QR code generation/scanning

**Blockchain:**
- Solana SDK 1.18
- Web3 libraries for EVM chains
- Custom RPC clients with retry logic

**Cryptography:**
- Ed25519 for signatures
- X25519 for key exchange
- AES-256-GCM for encryption
- Argon2 for password hashing
- Post-quantum hybrid mode (Kyber + X25519)

**External Services:**
- Birdeye for price data
- SideShift for conversions
- Claude for AI analysis
- Stripe for payments
- AWS for infrastructure

---

## Project Status

This project is feature-complete with the following major components implemented:

✅ Multi-chain wallet and portfolio management
✅ Whale detection and monitoring
✅ AI-powered position analysis and trimming
✅ P2P exchange with escrow
✅ Proximity-based transfers (BLE + mDNS)
✅ Stealth addresses for privacy
✅ BLE mesh networking for offline communication
✅ Decentralized price distribution mesh
✅ Comprehensive receipt system
✅ WebSocket real-time updates
✅ Full test coverage

See `.kiro/specs/` for detailed feature specifications and implementation tasks.

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! Please ensure all tests pass before submitting PRs:

```bash
cargo test --workspace
cargo clippy --workspace
cargo fmt --check
```
