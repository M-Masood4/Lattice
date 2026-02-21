// Property-Based Tests for Retry Logic
// Feature: solana-whale-tracker
// Property 21: Solana retry exponential backoff
// Validates: Requirements 9.3

use blockchain::retry::{retry_with_backoff, RetryConfig};
use proptest::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 21: For any sequence of Solana RPC failures, the retry delays 
    /// should follow exponential backoff (each delay >= 2x the previous delay).
    #[test]
    fn prop_exponential_backoff_timing(
        max_attempts in 2u32..=5u32,
        initial_delay_ms in 10u64..=100u64,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let config = RetryConfig {
                max_attempts,
                initial_delay: Duration::from_millis(initial_delay_ms),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
            };
            
            let attempt_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));
            let attempt_times_clone = attempt_times.clone();
            
            let start = Instant::now();
            
            // Operation that always fails to trigger all retries
            let _result = retry_with_backoff(
                "test_operation",
                &config,
                || {
                    let times = attempt_times_clone.clone();
                    async move {
                        times.lock().await.push(Instant::now());
                        Err::<(), String>("Always fails".to_string())
                    }
                },
            )
            .await;
            
            let times = attempt_times.lock().await;
            
            // Verify we attempted the correct number of times
            prop_assert_eq!(times.len(), max_attempts as usize);
            
            // Verify exponential backoff between attempts
            for i in 1..times.len() {
                let delay = times[i].duration_since(times[i - 1]);
                let expected_min_delay = config.calculate_delay((i - 1) as u32);
                
                // Allow 20ms tolerance for execution overhead
                let tolerance = Duration::from_millis(20);
                
                prop_assert!(
                    delay >= expected_min_delay.saturating_sub(tolerance),
                    "Delay between attempt {} and {} was {:?}, expected at least {:?}",
                    i - 1,
                    i,
                    delay,
                    expected_min_delay
                );
            }
            
            // Verify each delay is approximately 2x the previous (exponential)
            if times.len() >= 3 {
                for i in 2..times.len() {
                    let delay1 = times[i - 1].duration_since(times[i - 2]);
                    let delay2 = times[i].duration_since(times[i - 1]);
                    
                    // delay2 should be approximately 2x delay1 (within 30% tolerance for timing variance)
                    let ratio = delay2.as_millis() as f64 / delay1.as_millis() as f64;
                    
                    prop_assert!(
                        ratio >= 1.4 && ratio <= 2.6,
                        "Exponential backoff ratio between delays was {:.2}, expected ~2.0",
                        ratio
                    );
                }
            }
            
            Ok(())
        })?;
    }
    
    /// Property: Retry delays respect the configured max_delay cap
    #[test]
    fn prop_max_delay_cap(
        max_attempts in 3u32..=6u32,
        initial_delay_ms in 50u64..=200u64,
        max_delay_ms in 200u64..=500u64,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let config = RetryConfig {
                max_attempts,
                initial_delay: Duration::from_millis(initial_delay_ms),
                max_delay: Duration::from_millis(max_delay_ms),
                backoff_multiplier: 2.0,
            };
            
            let attempt_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));
            let attempt_times_clone = attempt_times.clone();
            
            let _result = retry_with_backoff(
                "test_operation",
                &config,
                || {
                    let times = attempt_times_clone.clone();
                    async move {
                        times.lock().await.push(Instant::now());
                        Err::<(), String>("Always fails".to_string())
                    }
                },
            )
            .await;
            
            let times = attempt_times.lock().await;
            
            // Verify no delay exceeds max_delay (with tolerance)
            for i in 1..times.len() {
                let delay = times[i].duration_since(times[i - 1]);
                let tolerance = Duration::from_millis(50);
                
                prop_assert!(
                    delay <= config.max_delay + tolerance,
                    "Delay {:?} exceeded max_delay {:?}",
                    delay,
                    config.max_delay
                );
            }
            
            Ok(())
        })?;
    }
    
    /// Property: Successful retry stops further attempts
    #[test]
    fn prop_success_stops_retries(
        max_attempts in 2u32..=5u32,
        success_on_attempt in 1u32..=4u32,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let success_attempt = success_on_attempt.min(max_attempts);
            
            let config = RetryConfig {
                max_attempts,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_secs(1),
                backoff_multiplier: 2.0,
            };
            
            let attempt_count = Arc::new(AtomicU32::new(0));
            let attempt_count_clone = attempt_count.clone();
            
            let result = retry_with_backoff(
                "test_operation",
                &config,
                || {
                    let count = attempt_count_clone.clone();
                    async move {
                        let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                        if current >= success_attempt {
                            Ok(42)
                        } else {
                            Err("Not yet".to_string())
                        }
                    }
                },
            )
            .await;
            
            // Should succeed
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), 42);
            
            // Should have attempted exactly success_attempt times
            prop_assert_eq!(attempt_count.load(Ordering::SeqCst), success_attempt);
            
            Ok(())
        })?;
    }
}
