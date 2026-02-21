use api::WebSocketService;

#[tokio::test]
async fn test_websocket_service_creation() {
    let ws_service = WebSocketService::new();
    
    // Should be able to subscribe to updates
    let _rx = ws_service.subscribe();
}

#[tokio::test]
async fn test_broadcast_price_update() {
    let ws_service = WebSocketService::new();
    let mut rx = ws_service.subscribe();
    
    // Broadcast a price update
    ws_service.broadcast_price_update(
        "SOL".to_string(),
        "Solana".to_string(),
        "100.50".to_string(),
        "5.2".to_string(),
    );
    
    // Receive the update
    let update = rx.recv().await.unwrap();
    
    match update {
        api::DashboardUpdate::PriceUpdate { asset, blockchain, price, change_24h, .. } => {
            assert_eq!(asset, "SOL");
            assert_eq!(blockchain, "Solana");
            assert_eq!(price, "100.50");
            assert_eq!(change_24h, "5.2");
        }
        _ => panic!("Expected PriceUpdate"),
    }
}

#[tokio::test]
async fn test_broadcast_trade_executed() {
    let ws_service = WebSocketService::new();
    let mut rx = ws_service.subscribe();
    
    let trade_id = uuid::Uuid::new_v4();
    
    // Broadcast a trade execution
    ws_service.broadcast_trade_executed(
        trade_id,
        "ETH".to_string(),
        "BUY".to_string(),
        "1.5".to_string(),
        "2000.00".to_string(),
    );
    
    // Receive the update
    let update = rx.recv().await.unwrap();
    
    match update {
        api::DashboardUpdate::TradeExecuted { trade_id: id, asset, action, amount, price, .. } => {
            assert_eq!(id, trade_id.to_string());
            assert_eq!(asset, "ETH");
            assert_eq!(action, "BUY");
            assert_eq!(amount, "1.5");
            assert_eq!(price, "2000.00");
        }
        _ => panic!("Expected TradeExecuted"),
    }
}

#[tokio::test]
async fn test_broadcast_trim_executed() {
    let ws_service = WebSocketService::new();
    let mut rx = ws_service.subscribe();
    
    let trim_id = uuid::Uuid::new_v4();
    
    // Broadcast a trim execution
    ws_service.broadcast_trim_executed(
        trim_id,
        "BTC".to_string(),
        "0.25".to_string(),
        "5000.00".to_string(),
        "Market conditions favorable for profit taking".to_string(),
    );
    
    // Receive the update
    let update = rx.recv().await.unwrap();
    
    match update {
        api::DashboardUpdate::TrimExecuted { trim_id: id, asset, amount_sold, profit_realized, reasoning, .. } => {
            assert_eq!(id, trim_id.to_string());
            assert_eq!(asset, "BTC");
            assert_eq!(amount_sold, "0.25");
            assert_eq!(profit_realized, "5000.00");
            assert_eq!(reasoning, "Market conditions favorable for profit taking");
        }
        _ => panic!("Expected TrimExecuted"),
    }
}

#[tokio::test]
async fn test_broadcast_benchmark_triggered() {
    let ws_service = WebSocketService::new();
    let mut rx = ws_service.subscribe();
    
    let benchmark_id = uuid::Uuid::new_v4();
    
    // Broadcast a benchmark trigger
    ws_service.broadcast_benchmark_triggered(
        benchmark_id,
        "SOL".to_string(),
        "150.00".to_string(),
        "151.50".to_string(),
        "ALERT".to_string(),
    );
    
    // Receive the update
    let update = rx.recv().await.unwrap();
    
    match update {
        api::DashboardUpdate::BenchmarkTriggered { benchmark_id: id, asset, target_price, current_price, action, .. } => {
            assert_eq!(id, benchmark_id.to_string());
            assert_eq!(asset, "SOL");
            assert_eq!(target_price, "150.00");
            assert_eq!(current_price, "151.50");
            assert_eq!(action, "ALERT");
        }
        _ => panic!("Expected BenchmarkTriggered"),
    }
}

#[tokio::test]
async fn test_broadcast_portfolio_update() {
    let ws_service = WebSocketService::new();
    let mut rx = ws_service.subscribe();
    
    // Broadcast a portfolio update
    ws_service.broadcast_portfolio_update(
        "50000.00".to_string(),
        "3.5".to_string(),
    );
    
    // Receive the update
    let update = rx.recv().await.unwrap();
    
    match update {
        api::DashboardUpdate::PortfolioUpdate { total_value_usd, change_24h, .. } => {
            assert_eq!(total_value_usd, "50000.00");
            assert_eq!(change_24h, "3.5");
        }
        _ => panic!("Expected PortfolioUpdate"),
    }
}

#[tokio::test]
async fn test_broadcast_conversion_completed() {
    let ws_service = WebSocketService::new();
    let mut rx = ws_service.subscribe();
    
    let conversion_id = uuid::Uuid::new_v4();
    
    // Broadcast a conversion completion
    ws_service.broadcast_conversion_completed(
        conversion_id,
        "USDC".to_string(),
        "SOL".to_string(),
        "1000.00".to_string(),
        "10.5".to_string(),
    );
    
    // Receive the update
    let update = rx.recv().await.unwrap();
    
    match update {
        api::DashboardUpdate::ConversionCompleted { conversion_id: id, from_asset, to_asset, from_amount, to_amount, .. } => {
            assert_eq!(id, conversion_id.to_string());
            assert_eq!(from_asset, "USDC");
            assert_eq!(to_asset, "SOL");
            assert_eq!(from_amount, "1000.00");
            assert_eq!(to_amount, "10.5");
        }
        _ => panic!("Expected ConversionCompleted"),
    }
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let ws_service = WebSocketService::new();
    let mut rx1 = ws_service.subscribe();
    let mut rx2 = ws_service.subscribe();
    
    // Broadcast a price update
    ws_service.broadcast_price_update(
        "BTC".to_string(),
        "Bitcoin".to_string(),
        "45000.00".to_string(),
        "2.1".to_string(),
    );
    
    // Both subscribers should receive the update
    let update1 = rx1.recv().await.unwrap();
    let update2 = rx2.recv().await.unwrap();
    
    match (update1, update2) {
        (
            api::DashboardUpdate::PriceUpdate { asset: asset1, .. },
            api::DashboardUpdate::PriceUpdate { asset: asset2, .. }
        ) => {
            assert_eq!(asset1, "BTC");
            assert_eq!(asset2, "BTC");
        }
        _ => panic!("Expected PriceUpdate for both subscribers"),
    }
}

#[tokio::test]
async fn test_late_subscriber_misses_old_updates() {
    let ws_service = WebSocketService::new();
    
    // Broadcast before subscribing
    ws_service.broadcast_price_update(
        "SOL".to_string(),
        "Solana".to_string(),
        "100.00".to_string(),
        "1.0".to_string(),
    );
    
    // Subscribe after broadcast
    let mut rx = ws_service.subscribe();
    
    // Broadcast a new update
    ws_service.broadcast_price_update(
        "ETH".to_string(),
        "Ethereum".to_string(),
        "2000.00".to_string(),
        "2.0".to_string(),
    );
    
    // Should only receive the second update
    let update = rx.recv().await.unwrap();
    
    match update {
        api::DashboardUpdate::PriceUpdate { asset, .. } => {
            assert_eq!(asset, "ETH"); // Not SOL
        }
        _ => panic!("Expected PriceUpdate"),
    }
}

#[tokio::test]
async fn test_broadcast_without_subscribers() {
    let ws_service = WebSocketService::new();
    
    // Should not panic when broadcasting without subscribers
    ws_service.broadcast_price_update(
        "SOL".to_string(),
        "Solana".to_string(),
        "100.00".to_string(),
        "1.0".to_string(),
    );
    
    ws_service.broadcast_trade_executed(
        uuid::Uuid::new_v4(),
        "ETH".to_string(),
        "BUY".to_string(),
        "1.0".to_string(),
        "2000.00".to_string(),
    );
}

#[tokio::test]
async fn test_update_serialization() {
    use serde_json;
    
    let ws_service = WebSocketService::new();
    let mut rx = ws_service.subscribe();
    
    // Broadcast an update
    ws_service.broadcast_price_update(
        "SOL".to_string(),
        "Solana".to_string(),
        "100.50".to_string(),
        "5.2".to_string(),
    );
    
    // Receive and serialize
    let update = rx.recv().await.unwrap();
    let json = serde_json::to_string(&update).unwrap();
    
    // Should contain expected fields
    assert!(json.contains("\"type\":\"price_update\""));
    assert!(json.contains("\"asset\":\"SOL\""));
    assert!(json.contains("\"blockchain\":\"Solana\""));
    assert!(json.contains("\"price\":\"100.50\""));
}
