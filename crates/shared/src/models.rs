use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// User models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Wallet models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub address: String,
    pub connected_at: DateTime<Utc>,
    pub last_synced: Option<DateTime<Utc>>,
}

// Portfolio models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub wallet_address: String,
    pub assets: Vec<Asset>,
    pub total_value_usd: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub token_mint: String,
    pub token_symbol: String,
    pub amount: String, // Using String to preserve precision
    pub value_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAsset {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub token_mint: String,
    pub token_symbol: String,
    pub amount: String,
    pub value_usd: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

// Whale models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whale {
    pub id: Uuid,
    pub address: String,
    pub total_value_usd: Option<f64>,
    pub first_detected: DateTime<Utc>,
    pub last_checked: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWhaleTracking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub whale_id: Uuid,
    pub token_mint: String,
    pub multiplier: Option<f64>,
    pub rank: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// Whale movement models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleMovement {
    pub id: Uuid,
    pub whale_id: Uuid,
    pub transaction_signature: String,
    pub movement_type: String, // BUY or SELL
    pub token_mint: String,
    pub amount: String,
    pub percent_of_position: Option<f64>,
    pub detected_at: DateTime<Utc>,
}

// AI Recommendation models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: Uuid,
    pub movement_id: Uuid,
    pub user_id: Uuid,
    pub action: String, // HOLD, BUY, SELL, TRIM
    pub confidence: i32,
    pub reasoning: String,
    pub suggested_amount: Option<String>,
    pub timeframe: Option<String>,
    pub risks: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// Trade execution models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    pub id: Uuid,
    pub user_id: Uuid,
    pub recommendation_id: Option<Uuid>,
    pub transaction_signature: String,
    pub action: String,
    pub token_mint: String,
    pub amount: String,
    pub price_usd: Option<f64>,
    pub total_value_usd: Option<f64>,
    pub status: String,
    pub executed_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

// Notification models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub priority: String,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub frequency: String,
    pub minimum_movement_percent: f64,
    pub minimum_confidence: i32,
}

// Subscription models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_subscription_id: String,
    pub tier: String,
    pub status: String,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User settings models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: Uuid,
    pub auto_trader_enabled: bool,
    pub max_trade_percentage: f64,
    pub max_daily_trades: i32,
    pub stop_loss_percentage: f64,
    pub risk_tolerance: String,
    pub updated_at: DateTime<Utc>,
}
