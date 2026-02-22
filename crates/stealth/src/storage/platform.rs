//! Secure storage module for stealth keys
//!
//! This module provides platform-agnostic and platform-specific storage implementations
//! for stealth key pairs. All implementations provide encryption at rest and key separation.

mod ios;
mod android;

#[cfg(test)]
mod platform_tests;

pub use ios::IOSKeychainStorage;
pub use android::AndroidKeystoreStorage;

// Re-export the SecureStorage trait from parent module
pub use super::SecureStorage;
