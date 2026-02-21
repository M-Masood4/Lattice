use reqwest::Client;
use serde::{Deserialize, Serialize};
use shared::models::{Asset, Portfolio, Recommendation, UserSettings, WhaleMovement};
use tracing::{error, info};
use uuid::Uuid;

pub mod error;
pub mod consumer;

pub use error::{AIServiceError, Result};
pub use consumer::{WhaleMovementConsumer, WhaleMovementEvent};

/// Claude API client for analyzing whale movements
/// Claude API client for analyzing whale movements
pub struct ClaudeClient {
    client: Client,
    api_key: String,
    api_url: String,
    max_retries: u32,
    initial_backoff_ms: u64,
}

/// Context for AI analysis
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisContext {
    pub whale_movement: WhaleMovementData,
    pub user_position: Option<Asset>,
    pub user_portfolio: Portfolio,
    pub user_risk_profile: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WhaleMovementData {
    pub whale_address: String,
    pub movement_type: String,
    pub token_mint: String,
    pub token_symbol: String,
    pub amount: String,
    pub percent_of_position: f64,
}

/// Claude API request structure
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API response structure
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

/// Parsed recommendation from Claude
#[derive(Debug, Deserialize)]
struct ClaudeRecommendation {
    action: String,
    confidence: i32,
    reasoning: String,
    suggested_amount: Option<String>,
    timeframe: Option<String>,
    risks: Option<Vec<String>>,
}

impl ClaudeClient {
    /// Create a new Claude API client
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_url: "https://api.anthropic.com/v1/messages".to_string(),
            max_retries: 3,
            initial_backoff_ms: 1000,
        }
    }

    /// Create a new Claude API client with custom retry settings
    pub fn with_retry_config(api_key: String, max_retries: u32, initial_backoff_ms: u64) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_url: "https://api.anthropic.com/v1/messages".to_string(),
            max_retries,
            initial_backoff_ms,
        }
    }

    /// Analyze a whale movement and generate recommendation with retry logic
    pub async fn analyze_movement(
        &self,
        movement: &WhaleMovement,
        user_id: Uuid,
        context: AnalysisContext,
    ) -> Result<Recommendation> {
        info!(
            "Analyzing whale movement {} for user {}",
            movement.transaction_signature, user_id
        );

        // Build the analysis prompt
        let prompt = self.build_analysis_prompt(&context);

        // Call Claude API with retry logic
        let response_text = self.call_claude_api_with_retry(&prompt).await?;

        // Parse the recommendation
        let recommendation = self.parse_recommendation(&response_text, movement.id, user_id)?;

        Ok(recommendation)
    }

    /// Call Claude API with exponential backoff retry
    async fn call_claude_api_with_retry(&self, prompt: &str) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.call_claude_api(prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.max_retries {
                        let backoff_ms = self.initial_backoff_ms * 2_u64.pow(attempt);
                        error!(
                            "Claude API call failed (attempt {}/{}), retrying in {}ms",
                            attempt + 1,
                            self.max_retries + 1,
                            backoff_ms
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AIServiceError::ApiError("All retry attempts failed".to_string())
        }))
    }

    /// Build structured prompt for Claude
    fn build_analysis_prompt(&self, context: &AnalysisContext) -> String {
        let user_position_text = if let Some(pos) = &context.user_position {
            format!(
                "- Holding: {} {}\n- Value: ${:.2}",
                pos.amount,
                pos.token_symbol,
                pos.value_usd.unwrap_or(0.0)
            )
        } else {
            "- No current position in this asset".to_string()
        };

        format!(
            r#"You are a cryptocurrency trading analyst. Analyze the following whale movement and provide a recommendation.

Whale Movement:
- Address: {}
- Action: {} {} {}
- Percentage of whale's position: {:.2}%

User's Current Position:
{}

User's Portfolio Value: ${:.2}
User Risk Profile: {}

Provide a recommendation in the following JSON format:
{{
  "action": "HOLD|BUY|SELL|TRIM",
  "confidence": 0-100,
  "reasoning": "detailed explanation",
  "suggestedAmount": "number or null",
  "timeframe": "immediate|short-term|long-term",
  "risks": ["risk1", "risk2"]
}}

Consider:
1. The significance of the whale's movement percentage
2. The user's current exposure to this asset
3. The user's risk tolerance
4. Market implications of the whale's action

Respond ONLY with the JSON object, no additional text."#,
            context.whale_movement.whale_address,
            context.whale_movement.movement_type,
            context.whale_movement.amount,
            context.whale_movement.token_symbol,
            context.whale_movement.percent_of_position,
            user_position_text,
            context.user_portfolio.total_value_usd,
            context.user_risk_profile
        )
    }

    /// Call Claude API with the prompt
    async fn call_claude_api(&self, prompt: &str) -> Result<String> {
        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIServiceError::ApiError(format!("Failed to call Claude API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Claude API error: {} - {}", status, error_text);
            return Err(AIServiceError::ApiError(format!(
                "Claude API returned error: {} - {}",
                status, error_text
            )));
        }

        let claude_response: ClaudeResponse = response.json().await.map_err(|e| {
            AIServiceError::ParseError(format!("Failed to parse Claude response: {}", e))
        })?;

        claude_response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| AIServiceError::ParseError("Empty response from Claude".to_string()))
    }

    /// Parse Claude's JSON response into a Recommendation
    fn parse_recommendation(
        &self,
        response_text: &str,
        movement_id: Uuid,
        user_id: Uuid,
    ) -> Result<Recommendation> {
        // Extract JSON from response (Claude might include markdown code blocks)
        let json_text = if response_text.contains("```json") {
            response_text
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response_text)
                .trim()
        } else if response_text.contains("```") {
            response_text
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response_text)
                .trim()
        } else {
            response_text.trim()
        };

        let claude_rec: ClaudeRecommendation = serde_json::from_str(json_text).map_err(|e| {
            error!("Failed to parse recommendation JSON: {}", e);
            error!("Response text: {}", json_text);
            AIServiceError::ParseError(format!("Failed to parse recommendation: {}", e))
        })?;

        // Validate action
        let action = claude_rec.action.to_uppercase();
        if !["HOLD", "BUY", "SELL", "TRIM"].contains(&action.as_str()) {
            return Err(AIServiceError::ParseError(format!(
                "Invalid action: {}",
                action
            )));
        }

        // Validate confidence
        if !(0..=100).contains(&claude_rec.confidence) {
            return Err(AIServiceError::ParseError(format!(
                "Invalid confidence: {}",
                claude_rec.confidence
            )));
        }

        Ok(Recommendation {
            id: Uuid::new_v4(),
            movement_id,
            user_id,
            action,
            confidence: claude_rec.confidence,
            reasoning: claude_rec.reasoning,
            suggested_amount: claude_rec.suggested_amount,
            timeframe: claude_rec.timeframe,
            risks: claude_rec
                .risks
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)),
            created_at: chrono::Utc::now(),
        })
    }
}

/// Service for building analysis context with historical data
pub struct AnalysisContextBuilder {
    // In a real implementation, this would have database and blockchain clients
    // For now, we'll keep it simple
}

impl AnalysisContextBuilder {
    pub fn new() -> Self {
        Self {}
    }

    /// Build complete analysis context from whale movement and user data
    pub async fn build_context(
        &self,
        movement: &WhaleMovement,
        whale_address: String,
        token_symbol: String,
        user_portfolio: Portfolio,
        user_settings: &UserSettings,
    ) -> Result<AnalysisContext> {
        // Find user's position in the moved token
        let user_position = user_portfolio
            .assets
            .iter()
            .find(|a| a.token_mint == movement.token_mint)
            .cloned();

        Ok(AnalysisContext {
            whale_movement: WhaleMovementData {
                whale_address,
                movement_type: movement.movement_type.clone(),
                token_mint: movement.token_mint.clone(),
                token_symbol,
                amount: movement.amount.clone(),
                percent_of_position: movement.percent_of_position.unwrap_or(0.0),
            },
            user_position,
            user_portfolio,
            user_risk_profile: user_settings.risk_tolerance.clone(),
        })
    }
}

impl Default for AnalysisContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Build analysis context from whale movement and user data (simplified version)
pub fn build_analysis_context(
    movement: &WhaleMovement,
    whale_address: String,
    token_symbol: String,
    user_portfolio: Portfolio,
    user_settings: &UserSettings,
) -> AnalysisContext {
    // Find user's position in the moved token
    let user_position = user_portfolio
        .assets
        .iter()
        .find(|a| a.token_mint == movement.token_mint)
        .cloned();

    AnalysisContext {
        whale_movement: WhaleMovementData {
            whale_address,
            movement_type: movement.movement_type.clone(),
            token_mint: movement.token_mint.clone(),
            token_symbol,
            amount: movement.amount.clone(),
            percent_of_position: movement.percent_of_position.unwrap_or(0.0),
        },
        user_position,
        user_portfolio,
        user_risk_profile: user_settings.risk_tolerance.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_analysis_prompt() {
        let client = ClaudeClient::new("test_key".to_string());
        
        let context = AnalysisContext {
            whale_movement: WhaleMovementData {
                whale_address: "whale123".to_string(),
                movement_type: "BUY".to_string(),
                token_mint: "SOL".to_string(),
                token_symbol: "SOL".to_string(),
                amount: "1000".to_string(),
                percent_of_position: 15.5,
            },
            user_position: Some(Asset {
                token_mint: "SOL".to_string(),
                token_symbol: "SOL".to_string(),
                amount: "10".to_string(),
                value_usd: Some(1000.0),
            }),
            user_portfolio: Portfolio {
                wallet_address: "user123".to_string(),
                assets: vec![],
                total_value_usd: 5000.0,
                last_updated: chrono::Utc::now(),
            },
            user_risk_profile: "MEDIUM".to_string(),
        };

        let prompt = client.build_analysis_prompt(&context);
        
        assert!(prompt.contains("whale123"));
        assert!(prompt.contains("BUY"));
        assert!(prompt.contains("15.50%"));
        assert!(prompt.contains("MEDIUM"));
    }

    #[test]
    fn test_parse_recommendation_valid() {
        let client = ClaudeClient::new("test_key".to_string());
        
        let json_response = r#"{
            "action": "BUY",
            "confidence": 75,
            "reasoning": "Whale is accumulating, bullish signal",
            "suggestedAmount": "5",
            "timeframe": "short-term",
            "risks": ["market volatility", "whale could reverse"]
        }"#;

        let movement_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let result = client.parse_recommendation(json_response, movement_id, user_id);
        assert!(result.is_ok());
        
        let rec = result.unwrap();
        assert_eq!(rec.action, "BUY");
        assert_eq!(rec.confidence, 75);
        assert_eq!(rec.reasoning, "Whale is accumulating, bullish signal");
    }

    #[test]
    fn test_parse_recommendation_with_markdown() {
        let client = ClaudeClient::new("test_key".to_string());
        
        let json_response = r#"```json
{
    "action": "HOLD",
    "confidence": 60,
    "reasoning": "Wait and see",
    "suggestedAmount": null,
    "timeframe": "immediate",
    "risks": []
}
```"#;

        let movement_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let result = client.parse_recommendation(json_response, movement_id, user_id);
        assert!(result.is_ok());
        
        let rec = result.unwrap();
        assert_eq!(rec.action, "HOLD");
        assert_eq!(rec.confidence, 60);
    }

    #[test]
    fn test_parse_recommendation_invalid_action() {
        let client = ClaudeClient::new("test_key".to_string());
        
        let json_response = r#"{
            "action": "INVALID",
            "confidence": 75,
            "reasoning": "Test",
            "suggestedAmount": null,
            "timeframe": "immediate",
            "risks": []
        }"#;

        let movement_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let result = client.parse_recommendation(json_response, movement_id, user_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_recommendation_invalid_confidence() {
        let client = ClaudeClient::new("test_key".to_string());
        
        let json_response = r#"{
            "action": "BUY",
            "confidence": 150,
            "reasoning": "Test",
            "suggestedAmount": null,
            "timeframe": "immediate",
            "risks": []
        }"#;

        let movement_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let result = client.parse_recommendation(json_response, movement_id, user_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_analysis_context_with_position() {
        let movement = WhaleMovement {
            id: Uuid::new_v4(),
            whale_id: Uuid::new_v4(),
            transaction_signature: "sig123".to_string(),
            movement_type: "BUY".to_string(),
            token_mint: "SOL".to_string(),
            amount: "1000".to_string(),
            percent_of_position: Some(15.5),
            detected_at: chrono::Utc::now(),
        };

        let user_portfolio = Portfolio {
            wallet_address: "user123".to_string(),
            assets: vec![Asset {
                token_mint: "SOL".to_string(),
                token_symbol: "SOL".to_string(),
                amount: "10".to_string(),
                value_usd: Some(1000.0),
            }],
            total_value_usd: 5000.0,
            last_updated: chrono::Utc::now(),
        };

        let user_settings = UserSettings {
            user_id: Uuid::new_v4(),
            auto_trader_enabled: false,
            max_trade_percentage: 5.0,
            max_daily_trades: 10,
            stop_loss_percentage: 10.0,
            risk_tolerance: "MEDIUM".to_string(),
            updated_at: chrono::Utc::now(),
        };

        let context = build_analysis_context(
            &movement,
            "whale123".to_string(),
            "SOL".to_string(),
            user_portfolio,
            &user_settings,
        );

        assert_eq!(context.whale_movement.whale_address, "whale123");
        assert_eq!(context.whale_movement.movement_type, "BUY");
        assert_eq!(context.whale_movement.percent_of_position, 15.5);
        assert!(context.user_position.is_some());
        assert_eq!(context.user_risk_profile, "MEDIUM");
    }

    #[test]
    fn test_build_analysis_context_without_position() {
        let movement = WhaleMovement {
            id: Uuid::new_v4(),
            whale_id: Uuid::new_v4(),
            transaction_signature: "sig123".to_string(),
            movement_type: "SELL".to_string(),
            token_mint: "USDC".to_string(),
            amount: "5000".to_string(),
            percent_of_position: Some(20.0),
            detected_at: chrono::Utc::now(),
        };

        let user_portfolio = Portfolio {
            wallet_address: "user123".to_string(),
            assets: vec![Asset {
                token_mint: "SOL".to_string(),
                token_symbol: "SOL".to_string(),
                amount: "10".to_string(),
                value_usd: Some(1000.0),
            }],
            total_value_usd: 5000.0,
            last_updated: chrono::Utc::now(),
        };

        let user_settings = UserSettings {
            user_id: Uuid::new_v4(),
            auto_trader_enabled: false,
            max_trade_percentage: 5.0,
            max_daily_trades: 10,
            stop_loss_percentage: 10.0,
            risk_tolerance: "LOW".to_string(),
            updated_at: chrono::Utc::now(),
        };

        let context = build_analysis_context(
            &movement,
            "whale456".to_string(),
            "USDC".to_string(),
            user_portfolio,
            &user_settings,
        );

        assert_eq!(context.whale_movement.token_mint, "USDC");
        assert!(context.user_position.is_none());
        assert_eq!(context.user_risk_profile, "LOW");
    }

    #[tokio::test]
    async fn test_context_builder() {
        let builder = AnalysisContextBuilder::new();
        
        let movement = WhaleMovement {
            id: Uuid::new_v4(),
            whale_id: Uuid::new_v4(),
            transaction_signature: "sig123".to_string(),
            movement_type: "BUY".to_string(),
            token_mint: "SOL".to_string(),
            amount: "1000".to_string(),
            percent_of_position: Some(15.5),
            detected_at: chrono::Utc::now(),
        };

        let user_portfolio = Portfolio {
            wallet_address: "user123".to_string(),
            assets: vec![],
            total_value_usd: 5000.0,
            last_updated: chrono::Utc::now(),
        };

        let user_settings = UserSettings {
            user_id: Uuid::new_v4(),
            auto_trader_enabled: false,
            max_trade_percentage: 5.0,
            max_daily_trades: 10,
            stop_loss_percentage: 10.0,
            risk_tolerance: "HIGH".to_string(),
            updated_at: chrono::Utc::now(),
        };

        let result = builder
            .build_context(
                &movement,
                "whale789".to_string(),
                "SOL".to_string(),
                user_portfolio,
                &user_settings,
            )
            .await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.whale_movement.whale_address, "whale789");
        assert_eq!(context.user_risk_profile, "HIGH");
    }
}

    #[test]
    fn test_client_with_retry_config() {
        let client = ClaudeClient::with_retry_config("test_key".to_string(), 5, 500);
        assert_eq!(client.max_retries, 5);
        assert_eq!(client.initial_backoff_ms, 500);
    }

    #[test]
    fn test_client_default_retry_config() {
        let client = ClaudeClient::new("test_key".to_string());
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.initial_backoff_ms, 1000);
    }
