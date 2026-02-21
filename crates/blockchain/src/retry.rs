use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (typically 2.0)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate the delay for a given attempt number (0-indexed)
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.initial_delay.as_millis() as f64)
            * self.backoff_multiplier.powi(attempt as i32);
        
        let delay = Duration::from_millis(delay_ms as u64);
        
        // Cap at max_delay
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
}

/// Execute an operation with retry logic and exponential backoff
///
/// # Arguments
/// * `operation_name` - Name of the operation for logging
/// * `config` - Retry configuration
/// * `operation` - Async function to execute
///
/// # Returns
/// Result of the operation or the last error encountered
pub async fn retry_with_backoff<F, Fut, T, E>(
    operation_name: &str,
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;
    
    for attempt in 0..config.max_attempts {
        debug!(
            "Executing '{}' - attempt {}/{}",
            operation_name,
            attempt + 1,
            config.max_attempts
        );
        
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!(
                        "'{}' succeeded on attempt {}/{}",
                        operation_name,
                        attempt + 1,
                        config.max_attempts
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                warn!(
                    "'{}' failed on attempt {}/{}: {}",
                    operation_name,
                    attempt + 1,
                    config.max_attempts,
                    e
                );
                
                last_error = Some(e);
                
                // Don't sleep after the last attempt
                if attempt < config.max_attempts - 1 {
                    let delay = config.calculate_delay(attempt);
                    debug!(
                        "Retrying '{}' after {:?} (exponential backoff)",
                        operation_name, delay
                    );
                    sleep(delay).await;
                }
            }
        }
    }
    
    // All attempts failed, return the last error
    Err(last_error.expect("Should have at least one error"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_calculate_delay_exponential() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        };

        // Attempt 0: 100ms
        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        
        // Attempt 1: 200ms
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        
        // Attempt 2: 400ms
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        
        // Attempt 3: 800ms
        assert_eq!(config.calculate_delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_calculate_delay_capped_at_max() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        };

        // Attempt 0: 1s
        assert_eq!(config.calculate_delay(0), Duration::from_secs(1));
        
        // Attempt 1: 2s
        assert_eq!(config.calculate_delay(1), Duration::from_secs(2));
        
        // Attempt 2: 4s
        assert_eq!(config.calculate_delay(2), Duration::from_secs(4));
        
        // Attempt 3: would be 8s, but capped at 5s
        assert_eq!(config.calculate_delay(3), Duration::from_secs(5));
        
        // Attempt 4: still capped at 5s
        assert_eq!(config.calculate_delay(4), Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_retry_succeeds_first_attempt() {
        let config = RetryConfig::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            "test_operation",
            &config,
            || async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok::<_, String>(42)
            },
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        };
        
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            "test_operation",
            &config,
            || async {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err("Temporary failure".to_string())
                } else {
                    Ok(42)
                }
            },
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_all_attempts() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        };
        
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            "test_operation",
            &config,
            || async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>("Permanent failure".to_string())
            },
        )
        .await;

        assert_eq!(result, Err("Permanent failure".to_string()));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exponential_backoff_timing() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        };
        
        let start = std::time::Instant::now();
        
        let _result = retry_with_backoff(
            "test_operation",
            &config,
            || async { Err::<i32, _>("Always fails".to_string()) },
        )
        .await;

        let elapsed = start.elapsed();
        
        // Should take at least: 50ms (first retry) + 100ms (second retry) = 150ms
        // Allow some margin for test execution overhead
        assert!(elapsed >= Duration::from_millis(140));
        
        // Should not take too long (with overhead, should be under 300ms)
        assert!(elapsed < Duration::from_millis(300));
    }
}
