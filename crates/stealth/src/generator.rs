//! Stealth address generation (sender-side)

use crate::crypto::StealthCrypto;
use crate::error::{StealthError, StealthResult};
use crate::keypair::StealthKeyPair;
use curve25519_dalek::scalar::Scalar;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use lru::LruCache;
use solana_sdk::pubkey::Pubkey;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Cache capacity for stealth address derivations
const CACHE_CAPACITY: usize = 1000;

/// Cache key for stealth address lookups
#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    meta_address: String,
    ephemeral_public_key: [u8; 32],
}

/// Stealth address generator for senders with LRU caching
pub struct StealthAddressGenerator {
    /// LRU cache for derived stealth addresses
    /// Cache based on (meta_address, ephemeral_key) tuple
    cache: Arc<Mutex<LruCache<CacheKey, StealthAddressOutput>>>,
}

impl StealthAddressGenerator {
    /// Create a new stealth address generator with caching
    ///
    /// # Requirements
    /// Validates: Requirements 12.4
    pub fn new() -> Self {
        let capacity = NonZeroUsize::new(CACHE_CAPACITY).unwrap();
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }

    /// Generate stealth address from receiver's meta-address
    /// 
    /// This implements the sender-side stealth address derivation using ECDH.
    /// The process:
    /// 1. Parse the receiver's meta-address to get spending and viewing public keys
    /// 2. Generate (or use provided) ephemeral key pair
    /// 3. Check cache for existing derivation
    /// 4. Compute shared secret using ECDH(ephemeral_secret, viewing_public)
    /// 5. Derive stealth address = spending_public + hash(shared_secret) * G
    /// 6. Compute viewing tag for efficient scanning
    /// 7. Cache the result for future lookups
    /// 
    /// # Requirements
    /// Validates: Requirements 2.3, 2.4, 2.5, 2.6, 2.8, 12.4
    pub async fn generate_stealth_address(
        &self,
        meta_address: &str,
        ephemeral_keypair: Option<Keypair>,
    ) -> StealthResult<StealthAddressOutput> {
        // Generate ephemeral key pair if not provided (Requirement 2.5)
        let ephemeral_kp = match ephemeral_keypair {
            Some(kp) => kp,
            None => {
                // Generate random ephemeral key pair
                let mut secret_bytes = [0u8; 32];
                use rand::RngCore;
                rand::thread_rng().fill_bytes(&mut secret_bytes);
                
                let secret = SecretKey::from_bytes(&secret_bytes)
                    .map_err(|e| StealthError::KeyDerivationFailed(format!("Failed to create ephemeral secret: {}", e)))?;
                let public: PublicKey = (&secret).into();
                
                Keypair { secret, public }
            }
        };

        // Create cache key (Requirement 12.4)
        let cache_key = CacheKey {
            meta_address: meta_address.to_string(),
            ephemeral_public_key: ephemeral_kp.public.to_bytes(),
        };

        // Check cache first (Requirement 12.4)
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached_output) = cache.get(&cache_key) {
                return Ok(cached_output.clone());
            }
        }

        // Parse the meta-address to get receiver's public keys
        let receiver_keys = StealthKeyPair::from_meta_address(meta_address)?;
        let spending_public = receiver_keys.spending_public_key();
        let viewing_public = receiver_keys.viewing_public_key();
        
        // Convert viewing public key to Curve25519 for ECDH
        let viewing_public_bytes = viewing_public.to_bytes();
        let viewing_curve25519 = StealthCrypto::ed25519_to_curve25519(&viewing_public_bytes)?;
        
        // Convert ephemeral secret key to Curve25519 for ECDH
        let ephemeral_secret_bytes = ephemeral_kp.secret.to_bytes();
        
        // Compute shared secret using ECDH (Requirement 2.3, 2.4)
        let shared_secret = StealthCrypto::ecdh(&ephemeral_secret_bytes, &viewing_curve25519)?;
        
        // Derive viewing tag from shared secret (Requirement 2.8)
        let viewing_tag = StealthCrypto::derive_viewing_tag(&shared_secret);
        
        // Derive stealth address using point addition
        // stealth_address = spending_public + hash(shared_secret) * G
        
        // Hash the shared secret to get a scalar
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        let hash = hasher.finalize();
        let scalar = Scalar::from_bytes_mod_order(*hash.as_ref());
        
        // Compute hash(shared_secret) * G (basepoint)
        use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
        let offset_point = scalar * ED25519_BASEPOINT_POINT;
        let offset_compressed = offset_point.compress().to_bytes();
        
        // Add spending_public + offset_point to get stealth address
        let spending_public_bytes = spending_public.to_bytes();
        let stealth_address_bytes = StealthCrypto::point_add(&spending_public_bytes, &offset_compressed)?;
        
        // Convert to Solana Pubkey
        let stealth_address = Pubkey::new_from_array(stealth_address_bytes);
        
        // Get ephemeral public key as Solana Pubkey (Requirement 2.6)
        let ephemeral_public_key = Pubkey::new_from_array(ephemeral_kp.public.to_bytes());
        
        let output = StealthAddressOutput {
            stealth_address,
            ephemeral_public_key,
            viewing_tag,
            shared_secret,
        };

        // Cache the result (Requirement 12.4)
        {
            let mut cache = self.cache.lock().await;
            cache.put(cache_key, output.clone());
        }

        Ok(output)
    }

    /// Generate stealth address with hybrid mode
    ///
    /// This is a convenience method that delegates to HybridStealthKeyPair.
    /// For full hybrid functionality, use HybridStealthKeyPair directly.
    pub fn generate_hybrid_stealth_address(
        meta_address: &str,
        ephemeral_keypair: Option<Keypair>,
    ) -> StealthResult<crate::hybrid::HybridStealthAddressOutput> {
        // Parse the hybrid meta-address
        let hybrid_keypair = crate::hybrid::HybridStealthKeyPair::from_meta_address(meta_address)?;
        
        // Generate hybrid stealth address
        hybrid_keypair.generate_stealth_address(ephemeral_keypair)
    }

    /// Static convenience method for generating stealth addresses without caching
    ///
    /// This method is provided for backward compatibility and simple use cases
    /// where caching is not needed. For better performance with repeated derivations,
    /// create a StealthAddressGenerator instance and use the async method.
    pub fn generate_stealth_address_uncached(
        meta_address: &str,
        ephemeral_keypair: Option<Keypair>,
    ) -> StealthResult<StealthAddressOutput> {
        // Parse the meta-address to get receiver's public keys
        let receiver_keys = StealthKeyPair::from_meta_address(meta_address)?;
        let spending_public = receiver_keys.spending_public_key();
        let viewing_public = receiver_keys.viewing_public_key();
        
        // Generate ephemeral key pair if not provided (Requirement 2.5)
        let ephemeral_kp = match ephemeral_keypair {
            Some(kp) => kp,
            None => {
                // Generate random ephemeral key pair
                let mut secret_bytes = [0u8; 32];
                use rand::RngCore;
                rand::thread_rng().fill_bytes(&mut secret_bytes);
                
                let secret = SecretKey::from_bytes(&secret_bytes)
                    .map_err(|e| StealthError::KeyDerivationFailed(format!("Failed to create ephemeral secret: {}", e)))?;
                let public: PublicKey = (&secret).into();
                
                Keypair { secret, public }
            }
        };
        
        // Convert viewing public key to Curve25519 for ECDH
        let viewing_public_bytes = viewing_public.to_bytes();
        let viewing_curve25519 = StealthCrypto::ed25519_to_curve25519(&viewing_public_bytes)?;
        
        // Convert ephemeral secret key to Curve25519 for ECDH
        let ephemeral_secret_bytes = ephemeral_kp.secret.to_bytes();
        
        // Compute shared secret using ECDH (Requirement 2.3, 2.4)
        let shared_secret = StealthCrypto::ecdh(&ephemeral_secret_bytes, &viewing_curve25519)?;
        
        // Derive viewing tag from shared secret (Requirement 2.8)
        let viewing_tag = StealthCrypto::derive_viewing_tag(&shared_secret);
        
        // Derive stealth address using point addition
        // stealth_address = spending_public + hash(shared_secret) * G
        
        // Hash the shared secret to get a scalar
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&shared_secret);
        let hash = hasher.finalize();
        let scalar = Scalar::from_bytes_mod_order(*hash.as_ref());
        
        // Compute hash(shared_secret) * G (basepoint)
        use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
        let offset_point = scalar * ED25519_BASEPOINT_POINT;
        let offset_compressed = offset_point.compress().to_bytes();
        
        // Add spending_public + offset_point to get stealth address
        let spending_public_bytes = spending_public.to_bytes();
        let stealth_address_bytes = StealthCrypto::point_add(&spending_public_bytes, &offset_compressed)?;
        
        // Convert to Solana Pubkey
        let stealth_address = Pubkey::new_from_array(stealth_address_bytes);
        
        // Get ephemeral public key as Solana Pubkey (Requirement 2.6)
        let ephemeral_public_key = Pubkey::new_from_array(ephemeral_kp.public.to_bytes());
        
        Ok(StealthAddressOutput {
            stealth_address,
            ephemeral_public_key,
            viewing_tag,
            shared_secret,
        })
    }
}

/// Output from stealth address generation
#[derive(Clone)]
pub struct StealthAddressOutput {
    pub stealth_address: Pubkey,
    pub ephemeral_public_key: Pubkey,
    pub viewing_tag: [u8; 4],
    pub shared_secret: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair::StealthKeyPair;

    #[test]
    fn test_generate_stealth_address_basic() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        
        // Generate stealth address
        let result = StealthAddressGenerator::generate_stealth_address_uncached(&meta_address, None);
        assert!(result.is_ok(), "Stealth address generation should succeed");
        
        let output = result.unwrap();
        
        // Verify all fields are populated
        assert_ne!(output.stealth_address, Pubkey::default(), "Stealth address should not be default");
        assert_ne!(output.ephemeral_public_key, Pubkey::default(), "Ephemeral public key should not be default");
        assert_ne!(output.viewing_tag, [0u8; 4], "Viewing tag should not be all zeros");
        assert_ne!(output.shared_secret, [0u8; 32], "Shared secret should not be all zeros");
    }

    #[test]
    fn test_generate_stealth_address_uniqueness() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        
        // Generate two stealth addresses for the same receiver
        let output1 = StealthAddressGenerator::generate_stealth_address_uncached(&meta_address, None).unwrap();
        let output2 = StealthAddressGenerator::generate_stealth_address_uncached(&meta_address, None).unwrap();
        
        // They should be different (unique ephemeral keys)
        assert_ne!(
            output1.stealth_address, output2.stealth_address,
            "Each stealth address should be unique"
        );
        assert_ne!(
            output1.ephemeral_public_key, output2.ephemeral_public_key,
            "Each ephemeral key should be unique"
        );
        assert_ne!(
            output1.viewing_tag, output2.viewing_tag,
            "Viewing tags should differ with different ephemeral keys"
        );
    }

    #[test]
    fn test_generate_stealth_address_with_provided_ephemeral() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        
        // Generate ephemeral key pair
        let mut secret_bytes = [42u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        
        let secret = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public: PublicKey = (&secret).into();
        let ephemeral_kp = Keypair { secret, public };
        let ephemeral_pk = Pubkey::new_from_array(ephemeral_kp.public.to_bytes());
        
        // Generate stealth address with provided ephemeral key
        let output = StealthAddressGenerator::generate_stealth_address_uncached(
            &meta_address,
            Some(ephemeral_kp),
        ).unwrap();
        
        // Verify the ephemeral public key matches
        assert_eq!(
            output.ephemeral_public_key, ephemeral_pk,
            "Should use provided ephemeral key"
        );
    }

    #[test]
    fn test_generate_stealth_address_deterministic_with_same_ephemeral() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        
        // Generate ephemeral key pair
        let secret_bytes = [99u8; 32];
        let secret1 = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public1: PublicKey = (&secret1).into();
        let ephemeral_kp1 = Keypair { secret: secret1, public: public1 };
        
        let secret2 = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public2: PublicKey = (&secret2).into();
        let ephemeral_kp2 = Keypair { secret: secret2, public: public2 };
        
        // Generate stealth addresses with same ephemeral key
        let output1 = StealthAddressGenerator::generate_stealth_address_uncached(
            &meta_address,
            Some(ephemeral_kp1),
        ).unwrap();
        
        let output2 = StealthAddressGenerator::generate_stealth_address_uncached(
            &meta_address,
            Some(ephemeral_kp2),
        ).unwrap();
        
        // They should be identical (same inputs)
        assert_eq!(
            output1.stealth_address, output2.stealth_address,
            "Same ephemeral key should produce same stealth address"
        );
        assert_eq!(
            output1.viewing_tag, output2.viewing_tag,
            "Same ephemeral key should produce same viewing tag"
        );
    }

    #[test]
    fn test_generate_stealth_address_invalid_meta_address() {
        let invalid_cases = vec![
            "invalid",
            "stealth:1:invalid:invalid",
            "",
            "stealth:2:key1:key2", // Wrong version
        ];
        
        for invalid in invalid_cases {
            let result = StealthAddressGenerator::generate_stealth_address_uncached(invalid, None);
            assert!(result.is_err(), "Should reject invalid meta-address: {}", invalid);
        }
    }

    #[test]
    fn test_stealth_address_differs_from_spending_key() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        let spending_pk = receiver.spending_public_key();
        
        // Generate stealth address
        let output = StealthAddressGenerator::generate_stealth_address_uncached(&meta_address, None).unwrap();
        
        // Stealth address should be different from spending public key
        assert_ne!(
            output.stealth_address, spending_pk,
            "Stealth address should differ from spending public key"
        );
    }

    #[test]
    fn test_viewing_tag_is_first_4_bytes_of_hash() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        
        // Generate stealth address
        let output = StealthAddressGenerator::generate_stealth_address_uncached(&meta_address, None).unwrap();
        
        // Verify viewing tag is derived correctly from shared secret
        let expected_tag = StealthCrypto::derive_viewing_tag(&output.shared_secret);
        assert_eq!(
            output.viewing_tag, expected_tag,
            "Viewing tag should match hash of shared secret"
        );
    }

    #[tokio::test]
    async fn test_caching_same_inputs() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        
        // Create generator with cache
        let generator = StealthAddressGenerator::new();
        
        // Generate ephemeral key pair
        let secret_bytes = [123u8; 32];
        let secret1 = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public1: PublicKey = (&secret1).into();
        let ephemeral_kp1 = Keypair { secret: secret1, public: public1 };
        
        let secret2 = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public2: PublicKey = (&secret2).into();
        let ephemeral_kp2 = Keypair { secret: secret2, public: public2 };
        
        // Generate stealth addresses with same inputs
        let output1 = generator.generate_stealth_address(&meta_address, Some(ephemeral_kp1)).await.unwrap();
        let output2 = generator.generate_stealth_address(&meta_address, Some(ephemeral_kp2)).await.unwrap();
        
        // They should be identical (cached)
        assert_eq!(
            output1.stealth_address, output2.stealth_address,
            "Cached result should match"
        );
        assert_eq!(
            output1.viewing_tag, output2.viewing_tag,
            "Cached viewing tag should match"
        );
    }

    #[tokio::test]
    async fn test_caching_different_inputs() {
        // Generate a receiver's key pair
        let receiver = StealthKeyPair::generate_standard().unwrap();
        let meta_address = receiver.to_meta_address();
        
        // Create generator with cache
        let generator = StealthAddressGenerator::new();
        
        // Generate two different stealth addresses
        let output1 = generator.generate_stealth_address(&meta_address, None).await.unwrap();
        let output2 = generator.generate_stealth_address(&meta_address, None).await.unwrap();
        
        // They should be different (different ephemeral keys)
        assert_ne!(
            output1.stealth_address, output2.stealth_address,
            "Different ephemeral keys should produce different addresses"
        );
    }
}
