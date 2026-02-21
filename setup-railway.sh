#!/bin/bash

# Railway Setup Helper Script

echo "🚂 Railway.app Deployment Setup"
echo "================================"
echo ""

# Check if git repo exists
if [ ! -d .git ]; then
    echo "📦 Initializing git repository..."
    git init
    git add .
    git commit -m "Initial commit - Solana Whale Tracker"
    echo "✅ Git repository initialized"
else
    echo "✅ Git repository exists"
fi

echo ""
echo "📋 Next Steps:"
echo ""
echo "1. Create a GitHub repository:"
echo "   - Go to https://github.com/new"
echo "   - Create a new repository (e.g., 'solana-whale-tracker')"
echo "   - Don't initialize with README"
echo ""
echo "2. Push your code to GitHub:"
echo "   git remote add origin https://github.com/YOUR_USERNAME/YOUR_REPO.git"
echo "   git branch -M main"
echo "   git push -u origin main"
echo ""
echo "3. Deploy to Railway:"
echo "   - Go to https://railway.app"
echo "   - Sign up/Login with GitHub"
echo "   - Click 'New Project'"
echo "   - Select 'Deploy from GitHub repo'"
echo "   - Choose your repository"
echo "   - Add PostgreSQL database"
echo "   - Add Redis database"
echo "   - Set environment variables (see RAILWAY_DEPLOYMENT.md)"
echo ""
echo "4. Your app will be live at:"
echo "   https://your-app-name.railway.app"
echo ""
echo "📖 Full guide: See RAILWAY_DEPLOYMENT.md"
echo ""
