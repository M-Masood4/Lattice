use thiserror::Error;
use tracing::error;
use uuid::Uuid;
use chrono;

#[derive(Error, Debug)]
pub enum ProximityError {
    #[error("Discovery not enabled")]
    DiscoveryNotEnabled,

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Transfer request not found: {0}")]
    TransferNotFound(String),

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session not found: {0}")]
    SessionNotFound(uuid::Uuid),

    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(String),

    #[error("Challenge not found")]
    ChallengeNotFound,

    #[error("Challenge expired")]
    ChallengeExpired,

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("BLE error: {0}")]
    BleError(String),

    #[error("QR code error: {0}")]
    QrCodeError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<serde_json::Error> for ProximityError {
    fn from(err: serde_json::Error) -> Self {
        ProximityError::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ProximityError>;

/// Context for error logging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub user_id: Option<Uuid>,
    pub peer_id: Option<String>,
    pub transfer_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub additional_info: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            user_id: None,
            peer_id: None,
            transfer_id: None,
            session_id: None,
            additional_info: None,
        }
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_peer_id(mut self, peer_id: String) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn with_transfer_id(mut self, transfer_id: Uuid) -> Self {
        self.transfer_id = Some(transfer_id);
        self
    }

    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_info(mut self, info: String) -> Self {
        self.additional_info = Some(info);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ProximityError {
    /// Log error with structured context
    /// 
    /// **Validates: Requirements 15.5**
    pub fn log_with_context(&self, context: &ErrorContext) {
        error!(
            error = %self,
            error_type = ?self,
            user_id = ?context.user_id,
            peer_id = ?context.peer_id,
            transfer_id = ?context.transfer_id,
            session_id = ?context.session_id,
            additional_info = ?context.additional_info,
            timestamp = %chrono::Utc::now(),
            "Proximity transfer error occurred"
        );
    }

    /// Get user-friendly error message
    /// 
    /// **Validates: Requirements 15.1, 15.2**
    pub fn user_message(&self) -> String {
        match self {
            ProximityError::InsufficientBalance { required, available } => {
                format!(
                    "Insufficient balance. You need {} but only have {} available.",
                    required, available
                )
            }
            ProximityError::PeerNotFound(peer) => {
                format!("The peer '{}' could not be found. They may have disabled discovery or moved out of range.", peer)
            }
            ProximityError::TransactionFailed(reason) => {
                format!("Transaction failed: {}. Please check your network connection and try again.", reason)
            }
            ProximityError::Timeout(operation) => {
                format!("Operation timed out: {}. Please try again.", operation)
            }
            ProximityError::NetworkError(details) => {
                format!("Network error: {}. Please check your connection and try again.", details)
            }
            ProximityError::DiscoveryNotEnabled => {
                "Discovery is not enabled. Please enable discovery to find nearby peers.".to_string()
            }
            ProximityError::AuthenticationFailed(reason) => {
                format!("Authentication failed: {}. Please verify your wallet connection.", reason)
            }
            ProximityError::TransferNotFound(id) => {
                format!("Transfer request '{}' not found. It may have expired or been cancelled.", id)
            }
            ProximityError::PermissionDenied(permission) => {
                format!("Permission denied: {}. Please grant the necessary permissions in your device settings.", permission)
            }
            ProximityError::RateLimitExceeded => {
                "Too many requests. Please wait a moment and try again.".to_string()
            }
            ProximityError::SessionExpired => {
                "Your discovery session has expired. Please start a new session.".to_string()
            }
            ProximityError::SessionNotFound(id) => {
                format!("Session '{}' not found. It may have expired.", id)
            }
            ProximityError::InvalidWalletAddress(addr) => {
                format!("Invalid wallet address: '{}'. Please check the address and try again.", addr)
            }
            ProximityError::ChallengeNotFound => {
                "Authentication challenge not found. Please try reconnecting.".to_string()
            }
            ProximityError::ChallengeExpired => {
                "Authentication challenge expired. Please try again.".to_string()
            }
            ProximityError::InvalidPublicKey => {
                "Invalid public key. Please verify your wallet connection.".to_string()
            }
            ProximityError::InvalidSignature => {
                "Invalid signature. Please verify your wallet connection and try again.".to_string()
            }
            ProximityError::BleError(details) => {
                format!("Bluetooth error: {}. Please check your Bluetooth settings.", details)
            }
            ProximityError::QrCodeError(details) => {
                format!("QR code error: {}. Please try scanning again.", details)
            }
            ProximityError::InvalidInput(details) => {
                format!("Invalid input: {}. Please check your entry and try again.", details)
            }
            ProximityError::DatabaseError(err) => {
                format!("Database error occurred. Please try again later. (Error: {})", err)
            }
            ProximityError::SerializationError(details) => {
                format!("Data processing error: {}. Please try again.", details)
            }
            ProximityError::ConnectionFailed(details) => {
                format!("Connection failed: {}. Please check your network and try again.", details)
            }
            ProximityError::InternalError(details) => {
                format!("An internal error occurred: {}. Please try again or contact support.", details)
            }
        }
    }

    /// Get error category for metrics and monitoring
    pub fn category(&self) -> ErrorCategory {
        match self {
            ProximityError::InsufficientBalance { .. } => ErrorCategory::Validation,
            ProximityError::PeerNotFound(_) => ErrorCategory::NotFound,
            ProximityError::AuthenticationFailed(_) => ErrorCategory::Authentication,
            ProximityError::TransferNotFound(_) => ErrorCategory::NotFound,
            ProximityError::TransactionFailed(_) => ErrorCategory::Transaction,
            ProximityError::NetworkError(_) => ErrorCategory::Network,
            ProximityError::Timeout(_) => ErrorCategory::Timeout,
            ProximityError::PermissionDenied(_) => ErrorCategory::Permission,
            ProximityError::RateLimitExceeded => ErrorCategory::RateLimit,
            ProximityError::SessionExpired => ErrorCategory::Session,
            ProximityError::SessionNotFound(_) => ErrorCategory::NotFound,
            ProximityError::InvalidWalletAddress(_) => ErrorCategory::Validation,
            ProximityError::ChallengeNotFound => ErrorCategory::Authentication,
            ProximityError::ChallengeExpired => ErrorCategory::Authentication,
            ProximityError::InvalidPublicKey => ErrorCategory::Authentication,
            ProximityError::InvalidSignature => ErrorCategory::Authentication,
            ProximityError::BleError(_) => ErrorCategory::Network,
            ProximityError::QrCodeError(_) => ErrorCategory::Validation,
            ProximityError::InvalidInput(_) => ErrorCategory::Validation,
            ProximityError::DatabaseError(_) => ErrorCategory::Database,
            ProximityError::SerializationError(_) => ErrorCategory::Internal,
            ProximityError::ConnectionFailed(_) => ErrorCategory::Network,
            ProximityError::InternalError(_) => ErrorCategory::Internal,
            ProximityError::DiscoveryNotEnabled => ErrorCategory::Validation,
        }
    }
}

/// Error categories for monitoring and metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Validation,
    NotFound,
    Authentication,
    Transaction,
    Network,
    Timeout,
    Permission,
    RateLimit,
    Session,
    Database,
    Internal,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Validation => write!(f, "validation"),
            ErrorCategory::NotFound => write!(f, "not_found"),
            ErrorCategory::Authentication => write!(f, "authentication"),
            ErrorCategory::Transaction => write!(f, "transaction"),
            ErrorCategory::Network => write!(f, "network"),
            ErrorCategory::Timeout => write!(f, "timeout"),
            ErrorCategory::Permission => write!(f, "permission"),
            ErrorCategory::RateLimit => write!(f, "rate_limit"),
            ErrorCategory::Session => write!(f, "session"),
            ErrorCategory::Database => write!(f, "database"),
            ErrorCategory::Internal => write!(f, "internal"),
        }
    }
}

