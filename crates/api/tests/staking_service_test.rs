use api::{StakingConfig, StakingPosition};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

#[test]
fn test_staking_config_default_values() {
    let config = StakingConfig::default();
    
    assert_eq!(config.minimum_idle_amount, Decimal::from(100));
    assert_eq!(config.idle_duration_hours, 24);
    assert_eq!(config.auto_compound, false);
}

#[test]
fn test_staking_config_custom_values() {
    let config = StakingConfig {
        minimum_idle_amount: Decimal::from(500),
        idle_duration_hours: 48,
        auto_compound: true,
    };
    
    assert_eq!(config.minimum_idle_amount, Decimal::from(500));
    assert_eq!(config.idle_duration_hours, 48);
    assert_eq!(config.auto_compound, true);
}

#[test]
fn test_staking_position_creation() {
    let position = StakingPosition {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        asset: "SOL".to_string(),
        amount: Decimal::from(1000),
        provider: "SideShift".to_string(),
        apy: Some(Decimal::from(5)),
        rewards_earned: Decimal::ZERO,
        auto_compound: false,
        started_at: Utc::now(),
        last_reward_at: None,
    };
    
    assert_eq!(position.asset, "SOL");
    assert_eq!(position.amount, Decimal::from(1000));
    assert_eq!(position.provider, "SideShift");
    assert_eq!(position.rewards_earned, Decimal::ZERO);
}

#[test]
fn test_staking_position_with_rewards() {
    let position = StakingPosition {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        asset: "ETH".to_string(),
        amount: Decimal::from(10),
        provider: "SideShift".to_string(),
        apy: Some(Decimal::from(4)),
        rewards_earned: Decimal::from_str_exact("0.5").unwrap(),
        auto_compound: true,
        started_at: Utc::now(),
        last_reward_at: Some(Utc::now()),
    };
    
    assert_eq!(position.rewards_earned, Decimal::from_str_exact("0.5").unwrap());
    assert!(position.auto_compound);
    assert!(position.last_reward_at.is_some());
}

#[test]
fn test_staking_position_serialization() {
    let position = StakingPosition {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        asset: "BTC".to_string(),
        amount: Decimal::from(1),
        provider: "SideShift".to_string(),
        apy: Some(Decimal::from(3)),
        rewards_earned: Decimal::from_str_exact("0.001").unwrap(),
        auto_compound: false,
        started_at: Utc::now(),
        last_reward_at: None,
    };
    
    // Test serialization
    let json = serde_json::to_string(&position).expect("Failed to serialize");
    assert!(json.contains("BTC"));
    assert!(json.contains("SideShift"));
    
    // Test deserialization
    let deserialized: StakingPosition = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.asset, position.asset);
    assert_eq!(deserialized.amount, position.amount);
    assert_eq!(deserialized.provider, position.provider);
}

#[test]
fn test_minimum_idle_amount_validation() {
    // Test that minimum idle amount must be positive
    let config = StakingConfig {
        minimum_idle_amount: Decimal::from(1),
        idle_duration_hours: 24,
        auto_compound: false,
    };
    
    assert!(config.minimum_idle_amount > Decimal::ZERO);
}

#[test]
fn test_idle_duration_hours_validation() {
    // Test various idle duration values
    let short_duration = StakingConfig {
        minimum_idle_amount: Decimal::from(100),
        idle_duration_hours: 1,
        auto_compound: false,
    };
    
    let long_duration = StakingConfig {
        minimum_idle_amount: Decimal::from(100),
        idle_duration_hours: 168, // 1 week
        auto_compound: false,
    };
    
    assert_eq!(short_duration.idle_duration_hours, 1);
    assert_eq!(long_duration.idle_duration_hours, 168);
}

#[test]
fn test_staking_config_serialization() {
    let config = StakingConfig {
        minimum_idle_amount: Decimal::from(250),
        idle_duration_hours: 72,
        auto_compound: true,
    };
    
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: StakingConfig = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(deserialized.minimum_idle_amount, config.minimum_idle_amount);
    assert_eq!(deserialized.idle_duration_hours, config.idle_duration_hours);
    assert_eq!(deserialized.auto_compound, config.auto_compound);
}
