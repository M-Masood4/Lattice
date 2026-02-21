use api::auth::{AuthError, JwtConfig};
use uuid::Uuid;

#[tokio::test]
async fn test_complete_auth_flow() {
    // This is a unit test that verifies the auth logic without database
    let jwt_config = JwtConfig::new("test_secret_key_for_testing".to_string());
    
    // Test email validation
    assert!(jwt_config.validate_email("user@example.com").is_ok());
    assert!(jwt_config.validate_email("invalid").is_err());
    
    // Test password strength validation
    assert!(jwt_config.validate_password_strength("StrongPass123").is_ok());
    assert!(jwt_config.validate_password_strength("weak").is_err());
    
    // Test password hashing and verification
    let password = "TestPassword123";
    let hash = jwt_config.hash_password(password).unwrap();
    assert!(jwt_config.verify_password(password, &hash).unwrap());
    assert!(!jwt_config.verify_password("WrongPassword", &hash).unwrap());
    
    // Test JWT token generation and verification
    let user_id = Uuid::new_v4();
    let token = jwt_config.generate_token(user_id).unwrap();
    let claims = jwt_config.verify_token(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    
    // Test user tag generation
    let tag = jwt_config.generate_user_tag();
    assert!(tag.starts_with("Trader_"));
    assert_eq!(tag.len(), 13);
    
    // Test TOTP secret generation
    let secret = jwt_config.generate_totp_secret();
    assert!(!secret.is_empty());
    
    // Test TOTP code verification with a known secret and code
    // Note: This is a simplified test - in production, codes are time-based
    let test_secret = jwt_config.generate_totp_secret();
    // We can't test actual TOTP verification without mocking time,
    // but we can verify the function doesn't panic
    let _ = jwt_config.verify_totp_code(&test_secret, "123456");
}

#[test]
fn test_auth_error_types() {
    // Test that all error types can be created
    let _err1 = AuthError::InvalidCredentials;
    let _err2 = AuthError::UserAlreadyExists;
    let _err3 = AuthError::InvalidEmail;
    let _err4 = AuthError::WeakPassword("test".to_string());
    let _err5 = AuthError::DatabaseError("test".to_string());
    let _err6 = AuthError::HashingError("test".to_string());
    let _err7 = AuthError::UserTagGenerationFailed;
}

#[test]
fn test_jwt_token_expiration() {
    let jwt_config = JwtConfig::new("test_secret".to_string());
    let user_id = Uuid::new_v4();
    
    // Generate token
    let token = jwt_config.generate_token(user_id).unwrap();
    
    // Verify token
    let claims = jwt_config.verify_token(&token).unwrap();
    
    // Check that expiration is set (should be 24 hours from now)
    let now = chrono::Utc::now().timestamp() as usize;
    let expected_exp = now + (24 * 60 * 60);
    
    // Allow 10 second tolerance for test execution time
    assert!(claims.exp >= expected_exp - 10);
    assert!(claims.exp <= expected_exp + 10);
}
