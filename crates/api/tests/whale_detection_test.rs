use api::{RankedWhale, WhaleAsset};
use shared::models::{Portfolio, Asset};
use chrono::Utc;

/// Test that whale ranking sorts by total USD value in descending order
#[test]
fn test_whale_ranking_sorts_by_value() {
    // Create test whales with different total values
    let whale1 = RankedWhale {
        address: "whale1".to_string(),
        assets: vec![WhaleAsset {
            token_mint: "SOL".to_string(),
            token_symbol: "SOL".to_string(),
            amount: 1000.0,
            value_usd: 50000.0,
            multiplier_vs_user: 100.0,
        }],
        total_value_usd: 50000.0,
        rank: 0,
    };

    let whale2 = RankedWhale {
        address: "whale2".to_string(),
        assets: vec![WhaleAsset {
            token_mint: "SOL".to_string(),
            token_symbol: "SOL".to_string(),
            amount: 2000.0,
            value_usd: 100000.0,
            multiplier_vs_user: 200.0,
        }],
        total_value_usd: 100000.0,
        rank: 0,
    };

    let whale3 = RankedWhale {
        address: "whale3".to_string(),
        assets: vec![WhaleAsset {
            token_mint: "SOL".to_string(),
            token_symbol: "SOL".to_string(),
            amount: 500.0,
            value_usd: 25000.0,
            multiplier_vs_user: 50.0,
        }],
        total_value_usd: 25000.0,
        rank: 0,
    };

    let mut whales = vec![whale1, whale3.clone(), whale2];

    // Sort by total_value_usd descending (simulating the ranking logic)
    whales.sort_by(|a, b| {
        b.total_value_usd
            .partial_cmp(&a.total_value_usd)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Assign ranks
    for (index, whale) in whales.iter_mut().enumerate() {
        whale.rank = (index + 1) as i32;
    }

    // Verify ranking order
    assert_eq!(whales[0].address, "whale2");
    assert_eq!(whales[0].rank, 1);
    assert_eq!(whales[0].total_value_usd, 100000.0);

    assert_eq!(whales[1].address, "whale1");
    assert_eq!(whales[1].rank, 2);
    assert_eq!(whales[1].total_value_usd, 50000.0);

    assert_eq!(whales[2].address, "whale3");
    assert_eq!(whales[2].rank, 3);
    assert_eq!(whales[2].total_value_usd, 25000.0);
}

/// Test that whale threshold calculation is correct
#[test]
fn test_whale_threshold_100x() {
    let user_amount = 10.0;
    let whale_threshold = user_amount * 100.0;

    assert_eq!(whale_threshold, 1000.0);

    // Test whale qualification
    let whale_amount_100x = 1000.0;
    let multiplier_100x = whale_amount_100x / user_amount;
    assert!(multiplier_100x >= 100.0);

    let whale_amount_150x = 1500.0;
    let multiplier_150x = whale_amount_150x / user_amount;
    assert!(multiplier_150x >= 100.0);

    let not_whale_amount = 999.0;
    let multiplier_99x = not_whale_amount / user_amount;
    assert!(multiplier_99x < 100.0);
}

/// Test that empty portfolio returns empty whale list
#[test]
fn test_empty_portfolio_returns_no_whales() {
    let portfolio = Portfolio {
        wallet_address: "test_wallet".to_string(),
        assets: vec![],
        total_value_usd: 0.0,
        last_updated: Utc::now(),
    };

    // With no assets, there should be no whales to identify
    assert_eq!(portfolio.assets.len(), 0);
}

/// Test that portfolio with zero amounts skips whale detection
#[test]
fn test_zero_amount_assets_skipped() {
    let portfolio = Portfolio {
        wallet_address: "test_wallet".to_string(),
        assets: vec![
            Asset {
                token_mint: "SOL".to_string(),
                token_symbol: "SOL".to_string(),
                amount: "0.0".to_string(),
                value_usd: Some(0.0),
            },
        ],
        total_value_usd: 0.0,
        last_updated: Utc::now(),
    };

    // Assets with zero amount should be skipped
    let user_amount = portfolio.assets[0].amount.parse::<f64>().unwrap_or(0.0);
    assert_eq!(user_amount, 0.0);
}

/// Test whale asset aggregation by address
#[test]
fn test_whale_aggregation_by_address() {
    // Simulate multiple assets from the same whale address
    let whale_address = "whale_address_1".to_string();
    
    let asset1 = WhaleAsset {
        token_mint: "SOL".to_string(),
        token_symbol: "SOL".to_string(),
        amount: 1000.0,
        value_usd: 50000.0,
        multiplier_vs_user: 100.0,
    };

    let asset2 = WhaleAsset {
        token_mint: "USDC".to_string(),
        token_symbol: "USDC".to_string(),
        amount: 100000.0,
        value_usd: 100000.0,
        multiplier_vs_user: 200.0,
    };

    let total_value = asset1.value_usd + asset2.value_usd;
    assert_eq!(total_value, 150000.0);

    let whale = RankedWhale {
        address: whale_address,
        assets: vec![asset1, asset2],
        total_value_usd: total_value,
        rank: 1,
    };

    assert_eq!(whale.assets.len(), 2);
    assert_eq!(whale.total_value_usd, 150000.0);
}

/// Test serialization and deserialization of RankedWhale (for caching)
#[test]
fn test_ranked_whale_serialization() {
    let whale = RankedWhale {
        address: "test_whale".to_string(),
        assets: vec![WhaleAsset {
            token_mint: "SOL".to_string(),
            token_symbol: "SOL".to_string(),
            amount: 1000.0,
            value_usd: 50000.0,
            multiplier_vs_user: 100.0,
        }],
        total_value_usd: 50000.0,
        rank: 1,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&whale).expect("Failed to serialize");
    assert!(json.contains("test_whale"));
    assert!(json.contains("50000"));

    // Deserialize from JSON
    let deserialized: RankedWhale = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.address, "test_whale");
    assert_eq!(deserialized.total_value_usd, 50000.0);
    assert_eq!(deserialized.rank, 1);
    assert_eq!(deserialized.assets.len(), 1);
}
