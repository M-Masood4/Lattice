use anyhow::Context;
use shared::{Error, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::retry::{retry_with_backoff, RetryConfig};
use crate::types::{TokenAccount, WalletBalance};

/// Solana client wrapper for blockchain interactions
pub struct SolanaClient {
    primary_client: RpcClient,
    fallback_client: Option<RpcClient>,
    primary_circuit_breaker: Arc<CircuitBreaker>,
    fallback_circuit_breaker: Option<Arc<CircuitBreaker>>,
    retry_config: RetryConfig,
}

impl SolanaClient {
    /// Create a new Solana client with primary and optional fallback RPC endpoints
    pub fn new(rpc_url: String, fallback_url: Option<String>) -> Self {
        info!("Initializing Solana client with primary RPC: {}", rpc_url);
        
        let primary_client = RpcClient::new_with_commitment(
            rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        let fallback_client = fallback_url.as_ref().map(|url| {
            info!("Configuring fallback RPC: {}", url);
            RpcClient::new_with_commitment(url.clone(), CommitmentConfig::confirmed())
        });

        // Create circuit breakers for primary and fallback
        let circuit_breaker_config = CircuitBreakerConfig::default();
        let primary_circuit_breaker = Arc::new(CircuitBreaker::new(
            format!("primary-rpc-{}", rpc_url),
            circuit_breaker_config.clone(),
        ));

        let fallback_circuit_breaker = fallback_url.map(|url| {
            Arc::new(CircuitBreaker::new(
                format!("fallback-rpc-{}", url),
                circuit_breaker_config,
            ))
        });

        Self {
            primary_client,
            fallback_client,
            primary_circuit_breaker,
            fallback_circuit_breaker,
            retry_config: RetryConfig::default(),
        }
    }

    /// Create a new Solana client with custom retry and circuit breaker configurations
    pub fn new_with_config(
        rpc_url: String,
        fallback_url: Option<String>,
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
    ) -> Self {
        info!("Initializing Solana client with custom config");
        
        let primary_client = RpcClient::new_with_commitment(
            rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        let fallback_client = fallback_url.as_ref().map(|url| {
            RpcClient::new_with_commitment(url.clone(), CommitmentConfig::confirmed())
        });

        let primary_circuit_breaker = Arc::new(CircuitBreaker::new(
            format!("primary-rpc-{}", rpc_url),
            circuit_breaker_config.clone(),
        ));

        let fallback_circuit_breaker = fallback_url.map(|url| {
            Arc::new(CircuitBreaker::new(
                format!("fallback-rpc-{}", url),
                circuit_breaker_config,
            ))
        });

        Self {
            primary_client,
            fallback_client,
            primary_circuit_breaker,
            fallback_circuit_breaker,
            retry_config,
        }
    }

    /// Validate a Solana wallet address format
    pub fn validate_address(&self, address: &str) -> Result<Pubkey> {
        Pubkey::from_str(address).map_err(|e| {
            warn!("Invalid wallet address format: {} - {}", address, e);
            Error::InvalidWalletAddress(format!("Invalid Solana address format: {}", e))
        })
    }

    /// Get SOL balance for a wallet address (in lamports)
    pub async fn get_sol_balance(&self, address: &str) -> Result<u64> {
        let pubkey = self.validate_address(address)?;
        
        debug!("Fetching SOL balance for address: {}", address);
        
        // Try primary client with circuit breaker and retry
        let primary_result = self.execute_with_circuit_breaker(
            &self.primary_circuit_breaker,
            "get_sol_balance_primary",
            || {
                let client = &self.primary_client;
                let pk = pubkey;
                async move {
                    client.get_balance(&pk).map_err(|e| {
                        Error::SolanaRpc(format!("Primary RPC failed: {}", e))
                    })
                }
            },
        ).await;

        match primary_result {
            Ok(balance) => {
                debug!("Retrieved SOL balance from primary: {} lamports", balance);
                Ok(balance)
            }
            Err(e) => {
                warn!("Primary RPC failed for get_balance: {}", e);
                
                // Try fallback if available
                if let (Some(fallback), Some(fallback_cb)) = 
                    (&self.fallback_client, &self.fallback_circuit_breaker) 
                {
                    debug!("Attempting fallback RPC for get_balance");
                    
                    let fallback_result = self.execute_with_circuit_breaker(
                        fallback_cb,
                        "get_sol_balance_fallback",
                        || {
                            let client = fallback;
                            let pk = pubkey;
                            async move {
                                client.get_balance(&pk).map_err(|e| {
                                    Error::SolanaRpc(format!("Fallback RPC failed: {}", e))
                                })
                            }
                        },
                    ).await;

                    match fallback_result {
                        Ok(balance) => {
                            debug!("Retrieved SOL balance from fallback: {} lamports", balance);
                            Ok(balance)
                        }
                        Err(fallback_err) => {
                            error!("Both primary and fallback RPC failed: {}", fallback_err);
                            Err(fallback_err)
                        }
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Get all SPL token accounts for a wallet address
    pub async fn get_token_accounts(&self, address: &str) -> Result<Vec<TokenAccount>> {
        let pubkey = self.validate_address(address)?;
        
        debug!("Fetching token accounts for address: {}", address);
        
        // Try primary client with circuit breaker and retry
        let primary_result = self.execute_with_circuit_breaker(
            &self.primary_circuit_breaker,
            "get_token_accounts_primary",
            || {
                let client = &self.primary_client;
                let pk = pubkey;
                async move {
                    client
                        .get_token_accounts_by_owner(
                            &pk,
                            solana_client::rpc_request::TokenAccountsFilter::ProgramId(
                                spl_token::id(),
                            ),
                        )
                        .map_err(|e| {
                            Error::SolanaRpc(format!("Primary RPC failed: {}", e))
                        })
                }
            },
        ).await;

        let accounts = match primary_result {
            Ok(accounts) => accounts,
            Err(e) => {
                warn!("Primary RPC failed for get_token_accounts: {}", e);
                
                // Try fallback if available
                if let (Some(fallback), Some(fallback_cb)) = 
                    (&self.fallback_client, &self.fallback_circuit_breaker) 
                {
                    debug!("Attempting fallback RPC for get_token_accounts");
                    
                    self.execute_with_circuit_breaker(
                        fallback_cb,
                        "get_token_accounts_fallback",
                        || {
                            let client = fallback;
                            let pk = pubkey;
                            async move {
                                client
                                    .get_token_accounts_by_owner(
                                        &pk,
                                        solana_client::rpc_request::TokenAccountsFilter::ProgramId(
                                            spl_token::id(),
                                        ),
                                    )
                                    .map_err(|e| {
                                        Error::SolanaRpc(format!("Fallback RPC failed: {}", e))
                                    })
                            }
                        },
                    ).await?
                } else {
                    return Err(e);
                }
            }
        };

        let mut token_accounts = Vec::new();

        for account in accounts {
            // Parse token account data using the decoded info
            match self.parse_token_account_from_ui(address, &account) {
                Ok(token_account) => token_accounts.push(token_account),
                Err(e) => {
                    warn!("Failed to parse token account: {}", e);
                    continue;
                }
            }
        }

        debug!("Retrieved {} token accounts", token_accounts.len());
        Ok(token_accounts)
    }

    /// Get complete wallet balance including SOL and all SPL tokens
    pub async fn get_wallet_balance(&self, address: &str) -> Result<WalletBalance> {
        info!("Fetching complete wallet balance for: {}", address);
        
        // Validate address first
        self.validate_address(address)?;

        // Get SOL balance
        let sol_balance = self.get_sol_balance(address).await?;

        // Get token accounts
        let token_accounts = self.get_token_accounts(address).await?;

        Ok(WalletBalance {
            address: address.to_string(),
            sol_balance,
            token_accounts,
        })
    }

    /// Parse token account from RPC UI response
    fn parse_token_account_from_ui(
        &self,
        owner: &str,
        account: &solana_client::rpc_response::RpcKeyedAccount,
    ) -> anyhow::Result<TokenAccount> {
        use solana_account_decoder::UiAccountData;
        
        // Extract token account info from the UI account data
        match &account.account.data {
            UiAccountData::Json(parsed_account) => {
                // The parsed account should have token account info
                let info = parsed_account
                    .parsed
                    .get("info")
                    .ok_or_else(|| anyhow::anyhow!("Missing info field"))?;

                let mint = info
                    .get("mint")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing mint field"))?
                    .to_string();

                let token_amount = info
                    .get("tokenAmount")
                    .ok_or_else(|| anyhow::anyhow!("Missing tokenAmount field"))?;

                let amount_str = token_amount
                    .get("amount")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing amount field"))?;

                let amount = amount_str
                    .parse::<u64>()
                    .context("Failed to parse amount")?;

                let decimals = token_amount
                    .get("decimals")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing decimals field"))?
                    as u8;

                Ok(TokenAccount {
                    mint,
                    owner: owner.to_string(),
                    amount,
                    decimals,
                })
            }
            _ => Err(anyhow::anyhow!("Expected JSON parsed account data")),
        }
    }

    /// Get the primary RPC client (for advanced operations)
    pub fn primary_client(&self) -> &RpcClient {
        &self.primary_client
    }

    /// Get the fallback RPC client if configured
    pub fn fallback_client(&self) -> Option<&RpcClient> {
        self.fallback_client.as_ref()
    }

    /// Health check for Solana RPC connectivity
    /// 
    /// **Validates: Requirements 11.1, 11.2**
    pub async fn health_check(&self) -> Result<()> {
        // Try to get the latest blockhash as a simple health check
        self.execute_with_circuit_breaker(
            &self.primary_circuit_breaker,
            "health_check",
            || async {
                self.primary_client
                    .get_latest_blockhash()
                    .map_err(|e| Error::SolanaRpc(format!("Health check failed: {}", e)))
            },
        )
        .await?;

        Ok(())
    }

    /// Execute an operation with circuit breaker and retry logic
    async fn execute_with_circuit_breaker<F, Fut, T>(
        &self,
        circuit_breaker: &CircuitBreaker,
        operation_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check if circuit breaker allows the request
        if !circuit_breaker.is_request_allowed().await {
            let state = circuit_breaker.get_state().await;
            error!(
                "Circuit breaker is {:?} for operation: {}",
                state, operation_name
            );
            return Err(Error::CircuitBreakerOpen(format!(
                "Circuit breaker is open for {}",
                operation_name
            )));
        }

        // Execute with retry logic
        let result = retry_with_backoff(
            operation_name,
            &self.retry_config,
            operation,
        )
        .await;

        // Record success or failure in circuit breaker
        match &result {
            Ok(_) => {
                circuit_breaker.record_success().await;
            }
            Err(_) => {
                circuit_breaker.record_failure().await;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_address_valid() {
        let client = SolanaClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            None,
        );

        // Valid Solana address
        let result = client.validate_address("11111111111111111111111111111111");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address_invalid() {
        let client = SolanaClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            None,
        );

        // Invalid address
        let result = client.validate_address("invalid_address");
        assert!(result.is_err());
        
        if let Err(Error::InvalidWalletAddress(msg)) = result {
            assert!(msg.contains("Invalid Solana address format"));
        } else {
            panic!("Expected InvalidWalletAddress error");
        }
    }

    #[test]
    fn test_validate_address_empty() {
        let client = SolanaClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            None,
        );

        let result = client.validate_address("");
        assert!(result.is_err());
    }
}
