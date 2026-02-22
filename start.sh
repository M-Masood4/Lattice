#!/bin/bash

# Solana Whale Tracker - Quick Start Script

set -e

echo "🐋 Solana Whale Tracker - Quick Start"
echo "======================================"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    echo "   Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    echo "   Visit: https://docs.docker.com/compose/install/"
    exit 1
fi

# Check if .env file exists
if [ ! -f .env ]; then
    echo "⚠️  No .env file found. Creating from example..."
    if [ -f .env.example ]; then
        cp .env.example .env
        echo "✅ Created .env file from .env.example"
        echo "⚠️  Please edit .env with your configuration before continuing."
        echo ""
        read -p "Press Enter to continue after editing .env, or Ctrl+C to exit..."
    else
        echo "❌ No .env.example file found. Creating minimal .env..."
        cat > .env << EOF
# Database
DATABASE_URL=postgresql://postgres:password@postgres:5432/whale_tracker
POSTGRES_PASSWORD=password

# Redis
REDIS_URL=redis://redis:6379

# Solana
SOLANA_RPC_URL=https://api.devnet.solana.com

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# JWT (CHANGE THIS IN PRODUCTION!)
JWT_SECRET=change_this_secret_in_production

# Optional: Add your API keys
# CLAUDE_API_KEY=your_key_here
# STRIPE_SECRET_KEY=your_key_here
# AWS_ACCESS_KEY_ID=your_key_here
# AWS_SECRET_ACCESS_KEY=your_key_here
# SQS_QUEUE_URL=your_queue_url_here
EOF
        echo "✅ Created minimal .env file"
        echo "⚠️  Please edit .env with your API keys for full functionality."
        echo ""
    fi
fi

echo "📦 Building Docker images..."
docker-compose build

echo ""
echo "🚀 Starting services..."
docker-compose up -d

echo ""
echo "⏳ Waiting for services to be healthy..."
sleep 5

# Wait for health check
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -f http://localhost:3000/health &> /dev/null; then
        echo "✅ All services are healthy!"
        break
    fi
    
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo "   Waiting for services... ($RETRY_COUNT/$MAX_RETRIES)"
    sleep 2
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "❌ Services failed to start. Check logs with: docker-compose logs"
    exit 1
fi

echo ""
echo "======================================"
echo "✅ Solana Whale Tracker is running!"
echo "======================================"
echo ""
echo "🌐 Frontend:     http://localhost:3000"
echo "🔌 API:          http://localhost:3000/api"
echo "💚 Health Check: http://localhost:3000/health"
echo ""
echo "📊 View logs:    docker-compose logs -f"
echo "🛑 Stop:         docker-compose down"
echo "🔄 Restart:      docker-compose restart"
echo ""
echo "📖 For more information, see DEPLOYMENT.md"
echo ""
