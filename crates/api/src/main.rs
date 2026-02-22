use anyhow::Result;
use api::{AnalyticsService, AppState, BenchmarkService, ChatService, CoinMarketCapService, ConversionService, MeshPriceService, P2PService, PaymentReceiptService, PortfolioMonitor, PositionEvaluator, PositionManagementService, PriceMonitor, PrivacyService, ReceiptService, SideShiftClient, StakingService, TrimConfigService, TrimExecutor, VerificationService, WalletService, WebSocketService, WhaleDetectionService};
use blockchain::SolanaClient;
use database::{create_pool, create_redis_client, create_redis_pool, run_migrations};
use notification::NotificationService;
use shared::config::Config;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,solana_whale_tracker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Solana Whale Tracker API");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Create database pool
    let db_pool = create_pool(&config.database.url, config.database.max_connections).await?;
    tracing::info!("Database connection pool created");

    // Run migrations (skip if SKIP_MIGRATIONS=true)
    if std::env::var("SKIP_MIGRATIONS").unwrap_or_default() != "true" {
        run_migrations(&db_pool).await?;
        tracing::info!("Database migrations completed");
    } else {
        tracing::info!("Skipping database migrations (SKIP_MIGRATIONS=true)");
    }

    // Create Redis client and pool
    let redis_client = create_redis_client(&config.redis.url).await?;
    let redis_pool = create_redis_pool(redis_client).await?;
    tracing::info!("Redis connection established");

    // Initialize Solana client
    let solana_client = Arc::new(SolanaClient::new(config.solana.rpc_url.clone(), None));
    tracing::info!("Solana client initialized");

    // Initialize Helius client for wallet analytics
    let helius_api_key = std::env::var("HELIUS_API_KEY")
        .unwrap_or_else(|_| "1266cbb3-f966-49e2-91f0-d3d04e52e69a".to_string());
    let use_mainnet = std::env::var("USE_MAINNET")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    let tantum_client = Arc::new(api::TantumClient::new(
        helius_api_key,
        use_mainnet,
    ));
    tracing::info!("Helius client initialized (using {})", if use_mainnet { "mainnet" } else { "devnet" });

    // Initialize services
    let wallet_service = Arc::new(WalletService::new_with_tantum(
        solana_client.clone(),
        tantum_client.clone(),
        db_pool.clone(),
        redis_pool.clone(),
        use_mainnet,
    ));
    tracing::info!("Wallet service initialized with Helius API integration");

    let whale_detection_service = Arc::new(WhaleDetectionService::new(
        solana_client.clone(),
        db_pool.clone(),
        redis_pool.clone(),
    ));
    tracing::info!("Whale detection service initialized");

    let analytics_service = Arc::new(AnalyticsService::new(db_pool.clone()));
    tracing::info!("Analytics service initialized");

    let benchmark_service = Arc::new(BenchmarkService::new(db_pool.clone()));
    tracing::info!("Benchmark service initialized");

    // Initialize CoinMarketCap service for real-time crypto prices
    let coinmarketcap_api_key = std::env::var("COINMARKETCAP_API_KEY")
        .unwrap_or_else(|_| "7c900818e1a14a3eb98ce42e9ac293e5".to_string());
    let coinmarketcap_service = Arc::new(CoinMarketCapService::new(
        coinmarketcap_api_key,
        redis_pool.clone(),
    ));
    tracing::info!("CoinMarketCap service initialized");

    // Initialize SideShift client for conversions
    let sideshift_client = Arc::new(SideShiftClient::new(
        config.sideshift.affiliate_id.clone(),
    ));
    tracing::info!("SideShift client initialized");

    // Initialize multi-chain blockchain client for receipts
    let multi_chain_client = Arc::new(blockchain::MultiChainClient::new());
    tracing::info!("Multi-chain blockchain client initialized");

    // Initialize receipt service for blockchain receipts
    let receipt_service = Arc::new(ReceiptService::new(
        db_pool.clone(),
        multi_chain_client.clone(),
    ));
    tracing::info!("Receipt service initialized");

    // Initialize payment receipt service for user-facing receipts
    let payment_receipt_service = Arc::new(PaymentReceiptService::new(
        db_pool.clone(),
        receipt_service.clone(),
    ));
    tracing::info!("Payment receipt service initialized");

    // Initialize conversion service with receipt generation
    let conversion_service = Arc::new(ConversionService::new_with_receipts(
        db_pool.clone(),
        sideshift_client.clone(),
        coinmarketcap_service.clone(),
        payment_receipt_service.clone(),
    ));
    tracing::info!("Conversion service initialized with receipt generation");

    // Initialize staking service
    let staking_service = Arc::new(StakingService::new(
        db_pool.clone(),
        sideshift_client.clone(),
    ));
    tracing::info!("Staking service initialized");

    // Initialize trim configuration service
    let trim_config_service = Arc::new(TrimConfigService::new(db_pool.clone()));
    tracing::info!("Trim configuration service initialized");

    // Initialize chat service for encrypted messaging
    let chat_service = Arc::new(ChatService::new(
        db_pool.clone(),
        receipt_service.clone(),
    ));
    tracing::info!("Chat service initialized");

    // Initialize WebSocket service for real-time dashboard updates
    let websocket_service = Arc::new(WebSocketService::new());
    tracing::info!("WebSocket service initialized");

    // Initialize position management service for manual/automatic trading
    let position_management_service = Arc::new(PositionManagementService::new(db_pool.clone()));
    tracing::info!("Position management service initialized");

    // Initialize P2P exchange service
    let p2p_service = Arc::new(P2PService::new(db_pool.clone()));
    tracing::info!("P2P exchange service initialized");

    // Initialize verification service for identity and wallet verification
    let verification_service = Arc::new(VerificationService::new(db_pool.clone()));
    tracing::info!("Verification service initialized");

    // Initialize privacy service for temporary wallets and user tags
    let privacy_service = Arc::new(PrivacyService::new(db_pool.clone()));
    tracing::info!("Privacy service initialized");

    // Initialize position evaluator for agentic trimming
    let position_evaluator = Arc::new(PositionEvaluator::new(
        db_pool.clone(),
        trim_config_service.clone(),
        config.claude.api_key.clone(),
    ));
    tracing::info!("Position evaluator initialized");

    // Initialize notification service for alerts and trade notifications
    let notification_service = Arc::new(NotificationService::new());
    tracing::info!("Notification service initialized");

    // Initialize and start portfolio monitor background job
    let portfolio_monitor = Arc::new(PortfolioMonitor::new(
        wallet_service.clone(),
        whale_detection_service.clone(),
        analytics_service.clone(),
        db_pool.clone(),
        None, // Use default 5-minute interval
    ));
    
    let _monitor_handle = portfolio_monitor.start();
    tracing::info!("Portfolio monitor background job started");

    // Initialize and start price monitor for benchmark triggers
    // Wires benchmark triggers to notification service (Requirement 2.3, 2.5)
    let price_monitor = Arc::new(PriceMonitor::new(
        benchmark_service.clone(),
        coinmarketcap_service.clone(),
        position_management_service.clone(),
        notification_service.clone(),
        db_pool.clone(),
    ));
    
    // Spawn price monitor as a background task with WebSocket integration
    let price_monitor_clone = price_monitor.clone();
    let _websocket_service_clone = websocket_service.clone();
    tokio::spawn(async move {
        // Note: Price monitor would need to be updated to accept websocket_service
        // For now, we'll integrate WebSocket broadcasting in the handlers
        price_monitor_clone.start().await;
    });
    tracing::info!("Price monitor background job started (checking every 10 seconds)");

    // Start position evaluator background job for agentic trimming
    // Evaluates positions every 5 minutes (Requirement 7.1, 7.2)
    let position_evaluator_clone = position_evaluator.clone();
    tokio::spawn(async move {
        position_evaluator_clone.start();
    });
    tracing::info!("Position evaluator background job started (checking every 5 minutes)");

    // Initialize trading service for trim execution
    let trading_service = Arc::new(trading::TradingService::new());
    tracing::info!("Trading service initialized");

    // Initialize trim executor for executing pending trim recommendations
    // Processes pending trims every 1 minute (Requirement 7.3, 7.4, 7.5, 7.6)
    let trim_executor = Arc::new(TrimExecutor::new(
        db_pool.clone(),
        trim_config_service.clone(),
        position_management_service.clone(),
        trading_service.clone(),
        notification_service.clone(),
    ));
    tracing::info!("Trim executor initialized");

    // Start trim executor background job
    let trim_executor_clone = trim_executor.clone();
    tokio::spawn(async move {
        trim_executor_clone.start();
    });
    tracing::info!("Trim executor background job started (processing pending trims every 1 minute)");

    // Initialize JWT config for authentication
    let jwt_config = Arc::new(api::auth::JwtConfig::new(config.jwt.secret.clone()));
    tracing::info!("JWT config initialized");

    // Initialize metrics collector for monitoring
    let metrics = api::MetricsCollector::new();
    tracing::info!("Metrics collector initialized");

    // Initialize proximity services for P2P transfers
    let proximity_auth_service = Arc::new(proximity::AuthenticationService::new());
    tracing::info!("Proximity authentication service initialized");

    // Note: DiscoveryService requires user_tag, device_id, and wallet_address
    // These would typically come from the user's session/request
    // For now, we create a placeholder that will be initialized per-user
    let proximity_discovery_service = Arc::new(proximity::DiscoveryService::new(
        "system".to_string(),
        "server".to_string(),
        "system_wallet".to_string(),
    ));
    tracing::info!("Proximity discovery service initialized");

    let proximity_session_manager = Arc::new(proximity::SessionManager::new());
    tracing::info!("Proximity session manager initialized");

    let proximity_transfer_service = Arc::new(proximity::TransferService::new(
        db_pool.clone(),
        solana_client.clone(),
    ));
    tracing::info!("Proximity transfer service initialized");

    // Initialize mesh price service for P2P price data distribution
    // Uses the proximity P2P connection infrastructure for message routing
    let peer_connection_manager = Arc::new(proximity::PeerConnectionManager::new());
    let mesh_price_service = Arc::new(MeshPriceService::new(
        coinmarketcap_service.clone(),
        peer_connection_manager,
        redis_pool.clone(),
        db_pool.clone(),
        websocket_service.clone(),
    ));
    tracing::info!("Mesh price service initialized");

    // Create application state
    let app_state = Arc::new(AppState::new(
        wallet_service,
        whale_detection_service,
        analytics_service,
        benchmark_service,
        coinmarketcap_service,
        conversion_service,
        staking_service,
        trim_config_service,
        trim_executor.clone(),
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
        mesh_price_service,
        jwt_config,
        db_pool,
        redis_pool,
        solana_client,
        metrics.clone(),
    ));

    // Start alert manager background task (checks every 60 seconds)
    let alert_manager = api::AlertManager::new(metrics.clone());
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            alert_manager.check_and_alert().await;
        }
    });
    tracing::info!("Alert manager background task started (checking every 60 seconds)");

    // Create router with CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = api::routes::create_router(app_state)
        .layer(cors);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("API server listening on {}", addr);
    tracing::info!("Health check available at http://{}/health", addr);
    tracing::info!("Metrics available at http://{}/metrics", addr);

    axum::serve(listener, app)
        .await?;

    Ok(())
}
