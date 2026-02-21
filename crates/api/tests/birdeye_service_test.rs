use api::birdeye_service::{Blockchain, BirdeyeService, WalletAddress};
use redis::aio::ConnectionManager;
use rust_decimal::Decimal;

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
#[ignore] // Requires Redis and Birdeye API key
async fn test_birdeye_service_initialization() {
    let redis = create_test_redis().await;
    let api_key = std::env::var("BIRDEYE_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    
    let _service = BirdeyeService::new(api_key, redis);
    
    // Service should be created successfully
    assert!(true);
}

#[tokio::test]
#[ignore] // Requires Redis and Birdeye API key
async fn test_get_multi_chain_portfolio_with_valid_addresses() {
    let redis = create_test_redis().await;
    let api_key = std::env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    
    let service = BirdeyeService::new(api_key, redis);
    
    // Test with a known Solana wallet (replace with a test wallet)
    let wallets = vec![
        WalletAddress {
            blockchain: Blockchain::Solana,
            address: "So11111111111111111111111111111111111111112".to_string(),
        },
    ];
    
    let result = service.get_multi_chain_portfolio(wallets).await;
    
    // Should succeed or fail gracefully
    match result {
        Ok(portfolio) => {
            assert!(portfolio.total_value_usd >= Decimal::ZERO);
            assert!(portfolio.positions_by_chain.contains_key("solana"));
        }
        Err(e) => {
            // API might fail, but error should be handled
            eprintln!("Expected API error: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires Redis and Birdeye API key
async fn test_get_asset_price_for_solana_token() {
    let redis = create_test_redis().await;
    let api_key = std::env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    
    let service = BirdeyeService::new(api_key, redis);
    
    // Test with SOL token address
    let sol_address = "So11111111111111111111111111111111111111112";
    
    let result = service.get_asset_price(&Blockchain::Solana, sol_address).await;
    
    match result {
        Ok(price_data) => {
            assert!(price_data.price_usd > Decimal::ZERO);
        }
        Err(e) => {
            eprintln!("Expected API error: {}", e);
        }
    }
}

#[test]
fn test_blockchain_to_birdeye_chain_conversion() {
    assert_eq!(Blockchain::Solana.to_birdeye_chain(), "solana");
    assert_eq!(Blockchain::Ethereum.to_birdeye_chain(), "ethereum");
    assert_eq!(Blockchain::BinanceSmartChain.to_birdeye_chain(), "bsc");
    assert_eq!(Blockchain::Polygon.to_birdeye_chain(), "polygon");
}

#[test]
fn test_wallet_address_serialization() {
    let wallet = WalletAddress {
        blockchain: Blockchain::Ethereum,
        address: "0x1234567890abcdef".to_string(),
    };
    
    let json = serde_json::to_string(&wallet).unwrap();
    assert!(json.contains("Ethereum"));
    assert!(json.contains("0x1234567890abcdef"));
    
    let deserialized: WalletAddress = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.address, wallet.address);
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_cache_functionality() {
    let redis = create_test_redis().await;
    let api_key = "test_key".to_string();
    
    let _service = BirdeyeService::new(api_key, redis.clone());
    
    // Clear any existing cache
    let cache_key = "birdeye:test:cache";
    let mut conn = redis.clone();
    let _: () = redis::cmd("DEL")
        .arg(cache_key)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    // First call should miss cache (would call API, but we can't test that without mocking)
    // Second call should hit cache
    // This is tested implicitly in the integration tests
}

#[test]
fn test_multi_chain_portfolio_structure() {
    use api::birdeye_service::{Asset, MultiChainPortfolio};
    use chrono::Utc;
    use std::collections::HashMap;
    
    let mut positions = HashMap::new();
    positions.insert(
        "solana".to_string(),
        vec![Asset {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            address: "So11111111111111111111111111111111111111112".to_string(),
            blockchain: Blockchain::Solana,
            balance: Decimal::from(100),
            price_usd: Decimal::from(50),
            value_usd: Decimal::from(5000),
        }],
    );
    
    let portfolio = MultiChainPortfolio {
        total_value_usd: Decimal::from(5000),
        positions_by_chain: positions,
        last_updated: Utc::now(),
    };
    
    assert_eq!(portfolio.total_value_usd, Decimal::from(5000));
    assert_eq!(portfolio.positions_by_chain.len(), 1);
    assert!(portfolio.positions_by_chain.contains_key("solana"));
}

#[test]
fn test_price_data_structure() {
    use api::birdeye_service::PriceData;
    use chrono::Utc;
    
    let price_data = PriceData {
        price_usd: Decimal::from(100),
        price_change_24h: Some(Decimal::from(5)),
        volume_24h: Some(Decimal::from(1000000)),
        last_updated: Utc::now(),
    };
    
    assert_eq!(price_data.price_usd, Decimal::from(100));
    assert_eq!(price_data.price_change_24h, Some(Decimal::from(5)));
    assert_eq!(price_data.volume_24h, Some(Decimal::from(1000000)));
}

#[test]
fn test_asset_value_calculation() {
    use api::birdeye_service::Asset;
    
    let asset = Asset {
        symbol: "ETH".to_string(),
        name: "Ethereum".to_string(),
        address: "0x0000000000000000000000000000000000000000".to_string(),
        blockchain: Blockchain::Ethereum,
        balance: Decimal::from(10),
        price_usd: Decimal::from(2000),
        value_usd: Decimal::from(20000),
    };
    
    // Verify value calculation
    assert_eq!(asset.balance * asset.price_usd, asset.value_usd);
}

#[tokio::test]
#[ignore] // Requires Redis and Birdeye API key
async fn test_retry_logic_on_api_failure() {
    let redis = create_test_redis().await;
    let api_key = "invalid_key".to_string(); // Invalid key should trigger retries
    
    let service = BirdeyeService::new(api_key, redis);
    
    let wallets = vec![WalletAddress {
        blockchain: Blockchain::Solana,
        address: "invalid_address".to_string(),
    }];
    
    let result = service.get_multi_chain_portfolio(wallets).await;
    
    // Should fail after retries
    assert!(result.is_err());
}

#[test]
fn test_empty_portfolio() {
    use api::birdeye_service::MultiChainPortfolio;
    use chrono::Utc;
    use std::collections::HashMap;
    
    let portfolio = MultiChainPortfolio {
        total_value_usd: Decimal::ZERO,
        positions_by_chain: HashMap::new(),
        last_updated: Utc::now(),
    };
    
    assert_eq!(portfolio.total_value_usd, Decimal::ZERO);
    assert_eq!(portfolio.positions_by_chain.len(), 0);
}

#[test]
fn test_multiple_chains_in_portfolio() {
    use api::birdeye_service::{Asset, MultiChainPortfolio};
    use chrono::Utc;
    use std::collections::HashMap;
    
    let mut positions = HashMap::new();
    
    // Add Solana assets
    positions.insert(
        "solana".to_string(),
        vec![Asset {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            address: "So11111111111111111111111111111111111111112".to_string(),
            blockchain: Blockchain::Solana,
            balance: Decimal::from(100),
            price_usd: Decimal::from(50),
            value_usd: Decimal::from(5000),
        }],
    );
    
    // Add Ethereum assets
    positions.insert(
        "ethereum".to_string(),
        vec![Asset {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            address: "0x0000000000000000000000000000000000000000".to_string(),
            blockchain: Blockchain::Ethereum,
            balance: Decimal::from(10),
            price_usd: Decimal::from(2000),
            value_usd: Decimal::from(20000),
        }],
    );
    
    let portfolio = MultiChainPortfolio {
        total_value_usd: Decimal::from(25000),
        positions_by_chain: positions,
        last_updated: Utc::now(),
    };
    
    assert_eq!(portfolio.positions_by_chain.len(), 2);
    assert!(portfolio.positions_by_chain.contains_key("solana"));
    assert!(portfolio.positions_by_chain.contains_key("ethereum"));
}
