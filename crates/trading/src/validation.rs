use crate::{TradeRequest, Result};
use shared::models::{Subscription, UserSettings};
use tracing::info;

/// Validation result with errors and warnings
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Trade validator for safety checks
pub struct TradeValidator;

impl TradeValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate a trade request
    pub fn validate(
        &self,
        trade: &TradeRequest,
        user_settings: &UserSettings,
        subscription: Option<&Subscription>,
        portfolio_value: f64,
        daily_trade_count: usize,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate action
        if !["BUY", "SELL"].contains(&trade.action.as_str()) {
            result.add_error(format!("Invalid action: {}", trade.action));
        }

        // Validate amount
        if let Ok(amount) = trade.amount.parse::<f64>() {
            if amount <= 0.0 {
                result.add_error("Trade amount must be positive".to_string());
            }

            // Check position limits
            let trade_percentage = (amount / portfolio_value) * 100.0;
            if trade_percentage > user_settings.max_trade_percentage {
                result.add_error(format!(
                    "Trade exceeds position limit: {:.2}% > {:.2}%",
                    trade_percentage, user_settings.max_trade_percentage
                ));
            }

            // Warning for large trades
            if trade_percentage > user_settings.max_trade_percentage * 0.8 {
                result.add_warning(format!(
                    "Large trade: {:.2}% of portfolio",
                    trade_percentage
                ));
            }
        } else {
            result.add_error("Invalid trade amount format".to_string());
        }

        // Check daily trade limits
        if daily_trade_count >= user_settings.max_daily_trades as usize {
            result.add_error(format!(
                "Daily trade limit exceeded: {} >= {}",
                daily_trade_count, user_settings.max_daily_trades
            ));
        }

        // Check subscription for auto-trader
        if user_settings.auto_trader_enabled {
            if let Some(sub) = subscription {
                if sub.tier != "PREMIUM" {
                    result.add_error("Premium subscription required for auto-trader".to_string());
                }
                if sub.status != "ACTIVE" {
                    result.add_error(format!("Subscription not active: {}", sub.status));
                }
            } else {
                result.add_error("No subscription found for auto-trader".to_string());
            }
        }

        // Validate slippage tolerance
        if trade.slippage_tolerance < 0.0 || trade.slippage_tolerance > 10.0 {
            result.add_warning(format!(
                "Unusual slippage tolerance: {:.2}%",
                trade.slippage_tolerance
            ));
        }

        info!(
            "Trade validation for user {}: valid={}, errors={}, warnings={}",
            trade.user_id,
            result.valid,
            result.errors.len(),
            result.warnings.len()
        );

        Ok(result)
    }
}

impl Default for TradeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_settings() -> UserSettings {
        UserSettings {
            user_id: Uuid::new_v4(),
            auto_trader_enabled: false,
            max_trade_percentage: 5.0,
            max_daily_trades: 10,
            stop_loss_percentage: 10.0,
            risk_tolerance: "MEDIUM".to_string(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn create_test_trade(amount: &str, action: &str) -> TradeRequest {
        TradeRequest {
            user_id: Uuid::new_v4(),
            action: action.to_string(),
            token_mint: "SOL".to_string(),
            amount: amount.to_string(),
            slippage_tolerance: 1.0,
            recommendation_id: None,
        }
    }

    #[test]
    fn test_valid_trade() {
        let validator = TradeValidator::new();
        let settings = create_test_settings();
        let trade = create_test_trade("100", "BUY");

        let result = validator
            .validate(&trade, &settings, None, 10000.0, 0)
            .unwrap();

        assert!(result.valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_invalid_action() {
        let validator = TradeValidator::new();
        let settings = create_test_settings();
        let mut trade = create_test_trade("100", "INVALID");
        trade.action = "INVALID".to_string();

        let result = validator
            .validate(&trade, &settings, None, 10000.0, 0)
            .unwrap();

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid action")));
    }

    #[test]
    fn test_position_limit_exceeded() {
        let validator = TradeValidator::new();
        let settings = create_test_settings();
        let trade = create_test_trade("600", "BUY"); // 6% of 10000

        let result = validator
            .validate(&trade, &settings, None, 10000.0, 0)
            .unwrap();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("exceeds position limit")));
    }

    #[test]
    fn test_daily_limit_exceeded() {
        let validator = TradeValidator::new();
        let settings = create_test_settings();
        let trade = create_test_trade("100", "BUY");

        let result = validator
            .validate(&trade, &settings, None, 10000.0, 10)
            .unwrap();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Daily trade limit exceeded")));
    }

    #[test]
    fn test_negative_amount() {
        let validator = TradeValidator::new();
        let settings = create_test_settings();
        let trade = create_test_trade("-100", "BUY");

        let result = validator
            .validate(&trade, &settings, None, 10000.0, 0)
            .unwrap();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("must be positive")));
    }

    #[test]
    fn test_auto_trader_without_subscription() {
        let validator = TradeValidator::new();
        let mut settings = create_test_settings();
        settings.auto_trader_enabled = true;
        let trade = create_test_trade("100", "BUY");

        let result = validator
            .validate(&trade, &settings, None, 10000.0, 0)
            .unwrap();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("No subscription found")));
    }

    #[test]
    fn test_large_trade_warning() {
        let validator = TradeValidator::new();
        let settings = create_test_settings();
        let trade = create_test_trade("450", "BUY"); // 4.5% of 10000 (90% of limit)

        let result = validator
            .validate(&trade, &settings, None, 10000.0, 0)
            .unwrap();

        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("Large trade")));
    }
}
