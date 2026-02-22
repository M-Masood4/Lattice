#!/bin/bash

# Fly.io Deployment Script for Solana Whale Tracker
# This script will deploy the application to Fly.io free tier

set -e

echo "🚀 Fly.io Deployment Script"
echo "============================"
echo ""

# Check if flyctl is installed
if ! command -v flyctl &> /dev/null && ! command -v fly &> /dev/null; then
    echo "❌ Fly.io CLI not found. Installing..."
    curl -L https://fly.io/install.sh | sh
    export FLYCTL_INSTALL="/Users/$USER/.fly"
    export PATH="$FLYCTL_INSTALL/bin:$PATH"
    echo "✅ Fly.io CLI installed"
    echo ""
    echo "⚠️  Please run this command to add flyctl to your PATH:"
    echo "    export PATH=\"\$HOME/.fly/bin:\$PATH\""
    echo ""
    echo "Then run this script again."
    exit 0
fi

# Use flyctl or fly command
FLY_CMD="flyctl"
if ! command -v flyctl &> /dev/null; then
    FLY_CMD="fly"
fi

echo "📋 Step 1: Check Fly.io authentication"
if ! $FLY_CMD auth whoami &> /dev/null; then
    echo "❌ Not logged in to Fly.io"
    echo "Please run: $FLY_CMD auth login"
    exit 1
fi
echo "✅ Authenticated with Fly.io"
echo ""

echo "📋 Step 2: Create Fly.io app (if not exists)"
APP_NAME="solana-whale-tracker-$(whoami | tr '[:upper:]' '[:lower:]')"
echo "App name: $APP_NAME"

# Check if app exists
if $FLY_CMD apps list | grep -q "$APP_NAME"; then
    echo "✅ App already exists"
else
    echo "Creating new app..."
    $FLY_CMD apps create "$APP_NAME" --org personal || true
    echo "✅ App created"
fi
echo ""

echo "📋 Step 3: Set up PostgreSQL database"
DB_NAME="${APP_NAME}-db"
if $FLY_CMD postgres list | grep -q "$DB_NAME"; then
    echo "✅ Database already exists"
else
    echo "Creating PostgreSQL database (free tier)..."
    $FLY_CMD postgres create --name "$DB_NAME" --region iad --initial-cluster-size 1 --vm-size shared-cpu-1x --volume-size 1
    echo "✅ Database created"
fi

# Attach database to app
echo "Attaching database to app..."
$FLY_CMD postgres attach "$DB_NAME" --app "$APP_NAME" || echo "Database already attached"
echo ""

echo "📋 Step 4: Set up Redis"
REDIS_NAME="${APP_NAME}-redis"
if $FLY_CMD redis list | grep -q "$REDIS_NAME"; then
    echo "✅ Redis already exists"
else
    echo "Creating Redis instance (free tier)..."
    $FLY_CMD redis create --name "$REDIS_NAME" --region iad --no-replicas --plan free || true
    echo "✅ Redis created"
fi
echo ""

echo "📋 Step 5: Set environment variables"
$FLY_CMD secrets set \
    BIRDEYE_API_KEY="fb5da84450bf4d49963bb14c8ee845e9" \
    SIDESHIFT_SECRET="7b6aae30b7f443198db1d48c299db65b" \
    SIDESHIFT_AFFILIATE_ID="noG7TQWmjL" \
    CLAUDE_API_KEY="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" \
    JWT_SECRET="$(openssl rand -base64 32)" \
    SKIP_MIGRATIONS="false" \
    --app "$APP_NAME"
echo "✅ Environment variables set"
echo ""

echo "📋 Step 6: Deploy application"
echo "This will build and deploy your application..."
$FLY_CMD deploy --app "$APP_NAME" --ha=false
echo "✅ Application deployed"
echo ""

echo "📋 Step 7: Get application URL"
APP_URL=$($FLY_CMD info --app "$APP_NAME" | grep "Hostname" | awk '{print $3}')
echo ""
echo "🎉 Deployment Complete!"
echo "======================="
echo ""
echo "Your application is now live at:"
echo "  https://$APP_URL"
echo ""
echo "Mesh test page:"
echo "  https://$APP_URL/mesh-test.html"
echo ""
echo "API endpoints:"
echo "  https://$APP_URL/health"
echo "  https://$APP_URL/api/mesh/network/status"
echo "  https://$APP_URL/api/mesh/prices"
echo ""
echo "To view logs:"
echo "  $FLY_CMD logs --app $APP_NAME"
echo ""
echo "To check status:"
echo "  $FLY_CMD status --app $APP_NAME"
echo ""
