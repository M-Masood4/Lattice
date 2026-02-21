#!/bin/bash

echo "🛑 Stopping Solana Whale Tracker..."

if [ -f .pids/frontend.pid ]; then
    kill $(cat .pids/frontend.pid) 2>/dev/null && echo "✓ Frontend stopped"
    rm .pids/frontend.pid
fi

if [ -f .pids/backend.pid ]; then
    kill $(cat .pids/backend.pid) 2>/dev/null && echo "✓ Backend stopped"
    rm .pids/backend.pid
fi

# Also kill by process name as backup
pkill -f "http.server 8080" 2>/dev/null
pkill -f "cargo run.*api" 2>/dev/null

echo "✅ All services stopped"
