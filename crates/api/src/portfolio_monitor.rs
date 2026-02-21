use database::DbPool;
use shared::{models::Portfolio, Error, Result};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{AnalyticsService, WalletService, WhaleDetectionService};

/// Background service that monitors portfolio changes and updates whale lists
/// 
/// **Validates: Requirements 2.5, 8.1**
pub struct PortfolioMonitor {
    wallet_service: Arc<WalletService>,
    whale_detection_service: Arc<WhaleDetectionService>,
    analytics_service: Arc<AnalyticsService>,
    db_pool: DbPool,
    check_interval: Duration,
}

impl PortfolioMonitor {
    /// Create a new portfolio monitor
    /// 
    /// # Arguments
    /// * `wallet_service` - Service for fetching portfolio data
    /// * `whale_detection_service` - Service for updating whale lists
    /// * `analytics_service` - Service for creating portfolio snapshots
    /// * `db_pool` - Database connection pool
    /// * `check_interval` - How often to check for portfolio changes (default: 5 minutes per Requirement 2.5)
    pub fn new(
        wallet_service: Arc<WalletService>,
        whale_detection_service: Arc<WhaleDetectionService>,
        analytics_service: Arc<AnalyticsService>,
        db_pool: DbPool,
        check_interval: Option<Duration>,
    ) -> Self {
        Self {
            wallet_service,
            whale_detection_service,
            analytics_service,
            db_pool,
            check_interval: check_interval.unwrap_or(Duration::from_secs(300)), // 5 minutes default
        }
    }

    /// Start the background monitoring job
    /// 
    /// This spawns a Tokio task that periodically checks for portfolio changes
    /// and updates whale lists accordingly.
    /// 
    /// **Validates: Requirements 2.5**
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        info!(
            "Starting portfolio monitor with check interval: {:?}",
            self.check_interval
        );

        tokio::spawn(async move {
            let mut ticker = interval(self.check_interval);

            loop {
                ticker.tick().await;
                
                debug!("Portfolio monitor tick - checking for updates");

                if let Err(e) = self.check_and_update_portfolios().await {
                    error!("Error in portfolio monitor: {}", e);
                    // Continue monitoring even if one cycle fails
                }
            }
        })
    }

    /// Check all connected wallets for portfolio changes and update whale lists
    /// 
    /// **Validates: Requirements 2.5**
    async fn check_and_update_portfolios(&self) -> Result<()> {
        debug!("Fetching all connected wallets");

        // Get all connected wallets from database
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        let wallet_rows = client
            .query(
                "SELECT w.id, w.user_id, w.address, w.last_synced 
                 FROM wallets w
                 ORDER BY w.last_synced ASC NULLS FIRST",
                &[],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to query wallets: {}", e)))?;

        info!("Found {} wallets to check", wallet_rows.len());

        for row in wallet_rows {
            let wallet_id: Uuid = row.get(0);
            let user_id: Uuid = row.get(1);
            let wallet_address: String = row.get(2);

            debug!(
                "Checking portfolio for wallet {} (user: {})",
                wallet_address, user_id
            );

            // Refresh portfolio from blockchain
            match self.wallet_service.refresh_portfolio(&wallet_address).await {
                Ok(portfolio) => {
                    // Create portfolio snapshot for historical tracking (Requirement 8.1)
                    if let Err(e) = self.analytics_service.create_portfolio_snapshot(wallet_id).await {
                        warn!(
                            "Failed to create portfolio snapshot for wallet {}: {}",
                            wallet_address, e
                        );
                        // Continue even if snapshot creation fails
                    }

                    // Check if portfolio composition has changed
                    if self.has_portfolio_changed(&wallet_id, &portfolio).await? {
                        info!(
                            "Portfolio changed for wallet {}, updating whale list",
                            wallet_address
                        );

                        // Update whale list for this user
                        if let Err(e) = self
                            .whale_detection_service
                            .update_whales_for_user(user_id, &portfolio)
                            .await
                        {
                            warn!(
                                "Failed to update whales for user {}: {}",
                                user_id, e
                            );
                            // Continue with other wallets even if one fails
                        }
                    } else {
                        debug!("No portfolio changes detected for wallet {}", wallet_address);
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to refresh portfolio for wallet {}: {}",
                        wallet_address, e
                    );
                    // Continue with other wallets even if one fails
                }
            }
        }

        Ok(())
    }

    /// Check if portfolio composition has changed (new tokens or removed tokens)
    /// 
    /// This compares the current portfolio with the stored portfolio to detect changes
    /// in the token composition (not just balance changes).
    async fn has_portfolio_changed(&self, wallet_id: &Uuid, new_portfolio: &Portfolio) -> Result<bool> {
        let client = self.db_pool.get().await.map_err(|e| {
            Error::Database(format!("Failed to get database connection: {}", e))
        })?;

        // Get current token mints from database
        let rows = client
            .query(
                "SELECT token_mint FROM portfolio_assets WHERE wallet_id = $1",
                &[wallet_id],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to query portfolio assets: {}", e)))?;

        let stored_mints: std::collections::HashSet<String> = rows
            .iter()
            .map(|row| row.get::<_, String>(0))
            .collect();

        let new_mints: std::collections::HashSet<String> = new_portfolio
            .assets
            .iter()
            .map(|asset| asset.token_mint.clone())
            .collect();

        // Portfolio changed if token composition is different
        let changed = stored_mints != new_mints;

        if changed {
            debug!(
                "Portfolio composition changed: stored={:?}, new={:?}",
                stored_mints, new_mints
            );
        }

        Ok(changed)
    }

    /// Manually trigger a whale list update for a specific user
    /// 
    /// This can be called when a user explicitly requests a refresh
    /// or when a portfolio change is detected outside the background job.
    /// 
    /// **Validates: Requirements 2.5**
    pub async fn trigger_whale_update(&self, user_id: Uuid, wallet_address: &str) -> Result<()> {
        info!(
            "Manually triggering whale update for user {} (wallet: {})",
            user_id, wallet_address
        );

        // Get current portfolio
        let portfolio = self.wallet_service.get_portfolio(wallet_address).await?;

        // Update whale list
        self.whale_detection_service
            .update_whales_for_user(user_id, &portfolio)
            .await?;

        info!("Successfully updated whale list for user {}", user_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_monitor_creation() {
        // This is a placeholder - actual tests require database and service setup
        // Real tests will be implemented with test fixtures
    }

    #[tokio::test]
    #[ignore] // Only run with real database
    async fn test_portfolio_change_detection() {
        // Test that portfolio composition changes are detected correctly
        // This will be implemented with proper test fixtures
    }
}
