# BLE Mesh Networking Crate

Bluetooth Low Energy mesh networking for offline P2P communication.

## Features

- **Dual-Mode BLE**: Simultaneous Central and Peripheral operation
- **Packet Routing**: Multi-hop message relay with TTL management
- **Deduplication**: Bloom filter-based loop prevention
- **Store-and-Forward**: Message queueing for offline recipients
- **Fragmentation**: Automatic payload splitting for BLE MTU limits
- **Cross-Platform**: Works on iOS and Android via btleplug

## Module Structure

- `router`: Core mesh packet routing with TTL and deduplication
- `adapter`: Platform-agnostic BLE abstraction layer
- `store_forward`: Message queue for offline recipients
- `stealth_handler`: Integration with stealth payment requests
- `error`: Error types for mesh operations

## Packet Format

```rust
MeshPacket {
    id: PacketId,           // Unique packet identifier (UUID)
    source: DeviceId,       // Originating device
    destination: Option<DeviceId>, // Target device (None = broadcast)
    ttl: u8,                // Time-to-live (hops remaining)
    payload: Vec<u8>,       // Encrypted payload
    timestamp: SystemTime,  // Creation time
}
```

## Architecture

```
┌─────────────────────────────────────────┐
│         BLE Mesh Handler                │
│  (Stealth Payment Integration)          │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│         Mesh Router                     │
│  • Packet forwarding                    │
│  • TTL management                       │
│  • Deduplication (Bloom filter)         │
└─────────┬───────────────┬───────────────┘
          │               │
┌─────────▼─────┐  ┌──────▼──────────────┐
│ Store-Forward │  │   BLE Adapter       │
│     Queue     │  │  (btleplug)         │
└───────────────┘  └─────────────────────┘
```

## Implementation Status

This crate is currently in development. Module stubs have been created with `todo!()` placeholders.
Implementation will proceed according to the task list in `.kiro/specs/ble-mesh-stealth-transfers/tasks.md`.

## Dependencies

- `btleplug`: Cross-platform BLE support (iOS, Android, Linux, macOS, Windows)
- `bloomfilter`: Efficient packet deduplication
- `dashmap`: Concurrent HashMap for peer management
- `tokio`: Async runtime
- `proptest`: Property-based testing framework

## Testing

The crate uses a dual testing approach:
- **Unit tests**: BLE adapter initialization, connection management
- **Property-based tests**: Packet routing, TTL decrement, deduplication

Run tests with:
```bash
cargo test --package ble-mesh
```

## Platform Support

- **iOS**: CoreBluetooth via btleplug
- **Android**: Android Bluetooth API via btleplug
- **Desktop**: Native BLE stacks for development/testing
