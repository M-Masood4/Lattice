#!/bin/bash
set -e

echo "🚀 Setting up Solana Whale Tracker..."

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install from https://rustup.rs"
    exit 1
fi
echo "✅ Rust installed"

if ! command -v psql &> /dev/null; then
    echo "⚠️  PostgreSQL client not found. Please install PostgreSQL 14+"
fi

if ! command -v redis-cli &> /dev/null; then
    echo "⚠️  Redis client not found. Please install Redis 7+"
fi

# Create .env if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "✅ .env file created. Please edit it with your configuration."
else
    echo "✅ .env file already exists"
fi

# Build the project
echo "🔨 Building project..."
cargo build

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit .env file with your configuration"
echo "2. Start PostgreSQL and Redis"
echo "3. Run: cargo run --bin api"
