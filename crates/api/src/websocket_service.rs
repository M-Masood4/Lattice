use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::AppState;

/// Maximum number of messages to buffer in the broadcast channel
const CHANNEL_CAPACITY: usize = 100;

/// WebSocket update types that can be pushed to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardUpdate {
    /// Price change for an asset
    PriceUpdate {
        asset: String,
        blockchain: String,
        price: String,
        change_24h: String,
        timestamp: i64,
    },
    /// Trade execution notification
    TradeExecuted {
        trade_id: String,
        asset: String,
        action: String,
        amount: String,
        price: String,
        timestamp: i64,
    },
    /// Agentic trim execution
    TrimExecuted {
        trim_id: String,
        asset: String,
        amount_sold: String,
        profit_realized: String,
        reasoning: String,
        timestamp: i64,
    },
    /// Benchmark triggered
    BenchmarkTriggered {
        benchmark_id: String,
        asset: String,
        target_price: String,
        current_price: String,
        action: String,
        timestamp: i64,
    },
    /// Portfolio value update
    PortfolioUpdate {
        total_value_usd: String,
        change_24h: String,
        timestamp: i64,
    },
    /// Conversion completed
    ConversionCompleted {
        conversion_id: String,
        from_asset: String,
        to_asset: String,
        from_amount: String,
        to_amount: String,
        timestamp: i64,
    },
}

/// WebSocket service for managing real-time dashboard updates
#[derive(Clone)]
pub struct WebSocketService {
    tx: broadcast::Sender<DashboardUpdate>,
}

impl WebSocketService {
    /// Create a new WebSocket service
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    /// Get a receiver for dashboard updates
    pub fn subscribe(&self) -> broadcast::Receiver<DashboardUpdate> {
        self.tx.subscribe()
    }

    /// Broadcast a price update to all connected clients
    pub fn broadcast_price_update(
        &self,
        asset: String,
        blockchain: String,
        price: String,
        change_24h: String,
    ) {
        let update = DashboardUpdate::PriceUpdate {
            asset,
            blockchain,
            price,
            change_24h,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.tx.send(update) {
            warn!("Failed to broadcast price update: {}", e);
        }
    }

    /// Broadcast a trade execution to all connected clients
    pub fn broadcast_trade_executed(
        &self,
        trade_id: Uuid,
        asset: String,
        action: String,
        amount: String,
        price: String,
    ) {
        let update = DashboardUpdate::TradeExecuted {
            trade_id: trade_id.to_string(),
            asset,
            action,
            amount,
            price,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.tx.send(update) {
            warn!("Failed to broadcast trade execution: {}", e);
        }
    }

    /// Broadcast a trim execution to all connected clients
    pub fn broadcast_trim_executed(
        &self,
        trim_id: Uuid,
        asset: String,
        amount_sold: String,
        profit_realized: String,
        reasoning: String,
    ) {
        let update = DashboardUpdate::TrimExecuted {
            trim_id: trim_id.to_string(),
            asset,
            amount_sold,
            profit_realized,
            reasoning,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.tx.send(update) {
            warn!("Failed to broadcast trim execution: {}", e);
        }
    }

    /// Broadcast a benchmark trigger to all connected clients
    pub fn broadcast_benchmark_triggered(
        &self,
        benchmark_id: Uuid,
        asset: String,
        target_price: String,
        current_price: String,
        action: String,
    ) {
        let update = DashboardUpdate::BenchmarkTriggered {
            benchmark_id: benchmark_id.to_string(),
            asset,
            target_price,
            current_price,
            action,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.tx.send(update) {
            warn!("Failed to broadcast benchmark trigger: {}", e);
        }
    }

    /// Broadcast a portfolio value update to all connected clients
    pub fn broadcast_portfolio_update(&self, total_value_usd: String, change_24h: String) {
        let update = DashboardUpdate::PortfolioUpdate {
            total_value_usd,
            change_24h,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.tx.send(update) {
            warn!("Failed to broadcast portfolio update: {}", e);
        }
    }

    /// Broadcast a conversion completion to all connected clients
    pub fn broadcast_conversion_completed(
        &self,
        conversion_id: Uuid,
        from_asset: String,
        to_asset: String,
        from_amount: String,
        to_amount: String,
    ) {
        let update = DashboardUpdate::ConversionCompleted {
            conversion_id: conversion_id.to_string(),
            from_asset,
            to_asset,
            from_amount,
            to_amount,
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if let Err(e) = self.tx.send(update) {
            warn!("Failed to broadcast conversion completion: {}", e);
        }
    }
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket handler for dashboard updates
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to dashboard updates
    let mut rx = state.websocket_service.subscribe();
    
    info!("New WebSocket connection established");
    
    // Spawn a task to send updates to the client
    let mut send_task = tokio::spawn(async move {
        while let Ok(update) = rx.recv().await {
            // Serialize the update to JSON
            match serde_json::to_string(&update) {
                Ok(json) => {
                    if sender.send(Message::Text(json)).await.is_err() {
                        // Client disconnected
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize dashboard update: {}", e);
                }
            }
        }
    });
    
    // Spawn a task to receive messages from the client (for ping/pong)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    info!("WebSocket client requested close");
                    break;
                }
                Message::Ping(_data) => {
                    // Echo back pong (axum handles this automatically)
                    info!("Received ping from client");
                }
                Message::Pong(_) => {
                    // Client responded to our ping
                }
                Message::Text(text) => {
                    // Could handle client commands here if needed
                    info!("Received text message from client: {}", text);
                }
                _ => {}
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
            info!("WebSocket send task completed");
        }
        _ = (&mut recv_task) => {
            send_task.abort();
            info!("WebSocket receive task completed");
        }
    }
    
    info!("WebSocket connection closed");
}
