# Retry Logic and Circuit Breaker Implementation

This document describes the retry logic with exponential backoff and circuit breaker pattern implemented for Solana RPC calls.

## Overview

The implementation provides robust error handling for Solana RPC interactions through:

1. **Exponential Backoff Retry Logic**: Automatically retries failed operations with increasing delays
2. **Circuit Breaker Pattern**: Prevents cascading failures by temporarily blocking requests to failing services
3. **Fallback RPC Support**: Automatically switches to fallback RPC endpoints when primary fails

## Components

### 1. Retry Module (`retry.rs`)

Implements exponential backoff retry logic for transient failures.

**Key Features:**
- Configurable maximum retry attempts (default: 3)
- Exponential delay calculation with configurable multiplier (default: 2.0)
- Maximum delay cap to prevent excessive wait times
- Detailed logging of retry attempts

**Configuration:**
```rust
RetryConfig {
    max_attempts: 3,                           // Retry up to 3 times
    initial_delay: Duration::from_millis(100), // Start with 100ms delay
    max_delay: Duration::from_secs(10),        // Cap delays at 10 seconds
    backoff_multiplier: 2.0,                   // Double delay each retry
}
```

**Delay Progression:**
- Attempt 1: 100ms
- Attempt 2: 200ms
- Attempt 3: 400ms
- Attempt 4: 800ms (capped at max_delay if configured)

### 2. Circuit Breaker Module (`circuit_breaker.rs`)

Implements the circuit breaker pattern to prevent overwhelming failing services.

**States:**
- **Closed**: Normal operation, requests flow through
- **Open**: Service is failing, requests are rejected immediately
- **Half-Open**: Testing if service has recovered

**Configuration:**
```rust
CircuitBreakerConfig {
    failure_threshold: 5,      // Open after 5 consecutive failures
    success_threshold: 2,      // Close after 2 consecutive successes in half-open
    timeout: Duration::from_secs(30), // Wait 30s before trying half-open
}
```

**State Transitions:**
1. **Closed → Open**: After `failure_threshold` consecutive failures
2. **Open → Half-Open**: After `timeout` duration elapses
3. **Half-Open → Closed**: After `success_threshold` consecutive successes
4. **Half-Open → Open**: On any failure

### 3. Enhanced SolanaClient (`client.rs`)

The `SolanaClient` integrates both retry logic and circuit breaker for all RPC operations.

**Features:**
- Separate circuit breakers for primary and fallback RPCs
- Automatic fallback to secondary RPC when primary fails
- Configurable retry and circuit breaker settings
- Comprehensive error logging and monitoring

## Usage Examples

### Basic Usage (Default Configuration)

```rust
use blockchain::SolanaClient;

let client = SolanaClient::new(
    "https://api.mainnet-beta.solana.com".to_string(),
    Some("https://api.devnet.solana.com".to_string()), // Fallback
);

// Automatically retries with exponential backoff and circuit breaker
let balance = client.get_sol_balance("wallet_address").await?;
```

### Custom Configuration

```rust
use blockchain::{SolanaClient, RetryConfig, CircuitBreakerConfig};
use std::time::Duration;

let retry_config = RetryConfig {
    max_attempts: 5,
    initial_delay: Duration::from_millis(50),
    max_delay: Duration::from_secs(5),
    backoff_multiplier: 2.0,
};

let circuit_breaker_config = CircuitBreakerConfig {
    failure_threshold: 3,
    success_threshold: 2,
    timeout: Duration::from_secs(20),
};

let client = SolanaClient::new_with_config(
    "https://api.mainnet-beta.solana.com".to_string(),
    Some("https://api.devnet.solana.com".to_string()),
    retry_config,
    circuit_breaker_config,
);
```

## Error Handling Flow

### Scenario 1: Transient Failure (Network Glitch)

1. Primary RPC call fails
2. Retry logic waits 100ms (exponential backoff)
3. Retry succeeds
4. Circuit breaker records success
5. Operation completes successfully

### Scenario 2: Primary RPC Down, Fallback Available

1. Primary RPC call fails (attempt 1)
2. Retry after 100ms - fails (attempt 2)
3. Retry after 200ms - fails (attempt 3)
4. Circuit breaker records 3 failures
5. Fallback RPC is attempted
6. Fallback succeeds
7. Operation completes successfully

### Scenario 3: Circuit Breaker Opens

1. Primary RPC experiences 5 consecutive failures
2. Circuit breaker transitions to OPEN state
3. Subsequent requests immediately return `CircuitBreakerOpen` error
4. After 30 seconds, circuit breaker transitions to HALF-OPEN
5. Next request is allowed through (testing)
6. If successful, circuit closes; if failed, reopens

### Scenario 4: Both Primary and Fallback Fail

1. Primary RPC fails after all retries
2. Fallback RPC is attempted
3. Fallback also fails after all retries
4. Both circuit breakers record failures
5. Error is returned to caller with detailed context

## Monitoring and Logging

The implementation provides comprehensive logging at different levels:

- **INFO**: Circuit breaker state transitions, client initialization
- **DEBUG**: Individual retry attempts, successful operations, circuit breaker checks
- **WARN**: RPC failures, circuit breaker opening, fallback attempts
- **ERROR**: Complete operation failures, both primary and fallback failed

Example log output:
```
INFO  Initializing Solana client with primary RPC: https://api.mainnet-beta.solana.com
INFO  Configuring fallback RPC: https://api.devnet.solana.com
INFO  Initializing circuit breaker 'primary-rpc-...' with failure_threshold=5, success_threshold=2, timeout=30s
DEBUG Executing 'get_sol_balance_primary' - attempt 1/3
WARN  'get_sol_balance_primary' failed on attempt 1/3: Primary RPC failed: connection timeout
DEBUG Retrying 'get_sol_balance_primary' after 100ms (exponential backoff)
DEBUG Executing 'get_sol_balance_primary' - attempt 2/3
DEBUG 'get_sol_balance_primary' succeeded on attempt 2/3
DEBUG Circuit breaker 'primary-rpc-...': Success recorded, failure count reset
```

## Testing

The implementation includes comprehensive unit tests:

### Circuit Breaker Tests
- State transitions (Closed → Open → Half-Open → Closed)
- Failure threshold enforcement
- Success threshold in half-open state
- Timeout behavior
- Failure count reset on success

### Retry Tests
- Exponential backoff delay calculation
- Delay capping at maximum
- Success on first attempt
- Success after failures
- All attempts fail
- Timing verification

### Integration Tests
- Client with custom configuration
- Address validation
- Error handling flow

Run tests:
```bash
# Unit tests
cargo test --package blockchain --lib

# Integration tests
cargo test --package blockchain --test integration_test

# All tests
cargo test --package blockchain
```

## Performance Considerations

### Retry Logic
- **Best Case**: Single successful call (no overhead)
- **Worst Case**: 3 attempts with delays = ~700ms total (100ms + 200ms + 400ms)
- **Configurable**: Adjust `max_attempts` and delays based on requirements

### Circuit Breaker
- **Overhead**: Minimal (async RwLock reads/writes)
- **Memory**: ~200 bytes per circuit breaker instance
- **Benefit**: Prevents wasting time on failing services

### Fallback RPC
- **Latency**: Additional attempt only when primary fails
- **Reliability**: Significantly improves overall availability
- **Cost**: Requires additional RPC endpoint subscription

## Requirements Validation

This implementation validates **Requirement 9.3**:

> THE System SHALL handle Solana network congestion gracefully with exponential backoff retry logic

**Validation:**
✅ Exponential backoff implemented with configurable multiplier  
✅ Retry logic handles transient failures gracefully  
✅ Circuit breaker prevents cascading failures during congestion  
✅ Fallback RPC provides additional resilience  
✅ Comprehensive error handling and logging  
✅ Configurable for different network conditions  

## Future Enhancements

Potential improvements for production deployment:

1. **Adaptive Retry**: Adjust retry parameters based on error types
2. **Metrics Collection**: Export circuit breaker state and retry counts to monitoring systems
3. **Rate Limiting**: Add per-endpoint rate limiting to prevent overwhelming RPCs
4. **Health Checks**: Periodic health checks for RPC endpoints
5. **Dynamic Fallback**: Automatically discover and use healthy RPC endpoints
6. **Jitter**: Add random jitter to retry delays to prevent thundering herd
