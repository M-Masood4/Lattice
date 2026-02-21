# Solana Whale Tracker - Quick Start Guide

Get up and running in 5 minutes!

## Prerequisites

- Docker and Docker Compose installed
- 5 minutes of your time

## Step 1: Clone and Setup

```bash
git clone <repository-url>
cd solana-whale-tracker
```

## Step 2: Start the Application

```bash
./start.sh
```

That's it! The script will:
- Check for Docker installation
- Create a default `.env` file if needed
- Build Docker images
- Start all services (PostgreSQL, Redis, API)
- Wait for services to be healthy
- Display access URLs

## Step 3: Access the Application

Open your browser and navigate to:

**http://localhost:3000**

## Using the Application

### 1. Connect Your Wallet

- Enter a Solana wallet address in the input field
- Click "Connect Wallet"
- Your portfolio will load automatically

**Test Wallet Address** (for demo):
```
11111111111111111111111111111111
```

### 2. View Your Portfolio

- See your total portfolio value
- View all assets and their values
- Track the number of whales monitoring your assets

### 3. Explore Whales

- Navigate to the "Whales" tab
- See all whale accounts holding the same assets
- View whale rankings and position sizes
- Click "Refresh" to update whale data

### 4. Check Analytics

- Navigate to the "Analytics" tab
- Select a time period (24h, 7d, 30d)
- View:
  - Portfolio performance and gains/losses
  - Whale impact on your portfolio
  - AI recommendation accuracy

### 5. Configure Settings

- Navigate to the "Settings" tab
- Update API Base URL if needed
- Save your preferences

## Demo Mode

The frontend includes mock data for demonstration when the API is unavailable. This allows you to:
- Explore the UI without a backend
- Test features independently
- Demo to stakeholders

## Stopping the Application

```bash
docker-compose down
```

## Viewing Logs

```bash
# View all logs
docker-compose logs -f

# View specific service
docker-compose logs -f api
docker-compose logs -f postgres
docker-compose logs -f redis
```

## Troubleshooting

### Port Already in Use

If port 3000 is already in use, edit `docker-compose.yml`:

```yaml
api:
  ports:
    - "8080:3000"  # Change 3000 to 8080
```

Then access at http://localhost:8080

### Services Not Starting

```bash
# Check service status
docker-compose ps

# Restart services
docker-compose restart

# Rebuild and restart
docker-compose up -d --build
```

### Database Connection Issues

```bash
# Check database logs
docker-compose logs postgres

# Reset database
docker-compose down -v
docker-compose up -d
```

### API Not Responding

```bash
# Check API logs
docker-compose logs api

# Check health endpoint
curl http://localhost:3000/health
```

## Next Steps

### Add API Keys for Full Functionality

Edit `.env` and add your API keys:

```env
# Claude API (for AI analysis)
CLAUDE_API_KEY=your_key_here

# Stripe (for payments)
STRIPE_SECRET_KEY=your_key_here

# AWS (for message queue)
AWS_ACCESS_KEY_ID=your_key_here
AWS_SECRET_ACCESS_KEY=your_key_here
SQS_QUEUE_URL=your_queue_url_here
```

Then restart:
```bash
docker-compose restart api
```

### Deploy to Production

See [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment instructions.

### Customize the Frontend

The frontend is in the `frontend/` directory:
- `index.html` - Structure
- `styles.css` - Styling
- `app.js` - Logic

No build step required! Just edit and refresh.

## Common Use Cases

### Testing with Real Wallet

```bash
# Use your actual Solana wallet address
# The app will fetch real data from the blockchain
```

### Running Without Docker

```bash
# Start PostgreSQL
createdb whale_tracker

# Start Redis
redis-server

# Set environment variables
export DATABASE_URL=postgresql://localhost/whale_tracker
export REDIS_URL=redis://localhost:6379
export SOLANA_RPC_URL=https://api.devnet.solana.com

# Run migrations
cargo run --bin migrate

# Start API
cargo run --bin api
```

### Frontend Only

```bash
cd frontend
python3 -m http.server 8080
# Open http://localhost:8080
```

## Features Overview

✅ **Wallet Connection** - Connect any Solana wallet  
✅ **Portfolio Tracking** - Real-time portfolio monitoring  
✅ **Whale Detection** - Automatic whale identification  
✅ **Real-time Monitoring** - 30-second update intervals  
✅ **AI Analysis** - Claude-powered recommendations  
✅ **Performance Analytics** - Track gains and losses  
✅ **Whale Impact** - See how whales affect your portfolio  
✅ **Recommendation Tracking** - Monitor AI accuracy  
✅ **Dark Mode UI** - Modern, responsive interface  

## Support

- **Documentation**: [DEPLOYMENT.md](DEPLOYMENT.md)
- **Frontend Docs**: [frontend/README.md](frontend/README.md)
- **Issues**: GitHub Issues
- **API Reference**: [docs/API.md](docs/API.md)

## What's Next?

1. ✅ Connect your wallet
2. ✅ Explore the dashboard
3. ✅ Check whale activity
4. ✅ View analytics
5. 🚀 Deploy to production
6. 💰 Add payment integration
7. 🤖 Enable auto-trading

---

**Ready to deploy?** See [DEPLOYMENT.md](DEPLOYMENT.md)  
**Need help?** Check the troubleshooting section above  
**Want to contribute?** See [CONTRIBUTING.md](CONTRIBUTING.md)
