use shared::Result;
use std::collections::HashMap;
use tracing::info;

use crate::circuit_breaker::CircuitBreakerConfig;
use crate::evm_client::{EvmChain, EvmClient};
use crate::retry::RetryConfig;
use crate::SolanaClient;

/// Blockchain type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum Blockchain {
    Solana,
    Ethereum,
    BinanceSmartChain,
    Polygon,
}

impl Blockchain {
    pub fn name(&self) -> &'static str {
        match self {
            Blockchain::Solana => "Solana",
            Blockchain::Ethereum => "Ethereum",
            Blockchain::BinanceSmartChain => "Binance Smart Chain",
            Blockchain::Polygon => "Polygon",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "solana" => Blockchain::Solana,
            "ethereum" => Blockchain::Ethereum,
            "binancesmartchain" | "bsc" | "binance smart chain" => Blockchain::BinanceSmartChain,
            "polygon" => Blockchain::Polygon,
            _ => Blockchain::Solana, // Default fallback
        }
    }
}

/// Configuration for a blockchain RPC endpoint
#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    pub blockchain: Blockchain,
    pub primary_rpc_url: String,
    pub fallback_rpc_url: Option<String>,
}

/// Multi-chain blockchain client manager
/// 
/// Provides unified access to multiple blockchain clients (Solana, Ethereum, BSC, Polygon)
/// with automatic failover, circuit breakers, and retry logic.
pub struct MultiChainClient {
    solana_client: Option<SolanaClient>,
    ethereum_client: Option<EvmClient>,
    bsc_client: Option<EvmClient>,
    polygon_client: Option<EvmClient>,
    retry_config: RetryConfig,
    circuit_breaker_config: CircuitBreakerConfig,
}

impl MultiChainClient {
    /// Create a new multi-chain client with default configurations
    pub fn new() -> Self {
        info!("Initializing multi-chain blockchain client");
        Self {
            solana_client: None,
            ethereum_client: None,
            bsc_client: None,
            polygon_client: None,
            retry_config: RetryConfig::default(),
            circuit_breaker_config: CircuitBreakerConfig::default(),
        }
    }

    /// Create a new multi-chain client with custom retry and circuit breaker configs
    pub fn new_with_config(
        retry_config: RetryConfig,
        circuit_breaker_config: CircuitBreakerConfig,
    ) -> Self {
        info!("Initializing multi-chain blockchain client with custom config");
        Self {
            solana_client: None,
            ethereum_client: None,
            bsc_client: None,
            polygon_client: None,
            retry_config,
            circuit_breaker_config,
        }
    }

    /// Configure Solana client
    pub fn with_solana(mut self, primary_rpc: String, fallback_rpc: Option<String>) -> Self {
        info!("Configuring Solana client");
        self.solana_client = Some(SolanaClient::new_with_config(
            primary_rpc,
            fallback_rpc,
            self.retry_config.clone(),
            self.circuit_breaker_config.clone(),
        ));
        self
    }

    /// Configure Ethereum client
    pub fn with_ethereum(mut self, primary_rpc: String, fallback_rpc: Option<String>) -> Self {
        info!("Configuring Ethereum client");
        self.ethereum_client = Some(EvmClient::new_with_config(
            EvmChain::Ethereum,
            primary_rpc,
            fallback_rpc,
            self.retry_config.clone(),
            self.circuit_breaker_config.clone(),
        ));
        self
    }

    /// Configure Binance Smart Chain client
    pub fn with_bsc(mut self, primary_rpc: String, fallback_rpc: Option<String>) -> Self {
        info!("Configuring Binance Smart Chain client");
        self.bsc_client = Some(EvmClient::new_with_config(
            EvmChain::BinanceSmartChain,
            primary_rpc,
            fallback_rpc,
            self.retry_config.clone(),
            self.circuit_breaker_config.clone(),
        ));
        self
    }

    /// Configure Polygon client
    pub fn with_polygon(mut self, primary_rpc: String, fallback_rpc: Option<String>) -> Self {
        info!("Configuring Polygon client");
        self.polygon_client = Some(EvmClient::new_with_config(
            EvmChain::Polygon,
            primary_rpc,
            fallback_rpc,
            self.retry_config.clone(),
            self.circuit_breaker_config.clone(),
        ));
        self
    }

    /// Configure multiple blockchains from a list of configs
    pub fn with_configs(mut self, configs: Vec<BlockchainConfig>) -> Self {
        for config in configs {
            match config.blockchain {
                Blockchain::Solana => {
                    self = self.with_solana(config.primary_rpc_url, config.fallback_rpc_url);
                }
                Blockchain::Ethereum => {
                    self = self.with_ethereum(config.primary_rpc_url, config.fallback_rpc_url);
                }
                Blockchain::BinanceSmartChain => {
                    self = self.with_bsc(config.primary_rpc_url, config.fallback_rpc_url);
                }
                Blockchain::Polygon => {
                    self = self.with_polygon(config.primary_rpc_url, config.fallback_rpc_url);
                }
            }
        }
        self
    }

    /// Get the Solana client
    pub fn solana(&self) -> Option<&SolanaClient> {
        self.solana_client.as_ref()
    }

    /// Get the Ethereum client
    pub fn ethereum(&self) -> Option<&EvmClient> {
        self.ethereum_client.as_ref()
    }

    /// Get the Binance Smart Chain client
    pub fn bsc(&self) -> Option<&EvmClient> {
        self.bsc_client.as_ref()
    }

    /// Get the Polygon client
    pub fn polygon(&self) -> Option<&EvmClient> {
        self.polygon_client.as_ref()
    }

    /// Get a client for a specific blockchain
    pub fn get_client(&self, blockchain: Blockchain) -> Option<BlockchainClientRef<'_>> {
        match blockchain {
            Blockchain::Solana => self
                .solana_client
                .as_ref()
                .map(BlockchainClientRef::Solana),
            Blockchain::Ethereum => self
                .ethereum_client
                .as_ref()
                .map(BlockchainClientRef::Evm),
            Blockchain::BinanceSmartChain => {
                self.bsc_client.as_ref().map(BlockchainClientRef::Evm)
            }
            Blockchain::Polygon => self
                .polygon_client
                .as_ref()
                .map(BlockchainClientRef::Evm),
        }
    }

    /// Get list of configured blockchains
    pub fn configured_blockchains(&self) -> Vec<Blockchain> {
        let mut blockchains = Vec::new();
        if self.solana_client.is_some() {
            blockchains.push(Blockchain::Solana);
        }
        if self.ethereum_client.is_some() {
            blockchains.push(Blockchain::Ethereum);
        }
        if self.bsc_client.is_some() {
            blockchains.push(Blockchain::BinanceSmartChain);
        }
        if self.polygon_client.is_some() {
            blockchains.push(Blockchain::Polygon);
        }
        blockchains
    }

    /// Health check for all configured blockchain clients
    pub async fn health_check_all(&self) -> HashMap<Blockchain, Result<()>> {
        let mut results = HashMap::new();

        if let Some(client) = &self.solana_client {
            results.insert(Blockchain::Solana, client.health_check().await);
        }

        if let Some(client) = &self.ethereum_client {
            results.insert(Blockchain::Ethereum, client.health_check().await);
        }

        if let Some(client) = &self.bsc_client {
            results.insert(Blockchain::BinanceSmartChain, client.health_check().await);
        }

        if let Some(client) = &self.polygon_client {
            results.insert(Blockchain::Polygon, client.health_check().await);
        }

        results
    }
}

impl Default for MultiChainClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference to a blockchain client (either Solana or EVM)
pub enum BlockchainClientRef<'a> {
    Solana(&'a SolanaClient),
    Evm(&'a EvmClient),
}

impl<'a> BlockchainClientRef<'a> {
    /// Submit a raw transaction to the blockchain
    pub async fn submit_transaction(&self, raw_tx: &str) -> Result<String> {
        match self {
            BlockchainClientRef::Solana(_) => {
                // Solana transaction submission would go here
                // For now, return an error as it's not implemented in this task
                Err(shared::Error::Internal(
                    "Solana transaction submission not implemented in this task".to_string(),
                ))
            }
            BlockchainClientRef::Evm(client) => client.submit_transaction(raw_tx).await,
        }
    }

    /// Validate an address for this blockchain
    pub fn validate_address(&self, address: &str) -> Result<String> {
        match self {
            BlockchainClientRef::Solana(client) => {
                client.validate_address(address)?;
                Ok(address.to_string())
            }
            BlockchainClientRef::Evm(client) => client.validate_address(address),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_names() {
        assert_eq!(Blockchain::Solana.name(), "Solana");
        assert_eq!(Blockchain::Ethereum.name(), "Ethereum");
        assert_eq!(Blockchain::BinanceSmartChain.name(), "Binance Smart Chain");
        assert_eq!(Blockchain::Polygon.name(), "Polygon");
    }

    #[test]
    fn test_multi_chain_client_creation() {
        let client = MultiChainClient::new();
        assert!(client.solana().is_none());
        assert!(client.ethereum().is_none());
        assert!(client.bsc().is_none());
        assert!(client.polygon().is_none());
    }

    #[test]
    fn test_multi_chain_client_with_solana() {
        let client = MultiChainClient::new()
            .with_solana("https://api.mainnet-beta.solana.com".to_string(), None);
        
        assert!(client.solana().is_some());
        assert!(client.ethereum().is_none());
    }

    #[test]
    fn test_multi_chain_client_with_ethereum() {
        let client = MultiChainClient::new()
            .with_ethereum("https://eth.llamarpc.com".to_string(), None);
        
        assert!(client.solana().is_none());
        assert!(client.ethereum().is_some());
    }

    #[test]
    fn test_multi_chain_client_with_all_chains() {
        let client = MultiChainClient::new()
            .with_solana("https://api.mainnet-beta.solana.com".to_string(), None)
            .with_ethereum("https://eth.llamarpc.com".to_string(), None)
            .with_bsc("https://bsc-dataseed.binance.org".to_string(), None)
            .with_polygon("https://polygon-rpc.com".to_string(), None);
        
        assert!(client.solana().is_some());
        assert!(client.ethereum().is_some());
        assert!(client.bsc().is_some());
        assert!(client.polygon().is_some());
    }

    #[test]
    fn test_configured_blockchains() {
        let client = MultiChainClient::new()
            .with_solana("https://api.mainnet-beta.solana.com".to_string(), None)
            .with_ethereum("https://eth.llamarpc.com".to_string(), None);
        
        let configured = client.configured_blockchains();
        assert_eq!(configured.len(), 2);
        assert!(configured.contains(&Blockchain::Solana));
        assert!(configured.contains(&Blockchain::Ethereum));
    }

    #[test]
    fn test_get_client() {
        let client = MultiChainClient::new()
            .with_ethereum("https://eth.llamarpc.com".to_string(), None);
        
        assert!(client.get_client(Blockchain::Ethereum).is_some());
        assert!(client.get_client(Blockchain::Solana).is_none());
    }

    #[test]
    fn test_with_configs() {
        let configs = vec![
            BlockchainConfig {
                blockchain: Blockchain::Ethereum,
                primary_rpc_url: "https://eth.llamarpc.com".to_string(),
                fallback_rpc_url: None,
            },
            BlockchainConfig {
                blockchain: Blockchain::Polygon,
                primary_rpc_url: "https://polygon-rpc.com".to_string(),
                fallback_rpc_url: Some("https://polygon-backup.com".to_string()),
            },
        ];

        let client = MultiChainClient::new().with_configs(configs);
        
        assert!(client.ethereum().is_some());
        assert!(client.polygon().is_some());
        assert!(client.solana().is_none());
        assert!(client.bsc().is_none());
    }
}
