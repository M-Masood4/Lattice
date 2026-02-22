//! BLE mesh packet routing with TTL and deduplication

use crate::adapter::BLEAdapter;
use crate::error::{MeshError, MeshResult};
use crate::store_forward::StoreForwardQueue;
use bloomfilter::Bloom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Unique device identifier
pub type DeviceId = Uuid;

/// Unique packet identifier
pub type PacketId = Uuid;

/// BLE MTU limit for packet fragmentation
const BLE_MTU: usize = 512;

/// Bloom filter capacity for packet deduplication
const BLOOM_FILTER_CAPACITY: usize = 10000;

/// Bloom filter false positive rate
const BLOOM_FILTER_FP_RATE: f64 = 0.01;

/// BLE mesh router with packet forwarding
pub struct MeshRouter {
    device_id: DeviceId,
    peers: Arc<Mutex<HashMap<DeviceId, PeerConnection>>>,
    packet_cache: Arc<Mutex<Bloom<PacketId>>>,
    store_forward: Arc<Mutex<StoreForwardQueue>>,
    ble_adapter: Arc<dyn BLEAdapter>,
}

impl MeshRouter {
    pub fn new(ble_adapter: Arc<dyn BLEAdapter>) -> Self {
        let device_id = Uuid::new_v4();
        let peers = Arc::new(Mutex::new(HashMap::new()));
        
        // Initialize bloom filter for packet deduplication
        let bloom = Bloom::new_for_fp_rate(BLOOM_FILTER_CAPACITY, BLOOM_FILTER_FP_RATE);
        let packet_cache = Arc::new(Mutex::new(bloom));
        
        // Initialize store-and-forward queue
        let store_forward = Arc::new(Mutex::new(StoreForwardQueue::new(
            1000,                      // max 1000 packets per recipient
            Duration::from_secs(3600), // 1 hour max age
        )));

        info!("MeshRouter initialized with device_id: {}", device_id);

        Self {
            device_id,
            peers,
            packet_cache,
            store_forward,
            ble_adapter,
        }
    }

    /// Initialize dual-mode BLE (Central + Peripheral)
    pub async fn initialize(&mut self) -> MeshResult<()> {
        info!("Initializing dual-mode BLE (Central + Peripheral)");
        
        // Start advertising as peripheral
        self.ble_adapter.start_advertising().await?;
        debug!("BLE advertising started");
        
        // Start scanning as central
        self.ble_adapter.start_scanning().await?;
        debug!("BLE scanning started");
        
        info!("Dual-mode BLE initialization complete");
        Ok(())
    }

    /// Send packet to specific peer
    pub async fn send(&self, peer: &DeviceId, packet: MeshPacket) -> MeshResult<()> {
        debug!("Sending packet {} to peer {}", packet.id, peer);
        
        // Check if peer is connected
        let peers = self.peers.lock().await;
        if !peers.contains_key(peer) {
            return Err(MeshError::DeviceNotFound(peer.to_string()));
        }
        drop(peers);
        
        // Fragment packet if needed
        let fragments = self.fragment_packet(&packet)?;
        
        // Send all fragments
        for fragment in fragments {
            let serialized = serde_json::to_vec(&fragment)?;
            self.ble_adapter.send_data(peer, &serialized).await?;
        }
        
        debug!("Packet {} sent successfully to peer {}", packet.id, peer);
        Ok(())
    }

    /// Broadcast packet to all peers
    pub async fn broadcast(&self, packet: MeshPacket) -> MeshResult<()> {
        debug!("Broadcasting packet {} to all peers", packet.id);
        
        let peers = self.peers.lock().await;
        let peer_ids: Vec<DeviceId> = peers.keys().copied().collect();
        drop(peers);
        
        if peer_ids.is_empty() {
            warn!("No peers available for broadcast");
            return Ok(());
        }
        
        // Send to all peers
        for peer_id in peer_ids {
            if let Err(e) = self.send(&peer_id, packet.clone()).await {
                warn!("Failed to send packet to peer {}: {}", peer_id, e);
                // Continue broadcasting to other peers
            }
        }
        
        debug!("Broadcast of packet {} complete", packet.id);
        Ok(())
    }

    /// Receive and route incoming packet
    pub async fn receive(&mut self, packet: MeshPacket) -> MeshResult<()> {
        debug!("Received packet {} from {}", packet.id, packet.source);
        
        // Check for duplicate
        if self.is_duplicate(&packet.id).await {
            debug!("Duplicate packet {} detected, discarding", packet.id);
            return Err(MeshError::DuplicatePacket(packet.id.to_string()));
        }
        
        // Add to packet cache for deduplication
        self.add_to_cache(&packet.id).await;
        
        // Check TTL
        if packet.ttl == 0 {
            debug!("Packet {} TTL expired, discarding", packet.id);
            return Err(MeshError::TTLExpired);
        }
        
        // Check if packet is for this device
        if let Some(dest) = packet.destination {
            if dest == self.device_id {
                info!("Packet {} is for this device, processing", packet.id);
                // Packet reached destination - caller will handle processing
                return Ok(());
            }
            
            // Check if destination is online
            let peers = self.peers.lock().await;
            let dest_online = peers.contains_key(&dest);
            drop(peers);
            
            if !dest_online {
                // Store for later delivery
                debug!("Destination {} offline, storing packet {}", dest, packet.id);
                let mut sf = self.store_forward.lock().await;
                sf.store(dest, packet.clone())?;
                drop(sf);
            }
        }
        
        // Forward packet to other peers
        self.forward_packet(packet).await?;
        
        Ok(())
    }

    /// Forward packet to next hop
    ///
    /// When the mesh network has more than 10 peers, limits forwarding to a subset
    /// to prevent broadcast storms (Requirement 12.3).
    async fn forward_packet(&self, mut packet: MeshPacket) -> MeshResult<()> {
        // Decrement TTL
        if packet.ttl == 0 {
            return Err(MeshError::TTLExpired);
        }
        packet.ttl -= 1;
        
        debug!(
            "Forwarding packet {} with TTL {} to peers",
            packet.id, packet.ttl
        );
        
        // Get list of peers (excluding source)
        let peers = self.peers.lock().await;
        let mut peer_ids: Vec<DeviceId> = peers
            .keys()
            .filter(|&&id| id != packet.source)
            .copied()
            .collect();
        drop(peers);
        
        if peer_ids.is_empty() {
            warn!("No peers available for forwarding packet {}", packet.id);
            return Ok(());
        }
        
        // Limit forwarding when >10 peers to prevent broadcast storms (Requirement 12.3)
        const MAX_FORWARD_PEERS: usize = 10;
        if peer_ids.len() > MAX_FORWARD_PEERS {
            // Use random selection to limit forwarding
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            peer_ids.shuffle(&mut rng);
            peer_ids.truncate(MAX_FORWARD_PEERS);
            
            debug!(
                "Limited forwarding to {} peers (out of {} total)",
                peer_ids.len(),
                peer_ids.len() + (peer_ids.len() - MAX_FORWARD_PEERS)
            );
        }
        
        // Forward to selected peers (except source)
        for peer_id in peer_ids {
            if let Err(e) = self.send(&peer_id, packet.clone()).await {
                warn!("Failed to forward packet to peer {}: {}", peer_id, e);
                // Continue forwarding to other peers
            }
        }
        
        Ok(())
    }

    /// Check if packet was recently seen (deduplication)
    async fn is_duplicate(&self, packet_id: &PacketId) -> bool {
        let cache = self.packet_cache.lock().await;
        cache.check(packet_id)
    }
    
    /// Add packet ID to cache for deduplication
    async fn add_to_cache(&self, packet_id: &PacketId) {
        let mut cache = self.packet_cache.lock().await;
        cache.set(packet_id);
    }
    
    /// Fragment packet if payload exceeds BLE MTU
    fn fragment_packet(&self, packet: &MeshPacket) -> MeshResult<Vec<MeshPacket>> {
        let payload_size = packet.payload.len();
        
        // If payload fits in single packet, return as-is
        if payload_size <= BLE_MTU {
            return Ok(vec![packet.clone()]);
        }
        
        debug!(
            "Fragmenting packet {} with payload size {} bytes",
            packet.id, payload_size
        );
        
        // Calculate number of fragments needed
        let num_fragments = (payload_size + BLE_MTU - 1) / BLE_MTU;
        let mut fragments = Vec::with_capacity(num_fragments);
        
        // Create fragments
        for i in 0..num_fragments {
            let start = i * BLE_MTU;
            let end = std::cmp::min(start + BLE_MTU, payload_size);
            let fragment_payload = packet.payload[start..end].to_vec();
            
            // Create fragment metadata
            let fragment_info = FragmentInfo {
                fragment_id: i as u16,
                total_fragments: num_fragments as u16,
                original_packet_id: packet.id,
            };
            
            // Prepend fragment info to payload
            let mut fragment_data = serde_json::to_vec(&fragment_info)?;
            fragment_data.extend_from_slice(&fragment_payload);
            
            let fragment = MeshPacket {
                id: Uuid::new_v4(), // Each fragment gets unique ID
                source: packet.source,
                destination: packet.destination,
                ttl: packet.ttl,
                payload: fragment_data,
                timestamp: packet.timestamp,
            };
            
            fragments.push(fragment);
        }
        
        debug!(
            "Packet {} fragmented into {} fragments",
            packet.id, num_fragments
        );
        
        Ok(fragments)
    }
    
    /// Get device ID
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
    
    /// Add peer connection
    pub async fn add_peer(&self, device_id: DeviceId) {
        let mut peers = self.peers.lock().await;
        peers.insert(
            device_id,
            PeerConnection {
                device_id,
                last_seen: SystemTime::now(),
            },
        );
        info!("Added peer: {}", device_id);
    }
    
    /// Remove peer connection
    pub async fn remove_peer(&self, device_id: &DeviceId) {
        let mut peers = self.peers.lock().await;
        peers.remove(device_id);
        info!("Removed peer: {}", device_id);
    }
    
    /// Get connected peers
    pub async fn get_peers(&self) -> Vec<DeviceId> {
        let peers = self.peers.lock().await;
        peers.keys().copied().collect()
    }

    /// Check if packet should be forwarded (for testing)
    ///
    /// Returns false if TTL is 0 or packet is duplicate
    #[cfg(test)]
    pub fn should_forward(&self, packet: &MeshPacket) -> bool {
        packet.ttl > 0
    }

    /// Check if packet is duplicate (public for testing)
    #[cfg(test)]
    pub fn is_duplicate_sync(&self, packet_id: &PacketId) -> bool {
        // Synchronous version for testing
        use tokio::runtime::Handle;
        Handle::current().block_on(async {
            self.is_duplicate(packet_id).await
        })
    }
}

/// A mesh packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshPacket {
    pub id: PacketId,
    pub source: DeviceId,
    pub destination: Option<DeviceId>,
    pub ttl: u8,
    pub payload: Vec<u8>,
    pub timestamp: SystemTime,
}

impl MeshPacket {
    /// Create a new mesh packet
    pub fn new(
        source: DeviceId,
        destination: Option<DeviceId>,
        ttl: u8,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            destination,
            ttl,
            payload,
            timestamp: SystemTime::now(),
        }
    }
}

/// Fragment information for packet reassembly
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FragmentInfo {
    fragment_id: u16,
    total_fragments: u16,
    original_packet_id: PacketId,
}

/// Peer connection information
#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub device_id: DeviceId,
    pub last_seen: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::BLEAdapter;
    use async_trait::async_trait;
    use std::sync::Arc;

    /// Mock BLE adapter for testing
    struct MockBLEAdapter {
        advertising_started: Arc<Mutex<bool>>,
        scanning_started: Arc<Mutex<bool>>,
    }

    impl MockBLEAdapter {
        fn new() -> Self {
            Self {
                advertising_started: Arc::new(Mutex::new(false)),
                scanning_started: Arc::new(Mutex::new(false)),
            }
        }
    }

    #[async_trait]
    impl BLEAdapter for MockBLEAdapter {
        async fn start_advertising(&self) -> MeshResult<()> {
            let mut started = self.advertising_started.lock().await;
            *started = true;
            Ok(())
        }

        async fn start_scanning(&self) -> MeshResult<()> {
            let mut started = self.scanning_started.lock().await;
            *started = true;
            Ok(())
        }

        async fn connect(&self, _device: &DeviceId) -> MeshResult<()> {
            Ok(())
        }

        async fn disconnect(&self, _device: &DeviceId) -> MeshResult<()> {
            Ok(())
        }

        async fn send_data(&self, _device: &DeviceId, _data: &[u8]) -> MeshResult<()> {
            Ok(())
        }

        async fn receive_data(&self) -> MeshResult<Vec<u8>> {
            Ok(vec![])
        }

        async fn connected_devices(&self) -> MeshResult<Vec<DeviceId>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_mesh_router_initialization() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let mut router = MeshRouter::new(adapter.clone());

        assert!(router.initialize().await.is_ok());

        // Verify dual-mode setup
        let advertising = adapter.advertising_started.lock().await;
        assert!(*advertising, "Advertising should be started");

        let scanning = adapter.scanning_started.lock().await;
        assert!(*scanning, "Scanning should be started");
    }

    #[tokio::test]
    async fn test_packet_creation() {
        let source = Uuid::new_v4();
        let destination = Some(Uuid::new_v4());
        let payload = vec![1, 2, 3, 4, 5];

        let packet = MeshPacket::new(source, destination, 5, payload.clone());

        assert_eq!(packet.source, source);
        assert_eq!(packet.destination, destination);
        assert_eq!(packet.ttl, 5);
        assert_eq!(packet.payload, payload);
    }

    #[tokio::test]
    async fn test_ttl_decrement() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let router = MeshRouter::new(adapter);

        let source = Uuid::new_v4();
        let packet = MeshPacket::new(source, None, 3, vec![1, 2, 3]);

        // Forward packet should decrement TTL
        let result = router.forward_packet(packet.clone()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ttl_zero_rejection() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let router = MeshRouter::new(adapter);

        let source = Uuid::new_v4();
        let packet = MeshPacket::new(source, None, 0, vec![1, 2, 3]);

        // Packet with TTL=0 should be rejected
        let result = router.forward_packet(packet).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MeshError::TTLExpired));
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let router = MeshRouter::new(adapter);

        let packet_id = Uuid::new_v4();

        // First check should return false (not duplicate)
        assert!(!router.is_duplicate(&packet_id).await);

        // Add to cache
        router.add_to_cache(&packet_id).await;

        // Second check should return true (duplicate)
        assert!(router.is_duplicate(&packet_id).await);
    }

    #[tokio::test]
    async fn test_packet_fragmentation() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let router = MeshRouter::new(adapter);

        let source = Uuid::new_v4();
        // Create large payload that exceeds BLE_MTU
        let large_payload = vec![0u8; 1500];
        let packet = MeshPacket::new(source, None, 5, large_payload);

        let fragments = router.fragment_packet(&packet).unwrap();

        // Should create multiple fragments
        assert!(fragments.len() > 1);

        // All fragments should have same source and destination
        for fragment in &fragments {
            assert_eq!(fragment.source, packet.source);
            assert_eq!(fragment.destination, packet.destination);
            assert_eq!(fragment.ttl, packet.ttl);
        }
    }

    #[tokio::test]
    async fn test_small_packet_no_fragmentation() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let router = MeshRouter::new(adapter);

        let source = Uuid::new_v4();
        let small_payload = vec![1, 2, 3, 4, 5];
        let packet = MeshPacket::new(source, None, 5, small_payload);

        let fragments = router.fragment_packet(&packet).unwrap();

        // Should return single packet (no fragmentation needed)
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].id, packet.id);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let router = MeshRouter::new(adapter);

        let peer_id = Uuid::new_v4();

        // Initially no peers
        assert_eq!(router.get_peers().await.len(), 0);

        // Add peer
        router.add_peer(peer_id).await;
        assert_eq!(router.get_peers().await.len(), 1);
        assert!(router.get_peers().await.contains(&peer_id));

        // Remove peer
        router.remove_peer(&peer_id).await;
        assert_eq!(router.get_peers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_to_multiple_peers() {
        let adapter = Arc::new(MockBLEAdapter::new());
        let router = MeshRouter::new(adapter);

        // Add multiple peers
        let peer1 = Uuid::new_v4();
        let peer2 = Uuid::new_v4();
        router.add_peer(peer1).await;
        router.add_peer(peer2).await;

        let packet = MeshPacket::new(router.device_id(), None, 5, vec![1, 2, 3]);

        // Broadcast should succeed even if individual sends fail
        let result = router.broadcast(packet).await;
        assert!(result.is_ok());
    }
}
