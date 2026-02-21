use shared::{Error, Result};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::retry::{retry_with_backoff, RetryConfig};

/// Supported EVM-compatible blockchains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvmChain {
    Ethereum,
    BinanceSmartChain,
    Polygon,
}

impl EvmChain {
    pub fn name(&self) -> &'static str {
        match self {
            EvmChain::Ethereum => "Ethereum",
            EvmChain::BinanceSmartChain => "Binance Smart Chain",
            EvmChain::Polygon => "Polygon",
        }
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            EvmChain::Ethereum => 1,
            EvmChain::BinanceSmartChain => 56,
            EvmChain::Polygon => 137,
        }
    }
}

/// EVM client for Ethereum, BSC, and Polygon
pub struct EvmClient {
    chain: EvmChain,
    primary_rpc_url: String,
    fallback_rpc_url: Option<String>,
    primary_circuit_breaker: Arc<CircuitBreaker>,
    fallback_circuit_breaker: Option<Arc<CircuitBreaker>>,
    retry_config: RetryConfig,
}

impl EvmClient {
    /// Create a new EVM client with primary and optional fallback RPC endpoints
    pub fn new(
        chain: EvmChain,
        rpc_url: String,
        fallback_url: Option<String>,
    ) -> Self {
        info!(
            "Initializing {} client with primary RPC: {}",
            chain.name(),
            rpc_url
        );

        let circuit_breaker_config = CircuitBreakerConfig::default();
        let primary_circuit_breaker = Arc::new(CircuitBreaker::new(
            format!("{}-primary-rpc-{}", chain.name(), rpc_url),
            circuit_breaker_config.clone(),
        ));

        let fallback_circuit_breaker = fallback_url.as_ref().map(|url| {
            info!("Configuring fallback RPC: {}", url);
            Arc::new(CircuitBreaker::new(
                format!("{}-fallback-rpc-{}", chain.name(), url),
                circuit_breaker_config,
            ))
        });

        Self {
            chain,
            primary_rpc_url: rpc_url,
            fallback_rpc_url: fallback_url,
            primary_circuit_breaker,
            fallback_circuit_breaker,
            retry_config: RetryConfig::default(),
        }
    }

    /// Create a new EVM client with custom retry and circuit breaker configurations
    pub fn new_with_config(
        chain: EvmChain,
        rpc_url: String,
        fallback_url: Option<String>,
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
    ) -> Self {
        info!(
            "Initializing {} client with custom config",
            chain.name()
        );

        let primary_circuit_breaker = Arc::new(CircuitBreaker::new(
            format!("{}-primary-rpc-{}", chain.name(), rpc_url),
            circuit_breaker_config.clone(),
        ));

        let fallback_circuit_breaker = fallback_url.as_ref().map(|url| {
            Arc::new(CircuitBreaker::new(
                format!("{}-fallback-rpc-{}", chain.name(), url),
                circuit_breaker_config,
            ))
        });

        Self {
            chain,
            primary_rpc_url: rpc_url,
            fallback_rpc_url: fallback_url,
            primary_circuit_breaker,
            fallback_circuit_breaker,
            retry_config,
        }
    }

    /// Get the chain this client is configured for
    pub fn chain(&self) -> EvmChain {
        self.chain
    }

    /// Validate an Ethereum-compatible address format (0x + 40 hex chars)
    pub fn validate_address(&self, address: &str) -> Result<String> {
        if !address.starts_with("0x") {
            return Err(Error::InvalidWalletAddress(
                "Address must start with 0x".to_string(),
            ));
        }

        if address.len() != 42 {
            return Err(Error::InvalidWalletAddress(
                "Address must be 42 characters (0x + 40 hex)".to_string(),
            ));
        }

        // Validate hex characters
        if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidWalletAddress(
                "Address must contain only hexadecimal characters".to_string(),
            ));
        }

        Ok(address.to_lowercase())
    }

    /// Submit a raw transaction to the blockchain
    /// Returns the transaction hash
    pub async fn submit_transaction(&self, raw_tx: &str) -> Result<String> {
        debug!(
            "Submitting transaction to {} blockchain",
            self.chain.name()
        );

        // Try primary RPC with circuit breaker and retry
        let primary_result = self
            .execute_with_circuit_breaker(
                &self.primary_circuit_breaker,
                "submit_transaction_primary",
                || {
                    let url = self.primary_rpc_url.clone();
                    let tx = raw_tx.to_string();
                    async move { Self::send_raw_transaction(&url, &tx).await }
                },
            )
            .await;

        match primary_result {
            Ok(tx_hash) => {
                info!(
                    "Transaction submitted successfully to {} (primary): {}",
                    self.chain.name(),
                    tx_hash
                );
                Ok(tx_hash)
            }
            Err(e) => {
                warn!(
                    "Primary RPC failed for submit_transaction on {}: {}",
                    self.chain.name(),
                    e
                );

                // Try fallback if available
                if let (Some(fallback_url), Some(fallback_cb)) = (
                    &self.fallback_rpc_url,
                    &self.fallback_circuit_breaker,
                ) {
                    debug!(
                        "Attempting fallback RPC for submit_transaction on {}",
                        self.chain.name()
                    );

                    let fallback_result = self
                        .execute_with_circuit_breaker(
                            fallback_cb,
                            "submit_transaction_fallback",
                            || {
                                let url = fallback_url.clone();
                                let tx = raw_tx.to_string();
                                async move { Self::send_raw_transaction(&url, &tx).await }
                            },
                        )
                        .await;

                    match fallback_result {
                        Ok(tx_hash) => {
                            info!(
                                "Transaction submitted successfully to {} (fallback): {}",
                                self.chain.name(),
                                tx_hash
                            );
                            Ok(tx_hash)
                        }
                        Err(fallback_err) => {
                            error!(
                                "Both primary and fallback RPC failed for {}: {}",
                                self.chain.name(),
                                fallback_err
                            );
                            Err(fallback_err)
                        }
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Send raw transaction via JSON-RPC
    async fn send_raw_transaction(rpc_url: &str, raw_tx: &str) -> Result<String> {
        let client = reqwest::Client::new();

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [raw_tx],
            "id": 1
        });

        let response = client
            .post(rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::EvmRpc(format!("Failed to send RPC request: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::EvmRpc(format!(
                "RPC request failed with status: {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::EvmRpc(format!("Failed to parse RPC response: {}", e)))?;

        // Check for JSON-RPC error
        if let Some(error) = response_json.get("error") {
            let error_message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(Error::EvmRpc(format!(
                "RPC error: {}",
                error_message
            )));
        }

        // Extract transaction hash from result
        let tx_hash = response_json
            .get("result")
            .and_then(|r| r.as_str())
            .ok_or_else(|| Error::EvmRpc("Missing transaction hash in response".to_string()))?;

        Ok(tx_hash.to_string())
    }

    /// Health check for EVM RPC connectivity
    pub async fn health_check(&self) -> Result<()> {
        self.execute_with_circuit_breaker(
            &self.primary_circuit_breaker,
            "health_check",
            || async {
                Self::check_rpc_health(&self.primary_rpc_url).await
            },
        )
        .await?;

        Ok(())
    }

    /// Check RPC health by calling eth_blockNumber
    async fn check_rpc_health(rpc_url: &str) -> Result<()> {
        let client = reqwest::Client::new();

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });

        let response = client
            .post(rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::EvmRpc(format!("Failed to send health check request: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::EvmRpc(format!(
                "Health check failed with status: {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::EvmRpc(format!("Failed to parse health check response: {}", e)))?;

        if response_json.get("error").is_some() {
            return Err(Error::EvmRpc("Health check returned error".to_string()));
        }

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
                "Circuit breaker is {:?} for operation: {} on {}",
                state,
                operation_name,
                self.chain.name()
            );
            return Err(Error::CircuitBreakerOpen(format!(
                "Circuit breaker is open for {} on {}",
                operation_name,
                self.chain.name()
            )));
        }

        // Execute with retry logic
        let result = retry_with_backoff(operation_name, &self.retry_config, operation).await;

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
        let client = EvmClient::new(
            EvmChain::Ethereum,
            "https://eth.llamarpc.com".to_string(),
            None,
        );

        // Valid Ethereum address (42 chars: 0x + 40 hex)
        let result = client.validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address_lowercase() {
        let client = EvmClient::new(
            EvmChain::Ethereum,
            "https://eth.llamarpc.com".to_string(),
            None,
        );

        let result = client.validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0x742d35cc6634c0532925a3b844bc9e7595f0beb0"
        );
    }

    #[test]
    fn test_validate_address_invalid_prefix() {
        let client = EvmClient::new(
            EvmChain::Ethereum,
            "https://eth.llamarpc.com".to_string(),
            None,
        );

        let result = client.validate_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEb");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_address_invalid_length() {
        let client = EvmClient::new(
            EvmChain::Ethereum,
            "https://eth.llamarpc.com".to_string(),
            None,
        );

        let result = client.validate_address("0x742d35Cc");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_address_invalid_chars() {
        let client = EvmClient::new(
            EvmChain::Ethereum,
            "https://eth.llamarpc.com".to_string(),
            None,
        );

        let result = client.validate_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbZ");
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_ids() {
        assert_eq!(EvmChain::Ethereum.chain_id(), 1);
        assert_eq!(EvmChain::BinanceSmartChain.chain_id(), 56);
        assert_eq!(EvmChain::Polygon.chain_id(), 137);
    }

    #[test]
    fn test_chain_names() {
        assert_eq!(EvmChain::Ethereum.name(), "Ethereum");
        assert_eq!(EvmChain::BinanceSmartChain.name(), "Binance Smart Chain");
        assert_eq!(EvmChain::Polygon.name(), "Polygon");
    }
}
