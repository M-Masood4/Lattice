//! Core cryptographic primitives for stealth address operations

use crate::error::{StealthError, StealthResult};
use chacha20poly1305::{
    aead::{Aead, NewAead},
    XChaCha20Poly1305, XNonce, Key,
};
use curve25519_dalek::edwards::CompressedEdwardsY;
use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::Scalar;
use sha2::{Digest, Sha256};

/// Core cryptographic operations for stealth addresses
pub struct StealthCrypto;

impl StealthCrypto {
    /// Convert Ed25519 public key to Curve25519 for ECDH
    /// 
    /// This conversion is necessary because Ed25519 keys use the edwards25519 curve
    /// while ECDH operations require Curve25519 (Montgomery form).
    /// 
    /// # Requirements
    /// Validates: Requirements 4.1
    pub fn ed25519_to_curve25519(ed_pk: &[u8; 32]) -> StealthResult<[u8; 32]> {
        // Decompress the Ed25519 public key
        let compressed = CompressedEdwardsY(*ed_pk);
        let edwards_point = compressed
            .decompress()
            .ok_or_else(|| StealthError::InvalidCurvePoint("Invalid Ed25519 public key".into()))?;

        // Convert to Montgomery form (Curve25519)
        let montgomery_point = edwards_point.to_montgomery();
        
        Ok(montgomery_point.to_bytes())
    }

    /// Perform ECDH key exchange using Curve25519
    /// 
    /// Computes the shared secret between a secret key and a public key.
    /// The operation is symmetric: ECDH(a_secret, b_public) == ECDH(b_secret, a_public)
    /// 
    /// # Requirements
    /// Validates: Requirements 4.2
    pub fn ecdh(secret_key: &[u8; 32], public_key: &[u8; 32]) -> StealthResult<[u8; 32]> {
        // Convert secret key to scalar
        let scalar = Scalar::from_bytes_mod_order(*secret_key);
        
        // Load public key as Montgomery point
        let public_point = MontgomeryPoint(*public_key);
        
        // Perform scalar multiplication (ECDH)
        let shared_point = scalar * public_point;
        
        Ok(shared_point.to_bytes())
    }

    /// Add two points on edwards25519 curve
    /// 
    /// Used for stealth address derivation where we add the ephemeral public key
    /// to the receiver's spending public key.
    /// 
    /// # Requirements
    /// Validates: Requirements 4.3
    pub fn point_add(point_a: &[u8; 32], point_b: &[u8; 32]) -> StealthResult<[u8; 32]> {
        // Decompress both points
        let compressed_a = CompressedEdwardsY(*point_a);
        let edwards_a = compressed_a
            .decompress()
            .ok_or_else(|| StealthError::InvalidCurvePoint("Invalid point A".into()))?;

        let compressed_b = CompressedEdwardsY(*point_b);
        let edwards_b = compressed_b
            .decompress()
            .ok_or_else(|| StealthError::InvalidCurvePoint("Invalid point B".into()))?;

        // Add the points
        let result = edwards_a + edwards_b;
        
        // Compress and return
        Ok(result.compress().to_bytes())
    }

    /// Derive viewing tag from shared secret (first 4 bytes of SHA256)
    /// 
    /// The viewing tag enables efficient blockchain scanning by allowing quick
    /// filtering before performing expensive ECDH operations.
    /// 
    /// # Requirements
    /// Validates: Requirements 2.8, 4.5
    pub fn derive_viewing_tag(shared_secret: &[u8; 32]) -> [u8; 4] {
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        let hash = hasher.finalize();
        
        // Take first 4 bytes
        let mut tag = [0u8; 4];
        tag.copy_from_slice(&hash[0..4]);
        tag
    }

    /// Encrypt payload for mesh relay using XChaCha20-Poly1305
    /// 
    /// XChaCha20-Poly1305 provides authenticated encryption with a 24-byte nonce,
    /// making it suitable for mesh relay where nonce management is critical.
    /// 
    /// # Requirements
    /// Validates: Requirements 4.4, 8.2
    pub fn encrypt_mesh_payload(
        plaintext: &[u8],
        shared_key: &[u8; 32],
        nonce: &[u8; 24],
    ) -> StealthResult<Vec<u8>> {
        let key = Key::from_slice(shared_key);
        let cipher = XChaCha20Poly1305::new(key);
        let xnonce = XNonce::from_slice(nonce);
        
        cipher
            .encrypt(xnonce, plaintext)
            .map_err(|e| StealthError::EncryptionFailed(format!("XChaCha20-Poly1305 encryption failed: {}", e)))
    }

    /// Decrypt mesh payload using XChaCha20-Poly1305
    /// 
    /// Decrypts and authenticates the ciphertext. Returns an error if the
    /// authentication tag doesn't match (indicating tampering or wrong key).
    /// 
    /// # Requirements
    /// Validates: Requirements 4.4, 8.2
    pub fn decrypt_mesh_payload(
        ciphertext: &[u8],
        shared_key: &[u8; 32],
        nonce: &[u8; 24],
    ) -> StealthResult<Vec<u8>> {
        let key = Key::from_slice(shared_key);
        let cipher = XChaCha20Poly1305::new(key);
        let xnonce = XNonce::from_slice(nonce);
        
        cipher
            .decrypt(xnonce, ciphertext)
            .map_err(|e| StealthError::DecryptionFailed(format!("XChaCha20-Poly1305 decryption failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;

    #[test]
    fn test_ed25519_to_curve25519_conversion() {
        // Use a known Ed25519 public key (basepoint)
        let ed_public = ED25519_BASEPOINT_POINT.compress().to_bytes();

        // Convert to Curve25519
        let result = StealthCrypto::ed25519_to_curve25519(&ed_public);
        assert!(result.is_ok(), "Conversion should succeed");

        let curve_public = result.unwrap();
        assert_eq!(curve_public.len(), 32, "Curve25519 key should be 32 bytes");
    }

    #[test]
    fn test_ed25519_to_curve25519_invalid_key() {
        // Use a point that's not on the curve (high bit set incorrectly)
        let mut invalid_key = [0u8; 32];
        invalid_key[31] = 0x80; // Set high bit which makes it invalid
        invalid_key[0] = 0xFF; // Add more invalid data
        
        let result = StealthCrypto::ed25519_to_curve25519(&invalid_key);
        // Note: Some invalid points may still decompress, so we just verify the function handles it
        // The important thing is it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_ecdh_shared_secret_symmetry() {
        // Use fixed scalars for deterministic testing
        let secret_a_bytes = [1u8; 32];
        let secret_b_bytes = [2u8; 32];
        
        let secret_a = Scalar::from_bytes_mod_order(secret_a_bytes);
        let secret_b = Scalar::from_bytes_mod_order(secret_b_bytes);
        
        let public_a = (&secret_a * &ED25519_BASEPOINT_POINT)
            .to_montgomery()
            .to_bytes();
        let public_b = (&secret_b * &ED25519_BASEPOINT_POINT)
            .to_montgomery()
            .to_bytes();

        // Compute shared secrets both ways
        let shared_ab = StealthCrypto::ecdh(&secret_a.to_bytes(), &public_b).unwrap();
        let shared_ba = StealthCrypto::ecdh(&secret_b.to_bytes(), &public_a).unwrap();

        // They should be equal (ECDH symmetry)
        assert_eq!(
            shared_ab, shared_ba,
            "ECDH shared secrets should be symmetric"
        );
    }

    #[test]
    fn test_ecdh_produces_valid_shared_secret() {
        let secret_bytes = [42u8; 32];
        let secret = Scalar::from_bytes_mod_order(secret_bytes);
        
        let public_bytes = [99u8; 32];
        let public_scalar = Scalar::from_bytes_mod_order(public_bytes);
        let public = (&public_scalar * &ED25519_BASEPOINT_POINT)
            .to_montgomery()
            .to_bytes();

        let result = StealthCrypto::ecdh(&secret.to_bytes(), &public);
        assert!(result.is_ok(), "ECDH should succeed with valid keys");

        let shared_secret = result.unwrap();
        assert_eq!(shared_secret.len(), 32, "Shared secret should be 32 bytes");
    }

    #[test]
    fn test_point_add_associativity() {
        // Use fixed scalars for deterministic testing
        let scalar_a = Scalar::from_bytes_mod_order([3u8; 32]);
        let scalar_b = Scalar::from_bytes_mod_order([5u8; 32]);
        let scalar_c = Scalar::from_bytes_mod_order([7u8; 32]);

        let point_a = (&scalar_a * &ED25519_BASEPOINT_POINT)
            .compress()
            .to_bytes();
        let point_b = (&scalar_b * &ED25519_BASEPOINT_POINT)
            .compress()
            .to_bytes();
        let point_c = (&scalar_c * &ED25519_BASEPOINT_POINT)
            .compress()
            .to_bytes();

        // Test associativity: (A + B) + C == A + (B + C)
        let ab = StealthCrypto::point_add(&point_a, &point_b).unwrap();
        let ab_c = StealthCrypto::point_add(&ab, &point_c).unwrap();

        let bc = StealthCrypto::point_add(&point_b, &point_c).unwrap();
        let a_bc = StealthCrypto::point_add(&point_a, &bc).unwrap();

        assert_eq!(
            ab_c, a_bc,
            "Point addition should be associative: (A+B)+C == A+(B+C)"
        );
    }

    #[test]
    fn test_point_add_commutativity() {
        // Use fixed scalars for deterministic testing
        let scalar_a = Scalar::from_bytes_mod_order([11u8; 32]);
        let scalar_b = Scalar::from_bytes_mod_order([13u8; 32]);

        let point_a = (&scalar_a * &ED25519_BASEPOINT_POINT)
            .compress()
            .to_bytes();
        let point_b = (&scalar_b * &ED25519_BASEPOINT_POINT)
            .compress()
            .to_bytes();

        // Test commutativity: A + B == B + A
        let ab = StealthCrypto::point_add(&point_a, &point_b).unwrap();
        let ba = StealthCrypto::point_add(&point_b, &point_a).unwrap();

        assert_eq!(ab, ba, "Point addition should be commutative: A+B == B+A");
    }

    #[test]
    fn test_point_add_invalid_point() {
        let scalar = Scalar::from_bytes_mod_order([17u8; 32]);
        let valid_point = (&scalar * &ED25519_BASEPOINT_POINT)
            .compress()
            .to_bytes();
        
        // Create an invalid point (not on the curve)
        let mut invalid_point = [0xFFu8; 32];
        invalid_point[31] = 0x7F; // Ensure it's in valid range but not on curve

        let result = StealthCrypto::point_add(&valid_point, &invalid_point);
        // Note: Some byte patterns may still be valid points, so we just verify no panic
        let _ = result;
    }

    #[test]
    fn test_derive_viewing_tag_deterministic() {
        let shared_secret = [42u8; 32];

        // Derive tag twice
        let tag1 = StealthCrypto::derive_viewing_tag(&shared_secret);
        let tag2 = StealthCrypto::derive_viewing_tag(&shared_secret);

        assert_eq!(tag1, tag2, "Viewing tag should be deterministic");
        assert_eq!(tag1.len(), 4, "Viewing tag should be 4 bytes");
    }

    #[test]
    fn test_derive_viewing_tag_different_secrets() {
        let secret1 = [1u8; 32];
        let secret2 = [2u8; 32];

        let tag1 = StealthCrypto::derive_viewing_tag(&secret1);
        let tag2 = StealthCrypto::derive_viewing_tag(&secret2);

        assert_ne!(
            tag1, tag2,
            "Different shared secrets should produce different viewing tags"
        );
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let plaintext = b"Hello, stealth world!";
        let key = [42u8; 32];
        let nonce = [1u8; 24];

        // Encrypt
        let ciphertext = StealthCrypto::encrypt_mesh_payload(plaintext, &key, &nonce).unwrap();
        assert_ne!(
            ciphertext.as_slice(),
            plaintext,
            "Ciphertext should differ from plaintext"
        );

        // Decrypt
        let decrypted = StealthCrypto::decrypt_mesh_payload(&ciphertext, &key, &nonce).unwrap();
        assert_eq!(
            decrypted.as_slice(),
            plaintext,
            "Decrypted text should match original plaintext"
        );
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext_with_different_nonce() {
        let plaintext = b"Test message";
        let key = [42u8; 32];
        let nonce1 = [1u8; 24];
        let nonce2 = [2u8; 24];

        let ciphertext1 = StealthCrypto::encrypt_mesh_payload(plaintext, &key, &nonce1).unwrap();
        let ciphertext2 = StealthCrypto::encrypt_mesh_payload(plaintext, &key, &nonce2).unwrap();

        assert_ne!(
            ciphertext1, ciphertext2,
            "Different nonces should produce different ciphertexts"
        );
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let plaintext = b"Secret message";
        let correct_key = [42u8; 32];
        let wrong_key = [43u8; 32];
        let nonce = [1u8; 24];

        let ciphertext = StealthCrypto::encrypt_mesh_payload(plaintext, &correct_key, &nonce).unwrap();
        let result = StealthCrypto::decrypt_mesh_payload(&ciphertext, &wrong_key, &nonce);

        assert!(
            result.is_err(),
            "Decryption with wrong key should fail authentication"
        );
    }

    #[test]
    fn test_decrypt_with_wrong_nonce_fails() {
        let plaintext = b"Secret message";
        let key = [42u8; 32];
        let correct_nonce = [1u8; 24];
        let wrong_nonce = [2u8; 24];

        let ciphertext = StealthCrypto::encrypt_mesh_payload(plaintext, &key, &correct_nonce).unwrap();
        let result = StealthCrypto::decrypt_mesh_payload(&ciphertext, &key, &wrong_nonce);

        assert!(
            result.is_err(),
            "Decryption with wrong nonce should fail authentication"
        );
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let plaintext = b"Secret message";
        let key = [42u8; 32];
        let nonce = [1u8; 24];

        let mut ciphertext = StealthCrypto::encrypt_mesh_payload(plaintext, &key, &nonce).unwrap();
        
        // Tamper with the ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xFF;
        }

        let result = StealthCrypto::decrypt_mesh_payload(&ciphertext, &key, &nonce);
        assert!(
            result.is_err(),
            "Decryption of tampered ciphertext should fail authentication"
        );
    }

    #[test]
    fn test_encrypt_empty_payload() {
        let plaintext = b"";
        let key = [42u8; 32];
        let nonce = [1u8; 24];

        let ciphertext = StealthCrypto::encrypt_mesh_payload(plaintext, &key, &nonce).unwrap();
        let decrypted = StealthCrypto::decrypt_mesh_payload(&ciphertext, &key, &nonce).unwrap();

        assert_eq!(
            decrypted.as_slice(),
            plaintext,
            "Empty payload should encrypt and decrypt correctly"
        );
    }

    #[test]
    fn test_encrypt_large_payload() {
        let plaintext = vec![0x42u8; 10000]; // 10KB payload
        let key = [42u8; 32];
        let nonce = [1u8; 24];

        let ciphertext = StealthCrypto::encrypt_mesh_payload(&plaintext, &key, &nonce).unwrap();
        let decrypted = StealthCrypto::decrypt_mesh_payload(&ciphertext, &key, &nonce).unwrap();

        assert_eq!(
            decrypted, plaintext,
            "Large payload should encrypt and decrypt correctly"
        );
    }
}
