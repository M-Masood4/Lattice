#!/bin/bash

echo "🐋 Solana Whale Tracker - Status Check"
echo "======================================"
echo ""

# Check frontend
if curl -s http://localhost:8080 > /dev/null 2>&1; then
    echo "✅ Frontend: Running on http://localhost:8080"
else
    echo "❌ Frontend: Not running"
fi

# Check backend
if curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "✅ Backend: Running on http://localhost:3000"
else
    echo "❌ Backend: Not running"
fi

# Check PostgreSQL
if command -v pg_isready &> /dev/null; then
    if pg_isready -q; then
        echo "✅ PostgreSQL: Running"
    else
        echo "❌ PostgreSQL: Not running"
    fi
else
    echo "⚠️  PostgreSQL: Not installed"
fi

# Check Redis
if command -v redis-cli &> /dev/null; then
    if redis-cli ping > /dev/null 2>&1; then
        echo "✅ Redis: Running"
    else
        echo "❌ Redis: Not running"
    fi
else
    echo "⚠️  Redis: Not installed"
fi

echo ""
echo "For more information, see RUNNING.md"
