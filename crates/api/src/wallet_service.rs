use blockchain::{SolanaClient, WalletBalance};
use chrono::Utc;
use database::{DbPool, RedisPool};
use shared::{models::*, Error, PriceFeedService, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::portfolio_cache::PortfolioCache;
use crate::birdeye_service::{BirdeyeService, Blockchain, WalletAddress};

/// Wallet service for managing user wallet connections and portfolio data
/// Now supports multiple blockchains via Birdeye integration
/// 
/// # Multi-Chain Support
/// 
/// The wallet service now supports connecting wallets from multiple blockchains:
/// - Solana (native support via SolanaClient)
/// - Ethereum (via Birdeye API)
/// - Binance Smart Chain (via Birdeye API)
/// - Polygon (via Birdeye API)
/// 
/// # Usage Examples
/// 
/// ## Connect a Solana wallet (legacy method)
/// ```rust,ignore
/// let portfolio = wallet_service.connect_wallet(wallet_address, user_id).await?;
/// ```
/// 
/// ## Connect a multi-chain wallet
/// ```rust,ignore
/// let portfolio = wallet_service
///     .connect_multi_chain_wallet(wallet_address, Blockchain::Ethereum, user_id)
///     .await?;
/// ```
/// 
/// ## Get aggregated portfolio across all chains
/// ```rust,ignore
/// let portfolio = wallet_service.get_aggregated_portfolio(user_id).await?;
/// ```
/// 
/// **Validates: Requirements 1.1, 1.2, 1.6**
pub struct WalletService {
    solana_client: Arc<SolanaClient>,
    db_pool: DbPool,
    price_feed: PriceFeedService,
    cache: PortfolioCache,
    birdeye_service: Option<Arc<BirdeyeService>>,
}

impl WalletService {
    /// Create a new wallet service
    pub fn new(
        solana_client: Arc<SolanaClient>,
        db_pool: DbPool,
        redis_pool: RedisPool,
    ) -> Self {
        let price_feed = PriceFeedService::new();
        let cache = PortfolioCache::new(redis_pool);
        
        Self {
            solana_client,
            db_pool,
            price_feed,
            cache,
            birdeye_service: None,
        }
    }

    /// Create a new wallet service with Birdeye integration for multi-chain support
    /// 
    /// **Validates: Requirements 1.1, 1.2, 1.6**
    pub fn new_with_birdeye(
        solana_client: Arc<SolanaClient>,
        db_pool: DbPool,
        redis_pool: RedisPool,
        birdeye_service: Arc<BirdeyeService>,
    ) -> Self {
        let price_feed = PriceFeedService::new();
        let cache = PortfolioCache::new(redis_pool);
        
        Self {
            solana_client,
            db_pool,
            price_feed,
            cache,
            birdeye_service: Some(birdeye_service),
        }
    }

    /// Connect a user's Solana wallet and retrieve portfolio
    /// 
    /// This validates the wallet address, retrieves the portfolio from the blockchain,
    /// stores the wallet connection in the database, and persists portfolio assets.
    /// 
    /// **Validates: Requirements 1.1, 1.2, 1.3**
    pub async fn connect_wallet(&self, wallet_address: &str, user_id: Uuid) -> Result<Portfolio> {
        info!(
            "Connecting wallet {} for user {}",
            wallet_address, user_id
        );

        // Validate wallet address format (Requirement 1.2)
        self.solana_client
            .validate_address(wallet_address)
            .map_err(|e| {
                warn!("Invalid wallet address: {}", e);
                e
            })?;

        // Retrieve portfolio from blockchain (Requirement 1.1)
        let wallet_balance = self
            .solana_client
            .get_wallet_balance(wallet_address)
            .await?;

        // Store wallet connection in database (Requirement 1.3)
        let wallet = self.store_wallet_connection(user_id, wallet_address).await?;

        // Store portfolio assets in database
        let portfolio = self
            .store_portfolio_assets(&wallet, &wallet_balance)
            .await?;

        info!(
            "Successfully connected wallet {} with {} assets",
            wallet_address,
            portfolio.assets.len()
        );

        Ok(portfolio)
    }

    /// Get current portfolio holdings for a wallet
    /// 
    /// **Validates: Requirements 1.4, 1.5**
    pub async fn get_portfolio(&self, wallet_address: &str) -> Result<Portfolio> {
        debug!("Fetching portfolio for wallet: {}", wallet_address);

        // Validate address
        self.solana_client.validate_address(wallet_address)?;

        // Check cache first (60-second TTL)
        if let Some(cached_portfolio) = self.cache.get(wallet_address).await? {
            debug!("Returning cached portfolio for wallet: {}", wallet_address);
            return Ok(cached_portfolio);
        }

        // Cache miss - fetch from database
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let wallet_row = client
            .query_one(
                "SELECT id, user_id, address, connected_at, last_synced FROM wallets WHERE address = $1",
                &[&wallet_address],
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("no rows") {
                    Error::WalletNotFound(wallet_address.to_string())
                } else {
                    Error::Database(format!("Failed to query wallet: {}", e))
                }
            })?;

        let wallet_id: Uuid = wallet_row.get(0);

        // Get portfolio assets from database
        let asset_rows = client
            .query(
                "SELECT token_mint, token_symbol, amount, value_usd 
                 FROM portfolio_assets 
                 WHERE wallet_id = $1",
                &[&wallet_id],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to query portfolio assets: {}", e)))?;

        let mut assets = Vec::new();
        let mut total_value_usd = 0.0;

        for row in asset_rows {
            let token_mint: String = row.get(0);
            let token_symbol: String = row.get(1);
            let amount: rust_decimal::Decimal = row.get(2);
            let value_usd: Option<rust_decimal::Decimal> = row.get(3);

            let value_usd_f64 = value_usd.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0));
            
            if let Some(value) = value_usd_f64 {
                total_value_usd += value;
            }

            assets.push(Asset {
                token_mint,
                token_symbol,
                amount: amount.to_string(),
                value_usd: value_usd_f64,
            });
        }

        let portfolio = Portfolio {
            wallet_address: wallet_address.to_string(),
            assets,
            total_value_usd,
            last_updated: Utc::now(),
        };

        // Cache the portfolio with 60-second TTL
        self.cache.set(wallet_address, &portfolio).await?;

        Ok(portfolio)
    }

    /// Validate a Solana wallet address format
    /// 
    /// **Validates: Requirements 1.2**
    pub fn validate_wallet_address(&self, address: &str) -> Result<()> {
        self.solana_client.validate_address(address)?;
        Ok(())
    }

    /// Connect a multi-chain wallet and retrieve portfolio via Birdeye
    /// 
    /// **Validates: Requirements 1.1, 1.2, 1.6**
    pub async fn connect_multi_chain_wallet(
        &self,
        wallet_address: &str,
        blockchain: Blockchain,
        user_id: Uuid,
    ) -> Result<Portfolio> {
        info!(
            "Connecting {:?} wallet {} for user {}",
            blockchain, wallet_address, user_id
        );

        let birdeye = self.birdeye_service.as_ref().ok_or_else(|| {
            Error::Internal("Birdeye service not configured for multi-chain support".to_string())
        })?;

        // Fetch portfolio from Birdeye
        let wallet_addresses = vec![WalletAddress {
            blockchain: blockchain.clone(),
            address: wallet_address.to_string(),
        }];

        let multi_chain_portfolio = birdeye
            .get_multi_chain_portfolio(wallet_addresses)
            .await
            .map_err(|e| Error::ExternalService(format!("Birdeye API error: {}", e)))?;

        // Store wallet connection in database
        let wallet = self
            .store_multi_chain_wallet_connection(user_id, wallet_address, &blockchain)
            .await?;

        // Convert Birdeye assets to internal Portfolio format
        let assets: Vec<Asset> = multi_chain_portfolio
            .positions_by_chain
            .values()
            .flatten()
            .map(|birdeye_asset| Asset {
                token_mint: birdeye_asset.address.clone(),
                token_symbol: birdeye_asset.symbol.clone(),
                amount: birdeye_asset.balance.to_string(),
                value_usd: Some(
                    birdeye_asset
                        .value_usd
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
            })
            .collect();

        let total_value_usd = multi_chain_portfolio
            .total_value_usd
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

        // Store portfolio assets
        self.store_multi_chain_portfolio_assets(&wallet, &assets)
            .await?;

        let portfolio = Portfolio {
            wallet_address: wallet_address.to_string(),
            assets,
            total_value_usd,
            last_updated: Utc::now(),
        };

        info!(
            "Successfully connected {:?} wallet {} with {} assets",
            blockchain,
            wallet_address,
            portfolio.assets.len()
        );

        Ok(portfolio)
    }

    /// Get portfolio for a multi-chain wallet via Birdeye
    /// 
    /// **Validates: Requirements 1.6**
    pub async fn get_multi_chain_portfolio(
        &self,
        wallet_address: &str,
        blockchain: Blockchain,
    ) -> Result<Portfolio> {
        debug!(
            "Fetching {:?} portfolio for wallet: {}",
            blockchain, wallet_address
        );

        let birdeye = self.birdeye_service.as_ref().ok_or_else(|| {
            Error::Internal("Birdeye service not configured for multi-chain support".to_string())
        })?;

        // Check cache first
        let cache_key = format!("{}:{:?}", wallet_address, blockchain);
        if let Some(cached_portfolio) = self.cache.get(&cache_key).await? {
            debug!(
                "Returning cached portfolio for {:?} wallet: {}",
                blockchain, wallet_address
            );
            return Ok(cached_portfolio);
        }

        // Fetch from Birdeye
        let wallet_addresses = vec![WalletAddress {
            blockchain: blockchain.clone(),
            address: wallet_address.to_string(),
        }];

        let multi_chain_portfolio = birdeye
            .get_multi_chain_portfolio(wallet_addresses)
            .await
            .map_err(|e| Error::ExternalService(format!("Birdeye API error: {}", e)))?;

        // Convert to internal format
        let assets: Vec<Asset> = multi_chain_portfolio
            .positions_by_chain
            .values()
            .flatten()
            .map(|birdeye_asset| Asset {
                token_mint: birdeye_asset.address.clone(),
                token_symbol: birdeye_asset.symbol.clone(),
                amount: birdeye_asset.balance.to_string(),
                value_usd: Some(
                    birdeye_asset
                        .value_usd
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
            })
            .collect();

        let total_value_usd = multi_chain_portfolio
            .total_value_usd
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

        let portfolio = Portfolio {
            wallet_address: wallet_address.to_string(),
            assets,
            total_value_usd,
            last_updated: Utc::now(),
        };

        // Cache the portfolio
        self.cache.set(&cache_key, &portfolio).await?;

        Ok(portfolio)
    }

    /// Get aggregated portfolio across all connected wallets (multi-chain)
    /// 
    /// **Validates: Requirements 1.6**
    pub async fn get_aggregated_portfolio(&self, user_id: Uuid) -> Result<Portfolio> {
        debug!("Fetching aggregated portfolio for user: {}", user_id);

        let birdeye = self.birdeye_service.as_ref().ok_or_else(|| {
            Error::Internal("Birdeye service not configured for multi-chain support".to_string())
        })?;

        // Get all wallets for user from database
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let wallet_rows = client
            .query(
                "SELECT address, blockchain FROM multi_chain_wallets WHERE user_id = $1",
                &[&user_id],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to query wallets: {}", e)))?;

        if wallet_rows.is_empty() {
            return Ok(Portfolio {
                wallet_address: "aggregated".to_string(),
                assets: vec![],
                total_value_usd: 0.0,
                last_updated: Utc::now(),
            });
        }

        // Build wallet addresses for Birdeye
        let wallet_addresses: Vec<WalletAddress> = wallet_rows
            .iter()
            .map(|row| {
                let address: String = row.get(0);
                let blockchain_str: String = row.get(1);
                let blockchain = match blockchain_str.as_str() {
                    "Solana" => Blockchain::Solana,
                    "Ethereum" => Blockchain::Ethereum,
                    "BinanceSmartChain" => Blockchain::BinanceSmartChain,
                    "Polygon" => Blockchain::Polygon,
                    _ => Blockchain::Solana, // Default fallback
                };
                WalletAddress {
                    blockchain,
                    address,
                }
            })
            .collect();

        // Fetch aggregated portfolio from Birdeye
        let multi_chain_portfolio = birdeye
            .get_multi_chain_portfolio(wallet_addresses)
            .await
            .map_err(|e| Error::ExternalService(format!("Birdeye API error: {}", e)))?;

        // Convert to internal format
        let assets: Vec<Asset> = multi_chain_portfolio
            .positions_by_chain
            .values()
            .flatten()
            .map(|birdeye_asset| Asset {
                token_mint: birdeye_asset.address.clone(),
                token_symbol: birdeye_asset.symbol.clone(),
                amount: birdeye_asset.balance.to_string(),
                value_usd: Some(
                    birdeye_asset
                        .value_usd
                        .to_string()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
            })
            .collect();

        let total_value_usd = multi_chain_portfolio
            .total_value_usd
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

        Ok(Portfolio {
            wallet_address: "aggregated".to_string(),
            assets,
            total_value_usd,
            last_updated: Utc::now(),
        })
    }

    /// Refresh portfolio data from blockchain
    /// 
    /// **Validates: Requirements 1.5**
    pub async fn refresh_portfolio(&self, wallet_address: &str) -> Result<Portfolio> {
        info!("Refreshing portfolio for wallet: {}", wallet_address);

        // Validate address
        self.solana_client.validate_address(wallet_address)?;

        // Invalidate cache
        self.cache.invalidate(wallet_address).await?;

        // Get wallet from database
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let wallet_row = client
            .query_one(
                "SELECT id, user_id FROM wallets WHERE address = $1",
                &[&wallet_address],
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("no rows") {
                    Error::WalletNotFound(wallet_address.to_string())
                } else {
                    Error::Database(format!("Failed to query wallet: {}", e))
                }
            })?;

        let wallet_id: Uuid = wallet_row.get(0);

        // Retrieve fresh data from blockchain
        let wallet_balance = self
            .solana_client
            .get_wallet_balance(wallet_address)
            .await?;

        // Update last_synced timestamp
        client
            .execute(
                "UPDATE wallets SET last_synced = NOW() WHERE id = $1",
                &[&wallet_id],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to update wallet sync time: {}", e)))?;

        // Update portfolio assets
        let wallet = Wallet {
            id: wallet_id,
            user_id: wallet_row.get(1),
            address: wallet_address.to_string(),
            connected_at: Utc::now(),
            last_synced: Some(Utc::now()),
        };

        let portfolio = self
            .store_portfolio_assets(&wallet, &wallet_balance)
            .await?;

        info!(
            "Successfully refreshed portfolio for wallet {} with {} assets",
            wallet_address,
            portfolio.assets.len()
        );

        Ok(portfolio)
    }

    /// Store wallet connection in database
    async fn store_wallet_connection(&self, user_id: Uuid, address: &str) -> Result<Wallet> {
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        // Insert or update wallet connection
        let row = client
            .query_one(
                "INSERT INTO wallets (user_id, address, connected_at, last_synced)
                 VALUES ($1, $2, NOW(), NOW())
                 ON CONFLICT (user_id, address) 
                 DO UPDATE SET last_synced = NOW()
                 RETURNING id, user_id, address, connected_at, last_synced",
                &[&user_id, &address],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to insert wallet: {}", e)))?;

        Ok(Wallet {
            id: row.get(0),
            user_id: row.get(1),
            address: row.get(2),
            connected_at: row.get(3),
            last_synced: row.get(4),
        })
    }

    /// Store multi-chain wallet connection in database
    /// 
    /// **Validates: Requirements 1.6**
    async fn store_multi_chain_wallet_connection(
        &self,
        user_id: Uuid,
        address: &str,
        blockchain: &Blockchain,
    ) -> Result<Wallet> {
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let blockchain_str = format!("{:?}", blockchain);

        // Insert or update multi-chain wallet connection
        let row = client
            .query_one(
                "INSERT INTO multi_chain_wallets (user_id, blockchain, address, is_primary, created_at)
                 VALUES ($1, $2, $3, FALSE, NOW())
                 ON CONFLICT (blockchain, address) 
                 DO UPDATE SET user_id = $1
                 RETURNING id, user_id, address",
                &[&user_id, &blockchain_str, &address],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to insert multi-chain wallet: {}", e)))?;

        Ok(Wallet {
            id: row.get(0),
            user_id: row.get(1),
            address: row.get(2),
            connected_at: Utc::now(),
            last_synced: Some(Utc::now()),
        })
    }

    /// Store multi-chain portfolio assets in database
    async fn store_multi_chain_portfolio_assets(
        &self,
        wallet: &Wallet,
        assets: &[Asset],
    ) -> Result<()> {
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        for asset in assets {
            let amount_decimal = rust_decimal::Decimal::from_str_exact(&asset.amount)
                .map_err(|e| Error::Internal(format!("Failed to parse asset amount: {}", e)))?;

            let value_usd_decimal = asset
                .value_usd
                .and_then(rust_decimal::Decimal::from_f64_retain);

            client
                .execute(
                    "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd, updated_at)
                     VALUES ($1, $2, $3, $4, $5, NOW())
                     ON CONFLICT (wallet_id, token_mint)
                     DO UPDATE SET amount = $4, value_usd = $5, updated_at = NOW()",
                    &[
                        &wallet.id,
                        &asset.token_mint,
                        &asset.token_symbol,
                        &amount_decimal,
                        &value_usd_decimal,
                    ],
                )
                .await
                .map_err(|e| Error::Database(format!("Failed to insert portfolio asset: {}", e)))?;
        }

        Ok(())
    }

    /// Store portfolio assets in database
    async fn store_portfolio_assets(
        &self,
        wallet: &Wallet,
        wallet_balance: &WalletBalance,
    ) -> Result<Portfolio> {
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let mut assets = Vec::new();
        let mut total_value_usd = 0.0;

        // Store SOL balance if non-zero
        if wallet_balance.sol_balance > 0 {
            let sol_amount = wallet_balance.sol_balance as f64 / 1_000_000_000.0; // Convert lamports to SOL
            let sol_amount_str = format!("{:.9}", sol_amount);
            let sol_mint = "So11111111111111111111111111111111111111112";

            // Calculate USD value using price feed
            let sol_value_usd = self.price_feed
                .calculate_usd_value(sol_mint, sol_amount)
                .await
                .unwrap_or(0.0);

            total_value_usd += sol_value_usd;

            client
                .execute(
                    "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd, updated_at)
                     VALUES ($1, $2, $3, $4, $5, NOW())
                     ON CONFLICT (wallet_id, token_mint)
                     DO UPDATE SET amount = $4, value_usd = $5, updated_at = NOW()",
                    &[
                        &wallet.id,
                        &sol_mint,
                        &"SOL",
                        &rust_decimal::Decimal::from_str_exact(&sol_amount_str)
                            .map_err(|e| Error::Internal(format!("Failed to parse SOL amount: {}", e)))?,
                        &rust_decimal::Decimal::from_f64_retain(sol_value_usd)
                            .ok_or_else(|| Error::Internal("Failed to convert SOL USD value".to_string()))?,
                    ],
                )
                .await
                .map_err(|e| Error::Database(format!("Failed to insert SOL asset: {}", e)))?;

            assets.push(Asset {
                token_mint: sol_mint.to_string(),
                token_symbol: "SOL".to_string(),
                amount: sol_amount_str,
                value_usd: Some(sol_value_usd),
            });
        }

        // Store SPL token accounts
        for token_account in &wallet_balance.token_accounts {
            if token_account.amount == 0 {
                continue; // Skip zero balances
            }

            let token_amount = token_account.amount as f64 / 10_f64.powi(token_account.decimals as i32);
            let token_amount_str = format!("{:.18}", token_amount);

            // Calculate USD value using price feed
            let token_value_usd = self.price_feed
                .calculate_usd_value(&token_account.mint, token_amount)
                .await
                .unwrap_or(0.0);

            total_value_usd += token_value_usd;

            // For now, we don't have token symbols from the blockchain
            // These would come from a token metadata service (future enhancement)
            let token_symbol = format!("TOKEN_{}", &token_account.mint[..8]);

            client
                .execute(
                    "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd, updated_at)
                     VALUES ($1, $2, $3, $4, $5, NOW())
                     ON CONFLICT (wallet_id, token_mint)
                     DO UPDATE SET amount = $4, value_usd = $5, updated_at = NOW()",
                    &[
                        &wallet.id,
                        &token_account.mint,
                        &token_symbol,
                        &rust_decimal::Decimal::from_str_exact(&token_amount_str)
                            .map_err(|e| Error::Internal(format!("Failed to parse token amount: {}", e)))?,
                        &rust_decimal::Decimal::from_f64_retain(token_value_usd)
                            .ok_or_else(|| Error::Internal("Failed to convert token USD value".to_string()))?,
                    ],
                )
                .await
                .map_err(|e| Error::Database(format!("Failed to insert token asset: {}", e)))?;

            assets.push(Asset {
                token_mint: token_account.mint.clone(),
                token_symbol,
                amount: token_amount_str,
                value_usd: Some(token_value_usd),
            });
        }

        let portfolio = Portfolio {
            wallet_address: wallet.address.clone(),
            assets,
            total_value_usd,
            last_updated: Utc::now(),
        };

        // Cache the portfolio with 60-second TTL
        self.cache.set(&wallet.address, &portfolio).await?;

        Ok(portfolio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birdeye_service::Blockchain;

    // Unit tests will validate specific scenarios
    // Property tests will be in a separate test file

    #[test]
    fn test_wallet_service_creation() {
        // This is a placeholder - actual tests require database and Solana client setup
        // Real tests will be implemented with test fixtures
    }

    #[test]
    fn test_blockchain_enum_variants() {
        // Verify all blockchain variants are supported
        let blockchains = vec![
            Blockchain::Solana,
            Blockchain::Ethereum,
            Blockchain::BinanceSmartChain,
            Blockchain::Polygon,
        ];

        for blockchain in blockchains {
            let chain_str = blockchain.to_birdeye_chain();
            assert!(!chain_str.is_empty());
        }
    }

    #[test]
    fn test_wallet_address_creation() {
        // Test WalletAddress struct creation
        let wallet = WalletAddress {
            blockchain: Blockchain::Ethereum,
            address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
        };

        assert_eq!(wallet.blockchain.to_birdeye_chain(), "ethereum");
        assert!(!wallet.address.is_empty());
    }
}
