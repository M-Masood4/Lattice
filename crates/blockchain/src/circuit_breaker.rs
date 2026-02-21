use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Number of successes in half-open state before closing
    pub success_threshold: u32,
    /// Time to wait before transitioning from open to half-open
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
    name: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        info!(
            "Initializing circuit breaker '{}' with failure_threshold={}, success_threshold={}, timeout={:?}",
            name, config.failure_threshold, config.success_threshold, config.timeout
        );
        
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
            name,
        }
    }

    /// Check if the circuit breaker allows the request
    pub async fn is_request_allowed(&self) -> bool {
        let state = self.state.read().await;
        
        match *state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                drop(state);
                // Check if timeout has elapsed
                let last_failure = self.last_failure_time.read().await;
                if let Some(last_time) = *last_failure {
                    if last_time.elapsed() >= self.config.timeout {
                        drop(last_failure);
                        // Transition to half-open
                        self.transition_to_half_open().await;
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let state = self.state.read().await.clone();
        
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                let mut failure_count = self.failure_count.write().await;
                *failure_count = 0;
                debug!("Circuit breaker '{}': Success recorded, failure count reset", self.name);
            }
            CircuitState::HalfOpen => {
                let mut success_count = self.success_count.write().await;
                *success_count += 1;
                
                debug!(
                    "Circuit breaker '{}': Success in half-open state ({}/{})",
                    self.name, *success_count, self.config.success_threshold
                );
                
                if *success_count >= self.config.success_threshold {
                    drop(success_count);
                    self.transition_to_closed().await;
                }
            }
            CircuitState::Open => {
                // Should not happen, but log it
                warn!("Circuit breaker '{}': Success recorded in open state", self.name);
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let state = self.state.read().await.clone();
        
        match state {
            CircuitState::Closed => {
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;
                
                debug!(
                    "Circuit breaker '{}': Failure recorded ({}/{})",
                    self.name, *failure_count, self.config.failure_threshold
                );
                
                if *failure_count >= self.config.failure_threshold {
                    drop(failure_count);
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open immediately opens the circuit
                self.transition_to_open().await;
            }
            CircuitState::Open => {
                // Already open, just update the timestamp
                let mut last_failure = self.last_failure_time.write().await;
                *last_failure = Some(Instant::now());
            }
        }
    }

    /// Get the current state of the circuit breaker
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Open;
        
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = Some(Instant::now());
        
        warn!("Circuit breaker '{}': Transitioned to OPEN state", self.name);
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;
        
        let mut success_count = self.success_count.write().await;
        *success_count = 0;
        
        info!("Circuit breaker '{}': Transitioned to HALF_OPEN state", self.name);
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        
        let mut failure_count = self.failure_count.write().await;
        *failure_count = 0;
        
        let mut success_count = self.success_count.write().await;
        *success_count = 0;
        
        info!("Circuit breaker '{}': Transitioned to CLOSED state", self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_closed_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Initially closed
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert!(cb.is_request_allowed().await);

        // Record failures
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        
        cb.record_failure().await;
        // Should transition to open
        assert_eq!(cb.get_state().await, CircuitState::Open);
        assert!(!cb.is_request_allowed().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_to_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Trigger open state
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(60)).await;

        // Should allow request and transition to half-open
        assert!(cb.is_request_allowed().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Trigger open state
        cb.record_failure().await;
        cb.record_failure().await;

        // Wait and transition to half-open
        sleep(Duration::from_millis(60)).await;
        assert!(cb.is_request_allowed().await);

        // Record successes
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
        
        cb.record_success().await;
        // Should transition to closed
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Trigger open state
        cb.record_failure().await;
        cb.record_failure().await;

        // Wait and transition to half-open
        sleep(Duration::from_millis(60)).await;
        assert!(cb.is_request_allowed().await);

        // Record failure in half-open
        cb.record_failure().await;
        // Should immediately go back to open
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Record some failures
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Record success - should reset failure count
        cb.record_success().await;

        // Record more failures - should need 3 more to open
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }
}
