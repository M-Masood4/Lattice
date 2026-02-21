// Authentication Service - handles peer authentication via challenge-response

use crate::{PeerId, ProximityError, Result, ErrorContext};
use chrono::{DateTime, Utc};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use rand::RngCore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Challenge {
    pub nonce: [u8; 32],
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct AuthenticationProof {
    pub wallet_address: String,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

struct RateLimitEntry {
    attempts: u32,
    window_start: DateTime<Utc>,
}

pub struct AuthenticationService {
    challenge_cache: Arc<RwLock<HashMap<PeerId, Challenge>>>,
    rate_limits: Arc<RwLock<HashMap<PeerId, RateLimitEntry>>>,
}

impl AuthenticationService {
    pub fn new() -> Self {
        Self {
            challenge_cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a 32-byte random nonce for challenge
    /// Stores challenge with 60-second expiration
    pub async fn create_challenge(&self, peer_id: PeerId) -> Result<Challenge> {
        // Generate 32-byte random nonce
        let mut nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut nonce);

        let now = Utc::now();
        let challenge = Challenge {
            nonce,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(60),
        };

        // Store challenge in cache
        let mut cache = self.challenge_cache.write().await;
        cache.insert(peer_id, challenge.clone());

        Ok(challenge)
    }

    /// Verify peer signature using Ed25519
    /// Returns true if signature is valid and matches claimed wallet address
    pub async fn verify_peer(&self, peer_id: PeerId, proof: AuthenticationProof) -> Result<bool> {
        // Check rate limiting
        if !self.check_rate_limit(&peer_id).await? {
            let err = ProximityError::RateLimitExceeded;
            let context = ErrorContext::new()
                .with_peer_id(peer_id.clone())
                .with_info("Authentication rate limit exceeded".to_string());
            err.log_with_context(&context);
            return Err(err);
        }

        // Retrieve challenge from cache
        let mut cache = self.challenge_cache.write().await;
        let challenge = cache
            .remove(&peer_id)
            .ok_or_else(|| {
                let err = ProximityError::ChallengeNotFound;
                let context = ErrorContext::new()
                    .with_peer_id(peer_id.clone())
                    .with_info("Challenge not found in cache".to_string());
                err.log_with_context(&context);
                err
            })?;

        // Check if challenge has expired
        let now = Utc::now();
        if now > challenge.expires_at {
            let err = ProximityError::ChallengeExpired;
            let context = ErrorContext::new()
                .with_peer_id(peer_id.clone())
                .with_info(format!("Challenge expired at {}", challenge.expires_at));
            err.log_with_context(&context);
            return Err(err);
        }

        // Verify signature using Ed25519
        let public_key = PublicKey::from_bytes(&proof.public_key)
            .map_err(|e| {
                let err = ProximityError::InvalidPublicKey;
                let context = ErrorContext::new()
                    .with_peer_id(peer_id.clone())
                    .with_info(format!("Failed to parse public key: {}", e));
                err.log_with_context(&context);
                err
            })?;

        let signature = Signature::try_from(proof.signature.as_slice())
            .map_err(|e| {
                let err = ProximityError::InvalidSignature;
                let context = ErrorContext::new()
                    .with_peer_id(peer_id.clone())
                    .with_info(format!("Failed to parse signature: {}", e));
                err.log_with_context(&context);
                err
            })?;

        // Verify the signature against the challenge nonce
        match public_key.verify(&challenge.nonce, &signature) {
            Ok(_) => {
                // Verify that the public key matches the claimed wallet address
                let derived_address = bs58::encode(&proof.public_key).into_string();
                if derived_address == proof.wallet_address {
                    Ok(true)
                } else {
                    let err = ProximityError::AuthenticationFailed(
                        "Public key does not match wallet address".to_string()
                    );
                    let context = ErrorContext::new()
                        .with_peer_id(peer_id.clone())
                        .with_info(format!(
                            "Wallet mismatch: claimed={}, derived={}",
                            proof.wallet_address, derived_address
                        ));
                    err.log_with_context(&context);
                    Ok(false)
                }
            }
            Err(e) => {
                let err = ProximityError::AuthenticationFailed(
                    format!("Signature verification failed: {}", e)
                );
                let context = ErrorContext::new()
                    .with_peer_id(peer_id.clone())
                    .with_info("Signature verification failed".to_string());
                err.log_with_context(&context);
                Ok(false)
            }
        }
    }

    /// Check rate limiting: max 100 attempts per minute per peer
    async fn check_rate_limit(&self, peer_id: &PeerId) -> Result<bool> {
        let mut rate_limits = self.rate_limits.write().await;
        let now = Utc::now();

        let entry = rate_limits.entry(peer_id.clone()).or_insert(RateLimitEntry {
            attempts: 0,
            window_start: now,
        });

        // Check if we're in a new time window (1 minute)
        let window_duration = now.signed_duration_since(entry.window_start);
        if window_duration.num_seconds() >= 60 {
            // Reset window
            entry.attempts = 1;
            entry.window_start = now;
            Ok(true)
        } else {
            // Check if limit exceeded
            if entry.attempts >= 100 {
                Ok(false)
            } else {
                entry.attempts += 1;
                Ok(true)
            }
        }
    }

    /// Clean up expired challenges
    pub async fn cleanup_expired_challenges(&self) -> Result<u64> {
        let mut cache = self.challenge_cache.write().await;
        let now = Utc::now();
        let initial_count = cache.len();

        cache.retain(|_, challenge| now <= challenge.expires_at);

        Ok((initial_count - cache.len()) as u64)
    }
}
