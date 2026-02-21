use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("Stripe API error: {0}")]
    StripeError(#[from] stripe::StripeError),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    #[error("Invalid subscription tier: {0}")]
    InvalidTier(String),

    #[error("Payment processing failed: {0}")]
    PaymentFailed(String),

    #[error("Webhook verification failed: {0}")]
    WebhookVerificationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, PaymentError>;
