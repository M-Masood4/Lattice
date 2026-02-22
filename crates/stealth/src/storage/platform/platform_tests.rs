//! Unit tests for platform-specific storage implementations
//!
//! These tests verify:
//! - Key storage and retrieval (Requirement 9.1)
//! - Encryption at rest (Requirement 9.2)
//! - Key separation (Requirement 9.3)

#[cfg(test)]
mod tests {
    use crate::keypair::StealthKeyPair;
    use crate::storage::{InMemoryStorage, SecureStorage};

    /// Test: Key storage and retrieval
    /// 
    /// Validates: Requirement 9.1 (platform-specific storage)
    #[tokio::test]
    async fn test_store_and_retrieve_keypair() {
        let storage = InMemoryStorage::new(b"test-device-key-12345");
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let original_meta = keypair.to_meta_address();

        // Store keypair
        storage.store_keypair("test-wallet-1", &keypair).await.unwrap();

        // Retrieve keypair
        let loaded = storage.load_keypair("test-wallet-1").await.unwrap();
        let loaded_meta = loaded.to_meta_address();

        // Verify meta-addresses match (public keys are preserved)
        assert_eq!(
            original_meta, loaded_meta,
            "Stored and loaded keypairs should have identical meta-addresses"
        );
    }

    /// Test: Encryption at rest
    /// 
    /// Validates: Requirement 9.2 (encryption at rest using AES-256-GCM)
    #[tokio::test]
    async fn test_keys_encrypted_at_rest() {
        let storage = InMemoryStorage::new(b"test-device-key-67890");
        let keypair = StealthKeyPair::generate_standard().unwrap();

        // Store keypair
        storage.store_keypair("test-wallet-2", &keypair).await.unwrap();

        // Access internal storage to verify encryption
        let keypairs = storage.keypairs.read().await;
        let encrypted = keypairs.get("test-wallet-2").unwrap();

        // Verify encrypted data is longer than plaintext (includes auth tag)
        assert!(
            encrypted.spending_secret_encrypted.len() > 32,
            "Encrypted spending key should be longer than 32 bytes (plaintext + auth tag)"
        );
        assert!(
            encrypted.viewing_secret_encrypted.len() > 32,
            "Encrypted viewing key should be longer than 32 bytes (plaintext + auth tag)"
        );

        // Verify the encrypted data doesn't contain plaintext secret keys
        let spending_secret = keypair.spending_keypair().secret.to_bytes();
        let viewing_secret = keypair.viewing_keypair().secret.to_bytes();

        assert!(
            !encrypted.spending_secret_encrypted.windows(32).any(|w| w == spending_secret),
            "Encrypted spending key should not contain plaintext secret"
        );
        assert!(
            !encrypted.viewing_secret_encrypted.windows(32).any(|w| w == viewing_secret),
            "Encrypted viewing key should not contain plaintext secret"
        );
    }

    /// Test: Key separation (spending and viewing keys stored separately)
    /// 
    /// Validates: Requirement 9.3 (separate storage for spending and viewing keys)
    #[tokio::test]
    async fn test_key_separation() {
        let storage = InMemoryStorage::new(b"test-device-key-separation");
        let keypair = StealthKeyPair::generate_standard().unwrap();

        // Store keypair
        storage.store_keypair("test-wallet-3", &keypair).await.unwrap();

        // Access internal storage
        let keypairs = storage.keypairs.read().await;
        let encrypted = keypairs.get("test-wallet-3").unwrap();

        // Verify spending and viewing keys use different nonces
        assert_ne!(
            encrypted.spending_nonce, encrypted.viewing_nonce,
            "Spending and viewing keys should use different nonces"
        );

        // Verify spending and viewing keys have different ciphertexts
        assert_ne!(
            encrypted.spending_secret_encrypted, encrypted.viewing_secret_encrypted,
            "Spending and viewing keys should have different encrypted representations"
        );

        // Verify public keys are stored separately
        assert_ne!(
            encrypted.spending_public, encrypted.viewing_public,
            "Spending and viewing public keys should be different"
        );
    }

    /// Test: Multiple keypairs can be stored independently
    /// 
    /// Validates: Requirement 9.1 (storage of multiple keypairs)
    #[tokio::test]
    async fn test_store_multiple_keypairs() {
        let storage = InMemoryStorage::new(b"test-device-key-multi");
        let keypair1 = StealthKeyPair::generate_standard().unwrap();
        let keypair2 = StealthKeyPair::generate_standard().unwrap();
        let keypair3 = StealthKeyPair::generate_standard().unwrap();

        // Store multiple keypairs
        storage.store_keypair("wallet-1", &keypair1).await.unwrap();
        storage.store_keypair("wallet-2", &keypair2).await.unwrap();
        storage.store_keypair("wallet-3", &keypair3).await.unwrap();

        // Retrieve and verify each keypair
        let loaded1 = storage.load_keypair("wallet-1").await.unwrap();
        let loaded2 = storage.load_keypair("wallet-2").await.unwrap();
        let loaded3 = storage.load_keypair("wallet-3").await.unwrap();

        assert_eq!(keypair1.to_meta_address(), loaded1.to_meta_address());
        assert_eq!(keypair2.to_meta_address(), loaded2.to_meta_address());
        assert_eq!(keypair3.to_meta_address(), loaded3.to_meta_address());

        // Verify they're all different
        assert_ne!(loaded1.to_meta_address(), loaded2.to_meta_address());
        assert_ne!(loaded2.to_meta_address(), loaded3.to_meta_address());
        assert_ne!(loaded1.to_meta_address(), loaded3.to_meta_address());
    }

    /// Test: List all stored keypairs
    /// 
    /// Validates: Requirement 9.1 (keypair enumeration)
    #[tokio::test]
    async fn test_list_keypairs() {
        let storage = InMemoryStorage::new(b"test-device-key-list");
        let keypair1 = StealthKeyPair::generate_standard().unwrap();
        let keypair2 = StealthKeyPair::generate_standard().unwrap();

        // Initially empty
        let ids = storage.list_keypairs().await.unwrap();
        assert_eq!(ids.len(), 0, "Storage should be empty initially");

        // Store keypairs
        storage.store_keypair("wallet-alpha", &keypair1).await.unwrap();
        storage.store_keypair("wallet-beta", &keypair2).await.unwrap();

        // List keypairs
        let mut ids = storage.list_keypairs().await.unwrap();
        ids.sort();

        assert_eq!(ids.len(), 2, "Should have 2 stored keypairs");
        assert_eq!(ids, vec!["wallet-alpha", "wallet-beta"]);
    }

    /// Test: Delete keypair
    /// 
    /// Validates: Requirement 9.1 (keypair deletion)
    #[tokio::test]
    async fn test_delete_keypair() {
        let storage = InMemoryStorage::new(b"test-device-key-delete");
        let keypair = StealthKeyPair::generate_standard().unwrap();

        // Store keypair
        storage.store_keypair("wallet-to-delete", &keypair).await.unwrap();

        // Verify it exists
        assert!(storage.load_keypair("wallet-to-delete").await.is_ok());

        // Delete keypair
        storage.delete_keypair("wallet-to-delete").await.unwrap();

        // Verify it's gone
        let result = storage.load_keypair("wallet-to-delete").await;
        assert!(result.is_err(), "Deleted keypair should not be loadable");
    }

    /// Test: Load nonexistent keypair fails
    /// 
    /// Validates: Requirement 9.1 (error handling)
    #[tokio::test]
    async fn test_load_nonexistent_keypair() {
        let storage = InMemoryStorage::new(b"test-device-key-nonexistent");

        let result = storage.load_keypair("does-not-exist").await;
        assert!(result.is_err(), "Loading nonexistent keypair should fail");
    }

    /// Test: Delete nonexistent keypair fails
    /// 
    /// Validates: Requirement 9.1 (error handling)
    #[tokio::test]
    async fn test_delete_nonexistent_keypair() {
        let storage = InMemoryStorage::new(b"test-device-key-delete-fail");

        let result = storage.delete_keypair("does-not-exist").await;
        assert!(result.is_err(), "Deleting nonexistent keypair should fail");
    }

    /// Test: Different device keys produce different encryption
    /// 
    /// Validates: Requirement 9.2 (device-specific encryption)
    #[tokio::test]
    async fn test_device_key_affects_encryption() {
        let storage1 = InMemoryStorage::new(b"device-key-A");
        let storage2 = InMemoryStorage::new(b"device-key-B");
        let keypair = StealthKeyPair::generate_standard().unwrap();

        // Store same keypair with different device keys
        storage1.store_keypair("test-wallet", &keypair).await.unwrap();
        storage2.store_keypair("test-wallet", &keypair).await.unwrap();

        // Access internal storage
        let keypairs1 = storage1.keypairs.read().await;
        let keypairs2 = storage2.keypairs.read().await;
        let encrypted1 = keypairs1.get("test-wallet").unwrap();
        let encrypted2 = keypairs2.get("test-wallet").unwrap();

        // Verify encrypted data is different (different device keys)
        assert_ne!(
            encrypted1.spending_secret_encrypted, encrypted2.spending_secret_encrypted,
            "Different device keys should produce different encrypted spending keys"
        );
        assert_ne!(
            encrypted1.viewing_secret_encrypted, encrypted2.viewing_secret_encrypted,
            "Different device keys should produce different encrypted viewing keys"
        );
    }

    /// Test: Store and load arbitrary data
    /// 
    /// Validates: Requirement 9.1 (arbitrary data storage)
    #[tokio::test]
    async fn test_store_and_load_data() {
        let storage = InMemoryStorage::new(b"test-device-key-data");
        let data = b"sensitive payment queue data";

        // Store data
        storage.store_data("payment-queue", data).await.unwrap();

        // Load data
        let loaded = storage.load_data("payment-queue").await.unwrap();

        assert_eq!(data.as_slice(), loaded.as_slice(), "Loaded data should match stored data");
    }

    /// Test: Arbitrary data is encrypted at rest
    /// 
    /// Validates: Requirement 9.2 (encryption of arbitrary data)
    #[tokio::test]
    async fn test_arbitrary_data_encrypted() {
        let storage = InMemoryStorage::new(b"test-device-key-data-enc");
        let data = b"secret information that must be encrypted";

        // Store data
        storage.store_data("secret-key", data).await.unwrap();

        // Access internal storage
        let data_map = storage.data.read().await;
        let stored = data_map.get("secret-key").unwrap();

        // Verify stored data is longer (nonce + ciphertext + auth tag)
        assert!(
            stored.len() > data.len(),
            "Encrypted data should be longer than plaintext"
        );

        // Verify it's not plaintext
        assert!(
            !stored.windows(data.len()).any(|w| w == data),
            "Stored data should not contain plaintext"
        );
    }

    /// Test: Overwriting keypair updates storage
    /// 
    /// Validates: Requirement 9.1 (keypair updates)
    #[tokio::test]
    async fn test_overwrite_keypair() {
        let storage = InMemoryStorage::new(b"test-device-key-overwrite");
        let keypair1 = StealthKeyPair::generate_standard().unwrap();
        let keypair2 = StealthKeyPair::generate_standard().unwrap();

        // Store first keypair
        storage.store_keypair("wallet-id", &keypair1).await.unwrap();
        let loaded1 = storage.load_keypair("wallet-id").await.unwrap();
        assert_eq!(keypair1.to_meta_address(), loaded1.to_meta_address());

        // Overwrite with second keypair
        storage.store_keypair("wallet-id", &keypair2).await.unwrap();
        let loaded2 = storage.load_keypair("wallet-id").await.unwrap();
        assert_eq!(keypair2.to_meta_address(), loaded2.to_meta_address());

        // Verify it's the second keypair, not the first
        assert_ne!(loaded1.to_meta_address(), loaded2.to_meta_address());
    }

    /// Test: Storage isolation between different IDs
    /// 
    /// Validates: Requirement 9.1 (storage isolation)
    #[tokio::test]
    async fn test_storage_isolation() {
        let storage = InMemoryStorage::new(b"test-device-key-isolation");
        let keypair1 = StealthKeyPair::generate_standard().unwrap();
        let keypair2 = StealthKeyPair::generate_standard().unwrap();

        // Store keypairs with different IDs
        storage.store_keypair("user-alice", &keypair1).await.unwrap();
        storage.store_keypair("user-bob", &keypair2).await.unwrap();

        // Delete one keypair
        storage.delete_keypair("user-alice").await.unwrap();

        // Verify alice's keypair is gone but bob's remains
        assert!(storage.load_keypair("user-alice").await.is_err());
        assert!(storage.load_keypair("user-bob").await.is_ok());

        let loaded_bob = storage.load_keypair("user-bob").await.unwrap();
        assert_eq!(keypair2.to_meta_address(), loaded_bob.to_meta_address());
    }

    /// Test: Empty storage list returns empty vector
    /// 
    /// Validates: Requirement 9.1 (empty storage handling)
    #[tokio::test]
    async fn test_list_empty_storage() {
        let storage = InMemoryStorage::new(b"test-device-key-empty");

        let ids = storage.list_keypairs().await.unwrap();
        assert_eq!(ids.len(), 0, "Empty storage should return empty list");
    }

    /// Test: Concurrent access to storage
    /// 
    /// Validates: Requirement 9.1 (thread safety)
    #[tokio::test]
    async fn test_concurrent_storage_access() {
        use std::sync::Arc;

        let storage = Arc::new(InMemoryStorage::new(b"test-device-key-concurrent"));
        let keypair1 = StealthKeyPair::generate_standard().unwrap();
        let keypair2 = StealthKeyPair::generate_standard().unwrap();
        
        // Store the meta-addresses for later verification
        let meta1 = keypair1.to_meta_address();
        let meta2 = keypair2.to_meta_address();

        // Spawn concurrent tasks
        let storage1 = storage.clone();
        let task1 = tokio::spawn(async move {
            storage1.store_keypair("concurrent-1", &keypair1).await.unwrap();
        });

        let storage2 = storage.clone();
        let task2 = tokio::spawn(async move {
            storage2.store_keypair("concurrent-2", &keypair2).await.unwrap();
        });

        // Wait for both tasks
        task1.await.unwrap();
        task2.await.unwrap();

        // Verify both keypairs were stored
        let loaded1 = storage.load_keypair("concurrent-1").await.unwrap();
        let loaded2 = storage.load_keypair("concurrent-2").await.unwrap();

        assert_eq!(meta1, loaded1.to_meta_address());
        assert_eq!(meta2, loaded2.to_meta_address());
    }

    /// Test: Verify spending and viewing keys can be loaded independently
    /// 
    /// This test verifies the key separation requirement by ensuring that
    /// spending and viewing keys are truly stored separately and can be
    /// accessed independently (important for view-only wallet modes).
    /// 
    /// Validates: Requirement 9.3 (key separation for view-only wallets)
    #[tokio::test]
    async fn test_independent_key_access() {
        let storage = InMemoryStorage::new(b"test-device-key-independent");
        let keypair = StealthKeyPair::generate_standard().unwrap();

        // Store keypair
        storage.store_keypair("test-wallet", &keypair).await.unwrap();

        // Access internal storage to verify keys are stored separately
        let keypairs = storage.keypairs.read().await;
        let encrypted = keypairs.get("test-wallet").unwrap();

        // Verify we have separate encrypted blobs for spending and viewing
        assert!(
            encrypted.spending_secret_encrypted.len() > 0,
            "Spending key should be stored"
        );
        assert!(
            encrypted.viewing_secret_encrypted.len() > 0,
            "Viewing key should be stored"
        );

        // Verify they're truly separate (different encrypted data)
        assert_ne!(
            encrypted.spending_secret_encrypted, encrypted.viewing_secret_encrypted,
            "Spending and viewing keys should be stored as separate encrypted blobs"
        );
    }
}
