# Error Handling and Monitoring Implementation

## Overview

This document describes the comprehensive error handling, logging, and monitoring system implemented for the crypto trading platform. The system provides circuit breakers for external APIs, structured error responses, metrics collection, and health monitoring.

## Components

### 1. Error Module (`error.rs`)

Provides a comprehensive error type system with proper HTTP status code mapping.

#### Error Types

- **Database Errors**: Connection issues, query failures
- **External API Errors**: Birdeye, SideShift, Blockchain RPC failures
- **Circuit Breaker Errors**: Service temporarily unavailable
- **Validation Errors**: Invalid input, validation failures
- **Authentication/Authorization Errors**: Unauthorized, forbidden access
- **Resource Errors**: Not found, already exists
- **Business Logic Errors**: Insufficient balance, expired orders, frozen wallets
- **Rate Limiting**: Too many requests
- **Internal Errors**: Configuration issues, unexpected failures
- **Timeout Errors**: Request timeouts

#### Error Response Format

```json
{
  "error": "error_type",
  "message": "Human-readable error message",
  "details": null,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### Usage Example

```rust
use crate::error::{ApiError, ApiResult};

pub async fn some_operation() -> ApiResult<Data> {
    // Automatic conversion from common error types
    let data = sqlx::query!("SELECT * FROM users")
        .fetch_one(&pool)
        .await?; // Converts sqlx::Error to ApiError
    
    // Manual error creation
    if data.balance < required_amount {
        return Err(ApiError::InsufficientBalance(
            format!("Required: {}, Available: {}", required_amount, data.balance)
        ));
    }
    
    Ok(data)
}
```

### 2. Circuit Breakers

Circuit breakers protect the system from cascading failures when external services are unavailable.

#### Configuration

- **Failure Threshold**: 5 consecutive failures trigger circuit open
- **Success Threshold**: 2 consecutive successes in half-open state close circuit
- **Timeout**: 60 seconds before transitioning from open to half-open

#### Protected Services

1. **Birdeye API** - Multi-chain portfolio and price data
2. **SideShift API** - Cryptocurrency conversions and staking
3. **Blockchain RPCs** - Transaction submission and verification

#### Circuit States

- **Closed**: Normal operation, requests flow through
- **Open**: Service unavailable, requests immediately rejected
- **Half-Open**: Testing if service recovered, limited requests allowed

#### Usage in Services

```rust
// Check circuit breaker before making request
if !self.circuit_breaker.is_request_allowed().await {
    return Err(ApiError::CircuitBreakerOpen(
        "Service temporarily unavailable".to_string()
    ));
}

// Make request
match make_api_call().await {
    Ok(result) => {
        self.circuit_breaker.record_success().await;
        Ok(result)
    }
    Err(e) => {
        self.circuit_breaker.record_failure().await;
        Err(e)
    }
}
```

### 3. Monitoring Module (`monitoring.rs`)

Provides metrics collection, health checks, and alerting capabilities.

#### Metrics Collected

- **Request Counts**: Total, successful, failed requests per service
- **Error Rates**: Percentage of failed requests
- **Response Times**: Average response time in milliseconds
- **Circuit Breaker States**: Current state of each circuit breaker
- **Error Types**: Breakdown of errors by type
- **Last Success/Failure**: Timestamps of last successful/failed requests

#### Health Check Endpoints

1. **`/health`** - Overall system health with all service statuses
2. **`/health/ready`** - Readiness probe (for Kubernetes)
3. **`/health/live`** - Liveness probe (for Kubernetes)
4. **`/metrics`** - All service metrics
5. **`/metrics/:service`** - Metrics for specific service

#### Health Status Types

- **Healthy**: Service operating normally
- **Degraded**: Service experiencing issues but still functional
- **Unhealthy**: Service unavailable
- **Unknown**: No data available for service

#### Request Timer Usage

```rust
use crate::monitoring::RequestTimer;

pub async fn fetch_data(metrics: MetricsCollector) -> ApiResult<Data> {
    let timer = RequestTimer::new("birdeye_api".to_string(), metrics);
    
    match make_request().await {
        Ok(data) => {
            timer.success().await;
            Ok(data)
        }
        Err(e) => {
            timer.failure("api_error").await;
            Err(e)
        }
    }
}
```

#### Alert Manager

Monitors service health and sends alerts when issues are detected:

- **High Error Rate**: Alert if error rate exceeds 10%
- **Circuit Breaker Open**: Critical alert when service unavailable
- **Slow Response Times**: Warning if average response time exceeds 5 seconds

### 4. Logging Module (`logging.rs`)

Provides structured logging with JSON output for production and pretty output for development.

#### Log Levels

- **ERROR**: Critical errors requiring immediate attention
- **WARN**: Warning conditions that should be investigated
- **INFO**: Informational messages about normal operations
- **DEBUG**: Detailed debugging information
- **TRACE**: Very detailed trace information

#### Structured Logging

All logs include:
- Timestamp
- Log level
- Target (module path)
- Thread ID
- File and line number
- Message
- Structured fields (JSON format in production)

#### Configuration

Set log level via `RUST_LOG` environment variable:

```bash
# Development
RUST_LOG=debug cargo run

# Production
RUST_LOG=info,sqlx=warn,hyper=warn cargo run
```

#### Usage Example

```rust
use tracing::{info, warn, error};

// Simple logging
info!("User logged in: {}", user_id);

// Structured logging with fields
info!(
    user_id = %user_id,
    action = "login",
    ip_address = %ip,
    "User authentication successful"
);

// Error logging
error!(
    error = %e,
    service = "birdeye_api",
    "Failed to fetch portfolio data"
);
```

## Integration with Services

### BirdeyeService

- Circuit breaker protection for all API calls
- Retry logic with exponential backoff (3 attempts)
- Comprehensive error handling with ApiError types
- Metrics recording for success/failure rates

### SideShiftClient

- Circuit breaker protection for conversions and staking
- Detailed error messages with API response context
- Metrics tracking for conversion success rates

### Blockchain Clients

- Circuit breakers for each blockchain RPC
- Retry logic for transient failures
- Transaction submission monitoring

## Monitoring Best Practices

### 1. Health Checks

- Use `/health/ready` for Kubernetes readiness probes
- Use `/health/live` for Kubernetes liveness probes
- Monitor `/health` endpoint for overall system status

### 2. Metrics

- Regularly check `/metrics` endpoint for service health
- Set up alerts for high error rates (>10%)
- Monitor circuit breaker states
- Track response time trends

### 3. Logging

- Use structured logging for easy parsing and analysis
- Include context in log messages (user_id, request_id, etc.)
- Log at appropriate levels (don't spam with DEBUG in production)
- Use correlation IDs to trace requests across services

### 4. Alerting

- Configure alerts for circuit breaker open states
- Alert on sustained high error rates
- Monitor for slow response times
- Set up on-call rotation for critical alerts

## Production Deployment

### Environment Variables

```bash
# Logging
RUST_LOG=info,sqlx=warn,hyper=warn,reqwest=warn

# Circuit Breaker Configuration (optional, defaults shown)
CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
CIRCUIT_BREAKER_SUCCESS_THRESHOLD=2
CIRCUIT_BREAKER_TIMEOUT_SECONDS=60

# Metrics
METRICS_ENABLED=true
ALERT_THRESHOLD_ERROR_RATE=10.0
```

### Kubernetes Configuration

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: api
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
```

### Monitoring Integration

The system is designed to integrate with:

- **Prometheus**: Metrics endpoint can be scraped
- **Grafana**: Visualize metrics and create dashboards
- **PagerDuty**: Critical alerts for on-call engineers
- **Slack**: Warning and info alerts for team visibility
- **ELK Stack**: Centralized log aggregation and analysis

## Testing

### Unit Tests

All modules include comprehensive unit tests:

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --package api --lib error
cargo test --package api --lib monitoring
```

### Integration Tests

Test error handling and circuit breakers:

```bash
cargo test --package api --test '*'
```

### Load Testing

Monitor circuit breakers under load:

1. Generate high request volume
2. Observe circuit breaker behavior
3. Verify graceful degradation
4. Check recovery after load reduction

## Troubleshooting

### Circuit Breaker Stuck Open

**Symptoms**: Service returns 503 errors continuously

**Solutions**:
1. Check external service availability
2. Review error logs for root cause
3. Verify network connectivity
4. Wait for timeout period (60s) for automatic recovery
5. Restart service if issue persists

### High Error Rates

**Symptoms**: Error rate exceeds 10%

**Solutions**:
1. Check `/metrics` endpoint for error breakdown
2. Review logs for specific error messages
3. Verify external service status
4. Check database connection pool
5. Monitor resource usage (CPU, memory)

### Slow Response Times

**Symptoms**: Average response time exceeds 5 seconds

**Solutions**:
1. Check database query performance
2. Review external API response times
3. Verify cache hit rates
4. Monitor network latency
5. Consider scaling resources

## Future Enhancements

1. **Distributed Tracing**: Add OpenTelemetry for request tracing
2. **Custom Metrics**: Add business-specific metrics (trades/sec, etc.)
3. **Advanced Alerting**: Implement anomaly detection
4. **Performance Profiling**: Add continuous profiling
5. **Cost Monitoring**: Track external API usage and costs
