//! Error types for stealth address operations

use thiserror::Error;

/// Result type for stealth operations
pub type StealthResult<T> = Result<T, StealthError>;

/// Errors that can occur during stealth address operations
#[derive(Error, Debug)]
pub enum StealthError {
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Invalid meta-address format: {0}")]
    InvalidMetaAddress(String),

    #[error("Invalid curve point: {0}")]
    InvalidCurvePoint(String),

    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Storage operation failed: {0}")]
    StorageFailed(String),

    #[error("Blockchain operation failed: {0}")]
    BlockchainError(String),

    #[error("Payment queue error: {0}")]
    PaymentQueueError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid payment status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Payment not found: {0}")]
    PaymentNotFound(String),

    #[error("Queue full: maximum {0} entries")]
    QueueFull(usize),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("QR code operation failed: {0}")]
    QrCodeError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<std::io::Error> for StealthError {
    fn from(err: std::io::Error) -> Self {
        StealthError::StorageFailed(err.to_string())
    }
}

impl From<serde_json::Error> for StealthError {
    fn from(err: serde_json::Error) -> Self {
        StealthError::SerializationError(err.to_string())
    }
}
