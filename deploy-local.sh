#!/bin/bash

# Solana Whale Tracker - Local Production Deployment
# This script deploys the application using local PostgreSQL and Redis

set -e

echo "🐋 Solana Whale Tracker - Local Production Deployment"
echo "======================================================"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

if ! pg_isready -q 2>/dev/null; then
    echo "❌ PostgreSQL is not running. Please start PostgreSQL first."
    exit 1
fi

if ! redis-cli ping &> /dev/null; then
    echo "❌ Redis is not running. Please start Redis first."
    exit 1
fi

echo "✅ All prerequisites met"
echo ""

# Create directories
mkdir -p logs .pids

# Build the application in release mode
echo "🔨 Building application in release mode..."
echo "   (This may take a few minutes...)"
cargo build --release --bin api

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo "❌ .env file not found"
    exit 1
fi

echo "✅ Environment configuration found"
echo ""

# Stop any existing processes
echo "🧹 Stopping existing processes..."
pkill -f "python3.*8080" 2>/dev/null || true
pkill -f "target/release/api" 2>/dev/null || true
sleep 2

# Start frontend
echo "🎨 Starting frontend on port 8080..."
python3 -m http.server 8080 --directory frontend > logs/frontend-prod.log 2>&1 &
FRONTEND_PID=$!
echo $FRONTEND_PID > .pids/frontend-prod.pid

sleep 2

if ! kill -0 $FRONTEND_PID 2>/dev/null; then
    echo "❌ Frontend failed to start"
    exit 1
fi

echo "✅ Frontend started (PID: $FRONTEND_PID)"
echo ""

# Start backend in production mode
echo "🚀 Starting backend API on port 3000..."
export $(cat .env | grep -v '^#' | xargs)
export RUST_LOG=info

./target/release/api > logs/backend-prod.log 2>&1 &
BACKEND_PID=$!
echo $BACKEND_PID > .pids/backend-prod.pid

# Wait for backend to start
echo "⏳ Waiting for backend to start..."
MAX_WAIT=60
WAITED=0

while [ $WAITED -lt $MAX_WAIT ]; do
    if curl -f http://localhost:3000/health &> /dev/null 2>&1; then
        echo ""
        echo "✅ Backend started (PID: $BACKEND_PID)"
        break
    fi
    
    # Check if process is still running
    if ! kill -0 $BACKEND_PID 2>/dev/null; then
        echo ""
        echo "❌ Backend process died. Check logs/backend-prod.log for errors"
        tail -30 logs/backend-prod.log
        exit 1
    fi
    
    printf "."
    sleep 2
    WAITED=$((WAITED + 2))
done

if [ $WAITED -ge $MAX_WAIT ]; then
    echo ""
    echo "⚠️  Backend is taking longer than expected"
    echo "   Check logs/backend-prod.log for details"
    exit 1
fi

echo ""
echo "=========================================="
echo "✅ Deployment Successful!"
echo "=========================================="
echo ""
echo "🌐 Frontend:     http://localhost:8080"
echo "🔌 Backend API:  http://localhost:3000"
echo "💚 Health:       http://localhost:3000/health"
echo ""
echo "📝 View logs:"
echo "   Frontend:  tail -f logs/frontend-prod.log"
echo "   Backend:   tail -f logs/backend-prod.log"
echo ""
echo "🛑 To stop:"
echo "   kill \$(cat .pids/frontend-prod.pid)"
echo "   kill \$(cat .pids/backend-prod.pid)"
echo ""
echo "Or run: ./stop-prod.sh"
echo ""
echo "📊 Monitor health:"
echo "   curl http://localhost:3000/health | jq ."
echo ""
