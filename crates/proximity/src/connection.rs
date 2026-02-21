// Peer Connection Manager - manages peer-to-peer connections

use crate::{ConnectionQuality, ConnectionType, PeerId, PeerMessage, ProximityError, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

/// Represents an active peer connection
#[derive(Clone)]
pub struct PeerConnection {
    pub peer_id: PeerId,
    pub connection_type: ConnectionType,
    pub quality: ConnectionQuality,
    pub established_at: DateTime<Utc>,
    // Track ping/pong for latency measurement
    last_ping_sent: Option<Instant>,
    ping_count: u32,
    pong_count: u32,
}

impl PeerConnection {
    /// Create a new peer connection
    pub fn new(peer_id: PeerId, connection_type: ConnectionType) -> Self {
        Self {
            peer_id,
            connection_type,
            quality: ConnectionQuality {
                signal_strength: None,
                latency_ms: 0,
                packet_loss_percent: 0.0,
            },
            established_at: Utc::now(),
            last_ping_sent: None,
            ping_count: 0,
            pong_count: 0,
        }
    }

    /// Update connection quality metrics
    pub fn update_quality(&mut self, quality: ConnectionQuality) {
        self.quality = quality;
    }

    /// Record a ping sent
    pub fn record_ping(&mut self) {
        self.last_ping_sent = Some(Instant::now());
        self.ping_count += 1;
    }

    /// Record a pong received and calculate latency
    pub fn record_pong(&mut self) -> Option<u32> {
        self.pong_count += 1;
        if let Some(ping_time) = self.last_ping_sent {
            let latency = ping_time.elapsed().as_millis() as u32;
            self.quality.latency_ms = latency;
            self.last_ping_sent = None;
            Some(latency)
        } else {
            None
        }
    }

    /// Calculate packet loss percentage
    pub fn calculate_packet_loss(&self) -> f32 {
        if self.ping_count == 0 {
            return 0.0;
        }
        let lost = self.ping_count.saturating_sub(self.pong_count);
        (lost as f32 / self.ping_count as f32) * 100.0
    }
}

/// Manages peer-to-peer connections
/// Manages peer-to-peer connections
pub struct PeerConnectionManager {
    connections: Arc<RwLock<HashMap<PeerId, PeerConnection>>>,
    retry_config: RetryConfig,
    /// Optional message handler for routing incoming messages
    message_handler: Arc<RwLock<Option<Arc<dyn Fn(PeerId, PeerMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> + Send + Sync>>>>,
    /// Peer routing preferences based on hop count to providers
    routing_preferences: Arc<RwLock<HashMap<PeerId, u32>>>,
    /// Maximum number of connections to maintain
    max_connections: usize,
}

/// Configuration for connection retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
        }
    }
}

impl PeerConnectionManager {
    /// Create a new PeerConnectionManager with default retry configuration
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            retry_config: RetryConfig::default(),
            message_handler: Arc::new(RwLock::new(None)),
            routing_preferences: Arc::new(RwLock::new(HashMap::new())),
            max_connections: 10,
        }
    }

    /// Create a new PeerConnectionManager with custom retry configuration
    pub fn with_retry_config(retry_config: RetryConfig) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            retry_config,
            message_handler: Arc::new(RwLock::new(None)),
            routing_preferences: Arc::new(RwLock::new(HashMap::new())),
            max_connections: 10,
        }
    }
    
    /// Set a message handler for routing incoming messages
    /// 
    /// The handler will be called whenever a message is received from a peer.
    /// This allows external services (like MeshPriceService) to process messages.
    /// 
    /// # Arguments
    /// * `handler` - Async function that processes incoming messages
    pub async fn set_message_handler<F, Fut>(&self, handler: F)
    where
        F: Fn(PeerId, PeerMessage) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
    {
        let mut message_handler = self.message_handler.write().await;
        *message_handler = Some(Arc::new(move |peer_id, message| {
            Box::pin(handler(peer_id, message))
        }));
    }
    
    /// Handle an incoming message from a peer
    /// 
    /// Routes the message to the registered message handler if one exists.
    /// This method should be called when a message is received from the network.
    /// 
    /// # Arguments
    /// * `peer_id` - The peer that sent the message
    /// * `message` - The message received
    /// 
    /// # Returns
    /// * `Ok(())` - Message handled successfully
    /// * `Err(_)` - No handler registered or handler returned error
    pub async fn handle_incoming_message(&self, peer_id: PeerId, message: PeerMessage) -> Result<()> {
        debug!("Handling incoming message from peer {}: {:?}", peer_id, message);
        
        let handler = self.message_handler.read().await;
        if let Some(handler) = handler.as_ref() {
            handler(peer_id, message).await
                .map_err(|e| ProximityError::InternalError(format!("Message handler error: {}", e)))
        } else {
            warn!("No message handler registered, message discarded");
            Ok(())
        }
    }

    /// Establish a connection to a peer with retry logic
    pub async fn establish_connection(&self, peer_id: PeerId) -> Result<PeerConnection> {
        info!("Establishing connection to peer: {}", peer_id);

        // Check if connection already exists
        {
            let connections = self.connections.read().await;
            if let Some(existing) = connections.get(&peer_id) {
                debug!("Connection to peer {} already exists", peer_id);
                return Ok(PeerConnection {
                    peer_id: existing.peer_id.clone(),
                    connection_type: existing.connection_type,
                    quality: existing.quality.clone(),
                    established_at: existing.established_at,
                    last_ping_sent: None,
                    ping_count: 0,
                    pong_count: 0,
                });
            }
        }

        // Attempt connection with retry logic
        let connection = self.establish_connection_with_retry(&peer_id).await?;

        // Store the connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(peer_id.clone(), connection.clone());
        }

        info!("Successfully established connection to peer: {}", peer_id);
        
        // Notify handler about new connection (for network status exchange)
        self.on_connection_established(&peer_id).await;
        
        Ok(connection)
    }
    
    /// Called when a new connection is established
    /// 
    /// This allows external services to be notified of new connections
    /// and perform initialization like network status exchange.
    async fn on_connection_established(&self, peer_id: &PeerId) {
        debug!("Connection established to peer: {}", peer_id);
        // Handler can listen for this via message handler
        // Network status exchange will be initiated by the application layer
    }

    /// Internal method to establish connection with exponential backoff retry
    async fn establish_connection_with_retry(&self, peer_id: &PeerId) -> Result<PeerConnection> {
        let mut attempt = 0;
        let mut backoff_ms = self.retry_config.initial_backoff_ms;

        loop {
            attempt += 1;
            debug!("Connection attempt {} for peer {}", attempt, peer_id);

            match self.try_establish_connection(peer_id).await {
                Ok(connection) => {
                    return Ok(connection);
                }
                Err(e) => {
                    if attempt >= self.retry_config.max_retries {
                        error!(
                            "Failed to establish connection to peer {} after {} attempts: {}",
                            peer_id, attempt, e
                        );
                        return Err(ProximityError::ConnectionFailed(format!(
                            "Failed after {} attempts: {}",
                            attempt, e
                        )));
                    }

                    warn!(
                        "Connection attempt {} failed for peer {}: {}. Retrying in {}ms",
                        attempt, peer_id, e, backoff_ms
                    );

                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(self.retry_config.max_backoff_ms);
                }
            }
        }
    }

    /// Try to establish a single connection (platform-specific)
    async fn try_establish_connection(&self, peer_id: &PeerId) -> Result<PeerConnection> {
        // Determine connection type based on platform
        let connection_type = self.determine_connection_type();

        debug!(
            "Attempting {:?} connection to peer {}",
            connection_type, peer_id
        );

        // Platform-specific connection logic
        match connection_type {
            ConnectionType::WebRTC => {
                // WebRTC connection for web platform
                // In a real implementation, this would use WebRTC APIs
                debug!("Establishing WebRTC connection to {}", peer_id);
                Ok(PeerConnection::new(peer_id.clone(), ConnectionType::WebRTC))
            }
            ConnectionType::TcpSocket => {
                // TCP socket connection for native platforms
                // In a real implementation, this would establish a TCP connection
                debug!("Establishing TCP socket connection to {}", peer_id);
                Ok(PeerConnection::new(peer_id.clone(), ConnectionType::TcpSocket))
            }
            ConnectionType::BleConnection => {
                // BLE connection for Bluetooth-based discovery
                debug!("Establishing BLE connection to {}", peer_id);
                Ok(PeerConnection::new(
                    peer_id.clone(),
                    ConnectionType::BleConnection,
                ))
            }
        }
    }

    /// Determine the appropriate connection type based on platform
    fn determine_connection_type(&self) -> ConnectionType {
        // In a real implementation, this would detect the platform
        // For now, we'll default to TCP sockets for native platforms
        #[cfg(target_arch = "wasm32")]
        {
            ConnectionType::WebRTC
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            ConnectionType::TcpSocket
        }
    }

    /// Send a message to a peer
    pub async fn send_message(&self, peer_id: PeerId, message: PeerMessage) -> Result<()> {
        debug!("Sending message to peer {}: {:?}", peer_id, message);

        let connections = self.connections.read().await;
        let connection = connections
            .get(&peer_id)
            .ok_or_else(|| ProximityError::PeerNotFound(peer_id.clone()))?;

        // In a real implementation, this would send the message over the connection
        // For now, we'll just serialize it to verify it's valid
        let _serialized = serde_json::to_string(&message)
            .map_err(|e| ProximityError::SerializationError(e.to_string()))?;

        debug!(
            "Message sent to peer {} via {:?}",
            peer_id, connection.connection_type
        );
        Ok(())
    }

    /// Close a connection to a peer
    pub async fn close_connection(&self, peer_id: PeerId) -> Result<()> {
        info!("Closing connection to peer: {}", peer_id);

        let mut connections = self.connections.write().await;
        if connections.remove(&peer_id).is_some() {
            debug!("Connection to peer {} closed", peer_id);
            drop(connections); // Release lock before notification
            
            // Notify about disconnection
            self.on_connection_closed(&peer_id).await;
            
            Ok(())
        } else {
            warn!("Attempted to close non-existent connection to peer {}", peer_id);
            Err(ProximityError::PeerNotFound(peer_id))
        }
    }
    
    /// Called when a connection is closed
    /// 
    /// This allows external services to be notified of disconnections
    /// and update their state accordingly.
    async fn on_connection_closed(&self, peer_id: &PeerId) {
        debug!("Connection closed to peer: {}", peer_id);
        // Handler can listen for this via message handler or separate callback
    }

    /// Measure connection quality for a peer
    pub async fn measure_quality(&self, peer_id: PeerId) -> Result<ConnectionQuality> {
        debug!("Measuring connection quality for peer: {}", peer_id);

        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(&peer_id)
            .ok_or_else(|| ProximityError::PeerNotFound(peer_id.clone()))?;

        // Send ping to measure latency
        connection.record_ping();

        // In a real implementation, we would:
        // 1. Send a Ping message
        // 2. Wait for Pong response
        // 3. Calculate latency
        // 4. Update packet loss statistics

        // For now, simulate a measurement
        let simulated_latency = 50; // ms
        connection.quality.latency_ms = simulated_latency;
        connection.quality.packet_loss_percent = connection.calculate_packet_loss();

        debug!(
            "Connection quality for peer {}: latency={}ms, packet_loss={:.2}%",
            peer_id, connection.quality.latency_ms, connection.quality.packet_loss_percent
        );

        Ok(connection.quality.clone())
    }

    /// Measure connection quality with ping/pong
    pub async fn measure_quality_with_ping(&self, peer_id: PeerId) -> Result<ConnectionQuality> {
        debug!("Measuring connection quality with ping for peer: {}", peer_id);

        // Send ping message
        self.send_ping(&peer_id).await?;

        // Wait for pong response (with timeout)
        let quality = tokio::time::timeout(
            Duration::from_secs(5),
            self.wait_for_pong(&peer_id)
        ).await
            .map_err(|_| ProximityError::Timeout(format!("Ping timeout for peer {}", peer_id)))??;

        debug!(
            "Connection quality measured for peer {}: latency={}ms, packet_loss={:.2}%, signal_strength={:?}",
            peer_id, quality.latency_ms, quality.packet_loss_percent, quality.signal_strength
        );

        Ok(quality)
    }

    /// Send a ping message to a peer
    async fn send_ping(&self, peer_id: &PeerId) -> Result<()> {
        debug!("Sending ping to peer: {}", peer_id);

        // Record ping sent
        {
            let mut connections = self.connections.write().await;
            let connection = connections
                .get_mut(peer_id)
                .ok_or_else(|| ProximityError::PeerNotFound(peer_id.clone()))?;
            connection.record_ping();
        }

        // Send ping message
        self.send_message(peer_id.clone(), PeerMessage::Ping).await?;

        Ok(())
    }

    /// Wait for pong response from a peer
    async fn wait_for_pong(&self, peer_id: &PeerId) -> Result<ConnectionQuality> {
        // In a real implementation, this would wait for an actual Pong message
        // For now, we'll simulate receiving a pong after a short delay
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(peer_id)
            .ok_or_else(|| ProximityError::PeerNotFound(peer_id.clone()))?;

        // Record pong received and calculate latency
        if let Some(latency) = connection.record_pong() {
            connection.quality.latency_ms = latency;
        }

        // Update packet loss
        connection.quality.packet_loss_percent = connection.calculate_packet_loss();

        Ok(connection.quality.clone())
    }

    /// Update signal strength for a BLE connection
    pub async fn update_signal_strength(&self, peer_id: PeerId, signal_strength: i8) -> Result<()> {
        debug!("Updating signal strength for peer {}: {}dBm", peer_id, signal_strength);

        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(&peer_id)
            .ok_or_else(|| ProximityError::PeerNotFound(peer_id.clone()))?;

        connection.quality.signal_strength = Some(signal_strength);

        Ok(())
    }

    /// Calculate overall connection quality score (0-100)
    pub async fn calculate_quality_score(&self, peer_id: &PeerId) -> Result<u8> {
        let connections = self.connections.read().await;
        let connection = connections
            .get(peer_id)
            .ok_or_else(|| ProximityError::PeerNotFound(peer_id.clone()))?;

        let quality = &connection.quality;

        // Calculate score based on multiple factors
        let mut score = 100u8;

        // Latency penalty (0-40 points)
        // 0-50ms: no penalty
        // 50-200ms: linear penalty up to 20 points
        // 200-500ms: linear penalty up to 30 points
        // >500ms: 40 points penalty
        if quality.latency_ms > 500 {
            score = score.saturating_sub(40);
        } else if quality.latency_ms > 200 {
            let penalty = 20 + ((quality.latency_ms - 200) * 10 / 300);
            score = score.saturating_sub(penalty as u8);
        } else if quality.latency_ms > 50 {
            let penalty = (quality.latency_ms - 50) * 20 / 150;
            score = score.saturating_sub(penalty as u8);
        }

        // Packet loss penalty (0-40 points)
        // 0-1%: no penalty
        // 1-5%: linear penalty up to 20 points
        // 5-10%: linear penalty up to 30 points
        // >10%: 40 points penalty
        if quality.packet_loss_percent > 10.0 {
            score = score.saturating_sub(40);
        } else if quality.packet_loss_percent > 5.0 {
            let penalty = 20.0 + ((quality.packet_loss_percent - 5.0) * 10.0 / 5.0);
            score = score.saturating_sub(penalty as u8);
        } else if quality.packet_loss_percent > 1.0 {
            let penalty = (quality.packet_loss_percent - 1.0) * 20.0 / 4.0;
            score = score.saturating_sub(penalty as u8);
        }

        // Signal strength penalty for BLE (0-20 points)
        if let Some(signal_strength) = quality.signal_strength {
            // Signal strength in dBm (typically -100 to -30)
            // -30 to -50: excellent (no penalty)
            // -50 to -70: good (5 points penalty)
            // -70 to -90: fair (10 points penalty)
            // < -90: poor (20 points penalty)
            if signal_strength < -90 {
                score = score.saturating_sub(20);
            } else if signal_strength < -70 {
                score = score.saturating_sub(10);
            } else if signal_strength < -50 {
                score = score.saturating_sub(5);
            }
        }

        debug!("Connection quality score for peer {}: {}/100", peer_id, score);
        Ok(score)
    }

    /// Check if connection quality is poor (score < 50)
    pub async fn is_quality_poor(&self, peer_id: &PeerId) -> Result<bool> {
        let score = self.calculate_quality_score(peer_id).await?;
        Ok(score < 50)
    }

    /// Get all active connections
    pub async fn get_active_connections(&self) -> Vec<PeerId> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// Check if a connection exists for a peer
    pub async fn has_connection(&self, peer_id: &PeerId) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(peer_id)
    }

    /// Get connection info for a peer
    pub async fn get_connection(&self, peer_id: &PeerId) -> Option<ConnectionQuality> {
        let connections = self.connections.read().await;
        connections.get(peer_id).map(|c| c.quality.clone())
    }
    
    /// Update routing preference for a peer based on hop count to providers
    /// 
    /// Lower hop counts are preferred for routing. This allows the network
    /// to maintain connections on shortest paths to provider nodes.
    /// 
    /// Requirement: 13.1, 13.2
    /// 
    /// # Arguments
    /// * `peer_id` - The peer to update routing preference for
    /// * `hop_count` - Number of hops to nearest provider through this peer
    pub async fn update_routing_preference(&self, peer_id: PeerId, hop_count: u32) {
        debug!("Updating routing preference for peer {}: hop_count={}", peer_id, hop_count);
        
        let mut preferences = self.routing_preferences.write().await;
        preferences.insert(peer_id, hop_count);
    }
    
    /// Get routing preference (hop count) for a peer
    /// 
    /// # Arguments
    /// * `peer_id` - The peer to get routing preference for
    /// 
    /// # Returns
    /// The hop count to nearest provider through this peer, or None if not set
    pub async fn get_routing_preference(&self, peer_id: &PeerId) -> Option<u32> {
        let preferences = self.routing_preferences.read().await;
        preferences.get(peer_id).copied()
    }
    
    /// Get peers sorted by routing preference (lowest hop count first)
    /// 
    /// Returns active connections sorted by their hop count to providers.
    /// This allows selecting the best peers for message routing.
    /// 
    /// Requirement: 13.1, 13.3
    /// 
    /// # Returns
    /// Vector of (peer_id, hop_count) tuples sorted by hop count
    pub async fn get_peers_by_preference(&self) -> Vec<(PeerId, u32)> {
        let connections = self.connections.read().await;
        let preferences = self.routing_preferences.read().await;
        
        let mut peers: Vec<(PeerId, u32)> = connections
            .keys()
            .filter_map(|peer_id| {
                preferences.get(peer_id).map(|&hop_count| (peer_id.clone(), hop_count))
            })
            .collect();
        
        // Sort by hop count (ascending)
        peers.sort_by_key(|(_, hop_count)| *hop_count);
        
        peers
    }
    
    /// Check if we should accept a new connection based on connection limits
    /// 
    /// Enforces the maximum connection limit. If at capacity, determines
    /// whether the new peer should replace an existing one based on routing preference.
    /// 
    /// Requirement: 13.5
    /// 
    /// # Arguments
    /// * `new_peer_hop_count` - Hop count for the potential new peer
    /// 
    /// # Returns
    /// * `Ok(None)` - Can accept new connection without dropping any
    /// * `Ok(Some(peer_id))` - Should drop this peer to make room for new one
    /// * `Err(_)` - Should not accept new connection
    pub async fn should_accept_connection(&self, new_peer_hop_count: u32) -> Result<Option<PeerId>> {
        let connections = self.connections.read().await;
        let connection_count = connections.len();
        
        if connection_count < self.max_connections {
            // Have room for more connections
            return Ok(None);
        }
        
        // At capacity, check if new peer is better than worst existing peer
        let preferences = self.routing_preferences.read().await;
        
        // Find peer with highest hop count (worst routing preference)
        let worst_peer = connections
            .keys()
            .filter_map(|peer_id| {
                preferences.get(peer_id).map(|&hop_count| (peer_id.clone(), hop_count))
            })
            .max_by_key(|(_, hop_count)| *hop_count);
        
        if let Some((worst_peer_id, worst_hop_count)) = worst_peer {
            if new_peer_hop_count < worst_hop_count {
                // New peer is better, should replace worst peer
                debug!(
                    "At connection limit, new peer (hop={}) is better than worst peer {} (hop={})",
                    new_peer_hop_count, worst_peer_id, worst_hop_count
                );
                return Ok(Some(worst_peer_id));
            }
        }
        
        // New peer is not better than any existing peer
        Err(ProximityError::ConnectionFailed(
            "At maximum connection limit and new peer does not improve routing".to_string()
        ))
    }
    
    /// Optimize connections based on routing preferences
    /// 
    /// Evaluates current connections and drops peers with poor routing
    /// preferences if better alternatives are available. This maintains
    /// connections on shortest paths to providers.
    /// 
    /// Requirement: 13.2, 13.3, 13.4
    /// 
    /// # Returns
    /// Number of connections dropped during optimization
    pub async fn optimize_connections(&self) -> Result<usize> {
        debug!("Optimizing connections based on routing preferences");
        
        let connections = self.connections.read().await;
        let preferences = self.routing_preferences.read().await;
        
        // If under capacity, no need to optimize
        if connections.len() < self.max_connections {
            return Ok(0);
        }
        
        // Find peers with highest hop counts
        let mut peers_by_hop: Vec<(PeerId, u32)> = connections
            .keys()
            .filter_map(|peer_id| {
                preferences.get(peer_id).map(|&hop_count| (peer_id.clone(), hop_count))
            })
            .collect();
        
        peers_by_hop.sort_by_key(|(_, hop_count)| std::cmp::Reverse(*hop_count));
        
        drop(connections);
        drop(preferences);
        
        // Drop connections with poor routing (high hop counts)
        // Keep at least 3 connections for network resilience
        let min_connections = 3;
        let mut dropped = 0;
        
        for (peer_id, hop_count) in peers_by_hop {
            let current_count = self.get_active_connections().await.len();
            
            if current_count <= min_connections {
                break;
            }
            
            // Drop peers with hop count > 5 (too far from providers)
            if hop_count > 5 {
                debug!("Dropping peer {} with high hop count: {}", peer_id, hop_count);
                if let Err(e) = self.close_connection(peer_id).await {
                    warn!("Failed to close connection during optimization: {}", e);
                } else {
                    dropped += 1;
                }
            }
        }
        
        debug!("Connection optimization complete, dropped {} connections", dropped);
        Ok(dropped)
    }
    
    /// Get the maximum number of connections allowed
    pub fn max_connections(&self) -> usize {
        self.max_connections
    }
}

impl Default for PeerConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
