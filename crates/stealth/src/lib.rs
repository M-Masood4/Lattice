//! Stealth address implementation for privacy-preserving Solana payments
//!
//! This crate implements EIP-5564 adapted for Solana, providing stealth address
//! generation, scanning, and key management with optional post-quantum hybrid mode.

pub mod crypto;
pub mod error;
pub mod generator;
pub mod hybrid;
pub mod keypair;
pub mod network_monitor;
pub mod payment_queue;
pub mod qr;
pub mod scanner;
pub mod storage;
pub mod wallet_manager;

// Re-export main types
pub use crypto::StealthCrypto;
pub use error::{StealthError, StealthResult};
pub use generator::{StealthAddressGenerator, StealthAddressOutput};
pub use hybrid::{HybridStealthAddressOutput, HybridStealthKeyPair};
pub use keypair::StealthKeyPair;
pub use network_monitor::NetworkMonitor;
pub use payment_queue::{PaymentQueue, PaymentStatus, QueuedPayment};
pub use qr::QrCodeHandler;
pub use scanner::{DetectedPayment, StealthScanner};
pub use wallet_manager::{PreparedPayment, StealthWalletManager};
