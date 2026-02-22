use blockchain::{SolanaClient, WalletBalance};
use chrono::Utc;
use database::{DbPool, RedisPool};
use shared::{models::*, Error, PriceFeedService, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::portfolio_cache::PortfolioCache;
use crate::tantum_client::TantumClient;

/// Wallet service for managing user wallet connections and portfolio data
/// 
/// # Solana Support
/// 
/// The wallet service supports connecting Solana wallets:
/// - Solana (native support via SolanaClient and Tantum API)
/// 
/// # Usage Examples
/// 
/// ## Connect a Solana wallet
/// ```rust,ignore
/// let portfolio = wallet_service.connect_wallet(wallet_address, user_id).await?;
/// ```
/// 
/// **Validates: Requirements 1.1, 1.2, 1.6**
pub struct WalletService {
    solana_client: Arc<SolanaClient>,
    tantum_client: Option<Arc<TantumClient>>,
    db_pool: DbPool,
    price_feed: PriceFeedService,
    cache: PortfolioCache,
    use_mainnet: bool,
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
            tantum_client: None,
            db_pool,
            price_feed,
            cache,
            use_mainnet: false,
        }
    }

    /// Create a new wallet service with Tantum API integration
    pub fn new_with_tantum(
        solana_client: Arc<SolanaClient>,
        tantum_client: Arc<TantumClient>,
        db_pool: DbPool,
        redis_pool: RedisPool,
        use_mainnet: bool,
    ) -> Self {
        let price_feed = PriceFeedService::new();
        let cache = PortfolioCache::new(redis_pool);
        
        Self {
            solana_client,
            tantum_client: Some(tantum_client),
            db_pool,
            price_feed,
            cache,
            use_mainnet,
        }
    }

    /// Connect a user's Solana wallet and retrieve portfolio
    /// 
    /// This validates the wallet address, retrieves the portfolio from Tantum API (if available)
    /// or blockchain, stores the wallet connection in the database, and persists portfolio assets.
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

        // Store wallet connection in database (Requirement 1.3)
        let wallet = self.store_wallet_connection(user_id, wallet_address).await?;

        // Try to use Tantum API if available, otherwise fall back to blockchain client
        let portfolio = if let Some(tantum_client) = &self.tantum_client {
            info!("Using Tantum API to fetch wallet data");
            
            match tantum_client.get_wallet_info(wallet_address, self.use_mainnet).await {
                Ok(tantum_info) => {
                    info!(
                        "Successfully fetched wallet data from Tantum: {} SOL, {} tokens, ${:.2} total",
                        tantum_info.balance.sol,
                        tantum_info.tokens.len(),
                        tantum_info.total_value_usd
                    );
                    
                    self.store_portfolio_assets_from_tantum(&wallet, &tantum_info).await?
                }
                Err(e) => {
                    warn!("Tantum API failed, falling back to blockchain client: {}", e);
                    
                    // Fallback to blockchain client
                    let wallet_balance = self
                        .solana_client
                        .get_wallet_balance(wallet_address)
                        .await?;
                    
                    self.store_portfolio_assets(&wallet, &wallet_balance).await?
                }
            }
        } else {
            info!("Tantum API not configured, using blockchain client");
            
            // Retrieve portfolio from blockchain (Requirement 1.1)
            let wallet_balance = self
                .solana_client
                .get_wallet_balance(wallet_address)
                .await?;

            // Store portfolio assets in database
            self.store_portfolio_assets(&wallet, &wallet_balance).await?
        };

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
                "SELECT id, user_id, address, connected_at, last_synced 
                 FROM wallets 
                 WHERE address = $1 
                 ORDER BY last_synced DESC NULLS LAST, connected_at DESC 
                 LIMIT 1",
                &[&wallet_address],
            )
            .await
            .map_err(|e| {
                let error_str = e.to_string();
                debug!("Database error when querying wallet: {}", error_str);
                // Check for both "no rows" and "unexpected number of rows" (which can mean 0 rows)
                if error_str.contains("no rows") || error_str.contains("unexpected number of rows") {
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

    // NOTE: Multi-chain wallet methods have been disabled as they require Birdeye API
    // which has been removed in favor of CoinMarketCap. CoinMarketCap does not provide
    // multi-chain portfolio aggregation. Consider implementing these using individual
    // blockchain clients instead.
    
    /*
    /// Connect a multi-chain wallet and retrieve portfolio via Birdeye
    /// 
    /// **Validates: Requirements 1.1, 1.2, 1.6**
    pub async fn connect_multi_chain_wallet(
        &self,
        wallet_address: &str,
        blockchain: Blockchain,
        user_id: Uuid,
    ) -> Result<Portfolio> {
        Err(Error::Internal("Multi-chain wallet support has been disabled. Birdeye API removed.".to_string()))
    }

    /// Get portfolio for a multi-chain wallet via Birdeye
    /// 
    /// **Validates: Requirements 1.6**
    pub async fn get_multi_chain_portfolio(
        &self,
        wallet_address: &str,
        blockchain: Blockchain,
    ) -> Result<Portfolio> {
        Err(Error::Internal("Multi-chain portfolio support has been disabled. Birdeye API removed.".to_string()))
    }

    /// Get aggregated portfolio across all connected wallets (multi-chain)
    /// 
    /// **Validates: Requirements 1.6**
    pub async fn get_aggregated_portfolio(&self, user_id: Uuid) -> Result<Portfolio> {
        Err(Error::Internal("Aggregated portfolio support has been disabled. Birdeye API removed.".to_string()))
    }
    */

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
                "SELECT id, user_id FROM wallets WHERE address = $1 
                 ORDER BY last_synced DESC NULLS LAST, connected_at DESC 
                 LIMIT 1",
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
        info!("Storing wallet connection for user {} and address {}", user_id, address);
        
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        // Insert or update wallet connection
        // Note: We use ON CONFLICT (address) because there's a unique constraint on address alone
        // This means the same wallet can only be connected once, regardless of user
        let row = client
            .query_one(
                "INSERT INTO wallets (user_id, address, connected_at, last_synced)
                 VALUES ($1, $2, NOW(), NOW())
                 ON CONFLICT (address) 
                 DO UPDATE SET user_id = EXCLUDED.user_id, last_synced = NOW()
                 RETURNING id, user_id, address, connected_at, last_synced",
                &[&user_id, &address],
            )
            .await
            .map_err(|e| {
                warn!("Failed to insert/update wallet: {}", e);
                Error::Database(format!("Failed to insert wallet: {}", e))
            })?;

        info!("Successfully stored wallet connection with id {}", row.get::<_, Uuid>(0));

        Ok(Wallet {
            id: row.get(0),
            user_id: row.get(1),
            address: row.get(2),
            connected_at: row.get(3),
            last_synced: row.get(4),
        })
    }

    /// Store portfolio assets from Tantum API data
    /// 
    /// This method takes Tantum wallet info and stores it in the database
    /// **Validates: Requirements 1.1, 1.3**
    pub async fn store_portfolio_assets_from_tantum(
        &self,
        wallet: &Wallet,
        tantum_info: &crate::tantum_client::TantumWalletInfo,
    ) -> Result<Portfolio> {
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let mut assets = Vec::new();
        let total_value_usd = tantum_info.total_value_usd;

        // Store SOL balance if non-zero
        if tantum_info.balance.sol > 0.0 {
            let sol_mint = "So11111111111111111111111111111111111111112";
            let sol_amount_str = format!("{:.9}", tantum_info.balance.sol);

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
                        &rust_decimal::Decimal::from_f64_retain(tantum_info.balance.sol_usd)
                            .ok_or_else(|| Error::Internal("Failed to convert SOL USD value".to_string()))?,
                    ],
                )
                .await
                .map_err(|e| Error::Database(format!("Failed to insert SOL asset: {}", e)))?;

            assets.push(Asset {
                token_mint: sol_mint.to_string(),
                token_symbol: "SOL".to_string(),
                amount: sol_amount_str,
                value_usd: Some(tantum_info.balance.sol_usd),
            });
        }

        // Store SPL tokens from Tantum
        for token in &tantum_info.tokens {
            if token.amount == 0.0 {
                continue; // Skip zero balances
            }

            let token_amount_str = format!("{:.18}", token.amount);

            client
                .execute(
                    "INSERT INTO portfolio_assets (wallet_id, token_mint, token_symbol, amount, value_usd, updated_at)
                     VALUES ($1, $2, $3, $4, $5, NOW())
                     ON CONFLICT (wallet_id, token_mint)
                     DO UPDATE SET amount = $4, value_usd = $5, updated_at = NOW()",
                    &[
                        &wallet.id,
                        &token.mint,
                        &token.symbol,
                        &rust_decimal::Decimal::from_str_exact(&token_amount_str)
                            .map_err(|e| Error::Internal(format!("Failed to parse token amount: {}", e)))?,
                        &rust_decimal::Decimal::from_f64_retain(token.value_usd.unwrap_or(0.0))
                            .ok_or_else(|| Error::Internal("Failed to convert token USD value".to_string()))?,
                    ],
                )
                .await
                .map_err(|e| Error::Database(format!("Failed to insert token asset: {}", e)))?;

            assets.push(Asset {
                token_mint: token.mint.clone(),
                token_symbol: token.symbol.clone(),
                amount: token_amount_str,
                value_usd: token.value_usd,
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

    /*
    /// Store multi-chain wallet connection in database
    /// 
    /// **Validates: Requirements 1.6**
    /// NOTE: Disabled - requires Birdeye API
    async fn store_multi_chain_wallet_connection(
        &self,
        user_id: Uuid,
        address: &str,
        blockchain: &Blockchain,
    ) -> Result<Wallet> {
        Err(Error::Internal("Multi-chain support disabled".to_string()))
    }

    /// Store multi-chain portfolio assets in database
    /// NOTE: Disabled - requires Birdeye API
    async fn store_multi_chain_portfolio_assets(
        &self,
        wallet: &Wallet,
        assets: &[Asset],
    ) -> Result<()> {
        Err(Error::Internal("Multi-chain support disabled".to_string()))
    }
    */

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

    /// Generate a stealth meta-address for receiving private payments
    /// 
    /// This method creates a new stealth key pair and returns the meta-address
    /// that can be shared with senders. The meta-address format is:
    /// `stealth:1:<spending_pk>:<viewing_pk>`
    /// 
    /// # Arguments
    /// * `user_id` - The user ID to associate with this stealth wallet
    /// 
    /// # Requirements
    /// Validates: Requirements 10.2
    /// 
    /// # Returns
    /// The stealth meta-address string
    pub async fn generate_stealth_meta_address(&self, user_id: Uuid) -> Result<String> {
        use stealth::keypair::StealthKeyPair;
        
        info!("Generating stealth meta-address for user {}", user_id);
        
        // Generate new stealth key pair
        let keypair = StealthKeyPair::generate_standard()
            .map_err(|e| Error::Internal(format!("Failed to generate stealth keypair: {}", e)))?;
        
        let meta_address = keypair.to_meta_address();
        
        // Store the keypair in the database (encrypted)
        // Note: In production, this should use secure key storage (iOS Keychain, Android Keystore)
        // For now, we'll store it in the database with encryption
        let client = self.db_pool.get().await
            .map_err(|e| Error::Database(format!("Failed to get database connection: {}", e)))?;
        
        // Export encrypted keypair
        let password = format!("user_{}_stealth", user_id); // In production, use proper key derivation
        let encrypted_keypair = keypair.export_encrypted(&password)
            .map_err(|e| Error::Internal(format!("Failed to encrypt keypair: {}", e)))?;
        
        client
            .execute(
                "INSERT INTO stealth_wallets (user_id, meta_address, encrypted_keypair, created_at)
                 VALUES ($1, $2, $3, NOW())
                 ON CONFLICT (user_id) DO UPDATE SET meta_address = $2, encrypted_keypair = $3, updated_at = NOW()",
                &[&user_id, &meta_address, &encrypted_keypair],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to store stealth wallet: {}", e)))?;
        
        info!("Generated stealth meta-address: {}", meta_address);
        
        Ok(meta_address)
    }

    /// Prepare a stealth payment to a receiver
    /// 
    /// This method generates a one-time stealth address for the receiver using their
    /// meta-address. The stealth address can then be used to send a private payment.
    /// 
    /// # Arguments
    /// * `receiver_meta_address` - The receiver's stealth meta-address
    /// * `amount` - Amount in lamports to send
    /// 
    /// # Requirements
    /// Validates: Requirements 10.2
    /// 
    /// # Returns
    /// PreparedStealthPayment containing the stealth address and metadata
    pub async fn prepare_stealth_payment(
        &self,
        receiver_meta_address: &str,
        amount: u64,
    ) -> Result<PreparedStealthPayment> {
        use stealth::generator::StealthAddressGenerator;
        
        info!(
            "Preparing stealth payment of {} lamports to {}",
            amount, receiver_meta_address
        );
        
        // Generate stealth address
        let stealth_output = StealthAddressGenerator::generate_stealth_address_uncached(
            receiver_meta_address,
            None, // Generate random ephemeral key
        )
        .map_err(|e| Error::Internal(format!("Failed to generate stealth address: {}", e)))?;
        
        Ok(PreparedStealthPayment {
            stealth_address: stealth_output.stealth_address.to_string(),
            amount,
            ephemeral_public_key: stealth_output.ephemeral_public_key.to_string(),
            viewing_tag: stealth_output.viewing_tag,
        })
    }

    /// Send a stealth payment
    /// 
    /// This method submits a stealth payment transaction to the blockchain.
    /// The transaction includes the payment transfer and stealth metadata.
    /// 
    /// # Arguments
    /// * `payer_keypair` - The keypair paying for the transaction
    /// * `prepared` - The prepared stealth payment
    /// 
    /// # Requirements
    /// Validates: Requirements 10.2
    /// 
    /// # Returns
    /// Transaction signature on success
    pub async fn send_stealth_payment(
        &self,
        payer_keypair: &solana_sdk::signature::Keypair,
        prepared: &PreparedStealthPayment,
    ) -> Result<String> {
        use solana_sdk::pubkey::Pubkey;
        use std::str::FromStr;
        
        info!(
            "Sending stealth payment of {} lamports to {}",
            prepared.amount, prepared.stealth_address
        );
        
        // Parse addresses
        let stealth_address = Pubkey::from_str(&prepared.stealth_address)
            .map_err(|e| Error::InvalidWalletAddress(format!("Invalid stealth address: {}", e)))?;
        
        let ephemeral_public_key = Pubkey::from_str(&prepared.ephemeral_public_key)
            .map_err(|e| Error::Internal(format!("Invalid ephemeral key: {}", e)))?;
        
        // Submit transaction using blockchain client
        let signature = self.solana_client
            .submit_stealth_payment(
                payer_keypair,
                &stealth_address,
                prepared.amount,
                &ephemeral_public_key,
                &prepared.viewing_tag,
                1, // version 1 (standard mode)
            )
            .await?;
        
        info!("Stealth payment sent. Signature: {}", signature);
        
        Ok(signature.to_string())
    }

    /// Scan for incoming stealth payments
    /// 
    /// This method scans the blockchain for stealth payments sent to the user's
    /// stealth wallet. It uses the viewing key to detect payments without exposing
    /// the spending key.
    /// 
    /// # Arguments
    /// * `user_id` - The user ID to scan for
    /// * `from_slot` - Optional starting slot (defaults to last scanned slot)
    /// * `to_slot` - Optional ending slot (defaults to current slot)
    /// 
    /// # Requirements
    /// Validates: Requirements 10.2
    /// 
    /// # Returns
    /// Vector of detected stealth payments
    pub async fn scan_stealth_payments(
        &self,
        user_id: Uuid,
        from_slot: Option<u64>,
        to_slot: Option<u64>,
    ) -> Result<Vec<DetectedStealthPayment>> {
        use stealth::keypair::StealthKeyPair;
        use stealth::scanner::StealthScanner;
        
        info!("Scanning stealth payments for user {}", user_id);
        
        // Load user's stealth wallet from database
        let client = self.db_pool.get().await
            .map_err(|e| Error::Database(format!("Failed to get database connection: {}", e)))?;
        
        let row = client
            .query_one(
                "SELECT meta_address, encrypted_keypair FROM stealth_wallets WHERE user_id = $1",
                &[&user_id],
            )
            .await
            .map_err(|e| Error::Database(format!("Stealth wallet not found for user: {}", e)))?;
        
        let encrypted_keypair: Vec<u8> = row.get(1);
        
        // Decrypt keypair
        let password = format!("user_{}_stealth", user_id);
        let keypair = StealthKeyPair::import_encrypted(&encrypted_keypair, &password)
            .map_err(|e| Error::Internal(format!("Failed to decrypt keypair: {}", e)))?;
        
        // Create scanner
        let rpc_url = "https://api.devnet.solana.com"; // TODO: Use configured RPC URL
        let mut scanner = StealthScanner::new(&keypair, rpc_url);
        
        // Scan for payments
        let detected: Vec<_> = scanner.scan_for_payments(from_slot, to_slot).await
            .map_err(|e| Error::Internal(format!("Failed to scan for payments: {}", e)))?;
        
        // Convert to API response format
        let payments: Vec<DetectedStealthPayment> = detected
            .into_iter()
            .map(|p| DetectedStealthPayment {
                stealth_address: p.stealth_address.to_string(),
                amount: p.amount,
                ephemeral_public_key: p.ephemeral_public_key.to_string(),
                viewing_tag: p.viewing_tag,
                slot: p.slot,
                signature: p.signature.to_string(),
            })
            .collect();
        
        info!("Found {} stealth payments", payments.len());
        
        Ok(payments)
    }
}

/// Prepared stealth payment ready to send
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreparedStealthPayment {
    pub stealth_address: String,
    pub amount: u64,
    pub ephemeral_public_key: String,
    pub viewing_tag: [u8; 4],
}

/// Detected stealth payment from blockchain scan
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectedStealthPayment {
    pub stealth_address: String,
    pub amount: u64,
    pub ephemeral_public_key: String,
    pub viewing_tag: [u8; 4],
    pub slot: u64,
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    // NOTE: Blockchain type removed with Birdeye API

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
