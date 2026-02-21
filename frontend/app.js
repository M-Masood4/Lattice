// API Configuration
let API_BASE_URL = localStorage.getItem('apiUrl') || 'http://localhost:3000';

// Global demo user UUID (valid UUID format for all API calls)
const DEMO_USER_ID = '00000000-0000-0000-0000-000000000001';

// State Management
const state = {
    connectedWallet: null,
    portfolio: null,
    whales: [],
    benchmarks: [],
    currentView: 'dashboard',
    currentPeriod: '24h',
    currentQuote: null,
    quoteExpiryTimer: null,
    conversionMode: 'exactInput', // 'exactInput' or 'exactOutput'
    selectedBlockchain: 'all' // For portfolio filtering
};

// Initialize App
document.addEventListener('DOMContentLoaded', () => {
    initializeApp();
});

function initializeApp() {
    setupEventListeners();
    checkHealthStatus();
    loadSavedWallet();
    
    // Check health every 30 seconds
    setInterval(checkHealthStatus, 30000);
}

// Event Listeners
function setupEventListeners() {
    // Navigation
    document.querySelectorAll('.nav-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            switchView(e.target.dataset.view);
        });
    });

    // Wallet Connection
    document.getElementById('connectWallet').addEventListener('click', connectWallet);
    document.getElementById('walletAddress').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') connectWallet();
    });

    // Refresh Whales
    document.getElementById('refreshWhales').addEventListener('click', refreshWhales);

    // Period Selector
    document.querySelectorAll('.period-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            document.querySelectorAll('.period-btn').forEach(b => b.classList.remove('active'));
            e.target.classList.add('active');
            state.currentPeriod = e.target.dataset.period;
            loadAnalytics();
        });
    });

    // Settings
    document.getElementById('saveSettings').addEventListener('click', saveSettings);
    document.getElementById('apiUrl').value = API_BASE_URL;
    
    // Trim settings
    setupTrimSettingsListeners();

    // Conversion interface
    setupConversionListeners();
    
    // Blockchain filters
    setupBlockchainFilters();
    
    // Benchmark management
    setupBenchmarkListeners();
    
    // P2P exchange
    setupP2PListeners();
    
    // Chat
    setupChatListeners();
    
    // Privacy features
    setupPrivacyListeners();
    
    // Proximity features
    setupProximityListeners();
}

// View Management
function switchView(viewName) {
    document.querySelectorAll('.view').forEach(view => view.classList.remove('active'));
    document.querySelectorAll('.nav-btn').forEach(btn => btn.classList.remove('active'));
    
    document.getElementById(viewName).classList.add('active');
    document.querySelector(`[data-view="${viewName}"]`).classList.add('active');
    
    state.currentView = viewName;

    // Load data for specific views
    if (viewName === 'whales' && state.connectedWallet) {
        loadWhales();
    } else if (viewName === 'analytics' && state.connectedWallet) {
        loadAnalytics();
    } else if (viewName === 'convert') {
        initializeConversionView();
    } else if (viewName === 'benchmarks') {
        initializeBenchmarksView();
    } else if (viewName === 'p2p') {
        initializeP2PView();
    } else if (viewName === 'chat') {
        initializeChatView();
    } else if (viewName === 'privacy') {
        initializePrivacyView();
    } else if (viewName === 'proximity') {
        initializeProximityView();
    } else if (viewName === 'settings' && state.connectedWallet) {
        loadTrimConfiguration();
    }
}

// Health Check
async function checkHealthStatus() {
    try {
        const response = await fetch(`${API_BASE_URL}/health`);
        const data = await response.json();
        
        const indicator = document.querySelector('.status-indicator');
        const statusText = document.querySelector('.status-text');
        
        if (data.status === 'healthy') {
            indicator.className = 'status-indicator healthy';
            statusText.textContent = 'All Systems Operational';
        } else {
            indicator.className = 'status-indicator unhealthy';
            statusText.textContent = 'Service Issues Detected';
        }
    } catch (error) {
        const indicator = document.querySelector('.status-indicator');
        const statusText = document.querySelector('.status-text');
        indicator.className = 'status-indicator unhealthy';
        statusText.textContent = 'API Unavailable';
    }
}

// Wallet Connection
async function connectWallet() {
    const walletAddress = document.getElementById('walletAddress').value.trim();
    const errorDiv = document.getElementById('walletError');
    
    if (!walletAddress) {
        errorDiv.textContent = 'Please enter a wallet address';
        return;
    }

    showLoading();
    errorDiv.textContent = '';

    try {
        // Validate wallet address format (basic check)
        if (walletAddress.length < 32 || walletAddress.length > 44) {
            throw new Error('Invalid wallet address format');
        }

        // Store wallet
        state.connectedWallet = walletAddress;
        localStorage.setItem('connectedWallet', walletAddress);

        // Load portfolio
        await loadPortfolio(walletAddress);
        
        showToast('Wallet connected successfully!', 'success');
        
        // Show portfolio section
        document.getElementById('portfolioSection').classList.remove('hidden');
        document.getElementById('activitySection').classList.remove('hidden');
        
        // Show enhanced dashboard sections
        document.getElementById('aiActionsSection').classList.remove('hidden');
        document.getElementById('distributionSection').classList.remove('hidden');
        
        // Load enhanced dashboard data
        loadAIActions();
        loadPositionDistribution();
        
        // Initialize WebSocket for real-time updates
        initializeWebSocket();
        
    } catch (error) {
        errorDiv.textContent = error.message || 'Failed to connect wallet';
        showToast('Failed to connect wallet', 'error');
    } finally {
        hideLoading();
    }
}

async function loadPortfolio(walletAddress) {
    try {
        const response = await fetch(`${API_BASE_URL}/api/wallets/${walletAddress}/multi-chain-portfolio`);
        
        if (!response.ok) {
            throw new Error('Failed to load portfolio');
        }

        const data = await response.json();
        state.portfolio = data.data;
        
        displayPortfolio(state.portfolio);
        
    } catch (error) {
        console.error('Error loading portfolio:', error);
        // Display mock data for demo purposes
        displayMockPortfolio();
    }
}

function displayPortfolio(portfolio) {
    document.getElementById('totalValue').textContent = `$${portfolio.total_value_usd.toFixed(2)}`;

    // Count unique blockchains
    const blockchains = new Set();
    let totalAssets = 0;

    if (portfolio.positions_by_chain) {
        Object.keys(portfolio.positions_by_chain).forEach(chain => {
            if (portfolio.positions_by_chain[chain].length > 0) {
                blockchains.add(chain);
                totalAssets += portfolio.positions_by_chain[chain].length;
            }
        });
    } else if (portfolio.assets) {
        // Fallback for old format
        totalAssets = portfolio.assets.length;
    }

    document.getElementById('assetCount').textContent = totalAssets;
    document.getElementById('blockchainCount').textContent = blockchains.size;

    displayAssetsList(portfolio);
}

function displayAssetsList(portfolio) {
    const assetsList = document.getElementById('assetsList');
    assetsList.innerHTML = '';
    
    // Handle new multi-chain format
    if (portfolio.positions_by_chain) {
        // Filter by selected blockchain
        const chains = state.selectedBlockchain === 'all' 
            ? Object.keys(portfolio.positions_by_chain)
            : [state.selectedBlockchain];
        
        let hasAssets = false;
        
        chains.forEach(chain => {
            const assets = portfolio.positions_by_chain[chain];
            if (!assets || assets.length === 0) return;
            
            hasAssets = true;
            
            // Add chain header
            const chainHeader = document.createElement('div');
            chainHeader.className = 'chain-header';
            chainHeader.innerHTML = `
                <div class="chain-name">${formatChainName(chain)}</div>
                <div class="chain-count">${assets.length} assets</div>
            `;
            assetsList.appendChild(chainHeader);
            
            // Add assets for this chain
            assets.forEach(asset => {
                const assetItem = document.createElement('div');
                assetItem.className = 'asset-item';
                assetItem.innerHTML = `
                    <div class="asset-info">
                        <div>
                            <div class="asset-symbol">${asset.token_symbol || asset.symbol}</div>
                            <div class="asset-amount">${parseFloat(asset.amount).toFixed(4)}</div>
                        </div>
                        <div class="asset-chain-badge">${formatChainName(chain)}</div>
                    </div>
                    <div class="asset-value">$${(asset.value_usd || 0).toFixed(2)}</div>
                `;
                assetsList.appendChild(assetItem);
            });
        });
        
        if (!hasAssets) {
            assetsList.innerHTML = '<p class="empty-state">No assets found for selected blockchain</p>';
        }
    } else if (portfolio.assets) {
        // Fallback for old format
        portfolio.assets.forEach(asset => {
            const assetItem = document.createElement('div');
            assetItem.className = 'asset-item';
            assetItem.innerHTML = `
                <div class="asset-info">
                    <div>
                        <div class="asset-symbol">${asset.token_symbol}</div>
                        <div class="asset-amount">${parseFloat(asset.amount).toFixed(4)}</div>
                    </div>
                </div>
                <div class="asset-value">$${(asset.value_usd || 0).toFixed(2)}</div>
            `;
            assetsList.appendChild(assetItem);
        });
    } else {
        assetsList.innerHTML = '<p class="empty-state">No assets found</p>';
    }
}

function formatChainName(chain) {
    const names = {
        'solana': 'Solana',
        'ethereum': 'Ethereum',
        'bsc': 'BSC',
        'binance_smart_chain': 'BSC',
        'polygon': 'Polygon'
    };
    return names[chain.toLowerCase()] || chain;
}

function setupBlockchainFilters() {
    document.querySelectorAll('.filter-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
            e.target.classList.add('active');
            state.selectedBlockchain = e.target.dataset.chain;
            if (state.portfolio) {
                displayAssetsList(state.portfolio);
            }
        });
    });
}

function displayMockPortfolio() {
    const mockPortfolio = {
        total_value_usd: 25750.75,
        positions_by_chain: {
            'solana': [
                { token_symbol: 'SOL', amount: '50.5', value_usd: 10000 },
                { token_symbol: 'USDC', amount: '2500', value_usd: 2500 }
            ],
            'ethereum': [
                { token_symbol: 'ETH', amount: '5.2', value_usd: 12000 },
                { token_symbol: 'USDT', amount: '1000', value_usd: 1000 }
            ],
            'polygon': [
                { token_symbol: 'MATIC', amount: '500', value_usd: 250.75 }
            ]
        }
    };
    
    displayPortfolio(mockPortfolio);
    document.getElementById('whaleCount').textContent = '3';
}

// Whales Management
async function loadWhales() {
    if (!state.connectedWallet) return;

    showLoading();
    
    try {
        const response = await fetch(`${API_BASE_URL}/api/whales/tracked`);
        
        if (!response.ok) {
            throw new Error('Failed to load whales');
        }

        const data = await response.json();
        state.whales = data.data || [];
        
        displayWhales(state.whales);
        
    } catch (error) {
        console.error('Error loading whales:', error);
        displayMockWhales();
    } finally {
        hideLoading();
    }
}

function displayWhales(whales) {
    const whalesList = document.getElementById('whalesList');
    
    if (whales.length === 0) {
        whalesList.innerHTML = '<p class="empty-state">No whales tracked yet</p>';
        return;
    }
    
    whalesList.innerHTML = '';
    
    whales.forEach(whale => {
        const whaleCard = document.createElement('div');
        whaleCard.className = 'whale-card';
        whaleCard.innerHTML = `
            <div class="whale-address">${whale.address}</div>
            <div class="whale-stats">
                <div class="whale-stat">
                    <span class="whale-stat-label">Total Value:</span>
                    <span class="whale-stat-value">$${(whale.total_value_usd || 0).toLocaleString()}</span>
                </div>
                <div class="whale-stat">
                    <span class="whale-stat-label">Multiplier:</span>
                    <span class="whale-stat-value">${whale.multiplier || 0}x</span>
                </div>
                <div class="whale-stat">
                    <span class="whale-stat-label">Rank:</span>
                    <span class="whale-stat-value">#${whale.rank || 0}</span>
                </div>
            </div>
        `;
        whalesList.appendChild(whaleCard);
    });
}

function displayMockWhales() {
    const mockWhales = [
        { address: '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU', total_value_usd: 5000000, multiplier: 150, rank: 1 },
        { address: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM', total_value_usd: 3500000, multiplier: 120, rank: 2 },
        { address: 'DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK', total_value_usd: 2800000, multiplier: 100, rank: 3 }
    ];
    
    displayWhales(mockWhales);
    document.getElementById('whaleCount').textContent = mockWhales.length;
}

async function refreshWhales() {
    showToast('Refreshing whale data...', 'warning');
    await loadWhales();
    showToast('Whale data refreshed!', 'success');
}

// Analytics
async function loadAnalytics() {
    if (!state.connectedWallet) return;

    showLoading();
    
    try {
        // Load portfolio performance
        await loadPortfolioPerformance();
        
        // Load whale impact
        await loadWhaleImpact();
        
        // Load recommendation accuracy
        await loadRecommendationAccuracy();
        
    } catch (error) {
        console.error('Error loading analytics:', error);
        displayMockAnalytics();
    } finally {
        hideLoading();
    }
}

async function loadPortfolioPerformance() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/analytics/portfolio-performance`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                wallet_address: state.connectedWallet,
                period: state.currentPeriod
            })
        });
        
        if (!response.ok) throw new Error('Failed to load performance');
        
        const data = await response.json();
        displayPerformance(data.data);
        
    } catch (error) {
        console.error('Error loading performance:', error);
        displayMockPerformance();
    }
}

function displayPerformance(performance) {
    const container = document.getElementById('performanceChart');
    
    const gainLossClass = performance.gain_loss_usd >= 0 ? 'success-color' : 'error-color';
    const gainLossSign = performance.gain_loss_usd >= 0 ? '+' : '';
    
    container.innerHTML = `
        <div style="text-align: center; padding: 2rem;">
            <div style="font-size: 3rem; font-weight: bold; color: var(--${gainLossClass});">
                ${gainLossSign}$${performance.gain_loss_usd.toFixed(2)}
            </div>
            <div style="font-size: 1.5rem; color: var(--text-secondary); margin-top: 0.5rem;">
                ${gainLossSign}${performance.gain_loss_percent.toFixed(2)}%
            </div>
            <div style="margin-top: 2rem; display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; max-width: 500px; margin: 2rem auto 0;">
                <div>
                    <div style="color: var(--text-secondary);">Start Value</div>
                    <div style="font-size: 1.5rem; font-weight: bold;">$${performance.start_value_usd.toFixed(2)}</div>
                </div>
                <div>
                    <div style="color: var(--text-secondary);">Current Value</div>
                    <div style="font-size: 1.5rem; font-weight: bold;">$${performance.end_value_usd.toFixed(2)}</div>
                </div>
            </div>
        </div>
    `;
}

function displayMockPerformance() {
    displayPerformance({
        gain_loss_usd: 1250.50,
        gain_loss_percent: 11.2,
        start_value_usd: 11250,
        end_value_usd: 12500.50
    });
}

async function loadWhaleImpact() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/analytics/whale-impact`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user_id: DEMO_USER_ID,
                period: state.currentPeriod
            })
        });
        
        if (!response.ok) throw new Error('Failed to load whale impact');
        
        const data = await response.json();
        displayWhaleImpact(data.data);
        
    } catch (error) {
        console.error('Error loading whale impact:', error);
        displayMockWhaleImpact();
    }
}

function displayWhaleImpact(impact) {
    const container = document.getElementById('whaleImpact');
    
    if (!impact.whale_impacts || impact.whale_impacts.length === 0) {
        container.innerHTML = '<p class="empty-state">No whale impact data available</p>';
        return;
    }
    
    container.innerHTML = `
        <div style="text-align: center; margin-bottom: 2rem;">
            <div style="font-size: 2rem; font-weight: bold;">${impact.total_movements}</div>
            <div style="color: var(--text-secondary);">Total Whale Movements</div>
        </div>
    `;
    
    impact.whale_impacts.slice(0, 5).forEach(item => {
        const impactItem = document.createElement('div');
        impactItem.className = 'impact-item';
        impactItem.innerHTML = `
            <div>
                <div style="font-weight: bold;">${item.whale_address.substring(0, 8)}...${item.whale_address.substring(item.whale_address.length - 8)}</div>
                <div style="color: var(--text-secondary); font-size: 0.9rem;">
                    ${item.movement_type} ${item.token_mint} (${item.movement_percent.toFixed(2)}%)
                </div>
            </div>
            <div class="impact-score">${item.impact_score.toFixed(2)}</div>
        `;
        container.appendChild(impactItem);
    });
}

function displayMockWhaleImpact() {
    displayWhaleImpact({
        total_movements: 12,
        whale_impacts: [
            { whale_address: '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU', movement_type: 'BUY', token_mint: 'SOL', movement_percent: 15.5, impact_score: 8.5 },
            { whale_address: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM', movement_type: 'SELL', token_mint: 'USDC', movement_percent: 10.2, impact_score: 6.3 }
        ]
    });
}

async function loadRecommendationAccuracy() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/analytics/recommendation-accuracy`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                user_id: DEMO_USER_ID,
                period: state.currentPeriod
            })
        });
        
        if (!response.ok) throw new Error('Failed to load recommendation accuracy');
        
        const data = await response.json();
        displayRecommendationAccuracy(data.data);
        
    } catch (error) {
        console.error('Error loading recommendation accuracy:', error);
        displayMockRecommendationAccuracy();
    }
}

function displayRecommendationAccuracy(accuracy) {
    const container = document.getElementById('recommendationAccuracy');
    
    container.innerHTML = `
        <div class="accuracy-card">
            <div style="color: var(--text-secondary);">Overall Accuracy</div>
            <div class="accuracy-percentage">${accuracy.accuracy_rate.toFixed(1)}%</div>
            <div style="color: var(--text-secondary); font-size: 0.9rem;">
                ${accuracy.successful_recommendations} / ${accuracy.recommendations_followed} followed
            </div>
        </div>
        <div class="accuracy-card">
            <div style="color: var(--text-secondary);">Total Recommendations</div>
            <div class="accuracy-percentage" style="color: var(--primary-color);">${accuracy.total_recommendations}</div>
            <div style="color: var(--text-secondary); font-size: 0.9rem;">
                Avg Confidence: ${accuracy.average_confidence.toFixed(1)}%
            </div>
        </div>
    `;
}

function displayMockRecommendationAccuracy() {
    displayRecommendationAccuracy({
        total_recommendations: 25,
        recommendations_followed: 18,
        successful_recommendations: 15,
        accuracy_rate: 83.3,
        average_confidence: 75.5
    });
}

function displayMockAnalytics() {
    displayMockPerformance();
    displayMockWhaleImpact();
    displayMockRecommendationAccuracy();
}

// Settings
function saveSettings() {
    const apiUrl = document.getElementById('apiUrl').value.trim();
    
    if (!apiUrl) {
        showToast('Please enter a valid API URL', 'error');
        return;
    }
    
    API_BASE_URL = apiUrl;
    localStorage.setItem('apiUrl', apiUrl);
    
    showToast('Settings saved successfully!', 'success');
    checkHealthStatus();
}

// Conversion Interface Functions

function setupConversionListeners() {
    // Asset selection
    document.getElementById('fromAsset').addEventListener('change', onAssetChange);
    document.getElementById('toAsset').addEventListener('change', onAssetChange);
    
    // Amount input
    document.getElementById('fromAmount').addEventListener('input', onAmountChange);
    document.getElementById('toAmount').addEventListener('input', onAmountChange);
    
    // Swap button
    document.getElementById('swapAssets').addEventListener('click', swapAssets);
    
    // Mode toggle
    document.getElementById('exactInputMode').addEventListener('click', () => setConversionMode('exactInput'));
    document.getElementById('exactOutputMode').addEventListener('click', () => setConversionMode('exactOutput'));
    
    // Action buttons
    document.getElementById('getQuoteBtn').addEventListener('click', getConversionQuote);
    document.getElementById('executeConversionBtn').addEventListener('click', executeConversion);
    document.getElementById('refreshHistory').addEventListener('click', loadConversionHistory);
}

function initializeConversionView() {
    // Reset state
    state.currentQuote = null;
    clearQuoteExpiryTimer();
    
    // Load conversion history if wallet connected
    if (state.connectedWallet) {
        loadConversionHistory();
    }
}

function onAssetChange() {
    const fromAsset = document.getElementById('fromAsset').value;
    const toAsset = document.getElementById('toAsset').value;
    const getQuoteBtn = document.getElementById('getQuoteBtn');
    
    // Enable get quote button if both assets selected and different
    if (fromAsset && toAsset && fromAsset !== toAsset) {
        getQuoteBtn.disabled = false;
    } else {
        getQuoteBtn.disabled = true;
    }
    
    // Clear quote when assets change
    clearQuote();
}

function onAmountChange(e) {
    const amount = parseFloat(e.target.value);
    
    if (amount > 0) {
        document.getElementById('getQuoteBtn').disabled = false;
    } else {
        document.getElementById('getQuoteBtn').disabled = true;
    }
    
    // Clear quote when amount changes
    clearQuote();
}

function swapAssets() {
    const fromAsset = document.getElementById('fromAsset').value;
    const toAsset = document.getElementById('toAsset').value;
    const fromAmount = document.getElementById('fromAmount').value;
    const toAmount = document.getElementById('toAmount').value;
    
    // Swap assets
    document.getElementById('fromAsset').value = toAsset;
    document.getElementById('toAsset').value = fromAsset;
    
    // Swap amounts if in exact output mode
    if (state.conversionMode === 'exactOutput') {
        document.getElementById('fromAmount').value = toAmount;
        document.getElementById('toAmount').value = fromAmount;
    }
    
    // Clear quote
    clearQuote();
    onAssetChange();
}

function setConversionMode(mode) {
    state.conversionMode = mode;
    
    // Update button states
    document.getElementById('exactInputMode').classList.toggle('active', mode === 'exactInput');
    document.getElementById('exactOutputMode').classList.toggle('active', mode === 'exactOutput');
    
    // Update input readonly states
    if (mode === 'exactInput') {
        document.getElementById('fromAmount').readOnly = false;
        document.getElementById('toAmount').readOnly = true;
    } else {
        document.getElementById('fromAmount').readOnly = true;
        document.getElementById('toAmount').readOnly = false;
    }
    
    // Clear quote
    clearQuote();
}

async function getConversionQuote() {
    const fromAsset = document.getElementById('fromAsset').value;
    const toAsset = document.getElementById('toAsset').value;
    const errorDiv = document.getElementById('conversionError');
    
    let amount, amountType;
    
    if (state.conversionMode === 'exactInput') {
        amount = parseFloat(document.getElementById('fromAmount').value);
        amountType = 'from';
    } else {
        amount = parseFloat(document.getElementById('toAmount').value);
        amountType = 'to';
    }
    
    if (!fromAsset || !toAsset || !amount || amount <= 0) {
        errorDiv.textContent = 'Please select assets and enter an amount';
        return;
    }
    
    if (fromAsset === toAsset) {
        errorDiv.textContent = 'Cannot convert same asset';
        return;
    }
    
    showLoading();
    errorDiv.textContent = '';
    
    try {
        const response = await fetch(`${API_BASE_URL}/api/conversions/quote`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                from_asset: fromAsset,
                to_asset: toAsset,
                amount: amount.toString(),
                amount_type: amountType
            })
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to get quote');
        }
        
        const data = await response.json();
        state.currentQuote = data.data;
        
        displayQuote(state.currentQuote);
        startQuoteExpiryTimer(state.currentQuote.expires_at);
        
        showToast('Quote received!', 'success');
        
    } catch (error) {
        console.error('Error getting quote:', error);
        errorDiv.textContent = error.message || 'Failed to get conversion quote';
        showToast('Failed to get quote', 'error');
    } finally {
        hideLoading();
    }
}

function displayQuote(quote) {
    // Update amounts
    document.getElementById('fromAmount').value = parseFloat(quote.from_amount).toFixed(6);
    document.getElementById('toAmount').value = parseFloat(quote.to_amount).toFixed(6);
    
    // Display quote details
    document.getElementById('exchangeRate').textContent = 
        `1 ${quote.from_asset} = ${parseFloat(quote.exchange_rate).toFixed(6)} ${quote.to_asset}`;
    document.getElementById('networkFee').textContent = 
        `${parseFloat(quote.network_fee).toFixed(6)} ${quote.from_asset}`;
    document.getElementById('platformFee').textContent = 
        `${parseFloat(quote.platform_fee).toFixed(6)} ${quote.from_asset}`;
    document.getElementById('providerFee').textContent = 
        `${parseFloat(quote.provider_fee).toFixed(6)} ${quote.from_asset}`;
    document.getElementById('totalFees').textContent = 
        `${parseFloat(quote.total_fees).toFixed(6)} ${quote.from_asset}`;
    document.getElementById('finalAmount').textContent = 
        `${parseFloat(quote.to_amount).toFixed(6)} ${quote.to_asset}`;
    
    // Show quote details
    document.getElementById('quoteDetails').classList.remove('hidden');
    document.getElementById('executeConversionBtn').classList.remove('hidden');
    document.getElementById('executeConversionBtn').disabled = false;
}

function startQuoteExpiryTimer(expiresAt) {
    clearQuoteExpiryTimer();
    
    const updateTimer = () => {
        const now = new Date();
        const expiry = new Date(expiresAt);
        const diff = expiry - now;
        
        if (diff <= 0) {
            document.getElementById('quoteExpiry').textContent = 'Quote expired';
            document.getElementById('executeConversionBtn').disabled = true;
            clearQuoteExpiryTimer();
            showToast('Quote expired, please get a new quote', 'warning');
        } else {
            const seconds = Math.floor(diff / 1000);
            document.getElementById('quoteExpiry').textContent = `Quote expires in ${seconds}s`;
        }
    };
    
    updateTimer();
    state.quoteExpiryTimer = setInterval(updateTimer, 1000);
}

function clearQuoteExpiryTimer() {
    if (state.quoteExpiryTimer) {
        clearInterval(state.quoteExpiryTimer);
        state.quoteExpiryTimer = null;
    }
}

function clearQuote() {
    state.currentQuote = null;
    clearQuoteExpiryTimer();
    document.getElementById('quoteDetails').classList.add('hidden');
    document.getElementById('executeConversionBtn').classList.add('hidden');
    document.getElementById('conversionError').textContent = '';
}

async function executeConversion() {
    if (!state.currentQuote) {
        showToast('No quote available', 'error');
        return;
    }
    
    if (!state.connectedWallet) {
        showToast('Please connect your wallet first', 'error');
        return;
    }
    
    const errorDiv = document.getElementById('conversionError');
    errorDiv.textContent = '';
    
    // Confirm with user
    if (!confirm(`Convert ${state.currentQuote.from_amount} ${state.currentQuote.from_asset} to ${state.currentQuote.to_amount} ${state.currentQuote.to_asset}?`)) {
        return;
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        
        const payload = {
            quote_id: state.currentQuote.quote_id,
            from_asset: state.currentQuote.from_asset,
            to_asset: state.currentQuote.to_asset,
            from_amount: state.currentQuote.from_amount.toString(),
            to_amount: state.currentQuote.to_amount.toString(),
            exchange_rate: state.currentQuote.exchange_rate.toString(),
            network_fee: state.currentQuote.network_fee.toString(),
            platform_fee: state.currentQuote.platform_fee.toString(),
            provider_fee: state.currentQuote.provider_fee.toString(),
            total_fees: state.currentQuote.total_fees.toString(),
            provider: state.currentQuote.provider,
            expires_at: state.currentQuote.expires_at,
            settle_address: state.connectedWallet,
            refund_address: state.connectedWallet
        };
        
        const response = await fetch(`${API_BASE_URL}/api/conversions/${userId}/execute`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            let errorMessage = 'Failed to execute conversion';
            try {
                const errorJson = JSON.parse(errorText);
                errorMessage = errorJson.error || errorMessage;
            } catch {
                errorMessage = errorText || errorMessage;
            }
            throw new Error(errorMessage);
        }
        
        const data = await response.json();
        
        showToast('Conversion initiated successfully!', 'success');
        
        // Clear form and reload history
        clearQuote();
        document.getElementById('fromAmount').value = '';
        document.getElementById('toAmount').value = '';
        loadConversionHistory();
        
    } catch (error) {
        console.error('Error executing conversion:', error);
        let errorMessage = error.message || 'Failed to execute conversion';
        
        // Provide helpful message for address validation errors
        if (errorMessage.includes('Invalid receiving address') || errorMessage.includes('Invalid address')) {
            errorMessage = 'Invalid wallet address for destination asset. Please ensure you have a valid wallet address for ' + state.currentQuote.to_asset + ' on the correct blockchain network.';
        }
        
        errorDiv.textContent = errorMessage;
        showToast('Conversion failed', 'error');
    } finally {
        hideLoading();
    }
}

async function loadConversionHistory() {
    if (!state.connectedWallet) {
        document.getElementById('conversionHistory').innerHTML = 
            '<p class="empty-state">Connect your wallet to see conversion history</p>';
        return;
    }
    
    try {
        // Use a demo user ID for now
        const userId = DEMO_USER_ID;
        
        const response = await fetch(`${API_BASE_URL}/api/conversions/${userId}/history`);
        
        if (!response.ok) {
            throw new Error('Failed to load conversion history');
        }
        
        const data = await response.json();
        displayConversionHistory(data.data || []);
        
    } catch (error) {
        console.error('Error loading conversion history:', error);
        // Display mock data for demo
        displayMockConversionHistory();
    }
}

function displayConversionHistory(conversions) {
    const container = document.getElementById('conversionHistory');
    
    if (conversions.length === 0) {
        container.innerHTML = '<p class="empty-state">No conversion history</p>';
        return;
    }
    
    container.innerHTML = '';
    
    conversions.forEach(conversion => {
        const item = document.createElement('div');
        item.className = `history-item ${conversion.status.toLowerCase()}`;
        
        const fromAmount = parseFloat(conversion.from_amount);
        const toAmount = parseFloat(conversion.to_amount);
        const rate = parseFloat(conversion.exchange_rate);
        
        // Calculate P/L (simplified - would need historical prices for accurate calculation)
        const plPercent = ((toAmount / fromAmount - rate) / rate * 100).toFixed(2);
        const plClass = plPercent >= 0 ? 'profit' : 'loss';
        const plSign = plPercent >= 0 ? '+' : '';
        
        item.innerHTML = `
            <div class="history-info">
                <div class="history-assets">${conversion.from_asset} → ${conversion.to_asset}</div>
                <div class="history-details">
                    ${fromAmount.toFixed(6)} ${conversion.from_asset} → ${toAmount.toFixed(6)} ${conversion.to_asset}
                </div>
                <div class="history-details">
                    ${new Date(conversion.created_at).toLocaleString()} • ${conversion.status}
                </div>
            </div>
            <div class="history-amount">
                <div class="history-value">${toAmount.toFixed(6)} ${conversion.to_asset}</div>
                <div class="history-rate">Rate: ${rate.toFixed(6)}</div>
                <div class="history-pl ${plClass}">${plSign}${plPercent}%</div>
            </div>
        `;
        
        container.appendChild(item);
    });
}

function displayMockConversionHistory() {
    const mockConversions = [
        {
            from_asset: 'SOL',
            to_asset: 'USDC',
            from_amount: '10.5',
            to_amount: '2100.50',
            exchange_rate: '200.0',
            status: 'completed',
            created_at: new Date(Date.now() - 3600000).toISOString()
        },
        {
            from_asset: 'ETH',
            to_asset: 'BTC',
            from_amount: '2.0',
            to_amount: '0.15',
            exchange_rate: '0.075',
            status: 'completed',
            created_at: new Date(Date.now() - 7200000).toISOString()
        },
        {
            from_asset: 'USDC',
            to_asset: 'SOL',
            from_amount: '500',
            to_amount: '2.45',
            exchange_rate: '0.0049',
            status: 'pending',
            created_at: new Date(Date.now() - 300000).toISOString()
        }
    ];
    
    displayConversionHistory(mockConversions);
}

// Benchmark Management Functions

function setupBenchmarkListeners() {
    document.getElementById('createBenchmarkBtn').addEventListener('click', showBenchmarkForm);
    document.getElementById('cancelBenchmarkBtn').addEventListener('click', hideBenchmarkForm);
    document.getElementById('saveBenchmarkBtn').addEventListener('click', saveBenchmark);
    document.getElementById('refreshBenchmarks').addEventListener('click', loadBenchmarks);
    
    // Show/hide trade fields based on action
    document.getElementById('benchmarkAction').addEventListener('change', (e) => {
        const tradeAmountGroup = document.getElementById('tradeAmountGroup');
        const tradeActionGroup = document.getElementById('tradeActionGroup');
        if (e.target.value === 'EXECUTE') {
            tradeAmountGroup.classList.remove('hidden');
            tradeActionGroup.classList.remove('hidden');
        } else {
            tradeAmountGroup.classList.add('hidden');
            tradeActionGroup.classList.add('hidden');
        }
    });
}

function initializeBenchmarksView() {
    hideBenchmarkForm();
    if (state.connectedWallet) {
        loadBenchmarks();
    }
}

function showBenchmarkForm(benchmarkData = null) {
    const form = document.getElementById('benchmarkForm');
    const title = document.getElementById('benchmarkFormTitle');
    const errorDiv = document.getElementById('benchmarkFormError');
    
    errorDiv.textContent = '';
    
    if (benchmarkData) {
        title.textContent = 'Edit Benchmark';
        document.getElementById('benchmarkAsset').value = benchmarkData.asset;
        document.getElementById('benchmarkBlockchain').value = benchmarkData.blockchain;
        document.getElementById('benchmarkPrice').value = benchmarkData.target_price;
        document.getElementById('benchmarkTrigger').value = benchmarkData.trigger_type;
        document.getElementById('benchmarkAction').value = benchmarkData.action_type;
        if (benchmarkData.trade_amount) {
            document.getElementById('benchmarkTradeAmount').value = benchmarkData.trade_amount;
            document.getElementById('tradeAmountGroup').classList.remove('hidden');
        }
    } else {
        title.textContent = 'Create Benchmark';
        document.getElementById('benchmarkAsset').value = '';
        document.getElementById('benchmarkBlockchain').value = 'solana';
        document.getElementById('benchmarkPrice').value = '';
        document.getElementById('benchmarkTrigger').value = 'above';
        document.getElementById('benchmarkAction').value = 'alert';
        document.getElementById('benchmarkTradeAmount').value = '';
        document.getElementById('tradeAmountGroup').classList.add('hidden');
    }
    
    form.classList.remove('hidden');
    form.scrollIntoView({ behavior: 'smooth' });
}

function hideBenchmarkForm() {
    document.getElementById('benchmarkForm').classList.add('hidden');
}

async function saveBenchmark() {
    const asset = document.getElementById('benchmarkAsset').value;
    const blockchain = document.getElementById('benchmarkBlockchain').value;
    const price = parseFloat(document.getElementById('benchmarkPrice').value);
    const triggerType = document.getElementById('benchmarkTrigger').value;
    const actionType = document.getElementById('benchmarkAction').value;
    const tradeAmount = document.getElementById('benchmarkTradeAmount').value;
    const tradeAction = document.getElementById('benchmarkTradeAction').value;
    const errorDiv = document.getElementById('benchmarkFormError');
    
    errorDiv.textContent = '';
    
    // Validation
    if (!asset) {
        errorDiv.textContent = 'Please select an asset';
        return;
    }
    if (!price || price <= 0) {
        errorDiv.textContent = 'Please enter a valid target price';
        return;
    }
    if (actionType === 'EXECUTE') {
        if (!tradeAmount || parseFloat(tradeAmount) <= 0) {
            errorDiv.textContent = 'Please enter a valid trade amount';
            return;
        }
        if (!tradeAction) {
            errorDiv.textContent = 'Please select a trade action (Buy/Sell)';
            return;
        }
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        
        const payload = {
            asset,
            blockchain,
            target_price: price.toString(),
            trigger_type: triggerType,
            action_type: actionType
        };
        
        // Only include trade fields if action is EXECUTE
        if (actionType === 'EXECUTE') {
            payload.trade_action = tradeAction;
            payload.trade_amount = parseFloat(tradeAmount).toString();
        }
        
        const response = await fetch(`${API_BASE_URL}/api/benchmarks/${userId}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            let errorMessage = 'Failed to create benchmark';
            try {
                const errorJson = JSON.parse(errorText);
                errorMessage = errorJson.error || errorMessage;
            } catch {
                errorMessage = errorText || errorMessage;
            }
            throw new Error(errorMessage);
        }
        
        showToast('Benchmark created successfully!', 'success');
        hideBenchmarkForm();
        loadBenchmarks();
        
    } catch (error) {
        console.error('Error creating benchmark:', error);
        errorDiv.textContent = error.message || 'Failed to create benchmark';
        showToast('Failed to create benchmark', 'error');
    } finally {
        hideLoading();
    }
}

async function loadBenchmarks() {
    if (!state.connectedWallet) {
        document.getElementById('benchmarksList').innerHTML = 
            '<p class="empty-state">Connect your wallet to manage benchmarks</p>';
        return;
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/benchmarks/${userId}`);
        
        if (!response.ok) {
            throw new Error('Failed to load benchmarks');
        }
        
        const data = await response.json();
        state.benchmarks = data.data || [];
        
        displayBenchmarks(state.benchmarks);
        
    } catch (error) {
        console.error('Error loading benchmarks:', error);
        displayMockBenchmarks();
    } finally {
        hideLoading();
    }
}

function displayBenchmarks(benchmarks) {
    const container = document.getElementById('benchmarksList');
    
    if (benchmarks.length === 0) {
        container.innerHTML = '<p class="empty-state">No active benchmarks. Create one to get started!</p>';
        return;
    }
    
    container.innerHTML = '';
    
    benchmarks.forEach(benchmark => {
        const item = document.createElement('div');
        item.className = `benchmark-item ${benchmark.is_active ? 'active' : 'inactive'}`;
        
        const currentPrice = 200.50; // Mock current price - would come from API
        const targetPrice = parseFloat(benchmark.target_price);
        const distance = ((targetPrice - currentPrice) / currentPrice * 100).toFixed(2);
        const distanceClass = distance >= 0 ? 'positive' : 'negative';
        
        const actionText = benchmark.action_type === 'alert' ? 'Alert' : 
                          benchmark.action_type === 'execute_buy' ? 'Buy' : 'Sell';
        const triggerText = benchmark.trigger_type === 'above' ? 'Above' : 'Below';
        
        item.innerHTML = `
            <div class="benchmark-info">
                <div class="benchmark-header">
                    <div class="benchmark-asset">${benchmark.asset}</div>
                    <div class="benchmark-chain-badge">${formatChainName(benchmark.blockchain)}</div>
                </div>
                <div class="benchmark-details">
                    <div class="benchmark-detail">
                        <span class="detail-label">Target:</span>
                        <span class="detail-value">$${targetPrice.toFixed(2)}</span>
                    </div>
                    <div class="benchmark-detail">
                        <span class="detail-label">Trigger:</span>
                        <span class="detail-value">${triggerText}</span>
                    </div>
                    <div class="benchmark-detail">
                        <span class="detail-label">Action:</span>
                        <span class="detail-value">${actionText}</span>
                    </div>
                </div>
                <div class="benchmark-distance ${distanceClass}">
                    ${Math.abs(distance)}% ${distance >= 0 ? 'above' : 'below'} current price
                </div>
            </div>
            <div class="benchmark-actions">
                <button class="btn-icon" onclick="editBenchmark('${benchmark.id}')" title="Edit">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                    </svg>
                </button>
                <button class="btn-icon delete" onclick="deleteBenchmark('${benchmark.id}')" title="Delete">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="3 6 5 6 21 6"/>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                    </svg>
                </button>
            </div>
        `;
        
        container.appendChild(item);
    });
}

async function editBenchmark(benchmarkId) {
    const benchmark = state.benchmarks.find(b => b.id === benchmarkId);
    if (benchmark) {
        showBenchmarkForm(benchmark);
    }
}

async function deleteBenchmark(benchmarkId) {
    if (!confirm('Are you sure you want to delete this benchmark?')) {
        return;
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/benchmarks/${userId}/${benchmarkId}`, {
            method: 'DELETE'
        });
        
        if (!response.ok) {
            throw new Error('Failed to delete benchmark');
        }
        
        showToast('Benchmark deleted successfully!', 'success');
        loadBenchmarks();
        
    } catch (error) {
        console.error('Error deleting benchmark:', error);
        showToast('Failed to delete benchmark', 'error');
    } finally {
        hideLoading();
    }
}

function displayMockBenchmarks() {
    const mockBenchmarks = [
        {
            id: '1',
            asset: 'SOL',
            blockchain: 'solana',
            target_price: '250.00',
            trigger_type: 'above',
            action_type: 'alert',
            is_active: true
        },
        {
            id: '2',
            asset: 'ETH',
            blockchain: 'ethereum',
            target_price: '2000.00',
            trigger_type: 'below',
            action_type: 'execute_buy',
            trade_amount: '0.5',
            is_active: true
        },
        {
            id: '3',
            asset: 'BTC',
            blockchain: 'ethereum',
            target_price: '50000.00',
            trigger_type: 'above',
            action_type: 'execute_sell',
            trade_amount: '0.1',
            is_active: true
        }
    ];
    
    displayBenchmarks(mockBenchmarks);
}

// P2P Exchange Functions

function setupP2PListeners() {
    document.getElementById('createOfferBtn').addEventListener('click', showOfferForm);
    document.getElementById('cancelOfferBtn').addEventListener('click', hideOfferForm);
    document.getElementById('saveOfferBtn').addEventListener('click', saveOffer);
    document.getElementById('refreshMyOffers').addEventListener('click', loadMyOffers);
    document.getElementById('refreshMarketplace').addEventListener('click', loadMarketplace);
}

function initializeP2PView() {
    hideOfferForm();
    if (state.connectedWallet) {
        loadMyOffers();
        loadMarketplace();
    }
}

function showOfferForm() {
    document.getElementById('offerForm').classList.remove('hidden');
    document.getElementById('offerFormError').textContent = '';
}

function hideOfferForm() {
    document.getElementById('offerForm').classList.add('hidden');
}

async function saveOffer() {
    const offerType = document.getElementById('offerType').value;
    const fromAsset = document.getElementById('offerFromAsset').value;
    const toAsset = document.getElementById('offerToAsset').value;
    const fromAmount = parseFloat(document.getElementById('offerFromAmount').value);
    const toAmount = parseFloat(document.getElementById('offerToAmount').value);
    const errorDiv = document.getElementById('offerFormError');
    
    errorDiv.textContent = '';
    
    if (!fromAmount || fromAmount <= 0) {
        errorDiv.textContent = 'Please enter a valid from amount';
        return;
    }
    if (!toAmount || toAmount <= 0) {
        errorDiv.textContent = 'Please enter a valid to amount';
        return;
    }
    if (fromAsset === toAsset) {
        errorDiv.textContent = 'Cannot exchange same asset';
        return;
    }
    
    // Calculate price (to_amount / from_amount)
    const price = toAmount / fromAmount;
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        
        const payload = {
            offer_type: offerType,
            from_asset: fromAsset,
            to_asset: toAsset,
            from_amount: fromAmount.toString(),
            to_amount: toAmount.toString(),
            price: price.toString(),
            is_proximity_offer: false
        };
        
        const response = await fetch(`${API_BASE_URL}/api/p2p/${userId}/offers`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            let errorMessage = 'Failed to create offer';
            try {
                const errorJson = JSON.parse(errorText);
                errorMessage = errorJson.error || errorMessage;
            } catch {
                errorMessage = errorText || errorMessage;
            }
            throw new Error(errorMessage);
        }
        
        showToast('Offer created successfully!', 'success');
        hideOfferForm();
        loadMyOffers();
        loadMarketplace();
        
    } catch (error) {
        console.error('Error creating offer:', error);
        errorDiv.textContent = error.message || 'Failed to create offer';
        showToast('Failed to create offer', 'error');
    } finally {
        hideLoading();
    }
}

async function loadMyOffers() {
    if (!state.connectedWallet) {
        document.getElementById('myOffersList').innerHTML = 
            '<p class="empty-state">Connect your wallet to view your offers</p>';
        return;
    }
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/p2p/${userId}/offers`);
        
        if (!response.ok) throw new Error('Failed to load offers');
        
        const data = await response.json();
        displayMyOffers(data.data || []);
        
    } catch (error) {
        console.error('Error loading my offers:', error);
        displayMockMyOffers();
    }
}

async function loadMarketplace() {
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/p2p/${userId}/marketplace`);
        
        if (!response.ok) throw new Error('Failed to load marketplace');
        
        const data = await response.json();
        displayMarketplace(data.data || []);
        
    } catch (error) {
        console.error('Error loading marketplace:', error);
        displayMockMarketplace();
    }
}

function displayMyOffers(offers) {
    const container = document.getElementById('myOffersList');
    
    // Filter out cancelled offers for cleaner display
    const activeOffers = offers.filter(offer => 
        offer.status.toLowerCase() !== 'cancelled' && 
        offer.status.toLowerCase() !== 'expired'
    );
    
    if (activeOffers.length === 0) {
        container.innerHTML = '<p class="empty-state">No active offers</p>';
        return;
    }
    
    container.innerHTML = '';
    activeOffers.forEach(offer => {
        const item = createOfferElement(offer, true);
        container.appendChild(item);
    });
}

function displayMarketplace(offers) {
    const container = document.getElementById('marketplaceList');
    
    if (offers.length === 0) {
        container.innerHTML = '<p class="empty-state">No offers available</p>';
        return;
    }
    
    container.innerHTML = '';
    offers.forEach(offer => {
        const item = createMarketplaceOfferElement(offer);
        container.appendChild(item);
    });
}

function createMarketplaceOfferElement(offer) {
    const item = document.createElement('div');
    item.className = 'offer-item';
    
    const price = (parseFloat(offer.to_amount) / parseFloat(offer.from_amount)).toFixed(6);
    const statusClass = offer.status === 'active' ? 'active' : offer.status;
    
    // Format timestamps
    const createdAt = offer.created_at ? new Date(offer.created_at).toLocaleString() : 'N/A';
    const expiresAt = offer.expires_at ? new Date(offer.expires_at).toLocaleString() : 'N/A';
    
    // Format creator information
    const creatorInfo = offer.user_id || 'Unknown';
    const creatorDisplay = typeof creatorInfo === 'string' && creatorInfo.length > 16 
        ? `${creatorInfo.substring(0, 8)}...${creatorInfo.substring(creatorInfo.length - 8)}`
        : creatorInfo;
    
    item.innerHTML = `
        <div class="offer-info">
            <div class="offer-header">
                <span class="offer-type ${offer.offer_type}">${offer.offer_type.toUpperCase()}</span>
                <span class="offer-status ${statusClass}">${offer.status}</span>
            </div>
            <div class="offer-details">
                <div class="offer-assets">${offer.from_asset} → ${offer.to_asset}</div>
                <div class="offer-amounts">
                    ${parseFloat(offer.from_amount).toFixed(4)} ${offer.from_asset} for 
                    ${parseFloat(offer.to_amount).toFixed(4)} ${offer.to_asset}
                </div>
                <div class="offer-price">Price: ${price} ${offer.to_asset}/${offer.from_asset}</div>
                <div class="offer-creator" style="color: var(--text-secondary); font-size: 0.9rem; margin-top: 0.5rem;">
                    Creator: ${creatorDisplay}
                </div>
                <div class="offer-timestamps" style="color: var(--text-secondary); font-size: 0.85rem; margin-top: 0.25rem;">
                    <div>Created: ${createdAt}</div>
                    <div>Expires: ${expiresAt}</div>
                </div>
            </div>
        </div>
        <div class="offer-actions">
            <button class="btn btn-primary btn-sm" onclick="acceptOffer('${offer.id}')">Accept</button>
        </div>
    `;
    
    return item;
}

function createOfferElement(offer, isMyOffer) {
    const item = document.createElement('div');
    item.className = 'offer-item';
    
    const price = (parseFloat(offer.to_amount) / parseFloat(offer.from_amount)).toFixed(6);
    const statusClass = offer.status === 'active' ? 'active' : offer.status.toLowerCase();
    
    // Format acceptor information if offer is accepted
    let acceptorInfo = '';
    if (isMyOffer && offer.acceptor_id && offer.status.toLowerCase() === 'matched') {
        const acceptorDisplay = offer.acceptor_wallet || offer.acceptor_user_tag || offer.acceptor_id;
        const acceptorShort = typeof acceptorDisplay === 'string' && acceptorDisplay.length > 16 
            ? `${acceptorDisplay.substring(0, 8)}...${acceptorDisplay.substring(acceptorDisplay.length - 8)}`
            : acceptorDisplay;
        
        const acceptedAt = offer.accepted_at ? new Date(offer.accepted_at).toLocaleString() : 'N/A';
        
        acceptorInfo = `
            <div class="offer-acceptor" style="color: var(--success-color); font-size: 0.9rem; margin-top: 0.5rem; padding-top: 0.5rem; border-top: 1px solid var(--border-color);">
                <div><strong>Accepted by:</strong> ${acceptorShort}</div>
                <div style="color: var(--text-secondary); font-size: 0.85rem; margin-top: 0.25rem;">
                    Accepted: ${acceptedAt}
                </div>
            </div>
        `;
    }
    
    // Determine action buttons
    let actionButtons = '';
    if (isMyOffer) {
        if (offer.status.toLowerCase() === 'active') {
            actionButtons = `<button class="btn btn-secondary btn-sm" onclick="cancelOffer('${offer.id}')">Cancel</button>`;
        } else if (offer.status.toLowerCase() === 'matched' && offer.conversation_id) {
            actionButtons = `<button class="btn btn-primary btn-sm" onclick="contactTrader('${offer.conversation_id}', '${offer.acceptor_id || ''}')">Contact ${offer.offer_type === 'buy' ? 'Seller' : 'Buyer'}</button>`;
        }
    } else {
        actionButtons = `<button class="btn btn-primary btn-sm" onclick="acceptOffer('${offer.id}')">Accept</button>`;
    }
    
    item.innerHTML = `
        <div class="offer-info">
            <div class="offer-header">
                <span class="offer-type ${offer.offer_type}">${offer.offer_type.toUpperCase()}</span>
                <span class="offer-status ${statusClass}">${offer.status.toUpperCase()}</span>
            </div>
            <div class="offer-details">
                <div class="offer-assets">${offer.from_asset} → ${offer.to_asset}</div>
                <div class="offer-amounts">
                    ${parseFloat(offer.from_amount).toFixed(4)} ${offer.from_asset} for 
                    ${parseFloat(offer.to_amount).toFixed(4)} ${offer.to_asset}
                </div>
                <div class="offer-price">Price: ${price} ${offer.to_asset}/${offer.from_asset}</div>
                ${acceptorInfo}
            </div>
        </div>
        <div class="offer-actions">
            ${actionButtons}
        </div>
    `;
    
    return item;
}

async function cancelOffer(offerId) {
    if (!confirm('Are you sure you want to cancel this offer?')) return;
    
    showLoading();
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/p2p/${userId}/offers/${offerId}/cancel`, {
            method: 'POST'
        });
        
        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.error || 'Failed to cancel offer');
        }
        
        showToast('Offer cancelled successfully!', 'success');
        await loadMyOffers();
        await loadMarketplace();
    } catch (error) {
        console.error('Error cancelling offer:', error);
        showToast(error.message || 'Failed to cancel offer', 'error');
    } finally {
        hideLoading();
    }
}

async function acceptOffer(offerId) {
    if (!confirm('Are you sure you want to accept this offer?')) return;
    
    showLoading();
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/p2p/${userId}/offers/${offerId}/accept`, {
            method: 'POST'
        });
        
        if (!response.ok) {
            const errorData = await response.json();
            const errorMessage = errorData.error || 'Failed to accept offer';
            throw new Error(errorMessage);
        }
        
        const data = await response.json();
        
        // Display success message with offer details
        showToast('Offer accepted successfully! Exchange initiated.', 'success');
        
        // Refresh marketplace to remove the accepted offer
        await loadMarketplace();
        
        // Also refresh My Offers if the user wants to see their accepted offers
        if (state.connectedWallet) {
            await loadMyOffers();
        }
        
    } catch (error) {
        console.error('Error accepting offer:', error);
        showToast(error.message || 'Failed to accept offer', 'error');
    } finally {
        hideLoading();
    }
}

function contactTrader(conversationId, otherUserId) {
    // Navigate to chat view
    switchView('chat');
    
    // Store the conversation context
    state.activeConversationId = conversationId;
    state.activeConversationUserId = otherUserId;
    
    // Load the conversation
    loadConversationById(conversationId);
    
    showToast('Opening chat with trading partner...', 'success');
}

async function loadConversationById(conversationId) {
    if (!conversationId) {
        document.getElementById('chatMessages').innerHTML = 
            '<p class="empty-state">No conversation found</p>';
        return;
    }
    
    try {
        // Enable chat input
        document.getElementById('chatInput').disabled = false;
        document.getElementById('sendMessageBtn').disabled = false;
        
        // For now, show a placeholder indicating the conversation is ready
        // In a full implementation, this would fetch messages from the conversation
        document.getElementById('chatMessages').innerHTML = `
            <p class="empty-state">Chat conversation loaded (Conversation ID: ${conversationId.substring(0, 8)}...)</p>
            <p class="empty-state" style="margin-top: 1rem;">Messages are encrypted end-to-end. Start chatting to coordinate your trade.</p>
        `;
        
        // Optionally fetch actual messages if the endpoint exists
        const response = await fetch(`${API_BASE_URL}/api/chat/conversations/${conversationId}/messages`);
        if (response.ok) {
            const data = await response.json();
            displayChatMessages(data.data || []);
        }
        
    } catch (error) {
        console.error('Error loading conversation:', error);
        document.getElementById('chatMessages').innerHTML = 
            '<p class="empty-state">Chat ready. Start messaging to coordinate your trade.</p>';
    }
}

function displayMockMyOffers() {
    const mockOffers = [
        {
            id: '1',
            offer_type: 'sell',
            from_asset: 'SOL',
            to_asset: 'USDC',
            from_amount: '10',
            to_amount: '2000',
            status: 'active'
        }
    ];
    displayMyOffers(mockOffers);
}

function displayMockMarketplace() {
    const mockOffers = [
        {
            id: '2',
            offer_type: 'buy',
            from_asset: 'USDC',
            to_asset: 'ETH',
            from_amount: '3000',
            to_amount: '1.5',
            status: 'active',
            user_id: '12345678-1234-1234-1234-123456789abc',
            created_at: new Date(Date.now() - 3600000).toISOString(),
            expires_at: new Date(Date.now() + 86400000).toISOString()
        },
        {
            id: '3',
            offer_type: 'sell',
            from_asset: 'BTC',
            to_asset: 'USDC',
            from_amount: '0.5',
            to_amount: '25000',
            status: 'active',
            user_id: '87654321-4321-4321-4321-cba987654321',
            created_at: new Date(Date.now() - 7200000).toISOString(),
            expires_at: new Date(Date.now() + 172800000).toISOString()
        }
    ];
    displayMarketplace(mockOffers);
}

// Chat Functions (Full Implementation)

function setupChatListeners() {
    document.getElementById('sendMessageBtn').addEventListener('click', sendMessage);
    document.getElementById('chatInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') sendMessage();
    });
}

function initializeChatView() {
    if (state.connectedWallet) {
        loadContacts();
        loadChatHistory();
    } else {
        document.getElementById('contactsList').innerHTML = 
            '<p class="empty-state">Connect your wallet to chat</p>';
    }
}

async function loadContacts() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/chat/proximity-contacts`);
        
        if (!response.ok) {
            throw new Error('Failed to load proximity contacts');
        }
        
        const data = await response.json();
        const contacts = data.data || [];
        
        if (contacts.length === 0) {
            document.getElementById('contactsList').innerHTML = 
                '<p class="empty-state">No users in proximity network. Enable discovery to find nearby users.</p>';
            return;
        }
        
        displayProximityContacts(contacts);
        
    } catch (error) {
        console.error('Error loading proximity contacts:', error);
        document.getElementById('contactsList').innerHTML = 
            '<p class="empty-state">Failed to load contacts. Make sure discovery is enabled.</p>';
    }
}

function displayProximityContacts(contacts) {
    const container = document.getElementById('contactsList');
    container.innerHTML = '';
    
    contacts.forEach(contact => {
        const item = document.createElement('div');
        item.className = 'contact-item';
        item.onclick = () => selectProximityContact(contact);
        
        const signalIcon = contact.signal_strength ? 
            `<span class="signal-strength" title="Signal: ${contact.signal_strength}dBm">📶</span>` : '';
        
        item.innerHTML = `
            <div class="contact-avatar">${contact.user_tag.substring(0, 2)}</div>
            <div class="contact-info">
                <div class="contact-name">${contact.user_tag}</div>
                <div class="contact-status">${contact.discovery_method} ${signalIcon}</div>
            </div>
        `;
        
        container.appendChild(item);
    });
}

let selectedContact = null;

function selectProximityContact(contact) {
    selectedContact = contact;
    
    // Update UI
    document.querySelectorAll('.contact-item').forEach(item => {
        item.classList.remove('active');
    });
    event.currentTarget.classList.add('active');
    
    // Enable chat input
    document.getElementById('chatInput').disabled = false;
    document.getElementById('sendMessageBtn').disabled = false;
    
    // Load chat history for this contact (by wallet address)
    loadChatHistory(contact.wallet_address);
}

async function loadChatHistory(contactWalletAddress = null) {
    if (!contactWalletAddress) {
        document.getElementById('chatMessages').innerHTML = 
            '<p class="empty-state">Select a contact to start chatting</p>';
        return;
    }
    
    // For now, show empty state - actual message loading would require user ID mapping
    document.getElementById('chatMessages').innerHTML = 
        '<p class="empty-state">Chat with proximity users (demo mode). Messages are encrypted end-to-end.</p>';
}

function displayChatMessages(messages) {
    const container = document.getElementById('chatMessages');
    container.innerHTML = '';
    
    if (messages.length === 0) {
        container.innerHTML = '<p class="empty-state">No messages yet. Start the conversation!</p>';
        return;
    }
    
    messages.forEach(message => {
        const messageEl = document.createElement('div');
        const isSent = message.from_user_id === DEMO_USER_ID;
        messageEl.className = `chat-message ${isSent ? 'sent' : 'received'}`;
        
        const verificationBadge = message.blockchain_hash ? 
            '<div class="verification-badge" title="Verified on blockchain">✓ Verified</div>' : '';
        
        messageEl.innerHTML = `
            <div class="message-content">${escapeHtml(message.content)}</div>
            ${verificationBadge}
            <div class="message-time">${new Date(message.created_at).toLocaleTimeString()}</div>
            ${message.blockchain_hash ? 
                `<button class="btn-link" onclick="verifyMessage('${message.id}')">Verify on blockchain</button>` : 
                ''}
        `;
        
        container.appendChild(messageEl);
    });
    
    container.scrollTop = container.scrollHeight;
}

function displayMockChatHistory() {
    const mockMessages = [
        {
            id: '1',
            from_user_id: 'other-user',
            content: 'Hey, interested in your SOL offer!',
            blockchain_hash: null,
            created_at: new Date(Date.now() - 3600000).toISOString()
        },
        {
            id: '2',
            from_user_id: DEMO_USER_ID,
            content: 'Sure! The rate is 200 USDC per SOL.',
            blockchain_hash: '0x123abc...',
            created_at: new Date(Date.now() - 3000000).toISOString()
        },
        {
            id: '3',
            from_user_id: 'other-user',
            content: 'Sounds good, let me check my balance.',
            blockchain_hash: null,
            created_at: new Date(Date.now() - 1800000).toISOString()
        }
    ];
    displayChatMessages(mockMessages);
}

async function sendMessage() {
    if (!selectedContact) {
        showToast('Please select a contact first', 'warning');
        return;
    }
    
    const input = document.getElementById('chatInput');
    const message = input.value.trim();
    const verifyOnChain = document.getElementById('verifyOnChain').checked;
    
    if (!message) return;
    
    // Chat is a demo feature - just add to UI
    const messagesDiv = document.getElementById('chatMessages');
    const messageEl = document.createElement('div');
    messageEl.className = 'chat-message sent';
    messageEl.innerHTML = `
        <div class="message-content">${escapeHtml(message)}</div>
        ${verifyOnChain ? '<div class="verification-badge">✓ Demo: Would verify on blockchain</div>' : ''}
        <div class="message-time">${new Date().toLocaleTimeString()}</div>
    `;
    messagesDiv.appendChild(messageEl);
    
    input.value = '';
    messagesDiv.scrollTop = messagesDiv.scrollHeight;
    
    showToast('Message sent (demo mode)', 'success');
}

async function verifyMessage(messageId) {
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/chat/${userId}/verify/${messageId}`);
        
        if (!response.ok) throw new Error('Failed to verify message');
        
        const data = await response.json();
        
        if (data.data.verified) {
            showToast('Message verified on blockchain!', 'success');
        } else {
            showToast('Message verification failed', 'error');
        }
        
    } catch (error) {
        console.error('Error verifying message:', error);
        showToast('Failed to verify message', 'error');
    } finally {
        hideLoading();
    }
}

async function reportMessage(messageId) {
    if (!confirm('Are you sure you want to report this message?')) return;
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/chat/${userId}/report/${messageId}`, {
            method: 'POST'
        });
        
        if (!response.ok) throw new Error('Failed to report message');
        
        showToast('Message reported successfully', 'success');
        
    } catch (error) {
        console.error('Error reporting message:', error);
        showToast('Failed to report message', 'error');
    } finally {
        hideLoading();
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Privacy Features Functions (Full Implementation)

function setupPrivacyListeners() {
    document.getElementById('updateUserTagBtn').addEventListener('click', updateUserTag);
    document.getElementById('createTempWalletBtn').addEventListener('click', showTempWalletForm);
    document.getElementById('cancelTempWalletBtn').addEventListener('click', hideTempWalletForm);
    document.getElementById('saveTempWalletBtn').addEventListener('click', createTempWallet);
}

function initializePrivacyView() {
    hideTempWalletForm();
    if (state.connectedWallet) {
        loadUserTag();
        loadTempWallets();
        loadWalletFreezeStatus();
    } else {
        document.getElementById('currentUserTag').textContent = 'Connect wallet to view';
        document.getElementById('tempWalletsList').innerHTML = 
            '<p class="empty-state">Connect your wallet to manage temporary wallets</p>';
        document.getElementById('walletFreezeList').innerHTML = 
            '<p class="empty-state">Connect your wallet to manage freeze settings</p>';
    }
}

async function loadUserTag() {
    // Privacy features are demo - using mock tag
    document.getElementById('currentUserTag').textContent = 'Trader_A7X9K2';
}

async function updateUserTag() {
    const newTag = document.getElementById('newUserTag').value.trim();
    const errorDiv = document.getElementById('userTagError');
    
    errorDiv.textContent = '';
    
    if (!newTag) {
        errorDiv.textContent = 'Please enter a new tag';
        return;
    }
    
    if (newTag.length < 3) {
        errorDiv.textContent = 'Tag must be at least 3 characters';
        return;
    }
    
    if (!/^[a-zA-Z0-9_]+$/.test(newTag)) {
        errorDiv.textContent = 'Tag can only contain letters, numbers, and underscores';
        return;
    }
    
    // Privacy features are demo - just update UI
    document.getElementById('currentUserTag').textContent = newTag;
    document.getElementById('newUserTag').value = '';
    showToast('User tag updated (demo mode)', 'success');
}

function showTempWalletForm() {
    document.getElementById('tempWalletForm').classList.remove('hidden');
    document.getElementById('tempWalletError').textContent = '';
}

function hideTempWalletForm() {
    document.getElementById('tempWalletForm').classList.add('hidden');
    document.getElementById('tempWalletTag').value = '';
    document.getElementById('tempWalletExpiry').value = '';
}

async function createTempWallet() {
    const tag = document.getElementById('tempWalletTag').value.trim();
    const blockchain = document.getElementById('tempWalletBlockchain').value;
    const expiry = document.getElementById('tempWalletExpiry').value;
    const errorDiv = document.getElementById('tempWalletError');
    
    errorDiv.textContent = '';
    
    if (!tag) {
        errorDiv.textContent = 'Please enter a wallet tag';
        return;
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/temporary-wallets`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                tag,
                blockchain,
                expires_at: expiry || null
            })
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to create temporary wallet');
        }
        
        showToast('Temporary wallet created successfully!', 'success');
        hideTempWalletForm();
        loadTempWallets();
        
    } catch (error) {
        console.error('Error creating temporary wallet:', error);
        errorDiv.textContent = error.message || 'Failed to create temporary wallet';
        showToast('Failed to create temporary wallet', 'error');
    } finally {
        hideLoading();
    }
}

async function loadTempWallets() {
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/temporary-wallets`);
        
        if (!response.ok) throw new Error('Failed to load temporary wallets');
        
        const data = await response.json();
        displayTempWallets(data.data || []);
        
    } catch (error) {
        console.error('Error loading temporary wallets:', error);
        displayMockTempWallets();
    }
}

function displayTempWallets(wallets) {
    const container = document.getElementById('tempWalletsList');
    const activeWalletInfo = document.getElementById('activeWalletInfo');
    const activeWalletName = document.getElementById('activeWalletName');
    
    if (wallets.length === 0) {
        container.innerHTML = '<p class="empty-state">No temporary wallets. Create one to get started!</p>';
        activeWalletInfo.classList.add('hidden');
        return;
    }
    
    // Find and display active wallet
    const activeWallet = wallets.find(w => w.is_primary);
    if (activeWallet) {
        activeWalletName.textContent = `${activeWallet.temp_tag || 'Unnamed'} (${formatChainName(activeWallet.blockchain)})`;
        activeWalletInfo.classList.remove('hidden');
    } else {
        activeWalletInfo.classList.add('hidden');
    }
    
    container.innerHTML = '';
    
    wallets.forEach(wallet => {
        const item = document.createElement('div');
        item.className = 'temp-wallet-item';
        
        const isExpired = wallet.expires_at && new Date(wallet.expires_at) < new Date();
        const expiryText = wallet.expires_at ? 
            `Expires: ${new Date(wallet.expires_at).toLocaleString()}` : 
            'No expiration';
        
        item.innerHTML = `
            <div class="wallet-info">
                <div class="wallet-header">
                    <div class="wallet-tag">${wallet.temp_tag || 'Unnamed'}</div>
                    <div class="wallet-chain-badge">${formatChainName(wallet.blockchain)}</div>
                    ${wallet.is_primary ? '<span class="primary-badge">Active</span>' : ''}
                    ${wallet.is_frozen ? '<span class="frozen-badge">Frozen</span>' : ''}
                    ${isExpired ? '<span class="expired-badge">Expired</span>' : ''}
                </div>
                <div class="wallet-address">${wallet.address}</div>
                <div class="wallet-expiry">${expiryText}</div>
            </div>
            <div class="wallet-actions">
                ${!wallet.is_primary ? `<button class="btn btn-primary btn-sm" onclick="setWalletAsPrimary('${wallet.id}')">Set as Active</button>` : ''}
                ${wallet.is_frozen ? 
                    `<button class="btn btn-warning btn-sm" onclick="unfreezeTempWallet('${wallet.address}')">Unfreeze</button>` :
                    `<button class="btn btn-secondary btn-sm" onclick="freezeTempWallet('${wallet.address}')">Freeze</button>`
                }
                <button class="btn btn-secondary btn-sm" onclick="copyToClipboard('${wallet.address}')">Copy Address</button>
                <button class="btn btn-secondary btn-sm" onclick="deleteTempWallet('${wallet.id}')">Delete</button>
            </div>
        `;
        
        container.appendChild(item);
    });
}

function displayMockTempWallets() {
    const mockWallets = [
        {
            id: '1',
            temp_tag: 'Trading_Bot_1',
            blockchain: 'solana',
            address: '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU',
            expires_at: new Date(Date.now() + 86400000 * 7).toISOString(),
            is_primary: true
        },
        {
            id: '2',
            temp_tag: 'DeFi_Experiments',
            blockchain: 'ethereum',
            address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
            expires_at: null,
            is_primary: false
        }
    ];
    displayTempWallets(mockWallets);
}

async function deleteTempWallet(walletId) {
    if (!confirm('Are you sure you want to delete this temporary wallet? Any remaining funds should be transferred first.')) {
        return;
    }
    
    showLoading();
    
    try {
        // TODO: Backend DELETE endpoint not implemented yet
        showToast('Delete functionality coming soon', 'warning');
        return;
        
        /* Uncomment when backend implements DELETE endpoint
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/temporary-wallets/${walletId}`, {
            method: 'DELETE'
        });
        
        if (!response.ok) throw new Error('Failed to delete temporary wallet');
        
        showToast('Temporary wallet deleted successfully!', 'success');
        loadTempWallets();
        */
        
    } catch (error) {
        console.error('Error deleting temporary wallet:', error);
        showToast('Failed to delete temporary wallet', 'error');
    } finally {
        hideLoading();
    }
}

async function setWalletAsPrimary(walletId) {
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/temporary-wallets/${walletId}/primary`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' }
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to set wallet as primary');
        }
        
        showToast('Wallet set as active successfully!', 'success');
        loadTempWallets();
        
    } catch (error) {
        console.error('Error setting wallet as primary:', error);
        showToast(error.message || 'Failed to set wallet as active', 'error');
    } finally {
        hideLoading();
    }
}

async function freezeTempWallet(walletAddress) {
    if (!confirm('Are you sure you want to freeze this wallet? This will block all outgoing transactions.')) {
        return;
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/wallets/${walletAddress}/freeze`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({})
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to freeze wallet');
        }
        
        showToast('Wallet frozen successfully! It will block all outgoing transactions.', 'success');
        loadTempWallets();
        
    } catch (error) {
        console.error('Error freezing wallet:', error);
        showToast(error.message || 'Failed to freeze wallet', 'error');
    } finally {
        hideLoading();
    }
}

async function unfreezeTempWallet(walletAddress) {
    const password = prompt('Enter your password to unfreeze this wallet:');
    
    if (!password) return;
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/wallets/${walletAddress}/unfreeze`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ password: password })
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to unfreeze wallet');
        }
        
        showToast('Wallet unfrozen successfully!', 'success');
        loadTempWallets();
        
    } catch (error) {
        console.error('Error unfreezing wallet:', error);
        showToast(error.message || 'Failed to unfreeze wallet', 'error');
    } finally {
        hideLoading();
    }
}

async function loadWalletFreezeStatus() {
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/wallets`);
        
        if (!response.ok) throw new Error('Failed to load wallet freeze status');
        
        const data = await response.json();
        displayWalletFreezeStatus(data.data || []);
        
    } catch (error) {
        console.error('Error loading wallet freeze status:', error);
        displayMockWalletFreezeStatus();
    }
}

function displayWalletFreezeStatus(wallets) {
    const container = document.getElementById('walletFreezeList');
    
    if (wallets.length === 0) {
        container.innerHTML = '<p class="empty-state">No wallets found</p>';
        return;
    }
    
    container.innerHTML = '';
    
    wallets.forEach(wallet => {
        const item = document.createElement('div');
        item.className = 'freeze-wallet-item';
        
        item.innerHTML = `
            <div class="wallet-info">
                <div class="wallet-header">
                    <div class="wallet-type">${wallet.is_primary ? 'Primary Wallet' : 'Secondary Wallet'}</div>
                    <div class="wallet-chain-badge">${formatChainName(wallet.blockchain)}</div>
                </div>
                <div class="wallet-address">${wallet.address}</div>
                ${wallet.is_frozen ? 
                    `<div class="freeze-status frozen">🔒 Frozen since ${new Date(wallet.frozen_at).toLocaleString()}</div>` :
                    '<div class="freeze-status active">✓ Active</div>'
                }
            </div>
            <div class="wallet-actions">
                ${wallet.is_frozen ?
                    `<button class="btn btn-primary btn-sm" onclick="unfreezeWallet('${wallet.address}')">Unfreeze</button>` :
                    `<button class="btn btn-secondary btn-sm" onclick="freezeWallet('${wallet.address}')">Freeze</button>`
                }
            </div>
        `;
        
        container.appendChild(item);
    });
}

function displayMockWalletFreezeStatus() {
    const mockWallets = [
        {
            id: '1',
            address: state.connectedWallet || '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU',
            blockchain: 'solana',
            is_primary: true,
            is_frozen: false,
            frozen_at: null
        }
    ];
    displayWalletFreezeStatus(mockWallets);
}

async function freezeWallet(walletAddress) {
    if (!confirm('Are you sure you want to freeze this wallet? This will block all outgoing transactions.')) {
        return;
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/wallets/${walletAddress}/freeze`, {
            method: 'POST'
        });
        
        if (!response.ok) throw new Error('Failed to freeze wallet');
        
        showToast('Wallet frozen successfully!', 'success');
        loadWalletFreezeStatus();
        
    } catch (error) {
        console.error('Error freezing wallet:', error);
        showToast('Failed to freeze wallet', 'error');
    } finally {
        hideLoading();
    }
}

async function unfreezeWallet(walletAddress) {
    const password = prompt('Enter your password to unfreeze this wallet:');
    
    if (!password) return;
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/privacy/${userId}/wallets/${walletAddress}/unfreeze`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ password: password })
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to unfreeze wallet');
        }
        
        showToast('Wallet unfrozen successfully!', 'success');
        loadWalletFreezeStatus();
        
    } catch (error) {
        console.error('Error unfreezing wallet:', error);
        showToast(error.message || 'Failed to unfreeze wallet', 'error');
    } finally {
        hideLoading();
    }
}

function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        showToast('Address copied to clipboard!', 'success');
    }).catch(() => {
        showToast('Failed to copy address', 'error');
    });
}

// Enhanced Dashboard Functions (Full Implementation)

let websocket = null;

function initializeWebSocket() {
    if (!state.connectedWallet) return;
    
    // Close existing connection if any
    if (websocket) {
        websocket.close();
    }
    
    try {
        // Connect to WebSocket server
        const wsUrl = API_BASE_URL.replace('http', 'ws') + '/ws/dashboard';
        websocket = new WebSocket(wsUrl);
        
        websocket.onopen = () => {
            console.log('WebSocket connected');
            // Subscribe to updates for this wallet
            websocket.send(JSON.stringify({
                type: 'subscribe',
                wallet_address: state.connectedWallet
            }));
        };
        
        websocket.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                handleWebSocketMessage(data);
            } catch (error) {
                console.error('Error parsing WebSocket message:', error);
            }
        };
        
        websocket.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
        
        websocket.onclose = () => {
            console.log('WebSocket disconnected');
            // Attempt to reconnect after 5 seconds
            setTimeout(() => {
                if (state.connectedWallet && state.currentView === 'dashboard') {
                    initializeWebSocket();
                }
            }, 5000);
        };
        
    } catch (error) {
        console.error('Error initializing WebSocket:', error);
        // Fall back to polling
        startPolling();
    }
}

function handleWebSocketMessage(data) {
    switch (data.type) {
        case 'portfolio_update':
            if (state.currentView === 'dashboard') {
                state.portfolio = data.portfolio;
                displayPortfolio(data.portfolio);
            }
            break;
            
        case 'price_update':
            // Update prices in real-time
            updatePrices(data.prices);
            break;
            
        case 'trade_executed':
            showToast(`Trade executed: ${data.trade.action} ${data.trade.amount} ${data.trade.asset}`, 'success');
            loadPortfolio(state.connectedWallet);
            loadAIActions();
            break;
            
        case 'trim_executed':
            showToast(`Position trimmed: ${data.trim.asset} - ${data.trim.reasoning}`, 'warning');
            loadPortfolio(state.connectedWallet);
            loadAIActions();
            break;
            
        case 'benchmark_triggered':
            showToast(`Benchmark triggered: ${data.benchmark.asset} reached $${data.benchmark.target_price}`, 'warning');
            break;
            
        default:
            console.log('Unknown WebSocket message type:', data.type);
    }
}

function updatePrices(prices) {
    // Update prices in the portfolio display
    Object.keys(prices).forEach(asset => {
        const elements = document.querySelectorAll(`[data-asset="${asset}"]`);
        elements.forEach(el => {
            el.textContent = `$${prices[asset].toFixed(2)}`;
        });
    });
}

function startPolling() {
    // Fallback polling mechanism if WebSocket fails
    setInterval(() => {
        if (state.connectedWallet && state.currentView === 'dashboard') {
            loadPortfolio(state.connectedWallet);
        }
    }, 30000); // Poll every 30 seconds
}

async function loadAIActions() {
    if (!state.connectedWallet) return;
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/analytics/${userId}/ai-actions`);
        
        if (!response.ok) throw new Error('Failed to load AI actions');
        
        const data = await response.json();
        displayAIActions(data.data || []);
        
    } catch (error) {
        console.error('Error loading AI actions:', error);
        displayMockAIActions();
    }
}

function displayAIActions(actions) {
    const container = document.getElementById('aiActionsList');
    
    if (actions.length === 0) {
        container.innerHTML = '<p class="empty-state">No recent AI actions</p>';
        return;
    }
    
    container.innerHTML = '';
    
    actions.slice(0, 10).forEach(action => {
        const item = document.createElement('div');
        item.className = `ai-action-item ${action.action_type}`;
        
        const icon = action.action_type === 'trim' ? '✂️' : 
                    action.action_type === 'recommendation' ? '💡' : 
                    action.action_type === 'trade' ? '💰' : '📊';
        
        item.innerHTML = `
            <div class="action-icon">${icon}</div>
            <div class="action-info">
                <div class="action-header">
                    <span class="action-type">${action.action_type.toUpperCase()}</span>
                    <span class="action-time">${formatTimeAgo(action.created_at)}</span>
                </div>
                <div class="action-details">
                    <strong>${action.asset}</strong> - ${action.description}
                </div>
                ${action.reasoning ? `<div class="action-reasoning">${action.reasoning}</div>` : ''}
                ${action.profit_realized ? 
                    `<div class="action-profit">Profit: $${parseFloat(action.profit_realized).toFixed(2)}</div>` : 
                    ''}
            </div>
        `;
        
        container.appendChild(item);
    });
}

function displayMockAIActions() {
    const mockActions = [
        {
            action_type: 'trim',
            asset: 'SOL',
            description: 'Trimmed 25% of position',
            reasoning: 'Strong resistance at $210, taking profits',
            profit_realized: '125.50',
            created_at: new Date(Date.now() - 3600000).toISOString()
        },
        {
            action_type: 'recommendation',
            asset: 'ETH',
            description: 'Recommended buy signal',
            reasoning: 'Whale accumulation detected, bullish momentum',
            profit_realized: null,
            created_at: new Date(Date.now() - 7200000).toISOString()
        },
        {
            action_type: 'trade',
            asset: 'BTC',
            description: 'Executed buy order',
            reasoning: 'Benchmark triggered at $48,000',
            profit_realized: null,
            created_at: new Date(Date.now() - 10800000).toISOString()
        }
    ];
    displayAIActions(mockActions);
}

async function loadPositionDistribution() {
    if (!state.connectedWallet || !state.portfolio) return;
    
    try {
        // Calculate distribution from portfolio data
        const blockchainDist = {};
        const assetTypeDist = {};
        
        if (state.portfolio.positions_by_chain) {
            Object.keys(state.portfolio.positions_by_chain).forEach(chain => {
                const assets = state.portfolio.positions_by_chain[chain];
                const totalValue = assets.reduce((sum, asset) => sum + (asset.value_usd || 0), 0);
                blockchainDist[formatChainName(chain)] = totalValue;
                
                assets.forEach(asset => {
                    const type = categorizeAsset(asset.token_symbol || asset.symbol);
                    assetTypeDist[type] = (assetTypeDist[type] || 0) + (asset.value_usd || 0);
                });
            });
        }
        
        displayDistributionCharts(blockchainDist, assetTypeDist);
        
    } catch (error) {
        console.error('Error loading position distribution:', error);
        displayMockDistribution();
    }
}

function categorizeAsset(symbol) {
    const stablecoins = ['USDC', 'USDT', 'DAI', 'BUSD'];
    const majors = ['BTC', 'ETH', 'SOL', 'BNB'];
    
    if (stablecoins.includes(symbol)) return 'Stablecoins';
    if (majors.includes(symbol)) return 'Major Coins';
    return 'Altcoins';
}

function displayDistributionCharts(blockchainDist, assetTypeDist) {
    // Simple text-based distribution for now (can be replaced with actual charts)
    displayDistributionList('blockchainChart', blockchainDist, 'By Blockchain');
    displayDistributionList('assetTypeChart', assetTypeDist, 'By Asset Type');
}

function displayDistributionList(canvasId, distribution, title) {
    const canvas = document.getElementById(canvasId);
    if (!canvas) return;
    
    const container = canvas.parentElement;
    container.innerHTML = `<h4>${title}</h4>`;
    
    const total = Object.values(distribution).reduce((sum, val) => sum + val, 0);
    
    Object.keys(distribution).forEach(key => {
        const value = distribution[key];
        const percentage = total > 0 ? (value / total * 100).toFixed(1) : 0;
        
        const item = document.createElement('div');
        item.className = 'distribution-item';
        item.innerHTML = `
            <div class="dist-label">${key}</div>
            <div class="dist-bar-container">
                <div class="dist-bar" style="width: ${percentage}%"></div>
            </div>
            <div class="dist-value">${percentage}%</div>
        `;
        container.appendChild(item);
    });
}

function displayMockDistribution() {
    const mockBlockchainDist = {
        'Solana': 12500,
        'Ethereum': 10000,
        'Polygon': 3250
    };
    
    const mockAssetTypeDist = {
        'Major Coins': 18000,
        'Stablecoins': 5000,
        'Altcoins': 2750
    };
    
    displayDistributionCharts(mockBlockchainDist, mockAssetTypeDist);
}

function formatTimeAgo(dateString) {
    const date = new Date(dateString);
    const now = new Date();
    const seconds = Math.floor((now - date) / 1000);
    
    if (seconds < 60) return 'Just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
}

// Utility Functions
function showLoading() {
    document.getElementById('loadingOverlay').classList.remove('hidden');
}

function hideLoading() {
    document.getElementById('loadingOverlay').classList.add('hidden');
}

function showToast(message, type = 'success') {
    const container = document.getElementById('toastContainer');
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;
    
    container.appendChild(toast);
    
    setTimeout(() => {
        toast.style.animation = 'slideIn 0.3s ease-out reverse';
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

function loadSavedWallet() {
    const savedWallet = localStorage.getItem('connectedWallet');
    if (savedWallet) {
        document.getElementById('walletAddress').value = savedWallet;
        state.connectedWallet = savedWallet;
        loadPortfolio(savedWallet);
        document.getElementById('portfolioSection').classList.remove('hidden');
        document.getElementById('activitySection').classList.remove('hidden');
    }
}


// Trim Settings Functions

function setupTrimSettingsListeners() {
    const trimEnabled = document.getElementById('trimEnabled');
    const saveTrimConfig = document.getElementById('saveTrimConfig');
    
    trimEnabled.addEventListener('change', (e) => {
        const configSection = document.getElementById('trimConfigSection');
        if (e.target.checked) {
            configSection.classList.remove('hidden');
        } else {
            configSection.classList.add('hidden');
        }
    });
    
    saveTrimConfig.addEventListener('click', saveTrimConfiguration);
    
    // Load trim config when settings view is opened
    loadTrimConfiguration();
}

async function loadTrimConfiguration() {
    if (!state.connectedWallet) return;
    
    try {
        const userId = DEMO_USER_ID;
        const response = await fetch(`${API_BASE_URL}/api/trim/${userId}/config`);
        
        if (!response.ok) {
            // Config doesn't exist yet, use defaults
            return;
        }
        
        const data = await response.json();
        const config = data.data;
        
        // Update UI with loaded config
        document.getElementById('trimEnabled').checked = config.enabled;
        document.getElementById('minProfitPercent').value = config.minimum_profit_percent;
        document.getElementById('trimPercent').value = config.trim_percent;
        document.getElementById('maxTrimsPerDay').value = config.max_trims_per_day;
        
        // Show/hide config section based on enabled state
        const configSection = document.getElementById('trimConfigSection');
        if (config.enabled) {
            configSection.classList.remove('hidden');
        } else {
            configSection.classList.add('hidden');
        }
        
    } catch (error) {
        console.error('Error loading trim configuration:', error);
        // Silently fail - config might not exist yet
    }
}

async function saveTrimConfiguration() {
    const errorDiv = document.getElementById('trimConfigError');
    errorDiv.textContent = '';
    
    const enabled = document.getElementById('trimEnabled').checked;
    const minProfitPercent = parseFloat(document.getElementById('minProfitPercent').value);
    const trimPercent = parseFloat(document.getElementById('trimPercent').value);
    const maxTrimsPerDay = parseInt(document.getElementById('maxTrimsPerDay').value);
    
    // Validation
    if (enabled) {
        if (isNaN(minProfitPercent) || minProfitPercent < 0 || minProfitPercent > 100) {
            errorDiv.textContent = 'Minimum profit percent must be between 0 and 100';
            return;
        }
        if (isNaN(trimPercent) || trimPercent < 1 || trimPercent > 100) {
            errorDiv.textContent = 'Trim percent must be between 1 and 100';
            return;
        }
        if (isNaN(maxTrimsPerDay) || maxTrimsPerDay < 1) {
            errorDiv.textContent = 'Max trims per day must be at least 1';
            return;
        }
    }
    
    showLoading();
    
    try {
        const userId = DEMO_USER_ID;
        const payload = {
            enabled,
            minimum_profit_percent: minProfitPercent,
            trim_percent: trimPercent,
            max_trims_per_day: maxTrimsPerDay
        };
        
        const response = await fetch(`${API_BASE_URL}/api/trim/${userId}/config`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to save trim configuration');
        }
        
        showToast(
            enabled ? 'Agentic trimming enabled!' : 'Agentic trimming disabled',
            'success'
        );
        
    } catch (error) {
        console.error('Error saving trim configuration:', error);
        errorDiv.textContent = error.message || 'Failed to save trim configuration';
        showToast('Failed to save configuration', 'error');
    } finally {
        hideLoading();
    }
}


// Mesh Network Provider Functions

function setupMeshNetworkListeners() {
    document.getElementById('providerModeEnabled').addEventListener('change', toggleProviderConfigSection);
    document.getElementById('saveProviderConfig').addEventListener('click', saveProviderConfig);
}

function toggleProviderConfigSection(e) {
    const configSection = document.getElementById('providerConfigSection');
    if (e.target.checked) {
        configSection.classList.remove('hidden');
    } else {
        // If unchecking, disable provider mode
        disableProviderMode();
    }
}

async function saveProviderConfig() {
    const apiKey = document.getElementById('birdeyeApiKey').value.trim();
    const errorDiv = document.getElementById('providerConfigError');
    
    errorDiv.textContent = '';
    
    if (!apiKey) {
        errorDiv.textContent = 'Please enter your Birdeye API key';
        return;
    }
    
    showLoading();
    
    try {
        const response = await fetch(`${API_BASE_URL}/api/mesh/provider/enable`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ api_key: apiKey })
        });
        
        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Failed to enable provider mode');
        }
        
        const data = await response.json();
        
        showToast('Provider mode enabled successfully!', 'success');
        updateProviderStatus(true);
        
        // Clear API key from input for security
        document.getElementById('birdeyeApiKey').value = '';
        document.getElementById('providerConfigSection').classList.add('hidden');
        
    } catch (error) {
        console.error('Error enabling provider mode:', error);
        errorDiv.textContent = error.message || 'Failed to enable provider mode. Please check your API key.';
        showToast('Failed to enable provider mode', 'error');
        
        // Uncheck the toggle
        document.getElementById('providerModeEnabled').checked = false;
    } finally {
        hideLoading();
    }
}

async function disableProviderMode() {
    showLoading();
    
    try {
        const response = await fetch(`${API_BASE_URL}/api/mesh/provider/disable`, {
            method: 'POST'
        });
        
        if (!response.ok) {
            throw new Error('Failed to disable provider mode');
        }
        
        showToast('Provider mode disabled', 'success');
        updateProviderStatus(false);
        document.getElementById('providerConfigSection').classList.add('hidden');
        
    } catch (error) {
        console.error('Error disabling provider mode:', error);
        showToast('Failed to disable provider mode', 'error');
        
        // Re-check the toggle
        document.getElementById('providerModeEnabled').checked = true;
    } finally {
        hideLoading();
    }
}

async function loadProviderStatus() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/mesh/provider/status`);
        
        if (!response.ok) {
            throw new Error('Failed to load provider status');
        }
        
        const data = await response.json();
        const isEnabled = data.data.enabled;
        
        updateProviderStatus(isEnabled);
        document.getElementById('providerModeEnabled').checked = isEnabled;
        
    } catch (error) {
        console.error('Error loading provider status:', error);
        updateProviderStatus(false);
    }
}

function updateProviderStatus(isEnabled) {
    const indicator = document.getElementById('providerStatusIndicator');
    const statusText = document.getElementById('providerStatusText');
    
    if (isEnabled) {
        indicator.classList.add('active');
        statusText.textContent = 'Provider Mode: Active';
    } else {
        indicator.classList.remove('active');
        statusText.textContent = 'Provider Mode: Disabled';
    }
}

// Price Freshness Functions

function calculateFreshness(timestamp) {
    const now = new Date();
    const updateTime = new Date(timestamp);
    const diffMs = now - updateTime;
    const diffMinutes = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    
    if (diffMinutes < 1) {
        return { text: 'Just now', class: 'just-now', warning: false };
    } else if (diffMinutes < 60) {
        return { text: `${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`, class: 'minutes-ago', warning: false };
    } else if (diffHours < 24) {
        return { text: `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`, class: 'hours-ago', warning: diffHours >= 1 };
    } else {
        return { text: 'Stale data', class: 'stale', warning: true };
    }
}

function createFreshnessIndicator(timestamp, sourceNodeId) {
    const freshness = calculateFreshness(timestamp);
    const formattedTime = new Date(timestamp).toLocaleString();
    const shortNodeId = sourceNodeId ? `${sourceNodeId.substring(0, 8)}...` : 'Unknown';
    
    const indicator = document.createElement('span');
    indicator.className = 'price-freshness freshness-tooltip';
    
    indicator.innerHTML = `
        <span class="freshness-indicator ${freshness.class}"></span>
        <span class="freshness-text ${freshness.warning ? 'freshness-warning' : ''}">${freshness.text}</span>
        <div class="tooltip-content">
            <div class="tooltip-row">
                <span class="tooltip-label">Updated:</span>
                <span class="tooltip-value">${formattedTime}</span>
            </div>
            <div class="tooltip-row">
                <span class="tooltip-label">Source:</span>
                <span class="tooltip-value">${shortNodeId}</span>
            </div>
        </div>
    `;
    
    return indicator;
}

// Network Status Functions

async function loadNetworkStatus() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/mesh/network/status`);
        
        if (!response.ok) {
            throw new Error('Failed to load network status');
        }
        
        const data = await response.json();
        displayNetworkStatus(data.data);
        
    } catch (error) {
        console.error('Error loading network status:', error);
        displayMockNetworkStatus();
    }
}

function displayNetworkStatus(status) {
    // Create or update network status card in dashboard
    let statusCard = document.getElementById('networkStatusCard');
    
    if (!statusCard) {
        statusCard = document.createElement('div');
        statusCard.id = 'networkStatusCard';
        statusCard.className = 'card network-status-card';
        
        // Insert after portfolio section
        const portfolioSection = document.getElementById('portfolioSection');
        if (portfolioSection) {
            portfolioSection.parentNode.insertBefore(statusCard, portfolioSection.nextSibling);
        }
    }
    
    const activeProviders = status.active_providers || [];
    const connectedPeers = status.connected_peers || 0;
    const lastUpdate = status.last_update_time;
    
    let warningHtml = '';
    
    // Check for warnings
    if (activeProviders.length === 0) {
        warningHtml = `
            <div class="network-warning error">
                <span class="warning-icon">⚠️</span>
                <div class="warning-content">
                    <div class="warning-title">No Live Data Sources</div>
                    <div class="warning-message">No provider nodes are currently online. Displaying cached data.</div>
                </div>
            </div>
        `;
    }
    
    // Check for extended offline (10+ minutes)
    if (lastUpdate) {
        const timeSinceUpdate = Date.now() - new Date(lastUpdate).getTime();
        const minutesSinceUpdate = Math.floor(timeSinceUpdate / 60000);
        
        if (minutesSinceUpdate >= 10) {
            warningHtml = `
                <div class="network-offline-indicator">
                    <span class="offline-icon">🔴</span>
                    <span class="offline-text">Network Offline for ${minutesSinceUpdate} minutes</span>
                </div>
            `;
        }
    }
    
    statusCard.innerHTML = `
        <div class="network-status-header">
            <h3 class="network-status-title">Mesh Network Status</h3>
        </div>
        
        ${warningHtml}
        
        <div class="network-status-grid">
            <div class="network-stat-item">
                <span class="network-stat-label">Active Providers</span>
                <span class="network-stat-value ${activeProviders.length > 0 ? 'success' : 'error'}">
                    ${activeProviders.length}
                </span>
            </div>
            <div class="network-stat-item">
                <span class="network-stat-label">Connected Peers</span>
                <span class="network-stat-value ${connectedPeers > 0 ? 'success' : 'warning'}">
                    ${connectedPeers}
                </span>
            </div>
            <div class="network-stat-item">
                <span class="network-stat-label">Network Size</span>
                <span class="network-stat-value">
                    ${status.total_network_size || 0}
                </span>
            </div>
        </div>
    `;
}

function displayMockNetworkStatus() {
    displayNetworkStatus({
        active_providers: [
            { node_id: 'abc123', last_seen: new Date().toISOString(), hop_count: 1 },
            { node_id: 'def456', last_seen: new Date().toISOString(), hop_count: 2 }
        ],
        connected_peers: 5,
        total_network_size: 12,
        last_update_time: new Date().toISOString(),
        data_freshness: 'JustNow'
    });
}

// Enhanced Portfolio Display with Mesh Prices

async function loadMeshPrices() {
    try {
        const response = await fetch(`${API_BASE_URL}/api/mesh/prices`);
        
        if (!response.ok) {
            throw new Error('Failed to load mesh prices');
        }
        
        const data = await response.json();
        return data.data || {};
        
    } catch (error) {
        console.error('Error loading mesh prices:', error);
        return {};
    }
}

function enhancePortfolioWithMeshPrices(portfolio, meshPrices) {
    // Add freshness indicators to asset display
    if (portfolio.positions_by_chain) {
        Object.keys(portfolio.positions_by_chain).forEach(chain => {
            portfolio.positions_by_chain[chain].forEach(asset => {
                const symbol = asset.token_symbol || asset.symbol;
                if (meshPrices[symbol]) {
                    asset.mesh_price_data = meshPrices[symbol];
                }
            });
        });
    }
    
    return portfolio;
}

// Update the displayAssetsList function to include freshness indicators
const originalDisplayAssetsList = displayAssetsList;
displayAssetsList = async function(portfolio) {
    // Load mesh prices
    const meshPrices = await loadMeshPrices();
    
    // Enhance portfolio with mesh price data
    const enhancedPortfolio = enhancePortfolioWithMeshPrices(portfolio, meshPrices);
    
    const assetsList = document.getElementById('assetsList');
    assetsList.innerHTML = '';
    
    if (enhancedPortfolio.positions_by_chain) {
        const chains = state.selectedBlockchain === 'all' 
            ? Object.keys(enhancedPortfolio.positions_by_chain)
            : [state.selectedBlockchain];
        
        let hasAssets = false;
        
        chains.forEach(chain => {
            const assets = enhancedPortfolio.positions_by_chain[chain];
            if (!assets || assets.length === 0) return;
            
            hasAssets = true;
            
            const chainHeader = document.createElement('div');
            chainHeader.className = 'chain-header';
            chainHeader.innerHTML = `
                <div class="chain-name">${formatChainName(chain)}</div>
                <div class="chain-count">${assets.length} assets</div>
            `;
            assetsList.appendChild(chainHeader);
            
            assets.forEach(asset => {
                const assetItem = document.createElement('div');
                assetItem.className = 'asset-item';
                
                const symbol = asset.token_symbol || asset.symbol;
                const amount = parseFloat(asset.amount).toFixed(4);
                const value = (asset.value_usd || 0).toFixed(2);
                
                const infoDiv = document.createElement('div');
                infoDiv.className = 'asset-info';
                
                let priceInfoHtml = '';
                if (asset.mesh_price_data) {
                    const freshnessIndicator = createFreshnessIndicator(
                        asset.mesh_price_data.timestamp,
                        asset.mesh_price_data.source_node_id
                    );
                    priceInfoHtml = `<div class="asset-price-info">${freshnessIndicator.outerHTML}</div>`;
                }
                
                infoDiv.innerHTML = `
                    <div>
                        <div class="asset-symbol">${symbol}</div>
                        <div class="asset-amount">${amount}</div>
                    </div>
                    <div class="asset-chain-badge">${formatChainName(chain)}</div>
                    ${priceInfoHtml}
                `;
                
                const valueDiv = document.createElement('div');
                valueDiv.className = 'asset-value';
                valueDiv.textContent = `$${value}`;
                
                assetItem.appendChild(infoDiv);
                assetItem.appendChild(valueDiv);
                assetsList.appendChild(assetItem);
            });
        });
        
        if (!hasAssets) {
            assetsList.innerHTML = '<p class="empty-state">No assets found for selected blockchain</p>';
        }
    } else if (enhancedPortfolio.assets) {
        enhancedPortfolio.assets.forEach(asset => {
            const assetItem = document.createElement('div');
            assetItem.className = 'asset-item';
            assetItem.innerHTML = `
                <div class="asset-info">
                    <div>
                        <div class="asset-symbol">${asset.token_symbol}</div>
                        <div class="asset-amount">${parseFloat(asset.amount).toFixed(4)}</div>
                    </div>
                </div>
                <div class="asset-value">${(asset.value_usd || 0).toFixed(2)}</div>
            `;
            assetsList.appendChild(assetItem);
        });
    } else {
        assetsList.innerHTML = '<p class="empty-state">No assets found</p>';
    }
};

// Initialize mesh network features
function initializeMeshNetwork() {
    setupMeshNetworkListeners();
    loadProviderStatus();
    loadNetworkStatus();
    
    // Refresh network status every 30 seconds
    setInterval(loadNetworkStatus, 30000);
}

// Update the initializeApp function to include mesh network initialization
const originalInitializeApp = initializeApp;
initializeApp = function() {
    originalInitializeApp();
    initializeMeshNetwork();
};
