# Railway.app Deployment Guide

## Quick Setup (5 minutes)

### Step 1: Create Railway Account
1. Go to https://railway.app
2. Sign up with GitHub (recommended for auto-deploy)
3. Verify your email

### Step 2: Create New Project
1. Click "New Project"
2. Select "Deploy from GitHub repo"
3. Connect your GitHub account and select this repository
4. Railway will auto-detect the Rust project

### Step 3: Add PostgreSQL Database
1. In your project, click "New"
2. Select "Database" → "PostgreSQL"
3. Railway will automatically create and connect the database
4. The `DATABASE_URL` environment variable is auto-configured

### Step 4: Add Redis
1. Click "New" again
2. Select "Database" → "Redis"
3. Railway will automatically create and connect Redis
4. The `REDIS_URL` environment variable is auto-configured

### Step 5: Configure Environment Variables
In your Railway project settings, add these variables:

**Required:**
```
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_RPC_FALLBACK_URL=https://api.devnet.solana.com
SOLANA_NETWORK=devnet
JWT_SECRET=your_random_secret_key_here
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

**Optional (for full features):**
```
CLAUDE_API_KEY=your_claude_api_key
CLAUDE_MODEL=claude-3-sonnet-20240229
CLAUDE_MAX_TOKENS=4096
STRIPE_SECRET_KEY=your_stripe_secret_key
STRIPE_WEBHOOK_SECRET=your_stripe_webhook_secret
STRIPE_BASIC_PRICE_ID=your_basic_price_id
STRIPE_PREMIUM_PRICE_ID=your_premium_price_id
```

**Auto-configured by Railway:**
- `DATABASE_URL` (from PostgreSQL service)
- `REDIS_URL` (from Redis service)

### Step 6: Deploy
1. Railway will automatically deploy on every git push
2. First deployment takes ~5-10 minutes (Rust compilation)
3. Subsequent deploys are faster (~2-3 minutes)

### Step 7: Get Your URL
1. Go to your service settings
2. Click "Generate Domain"
3. Railway provides a free `.railway.app` subdomain
4. Your app will be available at: `https://your-app-name.railway.app`

## Manual Deployment (Alternative)

If you prefer CLI deployment:

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login
railway login

# Initialize project
railway init

# Link to your project
railway link

# Add PostgreSQL
railway add --database postgresql

# Add Redis
railway add --database redis

# Set environment variables
railway variables set SOLANA_RPC_URL=https://api.devnet.solana.com
railway variables set JWT_SECRET=your_secret_here
# ... add other variables

# Deploy
railway up
```

## Configuration Files

The following files are configured for Railway:

- `railway.json` - Railway-specific configuration
- `nixpacks.toml` - Build configuration
- `Procfile` - Start command
- `Dockerfile` - Alternative Docker deployment

## Monitoring

### View Logs
```bash
railway logs
```

Or in the Railway dashboard:
1. Go to your service
2. Click "Deployments"
3. Click on the latest deployment
4. View real-time logs

### Check Health
```bash
curl https://your-app-name.railway.app/health
```

## Troubleshooting

### Build Fails
- Check Railway build logs
- Ensure all dependencies are in `Cargo.toml`
- Verify Rust version compatibility

### Database Connection Issues
- Verify `DATABASE_URL` is set correctly
- Check PostgreSQL service is running
- Review connection pool settings

### App Crashes on Start
- Check environment variables are set
- Review startup logs in Railway dashboard
- Verify all required services (PostgreSQL, Redis) are running

## Cost Estimate

**Free Tier Includes:**
- $5 credit per month
- 500 hours of usage
- Unlimited projects
- Custom domains

**Typical Usage for This App:**
- API Service: ~$3/month
- PostgreSQL: ~$1/month
- Redis: ~$0.50/month
- **Total: ~$4.50/month** (within free tier!)

## Scaling

If you need more resources:
1. Go to service settings
2. Adjust resources (CPU, RAM)
3. Railway charges based on usage

## Custom Domain (Optional)

1. Go to service settings
2. Click "Custom Domain"
3. Add your domain
4. Configure DNS records as shown
5. SSL certificate is automatic

## GitHub Integration

Railway automatically:
- Deploys on every push to main branch
- Creates preview deployments for PRs
- Rolls back on deployment failures

## Environment-Specific Deployments

Create separate Railway projects for:
- **Development**: Uses devnet, test keys
- **Production**: Uses mainnet, real keys

## Backup Strategy

Railway automatically backs up PostgreSQL:
- Daily backups retained for 7 days
- Manual backups available anytime
- Point-in-time recovery available

## Support

- Railway Docs: https://docs.railway.app
- Discord: https://discord.gg/railway
- Status: https://status.railway.app

## Next Steps After Deployment

1. ✅ Test all endpoints
2. ✅ Verify database migrations ran
3. ✅ Check health endpoint
4. ✅ Test wallet connection
5. ✅ Monitor logs for errors
6. ✅ Set up custom domain (optional)
7. ✅ Add real API keys when ready

Your app is now live and accessible worldwide! 🚀
