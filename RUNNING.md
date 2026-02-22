# Running the Solana Whale Tracker

## Current Status

✅ **Frontend**: Running on http://localhost:8080
❌ **Backend**: Requires PostgreSQL and Redis (not currently running)

## Quick Start (Frontend Only)

The frontend is currently running and accessible at:
```
http://localhost:8080
```

### What Works Without Backend:
- UI navigation and layout
- All frontend components and styling
- Settings page (stores preferences locally)

### What Requires Backend:
- Wallet portfolio data
- Whale tracking
- Analytics
- Real-time updates

## Running the Full Stack

To run both frontend and backend, you need:

### Prerequisites:
1. **PostgreSQL** (database)
2. **Redis** (caching/message queue)

### Option 1: Using Docker (Recommended)
```bash
docker-compose up -d
```

### Option 2: Local Installation

1. Install PostgreSQL:
   ```bash
   # macOS
   brew install postgresql@14
   brew services start postgresql@14
   
   # Create database
   createdb solana_whale_tracker
   ```

2. Install Redis:
   ```bash
   # macOS
   brew install redis
   brew services start redis
   ```

3. Run the startup script:
   ```bash
   ./run-dev.sh
   ```

## Environment Configuration

The `.env` file has been created with the following configuration:
- Database: `postgresql://postgres:postgres@localhost:5432/solana_whale_tracker`
- Redis: `redis://localhost:6379`
- Solana RPC: Mainnet Beta (public endpoint)
- Server: Port 3000

### External Services (Optional for Demo):
- **Claude API**: Set `CLAUDE_API_KEY` for AI-powered recommendations
- **Stripe**: Set Stripe keys for payment processing

## Stopping Services

To stop the frontend:
```bash
kill $(cat .pids/frontend.pid)
```

Or use the stop script:
```bash
./stop-dev.sh
```

## Troubleshooting

### Backend Won't Start
- Check PostgreSQL is running: `pg_isready`
- Check Redis is running: `redis-cli ping`
- View backend logs: `tail -f logs/backend.log`

### Frontend Issues
- Check it's running: `curl http://localhost:8080`
- View frontend logs: `tail -f logs/frontend.log`

## Next Steps

1. Install PostgreSQL and Redis locally, OR
2. Use Docker Compose for the full stack, OR
3. Deploy to a cloud provider (see DEPLOYMENT.md)
