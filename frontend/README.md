# Solana Whale Tracker - Frontend

A minimal, functional web interface for the Solana Whale Tracker platform.

## Features

- 🔗 Wallet connection and portfolio viewing
- 🐋 Whale tracking and monitoring
- 📊 Portfolio performance analytics
- 📈 Whale impact analysis
- 🎯 AI recommendation accuracy tracking
- ⚙️ Configurable API endpoint
- 🎨 Dark mode UI with responsive design

## Quick Start

### Option 1: Serve with Python (Simplest)

```bash
cd frontend
python3 -m http.server 8080
```

Then open http://localhost:8080 in your browser.

### Option 2: Serve with Node.js

```bash
cd frontend
npx serve -p 8080
```

### Option 3: Serve with Rust API (Integrated)

The Rust API can serve the frontend automatically. See deployment instructions below.

## Configuration

1. Open the app in your browser
2. Go to Settings
3. Update the API Base URL to point to your backend (default: http://localhost:3000)
4. Click "Save Settings"

## Usage

### Connect Your Wallet

1. Enter your Solana wallet address in the input field
2. Click "Connect Wallet"
3. Your portfolio will be loaded automatically

### View Tracked Whales

1. Navigate to the "Whales" tab
2. See all whales that hold the same assets as you
3. Click "Refresh" to update whale data

### Analytics

1. Navigate to the "Analytics" tab
2. Select a time period (24h, 7d, 30d)
3. View:
   - Portfolio performance over time
   - Whale impact on your portfolio
   - AI recommendation accuracy

## Mock Data

The frontend includes mock data for demonstration purposes when the API is unavailable. This allows you to:
- Test the UI without a running backend
- Demo the platform to stakeholders
- Develop frontend features independently

## Browser Support

- Chrome/Edge (recommended)
- Firefox
- Safari
- Mobile browsers

## Development

The frontend is built with vanilla JavaScript, HTML, and CSS for simplicity and ease of deployment. No build step required!

### File Structure

```
frontend/
├── index.html      # Main HTML structure
├── styles.css      # All styling
├── app.js          # Application logic
└── README.md       # This file
```

### Customization

- **Colors**: Edit CSS variables in `styles.css` (`:root` section)
- **API Endpoints**: Modify `API_BASE_URL` in `app.js`
- **Mock Data**: Update mock data functions in `app.js`

## Deployment

### Deploy to Netlify/Vercel

1. Push the `frontend` folder to a Git repository
2. Connect to Netlify or Vercel
3. Set build command: (none)
4. Set publish directory: `frontend`
5. Deploy!

### Deploy with Docker

```dockerfile
FROM nginx:alpine
COPY frontend /usr/share/nginx/html
EXPOSE 80
```

### Deploy to AWS S3

```bash
aws s3 sync frontend/ s3://your-bucket-name --acl public-read
aws s3 website s3://your-bucket-name --index-document index.html
```

## API Integration

The frontend expects the following API endpoints:

- `GET /health` - Health check
- `GET /api/wallets/:address/portfolio` - Get portfolio
- `GET /api/whales/tracked` - Get tracked whales
- `POST /api/analytics/portfolio-performance` - Get performance data
- `POST /api/analytics/whale-impact` - Get whale impact
- `POST /api/analytics/recommendation-accuracy` - Get accuracy metrics

## Troubleshooting

### CORS Issues

If you encounter CORS errors, ensure your API has CORS enabled:

```rust
// In your Rust API
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

### API Connection Failed

1. Check that the API is running
2. Verify the API URL in Settings
3. Check browser console for errors
4. Ensure no firewall is blocking the connection

### Wallet Not Loading

1. Verify the wallet address format (32-44 characters)
2. Check that the wallet exists on Solana
3. Try with a known wallet address for testing

## License

MIT License - See main project LICENSE file
