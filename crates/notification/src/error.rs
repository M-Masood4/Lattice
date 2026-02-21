use std::fmt;

#[derive(Debug)]
pub enum NotificationError {
    DatabaseError(String),
    ValidationError(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            NotificationError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for NotificationError {}

pub type Result<T> = std::result::Result<T, NotificationError>;
