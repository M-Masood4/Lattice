use std::fmt;

#[derive(Debug)]
pub enum TradingError {
    ValidationError(String),
    InsufficientBalance(String),
    DailyLimitExceeded(String),
    SubscriptionRequired(String),
    TransactionError(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TradingError::InsufficientBalance(msg) => write!(f, "Insufficient balance: {}", msg),
            TradingError::DailyLimitExceeded(msg) => write!(f, "Daily limit exceeded: {}", msg),
            TradingError::SubscriptionRequired(msg) => {
                write!(f, "Subscription required: {}", msg)
            }
            TradingError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
        }
    }
}

impl std::error::Error for TradingError {}

pub type Result<T> = std::result::Result<T, TradingError>;
