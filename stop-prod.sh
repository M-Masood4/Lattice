#!/bin/bash

# Stop production deployment

echo "🛑 Stopping Solana Whale Tracker..."

if [ -f .pids/frontend-prod.pid ]; then
    FRONTEND_PID=$(cat .pids/frontend-prod.pid)
    if kill -0 $FRONTEND_PID 2>/dev/null; then
        kill $FRONTEND_PID
        echo "✅ Frontend stopped (PID: $FRONTEND_PID)"
    fi
    rm .pids/frontend-prod.pid
fi

if [ -f .pids/backend-prod.pid ]; then
    BACKEND_PID=$(cat .pids/backend-prod.pid)
    if kill -0 $BACKEND_PID 2>/dev/null; then
        kill $BACKEND_PID
        echo "✅ Backend stopped (PID: $BACKEND_PID)"
    fi
    rm .pids/backend-prod.pid
fi

echo "✅ All services stopped"
