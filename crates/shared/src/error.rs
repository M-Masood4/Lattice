use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Solana RPC error: {0}")]
    SolanaRpc(String),
    
    #[error("EVM RPC error: {0}")]
    EvmRpc(String),
    
    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(String),
    
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Subscription required: {0}")]
    SubscriptionRequired(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
}

pub type Result<T> = std::result::Result<T, Error>;
