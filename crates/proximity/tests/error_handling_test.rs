// Error handling tests

use proximity::{ProximityError, ErrorContext, ErrorCategory};
use uuid::Uuid;

#[test]
fn test_error_variants_exist() {
    // Test that all required error variants exist (Requirements 15.1, 15.2)
    let errors = vec![
        ProximityError::InsufficientBalance {
            required: "100".to_string(),
            available: "50".to_string(),
        },
        ProximityError::PeerNotFound("peer123".to_string()),
        ProximityError::TransactionFailed("network timeout".to_string()),
        ProximityError::Timeout("transfer acceptance".to_string()),
        ProximityError::NetworkError("connection lost".to_string()),
    ];

    for error in errors {
        // Verify Display trait works
        let display_msg = format!("{}", error);
        assert!(!display_msg.is_empty());
    }
}

#[test]
fn test_user_friendly_messages() {
    // Test user-friendly error messages (Requirements 15.1, 15.2)
    
    let error = ProximityError::InsufficientBalance {
        required: "100 SOL".to_string(),
        available: "50 SOL".to_string(),
    };
    let msg = error.user_message();
    assert!(msg.contains("Insufficient balance"));
    assert!(msg.contains("100 SOL"));
    assert!(msg.contains("50 SOL"));

    let error = ProximityError::PeerNotFound("Alice".to_string());
    let msg = error.user_message();
    assert!(msg.contains("Alice"));
    assert!(msg.contains("not be found"));

    let error = ProximityError::TransactionFailed("network timeout".to_string());
    let msg = error.user_message();
    assert!(msg.contains("Transaction failed"));
    assert!(msg.contains("network timeout"));

    let error = ProximityError::Timeout("transfer acceptance".to_string());
    let msg = error.user_message();
    assert!(msg.contains("timed out"));
    assert!(msg.contains("transfer acceptance"));

    let error = ProximityError::NetworkError("WiFi disconnected".to_string());
    let msg = error.user_message();
    assert!(msg.contains("Network error"));
    assert!(msg.contains("WiFi disconnected"));
}

#[test]
fn test_error_context_builder() {
    // Test error context builder pattern (Requirements 15.5)
    let user_id = Uuid::new_v4();
    let transfer_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let context = ErrorContext::new()
        .with_user_id(user_id)
        .with_peer_id("peer123".to_string())
        .with_transfer_id(transfer_id)
        .with_session_id(session_id)
        .with_info("Additional context".to_string());

    assert_eq!(context.user_id, Some(user_id));
    assert_eq!(context.peer_id, Some("peer123".to_string()));
    assert_eq!(context.transfer_id, Some(transfer_id));
    assert_eq!(context.session_id, Some(session_id));
    assert_eq!(context.additional_info, Some("Additional context".to_string()));
}

#[test]
fn test_error_context_partial() {
    // Test that context can be built with only some fields
    let user_id = Uuid::new_v4();

    let context = ErrorContext::new()
        .with_user_id(user_id);

    assert_eq!(context.user_id, Some(user_id));
    assert_eq!(context.peer_id, None);
    assert_eq!(context.transfer_id, None);
}

#[test]
fn test_error_categories() {
    // Test error categorization for metrics
    assert_eq!(
        ProximityError::InsufficientBalance {
            required: "100".to_string(),
            available: "50".to_string(),
        }.category(),
        ErrorCategory::Validation
    );

    assert_eq!(
        ProximityError::PeerNotFound("peer".to_string()).category(),
        ErrorCategory::NotFound
    );

    assert_eq!(
        ProximityError::TransactionFailed("error".to_string()).category(),
        ErrorCategory::Transaction
    );

    assert_eq!(
        ProximityError::Timeout("op".to_string()).category(),
        ErrorCategory::Timeout
    );

    assert_eq!(
        ProximityError::NetworkError("error".to_string()).category(),
        ErrorCategory::Network
    );

    assert_eq!(
        ProximityError::AuthenticationFailed("error".to_string()).category(),
        ErrorCategory::Authentication
    );

    assert_eq!(
        ProximityError::RateLimitExceeded.category(),
        ErrorCategory::RateLimit
    );
}

#[test]
fn test_error_category_display() {
    // Test error category display for logging
    assert_eq!(ErrorCategory::Validation.to_string(), "validation");
    assert_eq!(ErrorCategory::NotFound.to_string(), "not_found");
    assert_eq!(ErrorCategory::Authentication.to_string(), "authentication");
    assert_eq!(ErrorCategory::Transaction.to_string(), "transaction");
    assert_eq!(ErrorCategory::Network.to_string(), "network");
    assert_eq!(ErrorCategory::Timeout.to_string(), "timeout");
    assert_eq!(ErrorCategory::Permission.to_string(), "permission");
    assert_eq!(ErrorCategory::RateLimit.to_string(), "rate_limit");
    assert_eq!(ErrorCategory::Session.to_string(), "session");
    assert_eq!(ErrorCategory::Database.to_string(), "database");
    assert_eq!(ErrorCategory::Internal.to_string(), "internal");
}

#[test]
fn test_all_errors_have_user_messages() {
    // Ensure all error variants have user-friendly messages
    let errors = vec![
        ProximityError::DiscoveryNotEnabled,
        ProximityError::PeerNotFound("test".to_string()),
        ProximityError::AuthenticationFailed("test".to_string()),
        ProximityError::TransferNotFound("test".to_string()),
        ProximityError::InsufficientBalance {
            required: "100".to_string(),
            available: "50".to_string(),
        },
        ProximityError::TransactionFailed("test".to_string()),
        ProximityError::NetworkError("test".to_string()),
        ProximityError::Timeout("test".to_string()),
        ProximityError::PermissionDenied("test".to_string()),
        ProximityError::RateLimitExceeded,
        ProximityError::SessionExpired,
        ProximityError::SessionNotFound(Uuid::new_v4()),
        ProximityError::InvalidWalletAddress("test".to_string()),
        ProximityError::ChallengeNotFound,
        ProximityError::ChallengeExpired,
        ProximityError::InvalidPublicKey,
        ProximityError::InvalidSignature,
        ProximityError::BleError("test".to_string()),
        ProximityError::QrCodeError("test".to_string()),
        ProximityError::InvalidInput("test".to_string()),
        ProximityError::SerializationError("test".to_string()),
        ProximityError::ConnectionFailed("test".to_string()),
        ProximityError::InternalError("test".to_string()),
    ];

    for error in errors {
        let msg = error.user_message();
        assert!(!msg.is_empty(), "Error {:?} has empty user message", error);
        // User messages should be helpful and not just technical
        assert!(msg.len() > 10, "Error {:?} has too short user message", error);
    }
}

#[test]
fn test_error_logging_with_context() {
    // Test that error logging works (Requirements 15.5)
    let error = ProximityError::TransactionFailed("network timeout".to_string());
    let context = ErrorContext::new()
        .with_user_id(Uuid::new_v4())
        .with_transfer_id(Uuid::new_v4())
        .with_info("Test transaction".to_string());

    // This should not panic
    error.log_with_context(&context);
}

#[test]
fn test_insufficient_balance_error_details() {
    // Test specific error message formatting
    let error = ProximityError::InsufficientBalance {
        required: "100.5 SOL".to_string(),
        available: "50.25 SOL".to_string(),
    };

    let display = format!("{}", error);
    assert!(display.contains("100.5 SOL"));
    assert!(display.contains("50.25 SOL"));

    let user_msg = error.user_message();
    assert!(user_msg.contains("100.5 SOL"));
    assert!(user_msg.contains("50.25 SOL"));
    assert!(user_msg.contains("need"));
}
