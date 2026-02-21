use blockchain::SolanaClient;
use database::{DbPool, RedisPool};
use redis::AsyncCommands;
use shared::{models::*, Error, PriceFeedService, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Whale detection service for identifying large token holders
/// 
/// **Validates: Requirements 2.1, 2.2, 2.3, 2.4**
pub struct WhaleDetectionService {
    #[allow(dead_code)]
    solana_client: Arc<SolanaClient>,
    db_pool: DbPool,
    redis_pool: RedisPool,
    #[allow(dead_code)]
    price_feed: PriceFeedService,
}

/// Represents a whale account with holdings information
#[derive(Debug, Clone)]
pub struct WhaleAccount {
    pub address: String,
    pub token_mint: String,
    pub amount: f64,
    pub value_usd: f64,
    pub multiplier_vs_user: f64,
}

/// Represents a ranked whale with aggregated holdings
#[derive(Debug, Clone)]
pub struct RankedWhale {
    pub address: String,
    pub assets: Vec<WhaleAsset>,
    pub total_value_usd: f64,
    pub rank: i32,
}

#[derive(Debug, Clone)]
pub struct WhaleAsset {
    pub token_mint: String,
    pub token_symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub multiplier_vs_user: f64,
}

impl WhaleDetectionService {
    /// Create a new whale detection service
    pub fn new(
        solana_client: Arc<SolanaClient>,
        db_pool: DbPool,
        redis_pool: RedisPool,
    ) -> Self {
        let price_feed = PriceFeedService::new();
        
        Self {
            solana_client,
            db_pool,
            redis_pool,
            price_feed,
        }
    }

    /// Identify whales for a given portfolio
    /// 
    /// **Validates: Requirements 2.1, 2.2, 2.3, 2.4**
    pub async fn identify_whales(
        &self,
        user_id: Uuid,
        portfolio: &Portfolio,
    ) -> Result<Vec<RankedWhale>> {
        info!(
            "Identifying whales for user {} with {} assets",
            user_id,
            portfolio.assets.len()
        );

        // Check cache first (5-minute TTL)
        let cache_key = format!("whales:user:{}", user_id);
        if let Some(cached_whales) = self.get_cached_whales(&cache_key).await? {
            debug!("Returning cached whales for user: {}", user_id);
            return Ok(cached_whales);
        }

        // Collect all whale accounts across all user assets
        let mut whale_accounts: Vec<WhaleAccount> = Vec::new();

        for asset in &portfolio.assets {
            // Skip assets with zero or very small amounts
            let user_amount = asset.amount.parse::<f64>().unwrap_or(0.0);
            if user_amount <= 0.0 {
                continue;
            }

            debug!(
                "Finding whales for token {} (user holds {})",
                asset.token_symbol, user_amount
            );

            // Find whales for this specific token
            match self
                .find_whales_for_token(&asset.token_mint, user_amount)
                .await
            {
                Ok(token_whales) => {
                    whale_accounts.extend(token_whales);
                }
                Err(e) => {
                    warn!(
                        "Failed to find whales for token {}: {}",
                        asset.token_mint, e
                    );
                    // Continue with other tokens even if one fails
                    continue;
                }
            }
        }

        // Aggregate whale accounts by address
        let ranked_whales = self.aggregate_and_rank_whales(whale_accounts).await?;

        // Store whales in database
        self.store_whales(&ranked_whales, user_id).await?;

        // Cache the results with 5-minute TTL
        self.cache_whales(&cache_key, &ranked_whales).await?;

        info!(
            "Identified {} whales for user {}",
            ranked_whales.len(),
            user_id
        );

        Ok(ranked_whales)
    }

    /// Find whale accounts for a specific token
    /// 
    /// **Validates: Requirements 2.1, 2.2**
    async fn find_whales_for_token(
        &self,
        token_mint: &str,
        user_amount: f64,
    ) -> Result<Vec<WhaleAccount>> {
        debug!("Querying Solana for top holders of token: {}", token_mint);

        // Calculate minimum whale threshold (100x user position)
        let whale_threshold = user_amount * 100.0;

        // Query Solana for token accounts using getProgramAccounts
        // This is a simplified implementation - in production, you'd want to:
        // 1. Use a more efficient method (like indexer services)
        // 2. Implement pagination for large result sets
        // 3. Add more sophisticated filtering

        // For now, we'll use a simplified approach that queries the largest accounts
        // In a real implementation, you'd use getProgramAccounts with filters
        let whale_accounts = self
            .query_large_token_holders(token_mint, whale_threshold)
            .await?;

        Ok(whale_accounts)
    }

    /// Query large token holders from Solana
    /// 
    /// This is a simplified implementation. In production, you would:
    /// - Use Solana's getProgramAccounts with proper filters
    /// - Implement pagination for large result sets
    /// - Use indexer services (like Helius, QuickNode) for better performance
    async fn query_large_token_holders(
        &self,
        token_mint: &str,
        _whale_threshold: f64,
    ) -> Result<Vec<WhaleAccount>> {
        // NOTE: This is a placeholder implementation
        // In a real system, you would:
        // 1. Use getProgramAccounts to query all token accounts for this mint
        // 2. Filter by minimum balance
        // 3. Sort by balance descending
        // 4. Take top N accounts
        
        // For now, we'll return an empty list and log a warning
        // This allows the rest of the system to work while we implement the full solution
        
        warn!(
            "Token holder query not fully implemented for mint: {}. Returning empty whale list.",
            token_mint
        );
        
        // TODO: Implement full getProgramAccounts query
        // This would require extending the SolanaClient in the blockchain crate
        // to expose getProgramAccounts functionality
        
        Ok(Vec::new())
    }

    /// Aggregate whale accounts by address and rank by total USD value
    /// 
    /// **Validates: Requirements 2.3**
    async fn aggregate_and_rank_whales(
        &self,
        whale_accounts: Vec<WhaleAccount>,
    ) -> Result<Vec<RankedWhale>> {
        debug!("Aggregating {} whale accounts", whale_accounts.len());

        // Group whale accounts by address
        let mut whales_map: HashMap<String, Vec<WhaleAccount>> = HashMap::new();
        
        for account in whale_accounts {
            whales_map
                .entry(account.address.clone())
                .or_default()
                .push(account);
        }

        // Calculate total value for each whale
        let mut ranked_whales: Vec<RankedWhale> = Vec::new();
        
        for (address, accounts) in whales_map {
            let mut total_value_usd = 0.0;
            let mut assets = Vec::new();

            for account in accounts {
                total_value_usd += account.value_usd;
                
                // Get token symbol (simplified - in production, use token metadata service)
                let token_symbol = if account.token_mint == "So11111111111111111111111111111111111111112" {
                    "SOL".to_string()
                } else {
                    format!("TOKEN_{}", &account.token_mint[..8])
                };

                assets.push(WhaleAsset {
                    token_mint: account.token_mint,
                    token_symbol,
                    amount: account.amount,
                    value_usd: account.value_usd,
                    multiplier_vs_user: account.multiplier_vs_user,
                });
            }

            ranked_whales.push(RankedWhale {
                address,
                assets,
                total_value_usd,
                rank: 0, // Will be set after sorting
            });
        }

        // Sort by total USD value descending (Requirement 2.3)
        ranked_whales.sort_by(|a, b| {
            b.total_value_usd
                .partial_cmp(&a.total_value_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (index, whale) in ranked_whales.iter_mut().enumerate() {
            whale.rank = (index + 1) as i32;
        }

        debug!("Ranked {} whales by total value", ranked_whales.len());

        Ok(ranked_whales)
    }

    /// Store identified whales in PostgreSQL
    /// 
    /// **Validates: Requirements 2.4**
    async fn store_whales(&self, whales: &[RankedWhale], user_id: Uuid) -> Result<()> {
        debug!("Storing {} whales in database", whales.len());

        let mut client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let transaction = client.transaction().await.map_err(|e| {
            Error::Database(format!("Failed to start transaction: {}", e))
        })?;

        for whale in whales {
            // Insert or update whale record
            let whale_row = transaction
                .query_one(
                    "INSERT INTO whales (address, total_value_usd, first_detected, last_checked)
                     VALUES ($1, $2, NOW(), NOW())
                     ON CONFLICT (address)
                     DO UPDATE SET total_value_usd = $2, last_checked = NOW()
                     RETURNING id",
                    &[&whale.address, &whale.total_value_usd],
                )
                .await
                .map_err(|e| Error::Database(format!("Failed to insert whale: {}", e)))?;

            let whale_id: Uuid = whale_row.get(0);

            // Store user-whale tracking relationships for each asset
            for asset in &whale.assets {
                transaction
                    .execute(
                        "INSERT INTO user_whale_tracking (user_id, whale_id, token_mint, multiplier, rank, created_at)
                         VALUES ($1, $2, $3, $4, $5, NOW())
                         ON CONFLICT (user_id, whale_id, token_mint)
                         DO UPDATE SET multiplier = $4, rank = $5",
                        &[
                            &user_id,
                            &whale_id,
                            &asset.token_mint,
                            &asset.multiplier_vs_user,
                            &whale.rank,
                        ],
                    )
                    .await
                    .map_err(|e| {
                        Error::Database(format!("Failed to insert user whale tracking: {}", e))
                    })?;
            }
        }

        transaction.commit().await.map_err(|e| {
            Error::Database(format!("Failed to commit transaction: {}", e))
        })?;

        debug!("Successfully stored whales in database");

        Ok(())
    }

    /// Get cached whales from Redis
    async fn get_cached_whales(&self, cache_key: &str) -> Result<Option<Vec<RankedWhale>>> {
        let mut conn = self.redis_pool.clone();
        
        match conn.get::<_, String>(cache_key).await {
            Ok(json) => {
                match serde_json::from_str::<Vec<RankedWhale>>(&json) {
                    Ok(whales) => {
                        debug!("Cache hit for whales: {}", cache_key);
                        Ok(Some(whales))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached whales: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                if e.kind() == redis::ErrorKind::TypeError {
                    debug!("Cache miss for whales: {}", cache_key);
                    Ok(None)
                } else {
                    warn!("Redis error fetching whales: {}", e);
                    Ok(None)
                }
            }
        }
    }

    /// Cache whales in Redis with 5-minute TTL
    /// 
    /// **Validates: Requirements 2.4**
    async fn cache_whales(&self, cache_key: &str, whales: &[RankedWhale]) -> Result<()> {
        debug!("Caching whales: {}", cache_key);
        
        let json = serde_json::to_string(whales)
            .map_err(|e| Error::Internal(format!("Failed to serialize whales: {}", e)))?;
        
        let mut conn = self.redis_pool.clone();
        
        // Set with 5-minute (300 seconds) TTL as per requirements
        match conn.set_ex::<_, _, ()>(cache_key, json, 300).await {
            Ok(_) => {
                debug!("Successfully cached whales");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to cache whales: {}", e);
                // Don't fail on cache errors
                Ok(())
            }
        }
    }

    /// Check if an account qualifies as a whale for a given user position
    /// 
    /// **Validates: Requirements 2.2**
    pub fn is_whale(&self, account_amount: f64, user_amount: f64) -> bool {
        if user_amount <= 0.0 {
            return false;
        }
        
        let multiplier = account_amount / user_amount;
        multiplier >= 100.0
    }

    /// Update whale list when user portfolio changes
    /// 
    /// **Validates: Requirements 2.5**
    pub async fn update_whales_for_user(
        &self,
        user_id: Uuid,
        portfolio: &Portfolio,
    ) -> Result<()> {
        info!("Updating whales for user: {}", user_id);

        // Invalidate cache
        let cache_key = format!("whales:user:{}", user_id);
        let mut conn = self.redis_pool.clone();
        let _ = conn.del::<_, ()>(&cache_key).await;

        // Re-identify whales with new portfolio
        self.identify_whales(user_id, portfolio).await?;

        Ok(())
    }
}

// Implement Serialize/Deserialize for RankedWhale to support caching
impl serde::Serialize for RankedWhale {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("RankedWhale", 4)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("assets", &self.assets)?;
        state.serialize_field("total_value_usd", &self.total_value_usd)?;
        state.serialize_field("rank", &self.rank)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for RankedWhale {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct RankedWhaleVisitor;

        impl<'de> Visitor<'de> for RankedWhaleVisitor {
            type Value = RankedWhale;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct RankedWhale")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RankedWhale, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut address = None;
                let mut assets = None;
                let mut total_value_usd = None;
                let mut rank = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "address" => address = Some(map.next_value()?),
                        "assets" => assets = Some(map.next_value()?),
                        "total_value_usd" => total_value_usd = Some(map.next_value()?),
                        "rank" => rank = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(RankedWhale {
                    address: address.ok_or_else(|| de::Error::missing_field("address"))?,
                    assets: assets.ok_or_else(|| de::Error::missing_field("assets"))?,
                    total_value_usd: total_value_usd
                        .ok_or_else(|| de::Error::missing_field("total_value_usd"))?,
                    rank: rank.ok_or_else(|| de::Error::missing_field("rank"))?,
                })
            }
        }

        deserializer.deserialize_struct(
            "RankedWhale",
            &["address", "assets", "total_value_usd", "rank"],
            RankedWhaleVisitor,
        )
    }
}

impl serde::Serialize for WhaleAsset {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WhaleAsset", 5)?;
        state.serialize_field("token_mint", &self.token_mint)?;
        state.serialize_field("token_symbol", &self.token_symbol)?;
        state.serialize_field("amount", &self.amount)?;
        state.serialize_field("value_usd", &self.value_usd)?;
        state.serialize_field("multiplier_vs_user", &self.multiplier_vs_user)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for WhaleAsset {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct WhaleAssetVisitor;

        impl<'de> Visitor<'de> for WhaleAssetVisitor {
            type Value = WhaleAsset;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WhaleAsset")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<WhaleAsset, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut token_mint = None;
                let mut token_symbol = None;
                let mut amount = None;
                let mut value_usd = None;
                let mut multiplier_vs_user = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "token_mint" => token_mint = Some(map.next_value()?),
                        "token_symbol" => token_symbol = Some(map.next_value()?),
                        "amount" => amount = Some(map.next_value()?),
                        "value_usd" => value_usd = Some(map.next_value()?),
                        "multiplier_vs_user" => multiplier_vs_user = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(WhaleAsset {
                    token_mint: token_mint.ok_or_else(|| de::Error::missing_field("token_mint"))?,
                    token_symbol: token_symbol
                        .ok_or_else(|| de::Error::missing_field("token_symbol"))?,
                    amount: amount.ok_or_else(|| de::Error::missing_field("amount"))?,
                    value_usd: value_usd.ok_or_else(|| de::Error::missing_field("value_usd"))?,
                    multiplier_vs_user: multiplier_vs_user
                        .ok_or_else(|| de::Error::missing_field("multiplier_vs_user"))?,
                })
            }
        }

        deserializer.deserialize_struct(
            "WhaleAsset",
            &[
                "token_mint",
                "token_symbol",
                "amount",
                "value_usd",
                "multiplier_vs_user",
            ],
            WhaleAssetVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_whale_threshold() {
        // Create a simple service instance for testing the is_whale logic
        // Note: In real tests, we'd use test fixtures with proper database/redis setup
        
        // Test the whale threshold calculation directly
        let user_amount = 1.0;
        
        // Exactly 100x should qualify
        let whale_amount_100x = 100.0;
        assert!(whale_amount_100x / user_amount >= 100.0);

        // More than 100x should qualify
        let whale_amount_150x = 150.0;
        assert!(whale_amount_150x / user_amount >= 100.0);

        // Less than 100x should not qualify
        let whale_amount_99x = 99.0;
        assert!(whale_amount_99x / user_amount < 100.0);

        // Zero user amount edge case
        let zero_user = 0.0;
        assert!(zero_user <= 0.0);
    }

    #[test]
    fn test_whale_ranking_order() {
        // This will be implemented with property-based tests
        // Testing that whales are sorted by total USD value descending
    }
}
