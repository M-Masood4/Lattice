use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub solana: SolanaConfig,
    pub claude: ClaudeConfig,
    pub stripe: StripeConfig,
    pub jwt: JwtConfig,
    pub server: ServerConfig,
    pub monitoring: MonitoringConfig,
    pub rate_limit: RateLimitConfig,
    pub birdeye: BirdeyeConfig,
    pub sideshift: SideShiftConfig,
    pub mesh_network: MeshNetworkConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub rpc_fallback_url: String,
    pub network: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: String,
    pub basic_price_id: String,
    pub premium_price_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringConfig {
    pub whale_check_interval_seconds: u64,
    pub worker_pool_size: usize,
    pub whales_per_worker: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BirdeyeConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SideShiftConfig {
    pub affiliate_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeshNetworkConfig {
    /// Fetch interval for provider nodes in seconds (default: 30)
    pub provider_fetch_interval_secs: u64,
    /// Coordination window in seconds to avoid duplicate fetches (default: 5)
    pub coordination_window_secs: u64,
    /// Initial TTL value for price update messages (default: 10)
    pub message_ttl: u32,
    /// Maximum number of entries in the seen messages cache (default: 10000)
    pub seen_messages_cache_size: usize,
    /// Expiration time for seen messages in seconds (default: 300 = 5 minutes)
    pub seen_messages_expiration_secs: u64,
    /// Maximum number of peer connections per node (default: 10)
    pub max_peer_connections: usize,
    /// Minimum number of peer connections to maintain (default: 3)
    pub min_peer_connections: usize,
    /// Price data staleness threshold in seconds (default: 3600 = 1 hour)
    pub staleness_threshold_secs: u64,
    /// Provider offline indicator threshold in seconds (default: 600 = 10 minutes)
    pub offline_indicator_threshold_secs: u64,
    /// Price discrepancy threshold as percentage (default: 5.0 = 5%)
    pub price_discrepancy_threshold_percent: f64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();
        
        Ok(Config {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")?,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")?,
                pool_size: env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            solana: SolanaConfig {
                rpc_url: env::var("SOLANA_RPC_URL")?,
                rpc_fallback_url: env::var("SOLANA_RPC_FALLBACK_URL")?,
                network: env::var("SOLANA_NETWORK")?,
            },
            claude: ClaudeConfig {
                api_key: env::var("CLAUDE_API_KEY")?,
                model: env::var("CLAUDE_MODEL")?,
                max_tokens: env::var("CLAUDE_MAX_TOKENS")
                    .unwrap_or_else(|_| "4096".to_string())
                    .parse()?,
            },
            stripe: StripeConfig {
                secret_key: env::var("STRIPE_SECRET_KEY")?,
                webhook_secret: env::var("STRIPE_WEBHOOK_SECRET")?,
                basic_price_id: env::var("STRIPE_BASIC_PRICE_ID")?,
                premium_price_id: env::var("STRIPE_PREMIUM_PRICE_ID")?,
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")?,
                expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
            },
            monitoring: MonitoringConfig {
                whale_check_interval_seconds: env::var("WHALE_CHECK_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                worker_pool_size: env::var("WORKER_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                whales_per_worker: env::var("WHALES_PER_WORKER")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()?,
                burst: env::var("RATE_LIMIT_BURST")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            birdeye: BirdeyeConfig {
                api_key: env::var("BIRDEYE_API_KEY")
                    .unwrap_or_else(|_| "demo_key".to_string()),
            },
            sideshift: SideShiftConfig {
                affiliate_id: env::var("SIDESHIFT_AFFILIATE_ID").ok(),
            },
            mesh_network: MeshNetworkConfig {
                provider_fetch_interval_secs: env::var("MESH_PROVIDER_FETCH_INTERVAL_SECS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                coordination_window_secs: env::var("MESH_COORDINATION_WINDOW_SECS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                message_ttl: env::var("MESH_MESSAGE_TTL")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                seen_messages_cache_size: env::var("MESH_SEEN_MESSAGES_CACHE_SIZE")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()?,
                seen_messages_expiration_secs: env::var("MESH_SEEN_MESSAGES_EXPIRATION_SECS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()?,
                max_peer_connections: env::var("MESH_MAX_PEER_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                min_peer_connections: env::var("MESH_MIN_PEER_CONNECTIONS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                staleness_threshold_secs: env::var("MESH_STALENESS_THRESHOLD_SECS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()?,
                offline_indicator_threshold_secs: env::var("MESH_OFFLINE_INDICATOR_THRESHOLD_SECS")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()?,
                price_discrepancy_threshold_percent: env::var("MESH_PRICE_DISCREPANCY_THRESHOLD_PERCENT")
                    .unwrap_or_else(|_| "5.0".to_string())
                    .parse()?,
            },
        })
    }
}
