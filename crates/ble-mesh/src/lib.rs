//! BLE mesh networking for offline P2P communication
//!
//! This crate implements a BLE mesh network with packet routing, store-and-forward,
//! and integration with stealth payment requests.

pub mod adapter;
pub mod error;
pub mod router;
pub mod stealth_handler;
pub mod store_forward;

// Re-export main types
pub use adapter::{BLEAdapter, BLEAdapterImpl};
pub use error::{MeshError, MeshResult};
pub use router::{MeshPacket, MeshRouter};
pub use stealth_handler::BLEMeshHandler;
pub use store_forward::StoreForwardQueue;
