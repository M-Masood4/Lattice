//! Error types for BLE mesh operations

use thiserror::Error;

/// Result type for mesh operations
pub type MeshResult<T> = Result<T, MeshError>;

/// Errors that can occur during BLE mesh operations
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("BLE connection failed: {0}")]
    ConnectionFailed(String),

    #[error("BLE adapter error: {0}")]
    AdapterError(String),

    #[error("Packet transmission failed: {0}")]
    TransmissionFailed(String),

    #[error("Packet fragmentation error: {0}")]
    FragmentationError(String),

    #[error("Invalid packet format: {0}")]
    InvalidPacket(String),

    #[error("TTL expired")]
    TTLExpired,

    #[error("Duplicate packet: {0}")]
    DuplicatePacket(String),

    #[error("Store and forward queue full")]
    QueueFull,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("BLE permission denied: {0}")]
    PermissionDenied(String),

    #[error("BLE powered off")]
    PoweredOff,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<std::io::Error> for MeshError {
    fn from(err: std::io::Error) -> Self {
        MeshError::AdapterError(err.to_string())
    }
}

impl From<serde_json::Error> for MeshError {
    fn from(err: serde_json::Error) -> Self {
        MeshError::SerializationError(err.to_string())
    }
}
