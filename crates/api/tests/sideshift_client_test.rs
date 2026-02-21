use api::{AmountType, SideShiftClient};

#[tokio::test]
async fn test_sideshift_client_creation() {
    // Test that we can create a SideShift client
    let client = SideShiftClient::new(None);
    
    // Client should be created successfully
    // This is a basic smoke test
    drop(client);
}

#[tokio::test]
async fn test_sideshift_client_with_affiliate() {
    // Test that we can create a SideShift client with affiliate ID
    let client = SideShiftClient::new(Some("test-affiliate".to_string()));
    
    // Client should be created successfully
    drop(client);
}

#[tokio::test]
async fn test_amount_type_equality() {
    // Test AmountType enum equality
    assert_eq!(AmountType::From, AmountType::From);
    assert_eq!(AmountType::To, AmountType::To);
    assert_ne!(AmountType::From, AmountType::To);
}

// Note: Integration tests with actual API calls would require:
// 1. Network access
// 2. Valid SideShift API endpoints
// 3. Proper error handling for rate limits
// These should be tested in a separate integration test suite with proper mocking
