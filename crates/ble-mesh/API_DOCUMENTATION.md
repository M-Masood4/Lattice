# BLE Mesh Networking API Documentation

## Overview

This document provides comprehensive API documentation for the BLE mesh networking crate, including usage examples, error handling patterns, and integration guidelines.

## Table of Contents

1. [Mesh Router](#mesh-router)
2. [BLE Adapter](#ble-adapter)
3. [Store and Forward Queue](#store-and-forward-queue)
4. [Stealth Handler](#stealth-handler)
5. [Error Handling](#error-handling)
6. [Common Usage Patterns](#common-usage-patterns)

---

## Mesh Router

### `MeshRouter`

Core BLE mesh packet routing with TTL management and deduplication.

#### Methods

##### `new`

Create a new mesh router.

```rust
pub fn new(ble_adapter: Box<dyn BLEAdapter>) -> Self
```

**Parameters:**
- `ble_adapter`: Platform-specific BLE adapter implementation

**Example:**
```rust
use ble_mesh::{MeshRouter, BLEAdapterImpl};

let adapter = Box::new(BLEAdapterImpl::new());
let router = MeshRouter::new(adapter);
```

##### `initialize`

Initialize dual-mode BLE (Central + Peripheral).

```rust
pub async fn initialize(&mut self) -> MeshResult<()>
```

**Returns:**
- `Ok(())`: BLE initialized successfully
- `Err(MeshError::BLEInitFailed)`: Initialization failed

**Example:**
```rust
router.initialize().await?;
println!("BLE mesh network ready");
```


##### `send`

Send packet to specific peer.

```rust
pub async fn send(&self, peer: &DeviceId, packet: MeshPacket) -> MeshResult<()>
```

**Parameters:**
- `peer`: Target device ID
- `packet`: Packet to send

**Returns:**
- `Ok(())`: Packet sent successfully
- `Err(MeshError::PeerNotFound)`: Peer not connected
- `Err(MeshError::TransmissionFailed)`: Send failed

**Example:**
```rust
let packet = MeshPacket {
    id: PacketId::new(),
    source: my_device_id,
    destination: Some(peer_device_id),
    ttl: 5,
    payload: encrypted_data,
    timestamp: SystemTime::now(),
};

router.send(&peer_device_id, packet).await?;
```

##### `broadcast`

Broadcast packet to all connected peers.

```rust
pub async fn broadcast(&self, packet: MeshPacket) -> MeshResult<()>
```

**Parameters:**
- `packet`: Packet to broadcast (destination should be None)

**Example:**
```rust
let broadcast_packet = MeshPacket {
    id: PacketId::new(),
    source: my_device_id,
    destination: None, // Broadcast
    ttl: 3,
    payload: announcement_data,
    timestamp: SystemTime::now(),
};

router.broadcast(broadcast_packet).await?;
```

##### `receive`

Receive and route incoming packet.

```rust
pub async fn receive(&mut self, packet: MeshPacket) -> MeshResult<()>
```

**Parameters:**
- `packet`: Received packet

**Behavior:**
- If packet is for this device: deliver to application
- If packet is for another device: forward to next hop
- If TTL is 0: discard packet
- If packet is duplicate: discard packet

**Example:**
```rust
// Called automatically by BLE adapter when packet arrives
router.receive(incoming_packet).await?;
```

### `MeshPacket`

Mesh network packet structure.

```rust
pub struct MeshPacket {
    pub id: PacketId,
    pub source: DeviceId,
    pub destination: Option<DeviceId>,
    pub ttl: u8,
    pub payload: Vec<u8>,
    pub timestamp: SystemTime,
}
```

**Fields:**
- `id`: Unique packet identifier (UUID)
- `source`: Originating device
- `destination`: Target device (None = broadcast)
- `ttl`: Time-to-live (hops remaining, decrements each forward)
- `payload`: Encrypted payload data
- `timestamp`: Creation time

**Example:**
```rust
use ble_mesh::{MeshPacket, PacketId, DeviceId};
use std::time::SystemTime;

let packet = MeshPacket {
    id: PacketId::new(),
    source: DeviceId::from_bytes(&[1; 16]),
    destination: Some(DeviceId::from_bytes(&[2; 16])),
    ttl: 5,
    payload: vec![1, 2, 3, 4],
    timestamp: SystemTime::now(),
};
```


---

## BLE Adapter

### `BLEAdapter` Trait

Platform-agnostic BLE abstraction layer.

```rust
pub trait BLEAdapter: Send + Sync {
    fn start_advertising(&self) -> MeshResult<()>;
    fn start_scanning(&self) -> MeshResult<()>;
    fn connect(&self, device: &DeviceId) -> MeshResult<()>;
    fn disconnect(&self, device: &DeviceId) -> MeshResult<()>;
    fn send_data(&self, device: &DeviceId, data: &[u8]) -> MeshResult<()>;
    fn receive_data(&self) -> MeshResult<Vec<u8>>;
}
```

### `BLEAdapterImpl`

Cross-platform BLE adapter using btleplug.

#### Methods

##### `new`

Create a new BLE adapter.

```rust
pub fn new() -> Self
```

**Example:**
```rust
use ble_mesh::BLEAdapterImpl;

let adapter = BLEAdapterImpl::new();
```

##### `start_advertising`

Start BLE advertising (Peripheral mode).

```rust
fn start_advertising(&self) -> MeshResult<()>
```

**Returns:**
- `Ok(())`: Advertising started
- `Err(MeshError::BLEError)`: Failed to start advertising

**Example:**
```rust
adapter.start_advertising()?;
println!("Now discoverable by other devices");
```

##### `start_scanning`

Start BLE scanning (Central mode).

```rust
fn start_scanning(&self) -> MeshResult<()>
```

**Returns:**
- `Ok(())`: Scanning started
- `Err(MeshError::BLEError)`: Failed to start scanning

**Example:**
```rust
adapter.start_scanning()?;
println!("Scanning for nearby devices");
```

##### `connect`

Connect to a discovered device.

```rust
fn connect(&self, device: &DeviceId) -> MeshResult<()>
```

**Parameters:**
- `device`: Device ID to connect to

**Returns:**
- `Ok(())`: Connected successfully
- `Err(MeshError::ConnectionFailed)`: Connection failed

**Example:**
```rust
// After discovering device
adapter.connect(&discovered_device_id)?;
```

##### `disconnect`

Disconnect from a device.

```rust
fn disconnect(&self, device: &DeviceId) -> MeshResult<()>
```

**Example:**
```rust
adapter.disconnect(&device_id)?;
```

##### `send_data`

Send data to connected device.

```rust
fn send_data(&self, device: &DeviceId, data: &[u8]) -> MeshResult<()>
```

**Parameters:**
- `device`: Target device ID
- `data`: Data to send (automatically fragmented if > MTU)

**Returns:**
- `Ok(())`: Data sent successfully
- `Err(MeshError::TransmissionFailed)`: Send failed

**Example:**
```rust
let payload = b"Hello, mesh network!";
adapter.send_data(&device_id, payload)?;
```

##### `receive_data`

Receive data from connected device.

```rust
fn receive_data(&self) -> MeshResult<Vec<u8>>
```

**Returns:**
- `Ok(Vec<u8>)`: Received data (reassembled if fragmented)
- `Err(MeshError::ReceiveFailed)`: Receive failed

**Example:**
```rust
let data = adapter.receive_data()?;
println!("Received {} bytes", data.len());
```


---

## Store and Forward Queue

### `StoreForwardQueue`

Message queue for offline recipients.

#### Methods

##### `new`

Create a new store-and-forward queue.

```rust
pub fn new(max_queue_size: usize, max_packet_age: Duration) -> Self
```

**Parameters:**
- `max_queue_size`: Maximum packets per recipient
- `max_packet_age`: Maximum age before packet expiration

**Example:**
```rust
use ble_mesh::StoreForwardQueue;
use std::time::Duration;

let queue = StoreForwardQueue::new(
    100,  // Max 100 packets per recipient
    Duration::from_secs(3600)  // 1 hour expiration
);
```

##### `store`

Store packet for offline recipient.

```rust
pub fn store(&mut self, recipient: DeviceId, packet: MeshPacket) -> MeshResult<()>
```

**Parameters:**
- `recipient`: Offline device ID
- `packet`: Packet to store

**Returns:**
- `Ok(())`: Packet stored
- `Err(MeshError::QueueFull)`: Recipient queue at capacity

**Example:**
```rust
// When recipient is offline
queue.store(offline_device_id, packet)?;
println!("Packet queued for delivery");
```

##### `retrieve`

Retrieve packets when recipient comes online.

```rust
pub fn retrieve(&mut self, recipient: &DeviceId) -> Vec<MeshPacket>
```

**Parameters:**
- `recipient`: Device ID that came online

**Returns:**
- `Vec<MeshPacket>`: All queued packets for recipient

**Example:**
```rust
// When device comes online
let queued_packets = queue.retrieve(&device_id);
println!("Delivering {} queued packets", queued_packets.len());

for packet in queued_packets {
    router.send(&device_id, packet).await?;
}
```

##### `cleanup_expired`

Remove expired packets from queue.

```rust
pub fn cleanup_expired(&mut self)
```

**Example:**
```rust
// Call periodically
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await;
        queue.cleanup_expired();
    }
});
```

---

## Stealth Handler

### `BLEMeshHandler`

Integration between BLE mesh and stealth payments.

#### Methods

##### `new`

Create a new stealth handler.

```rust
pub fn new(
    mesh_router: Arc<Mutex<MeshRouter>>,
    wallet_manager: Arc<Mutex<StealthWalletManager>>
) -> Self
```

**Parameters:**
- `mesh_router`: Mesh router instance
- `wallet_manager`: Stealth wallet manager

**Example:**
```rust
use ble_mesh::BLEMeshHandler;
use std::sync::{Arc, Mutex};

let handler = BLEMeshHandler::new(
    Arc::new(Mutex::new(router)),
    Arc::new(Mutex::new(wallet_manager))
);
```

##### `send_payment_via_mesh`

Send stealth payment request through mesh network.

```rust
pub async fn send_payment_via_mesh(
    &self,
    receiver_meta_address: &str,
    amount: u64
) -> MeshResult<()>
```

**Parameters:**
- `receiver_meta_address`: Receiver's stealth meta-address
- `amount`: Payment amount in lamports

**Returns:**
- `Ok(())`: Payment request sent via mesh
- `Err(MeshError)`: Send failed

**Example:**
```rust
// Send payment without internet connectivity
handler.send_payment_via_mesh(
    "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...",
    1_000_000_000  // 1 SOL
).await?;

println!("Payment request sent via BLE mesh");
```

##### `handle_mesh_packet`

Handle incoming mesh packet (called automatically).

```rust
pub async fn handle_mesh_packet(&self, packet: MeshPacket) -> MeshResult<()>
```

**Parameters:**
- `packet`: Received mesh packet

**Behavior:**
- Decrypts payment request payload
- Adds to payment queue if offline
- Processes immediately if online

**Example:**
```rust
// Called automatically by mesh router
handler.handle_mesh_packet(incoming_packet).await?;
```


---

## Error Handling

### `MeshError`

All errors in the BLE mesh crate.

```rust
pub enum MeshError {
    BLEInitFailed(String),
    BLEError(String),
    ConnectionFailed(String),
    TransmissionFailed(String),
    ReceiveFailed(String),
    PeerNotFound,
    QueueFull,
    PacketTooLarge,
    InvalidPacket(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
}
```

### Error Handling Patterns

#### Pattern 1: Retry with exponential backoff

```rust
use tokio::time::{sleep, Duration};

async fn send_with_retry(
    router: &MeshRouter,
    peer: &DeviceId,
    packet: MeshPacket,
    max_retries: u32
) -> MeshResult<()> {
    let mut retries = 0;
    
    loop {
        match router.send(peer, packet.clone()).await {
            Ok(()) => return Ok(()),
            Err(MeshError::TransmissionFailed(_)) if retries < max_retries => {
                retries += 1;
                let backoff = Duration::from_millis(100 * 2u64.pow(retries));
                sleep(backoff).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

#### Pattern 2: Fallback to store-and-forward

```rust
async fn send_or_queue(
    router: &MeshRouter,
    queue: &mut StoreForwardQueue,
    peer: &DeviceId,
    packet: MeshPacket
) -> MeshResult<()> {
    match router.send(peer, packet.clone()).await {
        Ok(()) => {
            println!("Sent directly");
            Ok(())
        }
        Err(MeshError::PeerNotFound) => {
            queue.store(*peer, packet)?;
            println!("Peer offline - queued for later");
            Ok(())
        }
        Err(e) => Err(e),
    }
}
```

---

## Common Usage Patterns

### Complete Mesh Setup

```rust
use ble_mesh::{MeshRouter, BLEAdapterImpl, StoreForwardQueue};
use std::time::Duration;

async fn setup_mesh_network() -> MeshResult<MeshRouter> {
    // 1. Create BLE adapter
    let adapter = Box::new(BLEAdapterImpl::new());
    
    // 2. Create mesh router
    let mut router = MeshRouter::new(adapter);
    
    // 3. Initialize BLE
    router.initialize().await?;
    
    println!("BLE mesh network initialized");
    Ok(router)
}
```

### Peer Discovery and Connection

```rust
async fn discover_and_connect(router: &MeshRouter) -> MeshResult<Vec<DeviceId>> {
    // Start scanning for peers
    router.adapter.start_scanning()?;
    
    // Wait for discovery
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Get discovered peers
    let peers = router.get_discovered_peers();
    
    // Connect to all peers
    for peer in &peers {
        match router.adapter.connect(peer) {
            Ok(()) => println!("Connected to peer: {:?}", peer),
            Err(e) => eprintln!("Failed to connect: {}", e),
        }
    }
    
    Ok(peers)
}
```

### Multi-Hop Message Relay

```rust
async fn relay_message_example() -> MeshResult<()> {
    let router = setup_mesh_network().await?;
    
    // Create packet with TTL=5 (up to 5 hops)
    let packet = MeshPacket {
        id: PacketId::new(),
        source: my_device_id,
        destination: Some(distant_device_id),
        ttl: 5,
        payload: encrypted_message,
        timestamp: SystemTime::now(),
    };
    
    // Send packet - will be relayed through intermediate nodes
    router.broadcast(packet).await?;
    
    println!("Message will relay through up to 5 hops");
    Ok(())
}
```

### Offline Payment via Mesh

```rust
use ble_mesh::BLEMeshHandler;
use stealth::StealthWalletManager;

async fn offline_mesh_payment() -> MeshResult<()> {
    // Setup
    let router = Arc::new(Mutex::new(setup_mesh_network().await?));
    let wallet_manager = Arc::new(Mutex::new(/* ... */));
    let handler = BLEMeshHandler::new(router, wallet_manager);
    
    // Send payment via mesh (no internet required)
    handler.send_payment_via_mesh(
        "stealth:1:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp...",
        500_000_000  // 0.5 SOL
    ).await?;
    
    println!("Payment request sent through BLE mesh");
    println!("Will settle on-chain when recipient is online");
    
    Ok(())
}
```

### Store-and-Forward Pattern

```rust
async fn store_forward_example() -> MeshResult<()> {
    let mut router = setup_mesh_network().await?;
    let mut queue = StoreForwardQueue::new(100, Duration::from_secs(3600));
    
    // Try to send packet
    let packet = MeshPacket { /* ... */ };
    let recipient = DeviceId::from_bytes(&[1; 16]);
    
    match router.send(&recipient, packet.clone()).await {
        Ok(()) => {
            println!("Delivered immediately");
        }
        Err(MeshError::PeerNotFound) => {
            // Recipient offline - store for later
            queue.store(recipient, packet)?;
            println!("Stored for later delivery");
            
            // When recipient comes online
            tokio::spawn(async move {
                // Wait for peer to connect
                // ...
                
                // Retrieve and deliver queued packets
                let queued = queue.retrieve(&recipient);
                for pkt in queued {
                    router.send(&recipient, pkt).await.ok();
                }
            });
        }
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```


### Packet Deduplication

```rust
// Deduplication is automatic via bloom filter
// Same packet received multiple times is processed only once

let packet = MeshPacket {
    id: PacketId::from_bytes(&[1; 16]), // Same ID
    // ...
};

// First receive: processed
router.receive(packet.clone()).await?;

// Second receive: automatically discarded (duplicate)
router.receive(packet.clone()).await?; // No-op

println!("Duplicate packets automatically filtered");
```

### TTL Management

```rust
// Packet with TTL=3 can traverse 3 hops

let packet = MeshPacket {
    id: PacketId::new(),
    source: device_a,
    destination: Some(device_d),
    ttl: 3,
    payload: data,
    timestamp: SystemTime::now(),
};

// Hop 1: A -> B (TTL decremented to 2)
// Hop 2: B -> C (TTL decremented to 1)
// Hop 3: C -> D (TTL decremented to 0, delivered)
// If D forwards: TTL=0, packet discarded

router.broadcast(packet).await?;
```

### Payload Fragmentation

```rust
// Large payloads automatically fragmented to fit BLE MTU

let large_payload = vec![0u8; 5000]; // 5KB payload
let packet = MeshPacket {
    id: PacketId::new(),
    source: my_device_id,
    destination: Some(peer_id),
    ttl: 5,
    payload: large_payload, // Automatically fragmented
    timestamp: SystemTime::now(),
};

// Adapter handles fragmentation and reassembly
router.send(&peer_id, packet).await?;
```

---

## Performance Considerations

### Connection Management

```rust
// Limit concurrent connections to avoid resource exhaustion
const MAX_CONNECTIONS: usize = 10;

async fn manage_connections(router: &MeshRouter) -> MeshResult<()> {
    let peers = router.get_discovered_peers();
    
    // Connect to closest/strongest peers only
    let mut connected = 0;
    for peer in peers.iter().take(MAX_CONNECTIONS) {
        if router.adapter.connect(peer).is_ok() {
            connected += 1;
        }
    }
    
    println!("Connected to {} peers", connected);
    Ok(())
}
```

### Broadcast Storm Prevention

```rust
// When >10 peers, limit forwarding to prevent broadcast storms

async fn smart_forward(
    router: &MeshRouter,
    packet: MeshPacket
) -> MeshResult<()> {
    let peers = router.get_connected_peers();
    
    if peers.len() > 10 {
        // Forward to subset only (e.g., 3 random peers)
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let selected: Vec<_> = peers.choose_multiple(&mut rng, 3).collect();
        
        for peer in selected {
            router.send(peer, packet.clone()).await.ok();
        }
    } else {
        // Forward to all peers
        router.broadcast(packet).await?;
    }
    
    Ok(())
}
```

### Queue Management

```rust
// Periodically clean expired packets

async fn queue_maintenance(queue: Arc<Mutex<StoreForwardQueue>>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(300)).await;
            
            let mut q = queue.lock().unwrap();
            q.cleanup_expired();
            
            println!("Queue cleanup completed");
        }
    });
}
```

---

## Platform-Specific Notes

### iOS

```rust
// iOS requires specific permissions in Info.plist:
// - NSBluetoothAlwaysUsageDescription
// - NSBluetoothPeripheralUsageDescription

// Request permissions before initializing
#[cfg(target_os = "ios")]
async fn setup_ios_ble() -> MeshResult<()> {
    // Permissions handled by btleplug automatically
    let adapter = Box::new(BLEAdapterImpl::new());
    let mut router = MeshRouter::new(adapter);
    router.initialize().await?;
    Ok(())
}
```

### Android

```rust
// Android requires runtime permissions:
// - BLUETOOTH
// - BLUETOOTH_ADMIN
// - ACCESS_FINE_LOCATION (for BLE scanning)

#[cfg(target_os = "android")]
async fn setup_android_ble() -> MeshResult<()> {
    // Request permissions via JNI before initializing
    // (handled by application layer)
    
    let adapter = Box::new(BLEAdapterImpl::new());
    let mut router = MeshRouter::new(adapter);
    router.initialize().await?;
    Ok(())
}
```

---

## Security Considerations

### 1. Payload Encryption

```rust
// Always encrypt payloads before sending

use stealth::StealthCrypto;

let plaintext = b"sensitive payment data";
let shared_key = [1u8; 32]; // From ECDH
let nonce = [2u8; 24]; // Unique nonce

let encrypted = StealthCrypto::encrypt_mesh_payload(
    plaintext,
    &shared_key,
    &nonce
)?;

let packet = MeshPacket {
    payload: encrypted, // Encrypted payload
    // ...
};
```

### 2. Peer Authentication

```rust
// Verify peer identity before accepting packets

async fn verify_peer(peer_id: &DeviceId) -> bool {
    // Implement challenge-response or certificate verification
    // ...
    true
}

async fn handle_packet_securely(
    packet: MeshPacket
) -> MeshResult<()> {
    if !verify_peer(&packet.source).await {
        return Err(MeshError::InvalidPacket("Untrusted peer".into()));
    }
    
    // Process packet
    Ok(())
}
```

### 3. Rate Limiting

```rust
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

struct RateLimiter {
    requests: HashMap<DeviceId, Vec<SystemTime>>,
    max_per_minute: usize,
}

impl RateLimiter {
    fn check(&mut self, peer: &DeviceId) -> bool {
        let now = SystemTime::now();
        let cutoff = now - Duration::from_secs(60);
        
        let requests = self.requests.entry(*peer).or_insert_with(Vec::new);
        requests.retain(|&t| t > cutoff);
        
        if requests.len() >= self.max_per_minute {
            return false; // Rate limit exceeded
        }
        
        requests.push(now);
        true
    }
}
```

---

## Troubleshooting

### Issue: BLE initialization fails

**Possible causes:**
1. Bluetooth disabled
2. Missing permissions
3. Unsupported hardware

**Solution:**
```rust
match router.initialize().await {
    Err(MeshError::BLEInitFailed(msg)) => {
        eprintln!("BLE init failed: {}", msg);
        // Check Bluetooth is enabled
        // Verify permissions granted
        // Check hardware compatibility
    }
    Ok(()) => println!("BLE ready"),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Issue: Peers not discovered

**Possible causes:**
1. Not scanning
2. Peers not advertising
3. Out of range

**Solution:**
```rust
// Ensure both scanning and advertising
adapter.start_scanning()?;
adapter.start_advertising()?;

// Wait longer for discovery
tokio::time::sleep(Duration::from_secs(10)).await;

// Check signal strength
let peers = router.get_discovered_peers();
println!("Found {} peers", peers.len());
```

### Issue: Packets not delivered

**Possible causes:**
1. TTL too low
2. No route to destination
3. Packet too large

**Solution:**
```rust
// Increase TTL
let packet = MeshPacket {
    ttl: 10, // Increased from 5
    // ...
};

// Check payload size
if packet.payload.len() > 5000 {
    eprintln!("Payload too large, consider splitting");
}

// Use broadcast for better delivery
router.broadcast(packet).await?;
```

---

## Additional Resources

- **Requirements Document**: `.kiro/specs/ble-mesh-stealth-transfers/requirements.md`
- **Design Document**: `.kiro/specs/ble-mesh-stealth-transfers/design.md`
- **btleplug Documentation**: https://github.com/deviceplug/btleplug
- **Bluetooth Core Specification**: https://www.bluetooth.com/specifications/specs/

---

## Changelog

### Version 0.1.0 (Current)
- Initial implementation
- Dual-mode BLE support (Central + Peripheral)
- Multi-hop packet routing with TTL
- Bloom filter deduplication
- Store-and-forward queue
- Payload fragmentation
- Stealth payment integration
- Cross-platform support (iOS, Android)

