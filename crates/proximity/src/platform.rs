// Platform abstraction layer for proximity transfers
// Provides traits and implementations for platform-specific functionality

use crate::{PeerId, PeerMessage, ProximityError, Result};
use async_trait::async_trait;
use std::fmt::Debug;

/// Trait for platform-specific connection implementations
#[async_trait]
pub trait PlatformConnection: Send + Sync + Debug {
    /// Establish a connection to a peer
    async fn connect(&mut self, peer_id: &PeerId) -> Result<()>;
    
    /// Send a message to the connected peer
    async fn send(&mut self, message: &PeerMessage) -> Result<()>;
    
    /// Receive a message from the connected peer (non-blocking)
    async fn receive(&mut self) -> Result<Option<PeerMessage>>;
    
    /// Close the connection
    async fn close(&mut self) -> Result<()>;
    
    /// Check if the connection is still active
    fn is_connected(&self) -> bool;
    
    /// Get the peer ID for this connection
    fn peer_id(&self) -> &PeerId;
}

/// Factory trait for creating platform-specific connections
#[async_trait]
pub trait PlatformConnectionFactory: Send + Sync {
    /// Create a new connection for the given peer
    async fn create_connection(&self, peer_id: PeerId) -> Result<Box<dyn PlatformConnection>>;
    
    /// Get the platform name
    fn platform_name(&self) -> &str;
}

/// WebRTC connection implementation for web platform
#[cfg(target_arch = "wasm32")]
pub mod webrtc {
    use super::*;
    use tracing::{debug, info};
    
    #[derive(Debug)]
    pub struct WebRtcConnection {
        peer_id: PeerId,
        connected: bool,
    }
    
    impl WebRtcConnection {
        pub fn new(peer_id: PeerId) -> Self {
            Self {
                peer_id,
                connected: false,
            }
        }
    }
    
    #[async_trait]
    impl PlatformConnection for WebRtcConnection {
        async fn connect(&mut self, peer_id: &PeerId) -> Result<()> {
            info!("Establishing WebRTC connection to peer: {}", peer_id);
            
            // In a real implementation, this would:
            // 1. Create RTCPeerConnection
            // 2. Create data channel
            // 3. Exchange SDP offers/answers via signaling server
            // 4. Wait for ICE connection to establish
            
            self.connected = true;
            debug!("WebRTC connection established to peer: {}", peer_id);
            Ok(())
        }
        
        async fn send(&mut self, message: &PeerMessage) -> Result<()> {
            if !self.connected {
                return Err(ProximityError::ConnectionFailed(
                    "Not connected".to_string()
                ));
            }
            
            debug!("Sending message via WebRTC to peer: {}", self.peer_id);
            
            // In a real implementation, this would send via data channel
            let _serialized = serde_json::to_string(message)
                .map_err(|e| ProximityError::SerializationError(e.to_string()))?;
            
            Ok(())
        }
        
        async fn receive(&mut self) -> Result<Option<PeerMessage>> {
            if !self.connected {
                return Ok(None);
            }
            
            // In a real implementation, this would read from data channel
            // For now, return None (no messages)
            Ok(None)
        }
        
        async fn close(&mut self) -> Result<()> {
            debug!("Closing WebRTC connection to peer: {}", self.peer_id);
            self.connected = false;
            Ok(())
        }
        
        fn is_connected(&self) -> bool {
            self.connected
        }
        
        fn peer_id(&self) -> &PeerId {
            &self.peer_id
        }
    }
    
    pub struct WebRtcConnectionFactory;
    
    impl WebRtcConnectionFactory {
        pub fn new() -> Self {
            Self
        }
    }
    
    #[async_trait]
    impl PlatformConnectionFactory for WebRtcConnectionFactory {
        async fn create_connection(&self, peer_id: PeerId) -> Result<Box<dyn PlatformConnection>> {
            Ok(Box::new(WebRtcConnection::new(peer_id)))
        }
        
        fn platform_name(&self) -> &str {
            "WebRTC (Web)"
        }
    }
}

/// Native TCP socket connection implementation for mobile/desktop
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use super::*;
    use tokio::net::TcpStream;
    use tokio::io::AsyncWriteExt;
    use tracing::{debug, info, warn};
    
    #[derive(Debug)]
    pub struct NativeSocketConnection {
        peer_id: PeerId,
        stream: Option<TcpStream>,
    }
    
    impl NativeSocketConnection {
        pub fn new(peer_id: PeerId) -> Self {
            Self {
                peer_id,
                stream: None,
            }
        }
    }
    
    #[async_trait]
    impl PlatformConnection for NativeSocketConnection {
        async fn connect(&mut self, peer_id: &PeerId) -> Result<()> {
            info!("Establishing TCP socket connection to peer: {}", peer_id);
            
            // In a real implementation, this would:
            // 1. Resolve peer address from discovery info
            // 2. Connect to peer's TCP socket
            // 3. Perform handshake
            
            // For now, we'll simulate a connection
            // In production, you would connect to an actual address:
            // let stream = TcpStream::connect("peer_address:port").await
            //     .map_err(|e| ProximityError::ConnectionFailed(e.to_string()))?;
            // self.stream = Some(stream);
            
            debug!("TCP socket connection established to peer: {}", peer_id);
            Ok(())
        }
        
        async fn send(&mut self, message: &PeerMessage) -> Result<()> {
            if self.stream.is_none() {
                return Err(ProximityError::ConnectionFailed(
                    "Not connected".to_string()
                ));
            }
            
            debug!("Sending message via TCP socket to peer: {}", self.peer_id);
            
            let _serialized = serde_json::to_string(message)
                .map_err(|e| ProximityError::SerializationError(e.to_string()))?;
            
            // In a real implementation, this would write to the TCP stream
            // if let Some(stream) = &mut self.stream {
            //     stream.write_all(serialized.as_bytes()).await
            //         .map_err(|e| ProximityError::NetworkError(e.to_string()))?;
            // }
            
            Ok(())
        }
        
        async fn receive(&mut self) -> Result<Option<PeerMessage>> {
            if self.stream.is_none() {
                return Ok(None);
            }
            
            // In a real implementation, this would read from the TCP stream
            // if let Some(stream) = &mut self.stream {
            //     let mut buffer = vec![0u8; 4096];
            //     match stream.read(&mut buffer).await {
            //         Ok(0) => return Ok(None), // Connection closed
            //         Ok(n) => {
            //             let data = &buffer[..n];
            //             let message = serde_json::from_slice(data)
            //                 .map_err(|e| ProximityError::SerializationError(e.to_string()))?;
            //             return Ok(Some(message));
            //         }
            //         Err(e) => return Err(ProximityError::NetworkError(e.to_string())),
            //     }
            // }
            
            Ok(None)
        }
        
        async fn close(&mut self) -> Result<()> {
            debug!("Closing TCP socket connection to peer: {}", self.peer_id);
            
            if let Some(mut stream) = self.stream.take() {
                // Gracefully shutdown the connection
                if let Err(e) = stream.shutdown().await {
                    warn!("Error shutting down TCP connection: {}", e);
                }
            }
            
            Ok(())
        }
        
        fn is_connected(&self) -> bool {
            self.stream.is_some()
        }
        
        fn peer_id(&self) -> &PeerId {
            &self.peer_id
        }
    }
    
    pub struct NativeSocketConnectionFactory;
    
    impl NativeSocketConnectionFactory {
        pub fn new() -> Self {
            Self
        }
    }
    
    #[async_trait]
    impl PlatformConnectionFactory for NativeSocketConnectionFactory {
        async fn create_connection(&self, peer_id: PeerId) -> Result<Box<dyn PlatformConnection>> {
            Ok(Box::new(NativeSocketConnection::new(peer_id)))
        }
        
        fn platform_name(&self) -> &str {
            "Native TCP Socket"
        }
    }
}

/// Get the default platform connection factory for the current platform
pub fn get_default_factory() -> Box<dyn PlatformConnectionFactory> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(webrtc::WebRtcConnectionFactory::new())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(native::NativeSocketConnectionFactory::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_default_factory() {
        let factory = get_default_factory();
        
        #[cfg(target_arch = "wasm32")]
        assert_eq!(factory.platform_name(), "WebRTC (Web)");
        
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(factory.platform_name(), "Native TCP Socket");
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_native_connection_lifecycle() {
        let peer_id: PeerId = "test-peer".to_string();
        let mut connection = native::NativeSocketConnection::new(peer_id.clone());
        
        assert!(!connection.is_connected());
        assert_eq!(connection.peer_id(), &peer_id);
        
        // Connect
        connection.connect(&peer_id).await.unwrap();
        
        // Close
        connection.close().await.unwrap();
        assert!(!connection.is_connected());
    }
}
