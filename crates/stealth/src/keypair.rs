//! Stealth key pair management and meta-address generation

use crate::error::{StealthError, StealthResult};
use chacha20poly1305::{
    aead::{Aead, NewAead},
    Key, XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use sha2::{Digest, Sha256};
use solana_sdk::pubkey::Pubkey;
use zeroize::Zeroize;

/// Stealth key pair with spending and viewing keys
/// 
/// This struct manages the two key pairs required for stealth addresses:
/// - Spending key pair: Used to derive private keys for spending received funds
/// - Viewing key pair: Used to scan the blockchain for incoming payments without spending capability
/// 
/// # Requirements
/// Validates: Requirements 2.1, 2.2, 9.5
#[derive(Debug)]
pub struct StealthKeyPair {
    spending_keypair: Keypair,
    viewing_keypair: Keypair,
    version: u8,
}

impl StealthKeyPair {
    /// Generate new stealth key pair (version 1: standard)
    /// 
    /// Creates two independent Ed25519 key pairs for spending and viewing.
    /// Version 1 indicates standard mode (non-hybrid, no post-quantum).
    /// 
    /// # Requirements
    /// Validates: Requirements 2.1
    pub fn generate_standard() -> StealthResult<Self> {
        // Generate random bytes for secret keys
        let mut spending_secret_bytes = [0u8; 32];
        let mut viewing_secret_bytes = [0u8; 32];
        
        // Use rand 0.8 to fill the bytes
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut spending_secret_bytes);
        rand::thread_rng().fill_bytes(&mut viewing_secret_bytes);
        
        // Create secret keys from bytes
        let spending_secret = SecretKey::from_bytes(&spending_secret_bytes)
            .map_err(|e| StealthError::KeyDerivationFailed(format!("Failed to create spending secret: {}", e)))?;
        let viewing_secret = SecretKey::from_bytes(&viewing_secret_bytes)
            .map_err(|e| StealthError::KeyDerivationFailed(format!("Failed to create viewing secret: {}", e)))?;
        
        // Derive public keys
        let spending_public: PublicKey = (&spending_secret).into();
        let viewing_public: PublicKey = (&viewing_secret).into();
        
        let spending_keypair = Keypair {
            secret: spending_secret,
            public: spending_public,
        };
        
        let viewing_keypair = Keypair {
            secret: viewing_secret,
            public: viewing_public,
        };
        
        Ok(Self {
            spending_keypair,
            viewing_keypair,
            version: 1,
        })
    }

    /// Generate meta-address string
    /// 
    /// Formats the meta-address as: `stealth:1:<spending_pk>:<viewing_pk>`
    /// where public keys are base58-encoded.
    /// 
    /// # Requirements
    /// Validates: Requirements 2.2
    pub fn to_meta_address(&self) -> String {
        let spending_pk = Pubkey::new_from_array(self.spending_keypair.public.to_bytes());
        let viewing_pk = Pubkey::new_from_array(self.viewing_keypair.public.to_bytes());
        
        format!(
            "stealth:{}:{}:{}",
            self.version,
            spending_pk.to_string(),
            viewing_pk.to_string()
        )
    }

    /// Parse meta-address string
    /// 
    /// Parses a meta-address in the format: `stealth:1:<spending_pk>:<viewing_pk>`
    /// Note: This only reconstructs the public keys. Private keys are not available.
    /// 
    /// # Requirements
    /// Validates: Requirements 2.2
    pub fn from_meta_address(meta_addr: &str) -> StealthResult<Self> {
        let parts: Vec<&str> = meta_addr.split(':').collect();
        
        if parts.len() != 4 {
            return Err(StealthError::InvalidMetaAddress(
                "Expected format: stealth:version:spending_pk:viewing_pk".into(),
            ));
        }
        
        if parts[0] != "stealth" {
            return Err(StealthError::InvalidMetaAddress(
                "Meta-address must start with 'stealth:'".into(),
            ));
        }
        
        let version: u8 = parts[1]
            .parse()
            .map_err(|_| StealthError::InvalidMetaAddress("Invalid version number".into()))?;
        
        if version != 1 {
            return Err(StealthError::InvalidMetaAddress(format!(
                "Unsupported version: {}. Expected version 1 for standard mode",
                version
            )));
        }
        
        let spending_pk = parts[2]
            .parse::<Pubkey>()
            .map_err(|e| StealthError::InvalidMetaAddress(format!("Invalid spending public key: {}", e)))?;
        
        let viewing_pk = parts[3]
            .parse::<Pubkey>()
            .map_err(|e| StealthError::InvalidMetaAddress(format!("Invalid viewing public key: {}", e)))?;
        
        // Create keypairs with only public keys (no secret keys available)
        // This is used for sender-side operations where only public keys are needed
        let spending_public = PublicKey::from_bytes(spending_pk.as_ref())
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid spending public key: {}", e)))?;
        
        let viewing_public = PublicKey::from_bytes(viewing_pk.as_ref())
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid viewing public key: {}", e)))?;
        
        // Create dummy secret keys (all zeros) since we only have public keys
        // This is safe because these keypairs will only be used for public key operations
        let spending_secret = SecretKey::from_bytes(&[0u8; 32])
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Failed to create spending keypair: {}", e)))?;
        
        let viewing_secret = SecretKey::from_bytes(&[0u8; 32])
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Failed to create viewing keypair: {}", e)))?;
        
        let spending_keypair = Keypair {
            secret: spending_secret,
            public: spending_public,
        };
        
        let viewing_keypair = Keypair {
            secret: viewing_secret,
            public: viewing_public,
        };
        
        Ok(Self {
            spending_keypair,
            viewing_keypair,
            version,
        })
    }

    /// Get spending public key
    /// 
    /// Returns the spending public key as a Solana Pubkey.
    pub fn spending_public_key(&self) -> Pubkey {
        Pubkey::new_from_array(self.spending_keypair.public.to_bytes())
    }
    /// Create a StealthKeyPair from existing keypairs (internal use)
    ///
    /// This is used by the storage layer to reconstruct keypairs from encrypted storage.
    ///
    /// # Arguments
    /// * `spending_keypair` - The spending keypair
    /// * `viewing_keypair` - The viewing keypair
    /// * `version` - The version number (1 = standard, 2 = hybrid)
    pub(crate) fn from_parts(
        spending_keypair: Keypair,
        viewing_keypair: Keypair,
        version: u8,
    ) -> StealthResult<Self> {
        // Validate version
        if version != 1 && version != 2 {
            return Err(StealthError::InvalidMetaAddress(
                format!("Unsupported version: {}", version)
            ));
        }

        Ok(Self {
            spending_keypair,
            viewing_keypair,
            version,
        })
    }


    /// Get viewing public key
    /// 
    /// Returns the viewing public key as a Solana Pubkey.
    pub fn viewing_public_key(&self) -> Pubkey {
        Pubkey::new_from_array(self.viewing_keypair.public.to_bytes())
    }
    
    /// Get spending secret key
    /// 
    /// Returns the spending secret key bytes for deriving stealth private keys.
    /// This is needed for unshield operations.
    /// 
    /// # Security
    /// This method exposes the spending secret key. Use with caution and ensure
    /// the returned bytes are zeroized after use.
    pub fn spending_secret_key(&self) -> [u8; 32] {
        self.spending_keypair.secret.to_bytes()
    }
    
    /// Get spending keypair (internal use)
    /// 
    /// Returns a reference to the spending keypair for internal operations.
    pub(crate) fn spending_keypair(&self) -> &Keypair {
        &self.spending_keypair
    }
    
    /// Get viewing keypair (internal use)
    /// 
    /// Returns a reference to the viewing keypair for internal operations.
    pub(crate) fn viewing_keypair(&self) -> &Keypair {
        &self.viewing_keypair
    }

    /// Export encrypted keys for backup
    /// 
    /// Encrypts the private keys using a password-derived key and returns
    /// the encrypted data. The format includes a salt, nonce, and ciphertext.
    /// 
    /// Format: [salt (32 bytes)][nonce (24 bytes)][ciphertext]
    /// 
    /// # Requirements
    /// Validates: Requirements 9.5
    pub fn export_encrypted(&self, password: &str) -> StealthResult<Vec<u8>> {
        use rand::Rng;
        
        // Generate random salt for key derivation
        let mut salt = [0u8; 32];
        rand::thread_rng().fill(&mut salt);
        
        // Derive encryption key from password using SHA256
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&salt);
        let key_bytes = hasher.finalize();
        let key = Key::from_slice(&key_bytes);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 24];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        
        // Serialize the keypairs
        let mut plaintext = Vec::new();
        plaintext.extend_from_slice(&self.spending_keypair.secret.to_bytes());
        plaintext.extend_from_slice(&self.spending_keypair.public.to_bytes());
        plaintext.extend_from_slice(&self.viewing_keypair.secret.to_bytes());
        plaintext.extend_from_slice(&self.viewing_keypair.public.to_bytes());
        plaintext.push(self.version);
        
        // Encrypt
        let cipher = XChaCha20Poly1305::new(key);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| StealthError::EncryptionFailed(format!("Failed to encrypt keys: {}", e)))?;
        
        // Zeroize plaintext
        plaintext.zeroize();
        
        // Combine salt, nonce, and ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Import from encrypted backup
    /// 
    /// Decrypts and reconstructs the key pair from encrypted backup data.
    /// 
    /// # Requirements
    /// Validates: Requirements 9.5
    pub fn import_encrypted(data: &[u8], password: &str) -> StealthResult<Self> {
        if data.len() < 32 + 24 {
            return Err(StealthError::DecryptionFailed(
                "Invalid encrypted data: too short".into(),
            ));
        }
        
        // Extract salt, nonce, and ciphertext
        let salt = &data[0..32];
        let nonce_bytes = &data[32..56];
        let ciphertext = &data[56..];
        
        // Derive decryption key from password
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let key_bytes = hasher.finalize();
        let key = Key::from_slice(&key_bytes);
        
        let nonce = XNonce::from_slice(nonce_bytes);
        
        // Decrypt
        let cipher = XChaCha20Poly1305::new(key);
        let mut plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| StealthError::DecryptionFailed(format!("Failed to decrypt keys: {}", e)))?;
        
        // Expected length: 32 (spending secret) + 32 (spending public) + 32 (viewing secret) + 32 (viewing public) + 1 (version)
        if plaintext.len() != 129 {
            plaintext.zeroize();
            return Err(StealthError::DecryptionFailed(
                "Invalid decrypted data length".into(),
            ));
        }
        
        // Extract components
        let spending_secret_bytes: [u8; 32] = plaintext[0..32]
            .try_into()
            .map_err(|_| StealthError::DecryptionFailed("Failed to extract spending secret".into()))?;
        
        let spending_public_bytes: [u8; 32] = plaintext[32..64]
            .try_into()
            .map_err(|_| StealthError::DecryptionFailed("Failed to extract spending public".into()))?;
        
        let viewing_secret_bytes: [u8; 32] = plaintext[64..96]
            .try_into()
            .map_err(|_| StealthError::DecryptionFailed("Failed to extract viewing secret".into()))?;
        
        let viewing_public_bytes: [u8; 32] = plaintext[96..128]
            .try_into()
            .map_err(|_| StealthError::DecryptionFailed("Failed to extract viewing public".into()))?;
        
        let version = plaintext[128];
        
        // Zeroize plaintext
        plaintext.zeroize();
        
        // Reconstruct keypairs
        let spending_secret = SecretKey::from_bytes(&spending_secret_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid spending secret key: {}", e)))?;
        
        let spending_public = PublicKey::from_bytes(&spending_public_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid spending public key: {}", e)))?;
        
        let viewing_secret = SecretKey::from_bytes(&viewing_secret_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid viewing secret key: {}", e)))?;
        
        let viewing_public = PublicKey::from_bytes(&viewing_public_bytes)
            .map_err(|e| StealthError::InvalidKeyFormat(format!("Invalid viewing public key: {}", e)))?;
        
        let spending_keypair = Keypair {
            secret: spending_secret,
            public: spending_public,
        };
        
        let viewing_keypair = Keypair {
            secret: viewing_secret,
            public: viewing_public,
        };
        
        Ok(Self {
            spending_keypair,
            viewing_keypair,
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_standard_creates_version_1() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        assert_eq!(keypair.version, 1, "Standard keypair should be version 1");
    }

    #[test]
    fn test_generate_standard_creates_different_keys() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        
        let spending_pk = keypair.spending_public_key();
        let viewing_pk = keypair.viewing_public_key();
        
        assert_ne!(
            spending_pk, viewing_pk,
            "Spending and viewing public keys should be different"
        );
    }

    #[test]
    fn test_generate_standard_creates_unique_keypairs() {
        let keypair1 = StealthKeyPair::generate_standard().unwrap();
        let keypair2 = StealthKeyPair::generate_standard().unwrap();
        
        assert_ne!(
            keypair1.spending_public_key(),
            keypair2.spending_public_key(),
            "Each generation should create unique spending keys"
        );
        
        assert_ne!(
            keypair1.viewing_public_key(),
            keypair2.viewing_public_key(),
            "Each generation should create unique viewing keys"
        );
    }

    #[test]
    fn test_to_meta_address_format() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let meta_addr = keypair.to_meta_address();
        
        // Check format: stealth:1:<spending_pk>:<viewing_pk>
        assert!(meta_addr.starts_with("stealth:1:"), "Meta-address should start with 'stealth:1:'");
        
        let parts: Vec<&str> = meta_addr.split(':').collect();
        assert_eq!(parts.len(), 4, "Meta-address should have 4 parts");
        assert_eq!(parts[0], "stealth", "First part should be 'stealth'");
        assert_eq!(parts[1], "1", "Second part should be version '1'");
        
        // Verify public keys are valid base58
        let spending_pk = parts[2].parse::<Pubkey>();
        let viewing_pk = parts[3].parse::<Pubkey>();
        assert!(spending_pk.is_ok(), "Spending public key should be valid base58");
        assert!(viewing_pk.is_ok(), "Viewing public key should be valid base58");
    }

    #[test]
    fn test_to_meta_address_contains_correct_keys() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let meta_addr = keypair.to_meta_address();
        
        let parts: Vec<&str> = meta_addr.split(':').collect();
        let spending_pk = parts[2].parse::<Pubkey>().unwrap();
        let viewing_pk = parts[3].parse::<Pubkey>().unwrap();
        
        assert_eq!(spending_pk, keypair.spending_public_key());
        assert_eq!(viewing_pk, keypair.viewing_public_key());
    }

    #[test]
    fn test_from_meta_address_valid() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let meta_addr = keypair.to_meta_address();
        
        let parsed = StealthKeyPair::from_meta_address(&meta_addr).unwrap();
        
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.spending_public_key(), keypair.spending_public_key());
        assert_eq!(parsed.viewing_public_key(), keypair.viewing_public_key());
    }

    #[test]
    fn test_from_meta_address_round_trip() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let meta_addr1 = keypair.to_meta_address();
        
        let parsed = StealthKeyPair::from_meta_address(&meta_addr1).unwrap();
        let meta_addr2 = parsed.to_meta_address();
        
        assert_eq!(meta_addr1, meta_addr2, "Round-trip should preserve meta-address");
    }

    #[test]
    fn test_from_meta_address_invalid_format() {
        let invalid_cases = vec![
            "invalid",
            "stealth:1",
            "stealth:1:key",
            "notstealth:1:key1:key2",
            "",
        ];
        
        for invalid in invalid_cases {
            let result = StealthKeyPair::from_meta_address(invalid);
            assert!(result.is_err(), "Should reject invalid format: {}", invalid);
        }
    }

    #[test]
    fn test_from_meta_address_invalid_version() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let meta_addr = keypair.to_meta_address();
        
        // Change version to 2
        let invalid = meta_addr.replace("stealth:1:", "stealth:2:");
        let result = StealthKeyPair::from_meta_address(&invalid);
        
        assert!(result.is_err(), "Should reject unsupported version");
    }

    #[test]
    fn test_from_meta_address_invalid_public_keys() {
        let invalid_cases = vec![
            "stealth:1:invalid_key:invalid_key",
            "stealth:1:not_base58!:not_base58!",
            "stealth:1:tooshort:tooshort",
        ];
        
        for invalid in invalid_cases {
            let result = StealthKeyPair::from_meta_address(invalid);
            assert!(result.is_err(), "Should reject invalid public keys: {}", invalid);
        }
    }

    #[test]
    fn test_spending_public_key_returns_correct_key() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let spending_pk = keypair.spending_public_key();
        
        let expected = Pubkey::new_from_array(keypair.spending_keypair.public.to_bytes());
        assert_eq!(spending_pk, expected);
    }

    #[test]
    fn test_viewing_public_key_returns_correct_key() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let viewing_pk = keypair.viewing_public_key();
        
        let expected = Pubkey::new_from_array(keypair.viewing_keypair.public.to_bytes());
        assert_eq!(viewing_pk, expected);
    }

    #[test]
    fn test_export_encrypted_produces_output() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let password = "test_password_123";
        
        let encrypted = keypair.export_encrypted(password).unwrap();
        
        // Should have at least salt (32) + nonce (24) + some ciphertext
        assert!(encrypted.len() > 56, "Encrypted data should contain salt, nonce, and ciphertext");
    }

    #[test]
    fn test_export_encrypted_different_each_time() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let password = "test_password_123";
        
        let encrypted1 = keypair.export_encrypted(password).unwrap();
        let encrypted2 = keypair.export_encrypted(password).unwrap();
        
        // Should be different due to random salt and nonce
        assert_ne!(encrypted1, encrypted2, "Each export should use different salt/nonce");
    }

    #[test]
    fn test_import_encrypted_round_trip() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let password = "test_password_123";
        
        let spending_pk = keypair.spending_public_key();
        let viewing_pk = keypair.viewing_public_key();
        
        let encrypted = keypair.export_encrypted(password).unwrap();
        let imported = StealthKeyPair::import_encrypted(&encrypted, password).unwrap();
        
        assert_eq!(imported.version, keypair.version);
        assert_eq!(imported.spending_public_key(), spending_pk);
        assert_eq!(imported.viewing_public_key(), viewing_pk);
    }

    #[test]
    fn test_import_encrypted_wrong_password_fails() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        
        let encrypted = keypair.export_encrypted(correct_password).unwrap();
        let result = StealthKeyPair::import_encrypted(&encrypted, wrong_password);
        
        assert!(result.is_err(), "Import with wrong password should fail");
    }

    #[test]
    fn test_import_encrypted_invalid_data() {
        let password = "test_password";
        
        // Too short
        let short_data = vec![0u8; 10];
        let result = StealthKeyPair::import_encrypted(&short_data, password);
        assert!(result.is_err(), "Should reject data that's too short");
        
        // Random data
        let random_data = vec![0xFFu8; 200];
        let result = StealthKeyPair::import_encrypted(&random_data, password);
        assert!(result.is_err(), "Should reject random data");
    }

    #[test]
    fn test_import_encrypted_tampered_data_fails() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let password = "test_password";
        
        let mut encrypted = keypair.export_encrypted(password).unwrap();
        
        // Tamper with the ciphertext (after salt and nonce)
        if encrypted.len() > 60 {
            encrypted[60] ^= 0xFF;
        }
        
        let result = StealthKeyPair::import_encrypted(&encrypted, password);
        assert!(result.is_err(), "Should reject tampered data");
    }

    #[test]
    fn test_export_import_preserves_meta_address() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        let password = "test_password_123";
        let original_meta = keypair.to_meta_address();
        
        let encrypted = keypair.export_encrypted(password).unwrap();
        let imported = StealthKeyPair::import_encrypted(&encrypted, password).unwrap();
        let imported_meta = imported.to_meta_address();
        
        assert_eq!(original_meta, imported_meta, "Meta-address should be preserved through export/import");
    }

    #[test]
    fn test_keypair_internal_accessors() {
        let keypair = StealthKeyPair::generate_standard().unwrap();
        
        let spending_kp = keypair.spending_keypair();
        let viewing_kp = keypair.viewing_keypair();
        
        assert_eq!(
            spending_kp.public.to_bytes(),
            keypair.spending_public_key().to_bytes()
        );
        assert_eq!(
            viewing_kp.public.to_bytes(),
            keypair.viewing_public_key().to_bytes()
        );
    }
}
