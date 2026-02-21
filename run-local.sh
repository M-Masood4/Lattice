#!/bin/bash

# Solana Whale Tracker - Local Development Startup Script
# Runs frontend and backend without Docker

set -e

echo "🐋 Solana Whale Tracker - Local Development"
echo "==========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
FRONTEND_PORT=8080
BACKEND_PORT=3000
POSTGRES_PORT=5432
REDIS_PORT=6379

# PID file locations
PIDDIR="./.pids"
mkdir -p "$PIDDIR"

FRONTEND_PID="$PIDDIR/frontend.pid"
BACKEND_PID="$PIDDIR/backend.pid"
POSTGRES_PID="$PIDDIR/postgres.pid"
REDIS_PID="$PIDDIR/redis.pid"

# Cleanup function
cleanup() {
    echo ""
    echo "🛑 Stopping all services..."
    
    if [ -f "$FRONTEND_PID" ]; then
        kill $(cat "$FRONTEND_PID") 2>/dev/null || true
        rm "$FRONTEND_PID"
    fi
    
    if [ -f "$BACKEND_PID" ]; then
        kill $(cat "$BACKEND_PID") 2>/dev/null || true
        rm "$BACKEND_PID"
    fi
    
    if [ -f "$REDIS_PID" ]; then
        kill $(cat "$REDIS_PID") 2>/dev/null || true
        rm "$REDIS_PID"
    fi
    
    if [ -f "$POSTGRES_PID" ]; then
        kill $(cat "$POSTGRES_PID") 2>/dev/null || true
        rm "$POSTGRES_PID"
    fi
    
    echo "✅ All services stopped"
    exit 0
}

# Set up trap for cleanup
trap cleanup SIGINT SIGTERM EXIT

# Check for required tools
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed${NC}"
        return 1
    fi
    echo -e "${GREEN}✓${NC} $1 found"
    return 0
}

echo "📋 Checking prerequisites..."
MISSING_DEPS=0

check_command cargo || MISSING_DEPS=1
check_command python3 || MISSING_DEPS=1

# Check for PostgreSQL (optional)
if command -v postgres &> /dev/null || command -v psql &> /dev/null; then
    echo -e "${GREEN}✓${NC} PostgreSQL found"
    HAS_POSTGRES=1
else
    echo -e "${YELLOW}⚠${NC}  PostgreSQL not found (will use mock data)"
    HAS_POSTGRES=0
fi

# Check for Redis (optional)
if command -v redis-server &> /dev/null; then
    echo -e "${GREEN}✓${NC} Redis found"
    HAS_REDIS=1
else
    echo -e "${YELLOW}⚠${NC}  Redis not found (will use in-memory cache)"
    HAS_REDIS=0
fi

if [ $MISSING_DEPS -eq 1 ]; then
    echo ""
    echo -e "${RED}❌ Missing required dependencies. Please install them first.${NC}"
    exit 1
fi

echo ""
echo "🔧 Setting up environment..."

# Create minimal .env if it doesn't exist
if [ ! -f .env ]; then
    cat > .env << EOF
# Minimal configuration for local development
DATABASE_URL=postgresql://localhost:5432/whale_tracker
REDIS_URL=redis://localhost:6379
SOLANA_RPC_URL=https://api.devnet.solana.com
SERVER_HOST=127.0.0.1
SERVER_PORT=$BACKEND_PORT
JWT_SECRET=local_dev_secret_change_in_production
RUST_LOG=info,solana_whale_tracker=debug
EOF
    echo "✅ Created .env file"
fi

# Source environment variables
export $(cat .env | grep -v '^#' | xargs)

echo ""
echo "🚀 Starting services..."

# Start Redis if available
if [ $HAS_REDIS -eq 1 ]; then
    echo "Starting Redis..."
    redis-server --port $REDIS_PORT --daemonize yes --pidfile "$REDIS_PID"
    sleep 1
    if [ -f "$REDIS_PID" ]; then
        echo -e "${GREEN}✓${NC} Redis started on port $REDIS_PORT"
    else
        echo -e "${YELLOW}⚠${NC}  Redis failed to start, continuing without it"
    fi
fi

# Start PostgreSQL if available
if [ $HAS_POSTGRES -eq 1 ]; then
    echo "Checking PostgreSQL..."
    if pg_isready -q 2>/dev/null; then
        echo -e "${GREEN}✓${NC} PostgreSQL is already running"
        
        # Create database if it doesn't exist
        if ! psql -lqt | cut -d \| -f 1 | grep -qw whale_tracker; then
            echo "Creating database..."
            createdb whale_tracker 2>/dev/null || true
        fi
        
        # Run migrations
        echo "Running database migrations..."
        DATABASE_URL="postgresql://localhost:5432/whale_tracker" cargo run --bin migrate 2>/dev/null || echo -e "${YELLOW}⚠${NC}  Migrations skipped"
    else
        echo -e "${YELLOW}⚠${NC}  PostgreSQL not running, continuing without it"
    fi
fi

# Build the backend
echo ""
echo "🔨 Building backend..."
if cargo build --release --bin api 2>&1 | tail -5; then
    echo -e "${GREEN}✓${NC} Backend built successfully"
else
    echo -e "${RED}❌ Backend build failed${NC}"
    exit 1
fi

# Start the backend
echo ""
echo "🚀 Starting backend API..."
./target/release/api > logs/backend.log 2>&1 &
BACKEND_PID_NUM=$!
echo $BACKEND_PID_NUM > "$BACKEND_PID"

# Wait for backend to start
echo "Waiting for backend to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -f http://localhost:$BACKEND_PORT/health &> /dev/null; then
        echo -e "${GREEN}✓${NC} Backend API is ready"
        break
    fi
    
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
        echo -e "${RED}❌ Backend failed to start. Check logs/backend.log${NC}"
        exit 1
    fi
    
    sleep 1
done

# Start the frontend
echo ""
echo "🎨 Starting frontend..."
python3 -m http.server $FRONTEND_PORT --directory frontend > logs/frontend.log 2>&1 &
FRONTEND_PID_NUM=$!
echo $FRONTEND_PID_NUM > "$FRONTEND_PID"

sleep 2

if kill -0 $FRONTEND_PID_NUM 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Frontend server started"
else
    echo -e "${RED}❌ Frontend failed to start${NC}"
    exit 1
fi

# Display status
echo ""
echo "=========================================="
echo -e "${GREEN}✅ Solana Whale Tracker is running!${NC}"
echo "=========================================="
echo ""
echo "🌐 Frontend:     http://localhost:$FRONTEND_PORT"
echo "🔌 Backend API:  http://localhost:$BACKEND_PORT"
echo "💚 Health Check: http://localhost:$BACKEND_PORT/health"
echo ""
echo "📊 Service Status:"
echo "  - Frontend:    Running (PID: $FRONTEND_PID_NUM)"
echo "  - Backend:     Running (PID: $BACKEND_PID_NUM)"
if [ $HAS_REDIS -eq 1 ] && [ -f "$REDIS_PID" ]; then
    echo "  - Redis:       Running (PID: $(cat $REDIS_PID))"
fi
if [ $HAS_POSTGRES -eq 1 ]; then
    echo "  - PostgreSQL:  Running"
fi
echo ""
echo "📝 Logs:"
echo "  - Backend:  tail -f logs/backend.log"
echo "  - Frontend: tail -f logs/frontend.log"
echo ""
echo "🛑 Press Ctrl+C to stop all services"
echo ""

# Keep script running
wait
