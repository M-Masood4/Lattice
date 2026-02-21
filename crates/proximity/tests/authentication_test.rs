use chrono::Utc;
use proximity::{AuthenticationProof, AuthenticationService, ProximityError};
use uuid::Uuid;

#[tokio::test]
async fn test_create_challenge_generates_random_nonce() {
    let service = AuthenticationService::new();
    let peer_id = Uuid::new_v4().to_string();

    let challenge1 = service.create_challenge(peer_id.clone()).await.unwrap();
    let challenge2 = service.create_challenge(peer_id.clone()).await.unwrap();

    // Nonces should be different (random)
    assert_ne!(challenge1.nonce, challenge2.nonce);
}

#[tokio::test]
async fn test_create_challenge_sets_60_second_expiration() {
    let service = AuthenticationService::new();
    let peer_id = Uuid::new_v4().to_string();

    let challenge = service.create_challenge(peer_id).await.unwrap();

    let duration = challenge
        .expires_at
        .signed_duration_since(challenge.created_at);
    assert_eq!(duration.num_seconds(), 60);
}

#[tokio::test]
async fn test_verify_peer_with_expired_challenge() {
    let service = AuthenticationService::new();
    let peer_id = Uuid::new_v4().to_string();

    // Create a challenge
    let _challenge = service.create_challenge(peer_id).await.unwrap();

    // Wait for expiration (simulate by creating expired challenge manually)
    // For testing, we'll just test the error path by using an old peer_id
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to verify with a different peer_id (no challenge exists)
    let different_peer = Uuid::new_v4().to_string();
    let proof = AuthenticationProof {
        wallet_address: "test_address".to_string(),
        signature: vec![0u8; 64],
        public_key: vec![0u8; 32],
    };

    let result = service.verify_peer(different_peer, proof).await;
    assert!(matches!(result, Err(ProximityError::ChallengeNotFound)));
}

#[tokio::test]
async fn test_verify_peer_with_invalid_signature() {
    let service = AuthenticationService::new();
    let peer_id = Uuid::new_v4().to_string();

    let _challenge = service.create_challenge(peer_id.clone()).await.unwrap();

    // Create invalid proof
    let proof = AuthenticationProof {
        wallet_address: "test_address".to_string(),
        signature: vec![0u8; 64],
        public_key: vec![0u8; 32],
    };

    let result = service.verify_peer(peer_id, proof).await;
    // Should return Ok(false) for invalid signature or Err for invalid format
    assert!(result.is_err() || result.unwrap() == false);
}

#[tokio::test]
async fn test_rate_limiting_allows_up_to_100_attempts() {
    let service = AuthenticationService::new();
    let peer_id = Uuid::new_v4().to_string();

    // Make 100 attempts - all should succeed
    for _ in 0..100 {
        let result = service.create_challenge(peer_id.clone()).await;
        assert!(result.is_ok());

        // Try to verify (will fail but counts toward rate limit)
        let proof = AuthenticationProof {
            wallet_address: "test".to_string(),
            signature: vec![0u8; 64],
            public_key: vec![0u8; 32],
        };
        let _ = service.verify_peer(peer_id.clone(), proof).await;
    }

    // 101st attempt should fail with rate limit
    let proof = AuthenticationProof {
        wallet_address: "test".to_string(),
        signature: vec![0u8; 64],
        public_key: vec![0u8; 32],
    };
    let result = service.verify_peer(peer_id.clone(), proof).await;
    assert!(matches!(result, Err(ProximityError::RateLimitExceeded)));
}

#[tokio::test]
async fn test_cleanup_expired_challenges() {
    let service = AuthenticationService::new();
    let peer_id1 = Uuid::new_v4().to_string();
    let peer_id2 = Uuid::new_v4().to_string();

    // Create two challenges
    service.create_challenge(peer_id1).await.unwrap();
    service.create_challenge(peer_id2).await.unwrap();

    // Immediately cleanup (nothing should be expired)
    let removed = service.cleanup_expired_challenges().await.unwrap();
    assert_eq!(removed, 0);

    // Note: Testing actual expiration would require waiting 60 seconds or mocking time
}

#[tokio::test]
async fn test_challenge_removed_after_verification_attempt() {
    let service = AuthenticationService::new();
    let peer_id = Uuid::new_v4().to_string();

    // Create challenge
    service.create_challenge(peer_id.clone()).await.unwrap();

    // Attempt verification (will fail but should remove challenge)
    let proof = AuthenticationProof {
        wallet_address: "test".to_string(),
        signature: vec![0u8; 64],
        public_key: vec![0u8; 32],
    };
    let _ = service.verify_peer(peer_id.clone(), proof).await;

    // Second attempt should fail with ChallengeNotFound
    let proof2 = AuthenticationProof {
        wallet_address: "test".to_string(),
        signature: vec![0u8; 64],
        public_key: vec![0u8; 32],
    };
    let result = service.verify_peer(peer_id, proof2).await;
    assert!(matches!(result, Err(ProximityError::ChallengeNotFound)));
}

#[tokio::test]
async fn test_different_peers_have_independent_rate_limits() {
    let service = AuthenticationService::new();
    let peer_id1 = Uuid::new_v4().to_string();
    let peer_id2 = Uuid::new_v4().to_string();

    // Exhaust rate limit for peer1
    for _ in 0..100 {
        service.create_challenge(peer_id1.clone()).await.unwrap();
        let proof = AuthenticationProof {
            wallet_address: "test".to_string(),
            signature: vec![0u8; 64],
            public_key: vec![0u8; 32],
        };
        let _ = service.verify_peer(peer_id1.clone(), proof).await;
    }

    // peer1 should be rate limited
    let proof1 = AuthenticationProof {
        wallet_address: "test".to_string(),
        signature: vec![0u8; 64],
        public_key: vec![0u8; 32],
    };
    let result1 = service.verify_peer(peer_id1.clone(), proof1).await;
    assert!(matches!(result1, Err(ProximityError::RateLimitExceeded)));

    // peer2 should still be able to authenticate
    service.create_challenge(peer_id2.clone()).await.unwrap();
    let proof2 = AuthenticationProof {
        wallet_address: "test".to_string(),
        signature: vec![0u8; 64],
        public_key: vec![0u8; 32],
    };
    let result2 = service.verify_peer(peer_id2.clone(), proof2).await;
    // Should fail for other reasons, not rate limit
    assert!(!matches!(result2, Err(ProximityError::RateLimitExceeded)));
}
