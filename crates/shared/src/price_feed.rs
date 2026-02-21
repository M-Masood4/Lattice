use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Price feed service for fetching USD values of tokens
/// 
/// This is a simplified implementation that can be extended to use
/// real price APIs like CoinGecko, Jupiter, or Pyth Network
#[derive(Clone)]
pub struct PriceFeedService {
    // In a real implementation, this would connect to a price API
    // For now, we'll use a simple in-memory cache with mock prices
    mock_prices: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub token_mint: String,
    pub symbol: String,
    pub price_usd: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl PriceFeedService {
    /// Create a new price feed service
    pub fn new() -> Self {
        let mut mock_prices = HashMap::new();
        
        // Add some common Solana token prices (mock data)
        // In production, these would come from a real price API
        mock_prices.insert(
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            100.0,
        );
        mock_prices.insert(
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            1.0,
        );
        mock_prices.insert(
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
            1.0,
        );
        
        Self { mock_prices }
    }
    
    /// Get the USD price for a token by its mint address
    /// 
    /// **Validates: Requirements 1.4, 1.5**
    pub async fn get_token_price(&self, token_mint: &str) -> Result<f64> {
        debug!("Fetching price for token: {}", token_mint);
        
        // Check mock prices first
        if let Some(&price) = self.mock_prices.get(token_mint) {
            return Ok(price);
        }
        
        // In a real implementation, this would call an external API
        // For now, return a default price for unknown tokens
        warn!("No price found for token {}, using default", token_mint);
        Ok(0.0)
    }
    
    /// Get prices for multiple tokens at once
    /// 
    /// **Validates: Requirements 1.4, 1.5**
    pub async fn get_token_prices(&self, token_mints: &[String]) -> Result<HashMap<String, f64>> {
        let mut prices = HashMap::new();
        
        for mint in token_mints {
            match self.get_token_price(mint).await {
                Ok(price) => {
                    prices.insert(mint.clone(), price);
                }
                Err(e) => {
                    warn!("Failed to get price for {}: {}", mint, e);
                    prices.insert(mint.clone(), 0.0);
                }
            }
        }
        
        Ok(prices)
    }
    
    /// Calculate USD value for a token amount
    /// 
    /// **Validates: Requirements 1.4, 1.5**
    pub async fn calculate_usd_value(&self, token_mint: &str, amount: f64) -> Result<f64> {
        let price = self.get_token_price(token_mint).await?;
        Ok(amount * price)
    }
}

impl Default for PriceFeedService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_sol_price() {
        let service = PriceFeedService::new();
        let price = service.get_token_price("So11111111111111111111111111111111111111112").await;
        assert!(price.is_ok());
        assert_eq!(price.unwrap(), 100.0);
    }
    
    #[tokio::test]
    async fn test_get_unknown_token_price() {
        let service = PriceFeedService::new();
        let price = service.get_token_price("UnknownToken123").await;
        assert!(price.is_ok());
        assert_eq!(price.unwrap(), 0.0);
    }
    
    #[tokio::test]
    async fn test_calculate_usd_value() {
        let service = PriceFeedService::new();
        let value = service.calculate_usd_value(
            "So11111111111111111111111111111111111111112",
            10.0
        ).await;
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), 1000.0); // 10 SOL * $100
    }
    
    #[tokio::test]
    async fn test_get_multiple_prices() {
        let service = PriceFeedService::new();
        let mints = vec![
            "So11111111111111111111111111111111111111112".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        ];
        let prices = service.get_token_prices(&mints).await;
        assert!(prices.is_ok());
        let prices = prices.unwrap();
        assert_eq!(prices.len(), 2);
        assert_eq!(prices.get(&mints[0]), Some(&100.0));
        assert_eq!(prices.get(&mints[1]), Some(&1.0));
    }
}
