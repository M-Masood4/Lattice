use std::fmt;

#[derive(Debug)]
pub enum AIServiceError {
    ApiError(String),
    ParseError(String),
    ValidationError(String),
}

impl fmt::Display for AIServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIServiceError::ApiError(msg) => write!(f, "API error: {}", msg),
            AIServiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AIServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for AIServiceError {}

pub type Result<T> = std::result::Result<T, AIServiceError>;
