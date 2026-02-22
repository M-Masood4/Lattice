# 🚀 Deploy to Railway.app - Quick Start

## ✅ What's Ready

Your Solana Whale Tracker is **100% ready to deploy**! All configuration files are in place:

- ✅ `railway.json` - Railway configuration
- ✅ `nixpacks.toml` - Build settings
- ✅ `Procfile` - Start command
- ✅ `Dockerfile` - Container config
- ✅ `.railwayignore` - Deployment optimization
- ✅ Production-ready code

## 🎯 Deploy in 5 Steps (10 minutes)

### Step 1: Initialize Git (1 minute)

```bash
git init
git add .
git commit -m "Initial commit - Solana Whale Tracker"
```

### Step 2: Push to GitHub (2 minutes)

1. Go to https://github.com/new
2. Create repository: `solana-whale-tracker`
3. Don't initialize with README

```bash
git remote add origin https://github.com/YOUR_USERNAME/solana-whale-tracker.git
git branch -M main
git push -u origin main
```

### Step 3: Create Railway Project (2 minutes)

1. Go to https://railway.app
2. Click "Login" → Sign in with GitHub
3. Click "New Project"
4. Select "Deploy from GitHub repo"
5. Choose `solana-whale-tracker`
6. Railway will start building automatically

### Step 4: Add Databases (2 minutes)

**Add PostgreSQL:**
1. In your project, click "+ New"
2. Select "Database" → "PostgreSQL"
3. Done! `DATABASE_URL` is auto-configured

**Add Redis:**
1. Click "+ New" again
2. Select "Database" → "Redis"  
3. Done! `REDIS_URL` is auto-configured

### Step 5: Set Environment Variables (3 minutes)

Click on your API service → "Variables" tab → Add these:

**Required (copy-paste these):**
```
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_RPC_FALLBACK_URL=https://api.devnet.solana.com
SOLANA_NETWORK=devnet
JWT_SECRET=hackathon_secret_key_2024
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
DATABASE_MAX_CONNECTIONS=10
REDIS_POOL_SIZE=10
CLAUDE_API_KEY=demo_key
CLAUDE_MODEL=claude-3-sonnet-20240229
CLAUDE_MAX_TOKENS=4096
STRIPE_SECRET_KEY=sk_test_demo
STRIPE_WEBHOOK_SECRET=whsec_demo
STRIPE_BASIC_PRICE_ID=price_demo_basic
STRIPE_PREMIUM_PRICE_ID=price_demo_premium
RUST_LOG=info
```

**Note:** `DATABASE_URL` and `REDIS_URL` are automatically set by Railway!

### Step 6: Get Your Live URL

1. Go to your service settings
2. Click "Generate Domain"
3. Your app will be at: `https://your-app-name.railway.app`

## 🎉 That's It!

Your app is now live! Test it:

```bash
curl https://your-app-name.railway.app/health
```

## 📊 What You Get

- ✅ Live URL accessible worldwide
- ✅ Automatic HTTPS/SSL
- ✅ PostgreSQL database (managed)
- ✅ Redis cache (managed)
- ✅ Auto-deploy on git push
- ✅ Free for competition duration ($5/month credit)
- ✅ Logs and monitoring dashboard

## 🔧 Monitoring Your App

**View Logs:**
- Railway Dashboard → Your Service → "Deployments" → Latest deployment

**Check Health:**
```bash
curl https://your-app-name.railway.app/health
```

**View Metrics:**
- Railway Dashboard → Your Service → "Metrics"

## 🐛 Troubleshooting

**Build fails?**
- Check Railway build logs
- First build takes 5-10 minutes (Rust compilation)

**App crashes?**
- Verify all environment variables are set
- Check PostgreSQL and Redis services are running
- Review logs in Railway dashboard

**Database connection error?**
- Ensure PostgreSQL service is added
- `DATABASE_URL` should be auto-configured
- Check service is in same project

## 💰 Cost

**Free Tier:**
- $5 credit/month
- Your app uses ~$4.50/month
- **You're covered for the entire competition!**

## 🔄 Updates

After initial deployment, just push to GitHub:

```bash
git add .
git commit -m "Update feature"
git push
```

Railway auto-deploys in ~2-3 minutes!

## 🌐 Custom Domain (Optional)

1. Railway Dashboard → Service → "Settings"
2. Click "Custom Domain"
3. Add your domain
4. Configure DNS as shown
5. SSL is automatic

## 📱 Share Your App

Your live URL: `https://your-app-name.railway.app`

Perfect for:
- Competition judges
- Demo presentations
- Testing with real users
- Portfolio showcase

## 🎯 Next Steps

1. ✅ Deploy to Railway (follow steps above)
2. ✅ Test all features
3. ✅ Share URL with judges
4. ✅ Add real API keys later (optional)
5. ✅ Monitor usage in Railway dashboard

## 📚 Additional Resources

- Full guide: `RAILWAY_DEPLOYMENT.md`
- Railway docs: https://docs.railway.app
- Support: https://discord.gg/railway

---

**Ready to deploy? Run:**
```bash
./setup-railway.sh
```

Then follow the steps above! 🚀
