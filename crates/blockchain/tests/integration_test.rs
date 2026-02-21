use blockchain::{CircuitBreakerConfig, RetryConfig, SolanaClient};
use std::time::Duration;

#[tokio::test]
async fn test_client_with_custom_retry_config() {
    // Create a client with custom retry configuration
    let retry_config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    let circuit_breaker_config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(100),
    };

    let client = SolanaClient::new_with_config(
        "https://api.devnet.solana.com".to_string(),
        Some("https://api.mainnet-beta.solana.com".to_string()),
        retry_config,
        circuit_breaker_config,
    );

    // Test with an invalid address to verify error handling
    let result = client.get_sol_balance("invalid_address").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_validates_addresses() {
    let client = SolanaClient::new(
        "https://api.devnet.solana.com".to_string(),
        None,
    );

    // Valid address format
    let valid_result = client.validate_address("11111111111111111111111111111111");
    assert!(valid_result.is_ok());

    // Invalid address format
    let invalid_result = client.validate_address("not_a_valid_address");
    assert!(invalid_result.is_err());
}
