# Crypto Trading Platform - Deployment Guide

Complete guide for deploying the Crypto Trading Platform with multi-chain support, P2P exchange, and advanced features.

## Prerequisites

- Docker and Docker Compose (recommended)
- PostgreSQL 14+
- Redis 7+
- Multi-chain RPC endpoints (Solana, Ethereum, BSC, Polygon)
- Birdeye API key (for multi-chain data)
- SideShift API key (for conversions and staking)
- Claude API key (Anthropic) or local AI model
- Stripe account (for payments)
- Optional: Intercom API key (for voice trading)
- Optional: KYC provider API key (for identity verification)

## Quick Start with Docker

### 1. Clone and Setup

```bash
git clone <repository-url>
cd solana-whale-tracker
```

### 2. Configure Environment

Create a `.env` file in the root directory:

```env
# Database
DATABASE_URL=postgresql://postgres:password@postgres:5432/whale_tracker
DATABASE_MAX_CONNECTIONS=20

# Redis
REDIS_URL=redis://redis:6379

# Multi-Chain RPC Endpoints
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_FALLBACK_RPC_URL=https://api.devnet.solana.com
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
BSC_RPC_URL=https://bsc-dataseed.binance.org
POLYGON_RPC_URL=https://polygon-rpc.com

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# External APIs
BIRDEYE_API_KEY=your_birdeye_api_key
SIDESHIFT_API_KEY=your_sideshift_api_key
INTERCOM_API_KEY=your_intercom_api_key
KYC_PROVIDER_API_KEY=your_kyc_provider_key

# AI Configuration
CLAUDE_API_KEY=your_claude_api_key
USE_LOCAL_MODEL=false
# LOCAL_MODEL_PATH=/models/mistral-7b-instruct.gguf
# LOCAL_MODEL_TYPE=mistral-7b

# Stripe
STRIPE_SECRET_KEY=your_stripe_secret_key
STRIPE_WEBHOOK_SECRET=your_webhook_secret

# JWT
JWT_SECRET=your_random_secret_key_here

# Feature Flags
ENABLE_VOICE_TRADING=true
ENABLE_P2P_EXCHANGE=true
ENABLE_AGENTIC_TRIMMING=true
```

### 3. Build and Run

```bash
docker-compose up -d
```

The application will be available at:
- Frontend: http://localhost:3000
- API: http://localhost:3000/api
- Health Check: http://localhost:3000/health
- WebSocket: ws://localhost:3000/api/ws/dashboard

## Manual Deployment

### 1. Setup Database

```bash
# Install PostgreSQL
sudo apt-get install postgresql-14

# Create database
sudo -u postgres psql
CREATE DATABASE whale_tracker;
CREATE USER whale_user WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE whale_tracker TO whale_user;
\q
```

### 2. Setup Redis

```bash
# Install Redis
sudo apt-get install redis-server

# Start Redis
sudo systemctl start redis-server
sudo systemctl enable redis-server
```

### 3. Build Rust Backend

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build --release

# Run migrations
DATABASE_URL=postgresql://whale_user:your_password@localhost/whale_tracker \
cargo run --bin migrate
```

### 4. Optional: Setup Local AI Model

For privacy and cost savings, you can use a local AI model instead of Claude:

```bash
# Download Mistral-7B-Instruct (recommended)
wget https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf

# Or Llama-2-7B-Chat
wget https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF/resolve/main/llama-2-7b-chat.Q4_K_M.gguf

# Update .env
USE_LOCAL_MODEL=true
LOCAL_MODEL_PATH=/path/to/mistral-7b-instruct-v0.2.Q4_K_M.gguf
LOCAL_MODEL_TYPE=mistral-7b
```

### 5. Run the Application

```bash
# Set environment variables
export DATABASE_URL=postgresql://whale_user:your_password@localhost/whale_tracker
export REDIS_URL=redis://localhost:6379
export SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
export ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
export BSC_RPC_URL=https://bsc-dataseed.binance.org
export POLYGON_RPC_URL=https://polygon-rpc.com
export BIRDEYE_API_KEY=your_key
export SIDESHIFT_API_KEY=your_key
export CLAUDE_API_KEY=your_key
export SERVER_HOST=0.0.0.0
export SERVER_PORT=3000

# Run the API server
./target/release/api
```

### 6. Access the UI

Open your browser and navigate to http://localhost:3000

## Production Deployment

### AWS Deployment

#### 1. Setup RDS (PostgreSQL)

```bash
aws rds create-db-instance \
  --db-instance-identifier whale-tracker-db \
  --db-instance-class db.t3.micro \
  --engine postgres \
  --master-username admin \
  --master-user-password YourPassword123 \
  --allocated-storage 20
```

#### 2. Setup ElastiCache (Redis)

```bash
aws elasticache create-cache-cluster \
  --cache-cluster-id whale-tracker-redis \
  --cache-node-type cache.t3.micro \
  --engine redis \
  --num-cache-nodes 1
```

#### 3. Setup SQS Queue

```bash
aws sqs create-queue --queue-name whale-movements
```

#### 4. Deploy to ECS/Fargate

```bash
# Build and push Docker image
docker build -t whale-tracker:latest .
docker tag whale-tracker:latest your-ecr-repo/whale-tracker:latest
docker push your-ecr-repo/whale-tracker:latest

# Create ECS task definition and service
aws ecs create-service \
  --cluster whale-tracker-cluster \
  --service-name whale-tracker \
  --task-definition whale-tracker:1 \
  --desired-count 2 \
  --launch-type FARGATE
```

### DigitalOcean Deployment

#### 1. Create Droplet

```bash
# Create a droplet with Docker pre-installed
doctl compute droplet create whale-tracker \
  --image docker-20-04 \
  --size s-2vcpu-4gb \
  --region nyc1
```

#### 2. Setup Managed Database

```bash
# Create PostgreSQL database
doctl databases create whale-tracker-db \
  --engine pg \
  --region nyc1 \
  --size db-s-1vcpu-1gb
```

#### 3. Deploy Application

```bash
# SSH into droplet
ssh root@your-droplet-ip

# Clone repository
git clone <repository-url>
cd solana-whale-tracker

# Setup environment
cp .env.example .env
nano .env  # Edit with your values

# Run with Docker Compose
docker-compose up -d
```

### Heroku Deployment

```bash
# Login to Heroku
heroku login

# Create app
heroku create whale-tracker

# Add PostgreSQL
heroku addons:create heroku-postgresql:hobby-dev

# Add Redis
heroku addons:create heroku-redis:hobby-dev

# Set environment variables
heroku config:set SOLANA_RPC_URL=https://api.devnet.solana.com
heroku config:set CLAUDE_API_KEY=your_key
heroku config:set STRIPE_SECRET_KEY=your_key

# Deploy
git push heroku main
```

## Frontend-Only Deployment

If you want to deploy just the frontend (connecting to an existing API):

### Netlify

```bash
cd frontend
netlify deploy --prod
```

### Vercel

```bash
cd frontend
vercel --prod
```

### AWS S3 + CloudFront

```bash
# Sync to S3
aws s3 sync frontend/ s3://your-bucket-name --acl public-read

# Enable website hosting
aws s3 website s3://your-bucket-name \
  --index-document index.html \
  --error-document index.html

# Create CloudFront distribution
aws cloudfront create-distribution \
  --origin-domain-name your-bucket-name.s3.amazonaws.com
```

## Configuration

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Yes | - |
| `REDIS_URL` | Redis connection string | Yes | - |
| `SOLANA_RPC_URL` | Solana RPC endpoint | Yes | - |
| `ETHEREUM_RPC_URL` | Ethereum RPC endpoint | Yes | - |
| `BSC_RPC_URL` | Binance Smart Chain RPC | Yes | - |
| `POLYGON_RPC_URL` | Polygon RPC endpoint | Yes | - |
| `BIRDEYE_API_KEY` | Birdeye API key | Yes | - |
| `SIDESHIFT_API_KEY` | SideShift API key | Yes | - |
| `CLAUDE_API_KEY` | Claude API key | Yes* | - |
| `INTERCOM_API_KEY` | Intercom API key | No | - |
| `KYC_PROVIDER_API_KEY` | KYC provider key | No | - |
| `USE_LOCAL_MODEL` | Use local AI model | No | false |
| `LOCAL_MODEL_PATH` | Path to local model | No | - |
| `STRIPE_SECRET_KEY` | Stripe secret key | Yes | - |
| `JWT_SECRET` | JWT signing secret | Yes | - |
| `SERVER_HOST` | Server bind address | No | `0.0.0.0` |
| `SERVER_PORT` | Server port | No | `3000` |
| `ENABLE_VOICE_TRADING` | Enable voice features | No | true |
| `ENABLE_P2P_EXCHANGE` | Enable P2P trading | No | true |
| `ENABLE_AGENTIC_TRIMMING` | Enable AI trimming | No | true |

\* Required if not using local model

See [ENV_VARIABLES.md](ENV_VARIABLES.md) for complete configuration options.

### Frontend Configuration

The frontend can be configured through the Settings page in the UI:

1. Navigate to Settings
2. Update API Base URL
3. Click Save Settings

Configuration is stored in browser localStorage.

## Monitoring

### Health Checks

The application provides a health check endpoint:

```bash
curl http://localhost:3000/health
```

Response:
```json
{
  "status": "healthy",
  "services": {
    "database": "healthy",
    "redis": "healthy",
    "solana_rpc": "healthy"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Logs

```bash
# View Docker logs
docker-compose logs -f api

# View specific service logs
docker-compose logs -f postgres
docker-compose logs -f redis
```

### Metrics

Consider adding:
- Prometheus for metrics collection
- Grafana for visualization
- Datadog or New Relic for APM

## Scaling

### Horizontal Scaling

The application is stateless and can be scaled horizontally:

```bash
# Scale with Docker Compose
docker-compose up -d --scale api=3

# Scale with Kubernetes
kubectl scale deployment whale-tracker --replicas=5
```

### Database Scaling

- Use read replicas for read-heavy workloads
- Enable connection pooling (already configured)
- Consider partitioning large tables

### Redis Scaling

- Use Redis Cluster for high availability
- Enable persistence for critical data
- Consider Redis Sentinel for automatic failover

## Security

### SSL/TLS

Use a reverse proxy like Nginx or Caddy:

```nginx
server {
    listen 443 ssl http2;
    server_name whale-tracker.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Firewall

```bash
# Allow only necessary ports
sudo ufw allow 22/tcp   # SSH
sudo ufw allow 80/tcp   # HTTP
sudo ufw allow 443/tcp  # HTTPS
sudo ufw enable
```

### Secrets Management

Use AWS Secrets Manager, HashiCorp Vault, or similar:

```bash
# Store secret in AWS Secrets Manager
aws secretsmanager create-secret \
  --name whale-tracker/claude-api-key \
  --secret-string "your-api-key"
```

## Backup

### Database Backup

```bash
# Automated daily backup
pg_dump -h localhost -U whale_user whale_tracker > backup_$(date +%Y%m%d).sql

# Restore from backup
psql -h localhost -U whale_user whale_tracker < backup_20240101.sql
```

### Redis Backup

```bash
# Enable RDB snapshots in redis.conf
save 900 1
save 300 10
save 60 10000

# Manual backup
redis-cli BGSAVE
```

## Troubleshooting

### API Not Starting

1. Check logs: `docker-compose logs api`
2. Verify database connection: `psql $DATABASE_URL`
3. Check Redis: `redis-cli ping`
4. Verify environment variables

### Frontend Not Loading

1. Check if API is running: `curl http://localhost:3000/health`
2. Check browser console for errors
3. Verify CORS is enabled in API
4. Clear browser cache and localStorage

### Database Connection Issues

```bash
# Test connection
psql $DATABASE_URL -c "SELECT 1"

# Check connection pool
# Look for "Failed to get database connection" in logs
```

### High Memory Usage

```bash
# Check container stats
docker stats

# Adjust connection pool size in .env
DATABASE_MAX_CONNECTIONS=10
```

## Support

For issues and questions:
- GitHub Issues: <repository-url>/issues
- Documentation: <repository-url>/wiki
- Email: support@example.com

## License

MIT License - See LICENSE file for details
