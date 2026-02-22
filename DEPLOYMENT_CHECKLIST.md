# 🚀 Railway Deployment Checklist

## Pre-Deployment ✅

- [x] Code is production-ready
- [x] All tests passing
- [x] Environment variables documented
- [x] Railway configuration files created
- [x] .gitignore configured
- [x] Documentation complete

## Deployment Steps

### 1. Git Setup
- [ ] Run: `git init`
- [ ] Run: `git add .`
- [ ] Run: `git commit -m "Initial commit - Solana Whale Tracker"`

### 2. GitHub Setup
- [ ] Create GitHub repository at https://github.com/new
- [ ] Name: `solana-whale-tracker`
- [ ] Run: `git remote add origin https://github.com/YOUR_USERNAME/solana-whale-tracker.git`
- [ ] Run: `git branch -M main`
- [ ] Run: `git push -u origin main`

### 3. Railway Setup
- [ ] Go to https://railway.app
- [ ] Sign up/Login with GitHub
- [ ] Click "New Project"
- [ ] Select "Deploy from GitHub repo"
- [ ] Choose your repository

### 4. Add Services
- [ ] Add PostgreSQL database
- [ ] Add Redis database
- [ ] Wait for services to be ready

### 5. Configure Environment Variables
Copy these to Railway Variables:
```
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_RPC_FALLBACK_URL=https://api.devnet.solana.com
SOLANA_NETWORK=devnet
JWT_SECRET=hackathon_secret_key_2024
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
CLAUDE_API_KEY=demo_key
CLAUDE_MODEL=claude-3-sonnet-20240229
STRIPE_SECRET_KEY=sk_test_demo
STRIPE_WEBHOOK_SECRET=whsec_demo
STRIPE_BASIC_PRICE_ID=price_demo_basic
STRIPE_PREMIUM_PRICE_ID=price_demo_premium
RUST_LOG=info
```

### 6. Generate Domain
- [ ] Go to service settings
- [ ] Click "Generate Domain"
- [ ] Note your URL: `https://________.railway.app`

### 7. Verify Deployment
- [ ] Check build logs (should complete in 5-10 min)
- [ ] Test health endpoint: `curl https://your-app.railway.app/health`
- [ ] Verify response shows all services healthy
- [ ] Test frontend: Open URL in browser

## Post-Deployment ✅

### Testing
- [ ] Test wallet connection
- [ ] Test portfolio viewing
- [ ] Test whale detection
- [ ] Test all API endpoints
- [ ] Check logs for errors

### Monitoring
- [ ] Set up Railway alerts (optional)
- [ ] Bookmark Railway dashboard
- [ ] Note deployment URL for judges

### Documentation
- [ ] Update README with live URL
- [ ] Share URL with team/judges
- [ ] Document any issues

## Quick Commands

**View logs:**
```bash
# In Railway dashboard: Service → Deployments → Latest
```

**Test health:**
```bash
curl https://your-app.railway.app/health
```

**Redeploy:**
```bash
git add .
git commit -m "Update"
git push
```

## Troubleshooting

**Build fails:**
- Check Railway build logs
- Verify all Cargo.toml dependencies
- First build takes 5-10 minutes

**App crashes:**
- Check environment variables are set
- Verify PostgreSQL and Redis are running
- Review application logs

**Database errors:**
- Ensure PostgreSQL service is added
- Check DATABASE_URL is auto-configured
- Verify migrations ran successfully

## Success Criteria ✅

Your deployment is successful when:
- [ ] Health endpoint returns `{"status":"healthy"}`
- [ ] Frontend loads in browser
- [ ] No errors in Railway logs
- [ ] All services show as "Active"

## Your Live URLs

**Application:** `https://________.railway.app`
**Health Check:** `https://________.railway.app/health`
**API Docs:** `https://________.railway.app/api`

## Support

- Railway Docs: https://docs.railway.app
- Discord: https://discord.gg/railway
- This project: See RAILWAY_DEPLOYMENT.md

---

**Ready to deploy?** Follow the checklist above! 🚀

**Estimated time:** 10-15 minutes
**Cost:** Free (within $5/month credit)
**Difficulty:** Easy ⭐
