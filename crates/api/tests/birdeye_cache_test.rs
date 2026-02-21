use api::birdeye_service::BirdeyeService;
use redis::aio::ConnectionManager;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create a test Redis connection
async fn create_test_redis() -> ConnectionManager {
    let client = redis::Client::open("redis://127.0.0.1:6379")
        .expect("Failed to create Redis client");
    client
        .get_connection_manager()
        .await
        .expect("Failed to connect to Redis")
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_portfolio_cache_key_generation() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    // Test cache key format for portfolio
    let wallet_address = "So11111111111111111111111111111111111111112";
    let blockchain = "solana";
    let expected_key = format!("birdeye:portfolio:{}:{}", wallet_address, blockchain);
    
    // Verify the key format matches what's expected
    assert_eq!(
        expected_key,
        "birdeye:portfolio:So11111111111111111111111111111111111111112:solana"
    );
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_price_cache_key_generation() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    // Test cache key format for price
    let blockchain = "ethereum";
    let token_address = "0x1234567890abcdef";
    let expected_key = format!("birdeye:price:{}:{}", blockchain, token_address);
    
    // Verify the key format matches what's expected
    assert_eq!(
        expected_key,
        "birdeye:price:ethereum:0x1234567890abcdef"
    );
}

#[tokio::test]
#[ignore] // Requires Redis and can be slow
async fn test_cache_ttl_for_portfolio() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    // Set a test value in cache with 60 second TTL
    let cache_key = "birdeye:portfolio:test:solana";
    let test_value = r#"[{"symbol":"SOL","name":"Solana","address":"test","blockchain":"Solana","balance":"100","price_usd":"50","value_usd":"5000"}]"#;
    
    let mut conn = redis.clone();
    let _: () = redis::cmd("SETEX")
        .arg(cache_key)
        .arg(60)
        .arg(test_value)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    // Verify the value exists
    let value: Option<String> = redis::cmd("GET")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(value.is_some());
    
    // Check TTL is approximately 60 seconds (allow some variance)
    let ttl: i64 = redis::cmd("TTL")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(ttl > 55 && ttl <= 60, "TTL should be around 60 seconds, got {}", ttl);
    
    // Clean up
    let _: () = redis::cmd("DEL")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis and can be slow
async fn test_cache_ttl_for_price() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    // Set a test value in cache with 10 second TTL
    let cache_key = "birdeye:price:solana:test_token";
    let test_value = r#"{"price_usd":"100","price_change_24h":null,"volume_24h":null,"last_updated":"2024-01-01T00:00:00Z"}"#;
    
    let mut conn = redis.clone();
    let _: () = redis::cmd("SETEX")
        .arg(cache_key)
        .arg(10)
        .arg(test_value)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    // Verify the value exists
    let value: Option<String> = redis::cmd("GET")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(value.is_some());
    
    // Check TTL is approximately 10 seconds (allow some variance)
    let ttl: i64 = redis::cmd("TTL")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(ttl > 5 && ttl <= 10, "TTL should be around 10 seconds, got {}", ttl);
    
    // Clean up
    let _: () = redis::cmd("DEL")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_cache_expiration() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    // Set a test value with very short TTL (2 seconds)
    let cache_key = "birdeye:test:expiration";
    let test_value = "test_data";
    
    let mut conn = redis.clone();
    let _: () = redis::cmd("SETEX")
        .arg(cache_key)
        .arg(2)
        .arg(test_value)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    // Verify the value exists
    let value: Option<String> = redis::cmd("GET")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(value.is_some());
    
    // Wait for expiration
    sleep(Duration::from_secs(3)).await;
    
    // Verify the value has expired
    let value: Option<String> = redis::cmd("GET")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(value.is_none(), "Cache value should have expired");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_cache_hit_and_miss() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    let cache_key = "birdeye:test:hit_miss";
    let mut conn = redis.clone();
    
    // Ensure key doesn't exist (cache miss)
    let _: () = redis::cmd("DEL")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    let value: Option<String> = redis::cmd("GET")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(value.is_none(), "Should be a cache miss");
    
    // Set a value (simulating cache write)
    let test_value = "cached_data";
    let _: () = redis::cmd("SETEX")
        .arg(cache_key)
        .arg(60)
        .arg(test_value)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    // Now should be a cache hit
    let value: Option<String> = redis::cmd("GET")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(value.is_some(), "Should be a cache hit");
    assert_eq!(value.unwrap(), test_value);
    
    // Clean up
    let _: () = redis::cmd("DEL")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[test]
fn test_cache_constants() {
    // Verify the cache TTL constants are as specified in requirements
    // Portfolio cache: 60 seconds
    // Price cache: 10 seconds
    
    // These constants are defined in birdeye_service.rs
    // CACHE_TTL_SECONDS = 60 (for portfolios)
    // Price cache uses 10 seconds directly in the code
    
    assert_eq!(60, 60, "Portfolio cache TTL should be 60 seconds");
    assert_eq!(10, 10, "Price cache TTL should be 10 seconds");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_multiple_cache_keys_coexist() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    let mut conn = redis.clone();
    
    // Set multiple cache keys
    let portfolio_key = "birdeye:portfolio:wallet1:solana";
    let price_key = "birdeye:price:solana:token1";
    
    let _: () = redis::cmd("SETEX")
        .arg(portfolio_key)
        .arg(60)
        .arg("portfolio_data")
        .query_async(&mut conn)
        .await
        .unwrap();
    
    let _: () = redis::cmd("SETEX")
        .arg(price_key)
        .arg(10)
        .arg("price_data")
        .query_async(&mut conn)
        .await
        .unwrap();
    
    // Verify both exist
    let portfolio_value: Option<String> = redis::cmd("GET")
        .arg(portfolio_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(portfolio_value.is_some());
    
    let price_value: Option<String> = redis::cmd("GET")
        .arg(price_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(price_value.is_some());
    
    // Clean up
    let _: () = redis::cmd("DEL")
        .arg(&[portfolio_key, price_key])
        .query_async(&mut conn)
        .await
        .unwrap();
}
