#!/bin/bash

# Solana Whale Tracker - Quick Development Startup
# Runs frontend and backend in development mode (faster startup)

set -e

echo "🐋 Solana Whale Tracker - Development Mode"
echo "=========================================="
echo ""

# Create directories
mkdir -p logs .pids

# Check if .env exists
if [ ! -f .env ]; then
    echo "❌ .env file not found. Please create one from .env.example"
    exit 1
fi

echo "✅ Using existing .env configuration"
echo ""

# Export environment
export $(cat .env | grep -v '^#' | xargs)

# Stop any existing processes
echo "🧹 Cleaning up existing processes..."
pkill -f "http.server 8080" 2>/dev/null || true
pkill -f "cargo run" 2>/dev/null || true
sleep 1

# Start frontend
echo "🎨 Starting frontend on port 8080..."
python3 -m http.server 8080 --directory frontend > logs/frontend.log 2>&1 &
FRONTEND_PID=$!
echo $FRONTEND_PID > .pids/frontend.pid

sleep 2

if kill -0 $FRONTEND_PID 2>/dev/null; then
    echo "✅ Frontend started (PID: $FRONTEND_PID)"
else
    echo "❌ Frontend failed to start"
    exit 1
fi

# Start backend in development mode
echo ""
echo "🚀 Starting backend API on port 3000..."
echo "   (This may take a minute to compile...)"
echo ""

cargo run --bin api > logs/backend.log 2>&1 &
BACKEND_PID=$!
echo $BACKEND_PID > .pids/backend.pid

# Wait for backend
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
        echo "❌ Backend process died. Check logs/backend.log for errors"
        cat logs/backend.log | tail -20
        exit 1
    fi
    
    printf "."
    sleep 2
    WAITED=$((WAITED + 2))
done

if [ $WAITED -ge $MAX_WAIT ]; then
    echo ""
    echo "⚠️  Backend is taking longer than expected"
    echo "   It may still be compiling. Check logs/backend.log"
    echo ""
fi

echo ""
echo "=========================================="
echo "✅ Services Started!"
echo "=========================================="
echo ""
echo "🌐 Frontend:     http://localhost:8080"
echo "🔌 Backend API:  http://localhost:3000"
echo "💚 Health:       http://localhost:3000/health"
echo ""
echo "📝 View logs:"
echo "   Frontend:  tail -f logs/frontend.log"
echo "   Backend:   tail -f logs/backend.log"
echo ""
echo "🛑 To stop:"
echo "   kill \$(cat .pids/frontend.pid)"
echo "   kill \$(cat .pids/backend.pid)"
echo ""
echo "Or run: ./stop-dev.sh"
echo ""
