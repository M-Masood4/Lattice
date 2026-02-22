// CoinMarketCap API Integration for Real-Time Crypto Prices

const CMC_API_BASE = '/api/cmc';

/**
 * Get real-time price for a single cryptocurrency
 * @param {string} symbol - Cryptocurrency symbol (e.g., 'BTC', 'ETH', 'SOL')
 * @returns {Promise<Object>} Price data including USD price, 24h change, volume, market cap
 */
async function getCryptoPrice(symbol) {
    try {
        const response = await fetch(`${CMC_API_BASE}/price?symbol=${symbol.toUpperCase()}`);
        const data = await response.json();
        
        if (!data.success) {
            throw new Error(data.error || 'Failed to fetch price');
        }
        
        return data.data;
    } catch (error) {
        console.error(`Error fetching price for ${symbol}:`, error);
        throw error;
    }
}

/**
 * Get real-time prices for multiple cryptocurrencies
 * @param {string[]} symbols - Array of cryptocurrency symbols
 * @returns {Promise<Object[]>} Array of price data objects
 */
async function getCryptoPrices(symbols) {
    try {
        const symbolsStr = symbols.map(s => s.toUpperCase()).join(',');
        const response = await fetch(`${CMC_API_BASE}/prices?symbols=${symbolsStr}`);
        const data = await response.json();
        
        if (!data.success) {
            throw new Error(data.error || 'Failed to fetch prices');
        }
        
        return data.data;
    } catch (error) {
        console.error('Error fetching prices:', error);
        throw error;
    }
}

/**
 * Convert between two cryptocurrencies
 * @param {string} from - Source cryptocurrency symbol
 * @param {string} to - Target cryptocurrency symbol
 * @param {number|string} amount - Amount to convert
 * @returns {Promise<Object>} Conversion result with rate and converted amount
 */
async function convertCrypto(from, to, amount) {
    try {
        const response = await fetch(
            `${CMC_API_BASE}/convert?from=${from.toUpperCase()}&to=${to.toUpperCase()}&amount=${amount}`
        );
        const data = await response.json();
        
        if (!data.success) {
            throw new Error(data.error || 'Failed to convert');
        }
        
        return data.data;
    } catch (error) {
        console.error(`Error converting ${from} to ${to}:`, error);
        throw error;
    }
}

/**
 * Format price with appropriate decimal places
 * @param {number} price - Price to format
 * @returns {string} Formatted price string
 */
function formatPrice(price) {
    if (price >= 1000) {
        return price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    } else if (price >= 1) {
        return price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 4 });
    } else {
        return price.toLocaleString('en-US', { minimumFractionDigits: 4, maximumFractionDigits: 8 });
    }
}

/**
 * Format percentage change with color
 * @param {number} change - Percentage change
 * @returns {Object} Object with formatted text and color class
 */
function formatPriceChange(change) {
    const formatted = change >= 0 ? `+${change.toFixed(2)}%` : `${change.toFixed(2)}%`;
    const colorClass = change >= 0 ? 'positive' : 'negative';
    return { text: formatted, colorClass };
}

/**
 * Get price for a token with caching
 * Cache prices for 60 seconds to reduce API calls
 */
const priceCache = new Map();
const CACHE_DURATION = 60000; // 60 seconds

async function getCachedPrice(symbol) {
    const now = Date.now();
    const cached = priceCache.get(symbol);
    
    if (cached && (now - cached.timestamp) < CACHE_DURATION) {
        return cached.data;
    }
    
    const priceData = await getCryptoPrice(symbol);
    priceCache.set(symbol, { data: priceData, timestamp: now });
    
    return priceData;
}

/**
 * Get multiple prices with caching
 */
async function getCachedPrices(symbols) {
    const now = Date.now();
    const uncachedSymbols = [];
    const results = {};
    
    // Check cache first
    for (const symbol of symbols) {
        const cached = priceCache.get(symbol);
        if (cached && (now - cached.timestamp) < CACHE_DURATION) {
            results[symbol] = cached.data;
        } else {
            uncachedSymbols.push(symbol);
        }
    }
    
    // Fetch uncached prices
    if (uncachedSymbols.length > 0) {
        const freshPrices = await getCryptoPrices(uncachedSymbols);
        for (const priceData of freshPrices) {
            priceCache.set(priceData.symbol, { data: priceData, timestamp: now });
            results[priceData.symbol] = priceData;
        }
    }
    
    return Object.values(results);
}

/**
 * Update price display element
 * @param {string} elementId - ID of element to update
 * @param {string} symbol - Cryptocurrency symbol
 */
async function updatePriceDisplay(elementId, symbol) {
    try {
        const priceData = await getCachedPrice(symbol);
        const element = document.getElementById(elementId);
        
        if (element) {
            element.textContent = `$${formatPrice(parseFloat(priceData.price_usd))}`;
            
            // Add 24h change if available
            if (priceData.price_change_24h) {
                const change = formatPriceChange(parseFloat(priceData.price_change_24h));
                const changeSpan = document.createElement('span');
                changeSpan.className = `price-change ${change.colorClass}`;
                changeSpan.textContent = ` (${change.text})`;
                element.appendChild(changeSpan);
            }
        }
    } catch (error) {
        console.error(`Error updating price display for ${symbol}:`, error);
        const element = document.getElementById(elementId);
        if (element) {
            element.textContent = 'Price unavailable';
        }
    }
}

/**
 * Start auto-refresh for price displays
 * @param {Object[]} priceElements - Array of {elementId, symbol} objects
 * @param {number} intervalMs - Refresh interval in milliseconds (default: 60000 = 1 minute)
 */
function startPriceAutoRefresh(priceElements, intervalMs = 60000) {
    // Initial update
    priceElements.forEach(({ elementId, symbol }) => {
        updatePriceDisplay(elementId, symbol);
    });
    
    // Set up interval
    return setInterval(() => {
        priceElements.forEach(({ elementId, symbol }) => {
            updatePriceDisplay(elementId, symbol);
        });
    }, intervalMs);
}

// Export functions for use in other scripts
window.CMC = {
    getCryptoPrice,
    getCryptoPrices,
    convertCrypto,
    formatPrice,
    formatPriceChange,
    getCachedPrice,
    getCachedPrices,
    updatePriceDisplay,
    startPriceAutoRefresh
};
