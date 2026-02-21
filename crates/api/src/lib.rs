pub mod error;
pub mod monitoring;
pub mod logging;
pub mod wallet_service;
pub mod portfolio_cache;
pub mod whale_detection;
pub mod portfolio_monitor;
pub mod analytics;
pub mod auth;
pub mod routes;
pub mod handlers;
pub mod rate_limit;
pub mod security;
pub mod birdeye_service;
pub mod benchmark_service;
pub mod price_monitor;
pub mod sideshift_client;
pub mod conversion_service;
pub mod staking_service;
pub mod trim_config_service;
pub mod position_evaluator;
pub mod trim_executor;
pub mod receipt_service;
pub mod payment_receipt_service;
pub mod chat_service;
pub mod p2p_service;
pub mod verification_service;
pub mod privacy_service;
pub mod websocket_service;
pub mod position_management_service;
pub mod token_metadata_service;
pub mod cross_chain_transaction_service;
pub mod proximity_receipt_integration;
pub mod proximity_service;
pub mod proximity_handlers;
pub mod proximity_websocket;

pub use wallet_service::WalletService;
pub use portfolio_cache::PortfolioCache;
pub use whale_detection::{WhaleDetectionService, RankedWhale, WhaleAsset};
pub use portfolio_monitor::PortfolioMonitor;
pub use analytics::AnalyticsService;
pub use birdeye_service::{BirdeyeService, Blockchain, WalletAddress};
pub use benchmark_service::{BenchmarkService, Benchmark, CreateBenchmarkRequest, UpdateBenchmarkRequest, TriggerType, ActionType, TradeAction};
pub use price_monitor::{PriceMonitor, TriggeredBenchmark};
pub use sideshift_client::{SideShiftClient, ConversionQuote, ConversionOrder, OrderStatus, SupportedCoin, StakingInfo, AmountType};
pub use conversion_service::{ConversionService, ConversionQuoteWithFees, ConversionResult, ConversionRecord, ConversionProvider, ConversionStatus};
pub use staking_service::{StakingService, StakingConfig, StakingPosition, StakingApprovalRequest, StakingInitiationResult};
pub use trim_config_service::{TrimConfigService, TrimConfig, UpdateTrimConfigRequest};
pub use position_evaluator::{PositionEvaluator, Position, TrimRecommendation};
pub use trim_executor::{TrimExecutor, TrimExecution, PendingTrim};
pub use receipt_service::{ReceiptService, Receipt, ReceiptData, VerificationStatus};
pub use payment_receipt_service::{
    PaymentReceiptService, PaymentReceipt, TransactionType, TransactionFees, 
    BlockchainConfirmation, ReceiptSearchFilters, Pagination, ReceiptSearchResults
};
pub use chat_service::{ChatService, ChatMessage};
pub use p2p_service::{P2PService, P2POffer, P2PExchange, OfferType, OfferStatus};
pub use verification_service::{VerificationService, WalletVerification, VerificationLevel, VerificationStatus as IdentityVerificationStatus};
pub use privacy_service::{PrivacyService, TemporaryWallet};
pub use websocket_service::{WebSocketService, DashboardUpdate, websocket_handler};
pub use position_management_service::{
    PositionManagementService, PositionMode, PositionModeConfig, ManualOrder, 
    ManualOrderRequest, PendingAutomaticOrder
};
pub use token_metadata_service::{TokenMetadataService, TokenMetadata, TokenType};
pub use cross_chain_transaction_service::{
    CrossChainTransactionService, NormalizedTransaction, TransactionStatus,
    TransactionFees as CrossChainTransactionFees
};
pub use proximity_receipt_integration::{create_proximity_receipt, create_receipt_service};
pub use proximity_websocket::{ProximityWebSocketService, ProximityEvent, proximity_websocket_handler};
pub use error::{ApiError, ApiResult, ErrorResponse};
pub use monitoring::{MetricsCollector, ServiceMetrics, ServiceMetric, HealthStatus, RequestTimer, AlertManager};

use blockchain::SolanaClient;
use deadpool_postgres::Pool;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use proximity::{DiscoveryService, TransferService, SessionManager, AuthenticationService};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub wallet_service: Arc<WalletService>,
    pub whale_detection_service: Arc<WhaleDetectionService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub benchmark_service: Arc<BenchmarkService>,
    pub birdeye_service: Arc<BirdeyeService>,
    pub conversion_service: Arc<ConversionService>,
    pub staking_service: Arc<StakingService>,
    pub trim_config_service: Arc<TrimConfigService>,
    pub trim_executor: Arc<TrimExecutor>,
    pub payment_receipt_service: Arc<PaymentReceiptService>,
    pub chat_service: Arc<ChatService>,
    pub websocket_service: Arc<WebSocketService>,
    pub position_management_service: Arc<PositionManagementService>,
    pub p2p_service: Arc<P2PService>,
    pub verification_service: Arc<VerificationService>,
    pub privacy_service: Arc<PrivacyService>,
    pub proximity_discovery_service: Arc<DiscoveryService>,
    pub proximity_transfer_service: Arc<TransferService>,
    pub proximity_session_manager: Arc<SessionManager>,
    pub proximity_auth_service: Arc<AuthenticationService>,
    pub jwt_config: Arc<auth::JwtConfig>,
    pub db_pool: Pool,
    pub redis_pool: ConnectionManager,
    pub solana_client: Arc<SolanaClient>,
    pub metrics: MetricsCollector,
}

impl AppState {
    pub fn new(
        wallet_service: Arc<WalletService>,
        whale_detection_service: Arc<WhaleDetectionService>,
        analytics_service: Arc<AnalyticsService>,
        benchmark_service: Arc<BenchmarkService>,
        birdeye_service: Arc<BirdeyeService>,
        conversion_service: Arc<ConversionService>,
        staking_service: Arc<StakingService>,
        trim_config_service: Arc<TrimConfigService>,
        trim_executor: Arc<TrimExecutor>,
        payment_receipt_service: Arc<PaymentReceiptService>,
        chat_service: Arc<ChatService>,
        websocket_service: Arc<WebSocketService>,
        position_management_service: Arc<PositionManagementService>,
        p2p_service: Arc<P2PService>,
        verification_service: Arc<VerificationService>,
        privacy_service: Arc<PrivacyService>,
        proximity_discovery_service: Arc<DiscoveryService>,
        proximity_transfer_service: Arc<TransferService>,
        proximity_session_manager: Arc<SessionManager>,
        proximity_auth_service: Arc<AuthenticationService>,
        jwt_config: Arc<auth::JwtConfig>,
        db_pool: Pool,
        redis_pool: ConnectionManager,
        solana_client: Arc<SolanaClient>,
        metrics: MetricsCollector,
    ) -> Self {
        Self {
            wallet_service,
            whale_detection_service,
            analytics_service,
            benchmark_service,
            birdeye_service,
            conversion_service,
            staking_service,
            trim_config_service,
            trim_executor,
            payment_receipt_service,
            chat_service,
            websocket_service,
            position_management_service,
            p2p_service,
            verification_service,
            privacy_service,
            proximity_discovery_service,
            proximity_transfer_service,
            proximity_session_manager,
            proximity_auth_service,
            jwt_config,
            db_pool,
            redis_pool,
            solana_client,
            metrics,
        }
    }
}
