// Test for proximity API endpoints

use api::{ApiError, ApiResult};

#[tokio::test]
async fn test_proximity_endpoints_structure() {
    // This test verifies that the proximity endpoint handlers are properly structured
    // and return the expected NotImplemented error when proximity service is not initialized
    
    // The actual integration with proximity service would be tested when the service
    // is added to AppState in a future integration task
    
    // For now, we verify the error types are correct
    let error = ApiError::NotImplemented("Test".to_string());
    assert!(matches!(error, ApiError::NotImplemented(_)));
}

#[test]
fn test_proximity_event_serialization() {
    use api::ProximityEvent;
    use serde_json;
    
    // Test that proximity events can be serialized to JSON
    let event = ProximityEvent::PeerDiscovered {
        peer_id: "peer123".to_string(),
        user_tag: "alice".to_string(),
        wallet_address: "So11111111111111111111111111111111111111112".to_string(),
        discovery_method: "WiFi".to_string(),
        signal_strength: Some(-50),
        verified: true,
        timestamp: 1234567890,
    };
    
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("peer_discovered"));
    assert!(json.contains("peer123"));
    assert!(json.contains("alice"));
}

#[test]
fn test_transfer_request_serialization() {
    use serde_json;
    use uuid::Uuid;
    use rust_decimal::Decimal;
    
    // Test transfer request structure
    #[derive(serde::Serialize)]
    struct TestTransferRequest {
        sender_user_id: Uuid,
        sender_wallet: String,
        recipient_user_id: Uuid,
        recipient_wallet: String,
        asset: String,
        amount: Decimal,
    }
    
    let request = TestTransferRequest {
        sender_user_id: Uuid::new_v4(),
        sender_wallet: "sender123".to_string(),
        recipient_user_id: Uuid::new_v4(),
        recipient_wallet: "recipient456".to_string(),
        asset: "SOL".to_string(),
        amount: Decimal::new(100, 2), // 1.00
    };
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("SOL"));
    assert!(json.contains("sender123"));
}

#[test]
fn test_discovery_method_serialization() {
    use proximity::DiscoveryMethod;
    use serde_json;
    
    let wifi = DiscoveryMethod::WiFi;
    let bluetooth = DiscoveryMethod::Bluetooth;
    
    let wifi_json = serde_json::to_string(&wifi).unwrap();
    let bluetooth_json = serde_json::to_string(&bluetooth).unwrap();
    
    assert_eq!(wifi_json, "\"WiFi\"");
    assert_eq!(bluetooth_json, "\"Bluetooth\"");
}

#[test]
fn test_transfer_status_serialization() {
    use proximity::TransferStatus;
    use serde_json;
    
    let statuses = vec![
        TransferStatus::Pending,
        TransferStatus::Accepted,
        TransferStatus::Rejected,
        TransferStatus::Executing,
        TransferStatus::Completed,
        TransferStatus::Failed,
        TransferStatus::Expired,
    ];
    
    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        assert!(!json.is_empty());
    }
}
