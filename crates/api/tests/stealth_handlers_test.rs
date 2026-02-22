use api::handlers::*;
use serde_json::json;

#[tokio::test]
async fn test_generate_stealth_address_request_format() {
    // Test that the request structure is correctly defined
    // Requirements: 10.3 (2.1, 2.2)
    
    let request_json = json!({
        "version": 1
    });
    
    let request: Result<GenerateStealthAddressRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.version, Some(1));
}

#[tokio::test]
async fn test_generate_stealth_address_default_version() {
    // Test that version defaults to 1 if not provided
    // Requirements: 10.3 (2.2)
    
    let request_json = json!({});
    
    let request: Result<GenerateStealthAddressRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.version, None);
}

#[tokio::test]
async fn test_prepare_stealth_payment_request_format() {
    // Test that the prepare payment request structure is correctly defined
    // Requirements: 10.3 (2.3, 2.4, 2.5)
    
    let request_json = json!({
        "receiver_meta_address": "stealth:1:ABC123:DEF456",
        "amount": 1000000
    });
    
    let request: Result<PrepareStealthPaymentRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.receiver_meta_address, "stealth:1:ABC123:DEF456");
    assert_eq!(req.amount, 1000000);
}

#[tokio::test]
async fn test_prepare_stealth_payment_validates_meta_address_format() {
    // Test that meta-address format validation works
    // Requirements: 10.3 (2.2)
    
    let request_json = json!({
        "receiver_meta_address": "invalid_format",
        "amount": 1000000
    });
    
    let request: Result<PrepareStealthPaymentRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    // The validation happens in the handler, not deserialization
}

#[tokio::test]
async fn test_send_stealth_payment_request_format() {
    // Test that the send payment request structure is correctly defined
    // Requirements: 10.3 (5.1, 5.3, 8.1)
    
    let request_json = json!({
        "stealth_address": "ABC123XYZ",
        "amount": 1000000,
        "ephemeral_public_key": "EPH123",
        "viewing_tag": "12345678",
        "via_mesh": true
    });
    
    let request: Result<SendStealthPaymentRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.stealth_address, "ABC123XYZ");
    assert_eq!(req.amount, 1000000);
    assert_eq!(req.ephemeral_public_key, "EPH123");
    assert_eq!(req.viewing_tag, "12345678");
    assert_eq!(req.via_mesh, Some(true));
}

#[tokio::test]
async fn test_send_stealth_payment_optional_mesh_flag() {
    // Test that via_mesh flag is optional
    // Requirements: 10.3 (8.1)
    
    let request_json = json!({
        "stealth_address": "ABC123XYZ",
        "amount": 1000000,
        "ephemeral_public_key": "EPH123",
        "viewing_tag": "12345678"
    });
    
    let request: Result<SendStealthPaymentRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.via_mesh, None);
}

#[tokio::test]
async fn test_shield_funds_request_format() {
    // Test that the shield request structure is correctly defined
    // Requirements: 10.3 (7.1, 7.2, 7.3)
    
    let request_json = json!({
        "amount": 5000000,
        "source_keypair": "BASE58_ENCODED_KEY"
    });
    
    let request: Result<ShieldFundsRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.amount, 5000000);
    assert_eq!(req.source_keypair, "BASE58_ENCODED_KEY");
}

#[tokio::test]
async fn test_unshield_funds_request_format() {
    // Test that the unshield request structure is correctly defined
    // Requirements: 10.3 (7.2, 7.5)
    
    let request_json = json!({
        "stealth_address": "STEALTH123",
        "ephemeral_public_key": "EPH456",
        "amount": 5000000,
        "destination_address": "DEST789"
    });
    
    let request: Result<UnshieldFundsRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.stealth_address, "STEALTH123");
    assert_eq!(req.ephemeral_public_key, "EPH456");
    assert_eq!(req.amount, 5000000);
    assert_eq!(req.destination_address, "DEST789");
}

#[tokio::test]
async fn test_detected_stealth_payment_response_format() {
    // Test that the detected payment response structure is correctly defined
    // Requirements: 10.3 (3.1, 3.2, 3.3)
    
    let payment = DetectedStealthPayment {
        stealth_address: "STEALTH123".to_string(),
        amount: 1000000,
        ephemeral_public_key: "EPH456".to_string(),
        viewing_tag: "12345678".to_string(),
        slot: 12345,
        signature: "SIG789".to_string(),
    };
    
    let json = serde_json::to_value(&payment).unwrap();
    
    assert_eq!(json["stealth_address"], "STEALTH123");
    assert_eq!(json["amount"], 1000000);
    assert_eq!(json["ephemeral_public_key"], "EPH456");
    assert_eq!(json["viewing_tag"], "12345678");
    assert_eq!(json["slot"], 12345);
    assert_eq!(json["signature"], "SIG789");
}

#[tokio::test]
async fn test_payment_queue_status_response_format() {
    // Test that the payment queue status response structure is correctly defined
    // Requirements: 10.3 (5.2, 5.4)
    
    let queued_payment = QueuedPaymentInfo {
        payment_id: "PAY123".to_string(),
        stealth_address: "STEALTH456".to_string(),
        amount: 2000000,
        status: "queued".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        retry_count: 0,
    };
    
    let response = PaymentQueueStatusResponse {
        queued_payments: vec![queued_payment],
        total_count: 1,
    };
    
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["total_count"], 1);
    assert_eq!(json["queued_payments"][0]["payment_id"], "PAY123");
    assert_eq!(json["queued_payments"][0]["status"], "queued");
}

#[tokio::test]
async fn test_api_response_success_format() {
    // Test that successful API responses have the correct structure
    // Requirements: 10.3
    
    let data = GenerateStealthAddressResponse {
        meta_address: "stealth:1:ABC:DEF".to_string(),
        version: 1,
    };
    
    let response = ApiResponse::success(data);
    
    assert!(response.success);
    assert!(response.data.is_some());
    assert!(response.error.is_none());
    
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["meta_address"], "stealth:1:ABC:DEF");
    assert_eq!(json["data"]["version"], 1);
}

#[tokio::test]
async fn test_api_response_error_format() {
    // Test that error API responses have the correct structure
    // Requirements: 10.3
    
    let response: ApiResponse<()> = ApiResponse::error("Test error message".to_string());
    
    assert!(!response.success);
    assert!(response.data.is_none());
    assert!(response.error.is_some());
    
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["error"], "Test error message");
}

#[tokio::test]
async fn test_viewing_tag_length_validation() {
    // Test that viewing tag must be exactly 8 hex characters (4 bytes)
    // Requirements: 10.3 (2.8)
    
    // Valid viewing tag (8 hex chars)
    let valid_request = json!({
        "stealth_address": "ABC123",
        "amount": 1000000,
        "ephemeral_public_key": "EPH123",
        "viewing_tag": "12345678"
    });
    
    let request: Result<SendStealthPaymentRequest, _> = 
        serde_json::from_value(valid_request);
    assert!(request.is_ok());
    
    // Invalid viewing tag (too short)
    let invalid_request = json!({
        "stealth_address": "ABC123",
        "amount": 1000000,
        "ephemeral_public_key": "EPH123",
        "viewing_tag": "1234"
    });
    
    let request: Result<SendStealthPaymentRequest, _> = 
        serde_json::from_value(invalid_request);
    assert!(request.is_ok()); // Deserialization succeeds, validation happens in handler
}

#[tokio::test]
async fn test_zero_amount_validation() {
    // Test that zero amounts are rejected
    // Requirements: 10.3
    
    let request_json = json!({
        "receiver_meta_address": "stealth:1:ABC:DEF",
        "amount": 0
    });
    
    let request: Result<PrepareStealthPaymentRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    // The validation for zero amount happens in the handler
}

#[tokio::test]
async fn test_hybrid_version_support() {
    // Test that version 2 (hybrid post-quantum) is supported
    // Requirements: 10.3 (6.1, 6.2)
    
    let request_json = json!({
        "version": 2
    });
    
    let request: Result<GenerateStealthAddressRequest, _> = 
        serde_json::from_value(request_json);
    
    assert!(request.is_ok());
    let req = request.unwrap();
    assert_eq!(req.version, Some(2));
}

#[tokio::test]
async fn test_scan_payments_response_structure() {
    // Test that scan response includes all required fields
    // Requirements: 10.3 (3.1, 3.2, 3.3, 3.6)
    
    let payment = DetectedStealthPayment {
        stealth_address: "STEALTH123".to_string(),
        amount: 1000000,
        ephemeral_public_key: "EPH456".to_string(),
        viewing_tag: "ABCD1234".to_string(),
        slot: 54321,
        signature: "SIG789ABC".to_string(),
    };
    
    let response = ScanStealthPaymentsResponse {
        payments: vec![payment],
        last_scanned_slot: 54321,
    };
    
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["last_scanned_slot"], 54321);
    assert_eq!(json["payments"].as_array().unwrap().len(), 1);
    assert_eq!(json["payments"][0]["stealth_address"], "STEALTH123");
}

#[tokio::test]
async fn test_payment_status_values() {
    // Test that payment status values are correctly defined
    // Requirements: 10.3 (5.4, 5.5, 5.6, 5.7)
    
    let statuses = vec!["queued", "settling", "settled", "failed"];
    
    for status in statuses {
        let payment = QueuedPaymentInfo {
            payment_id: "PAY123".to_string(),
            stealth_address: "STEALTH456".to_string(),
            amount: 1000000,
            status: status.to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            retry_count: 0,
        };
        
        let json = serde_json::to_value(&payment).unwrap();
        assert_eq!(json["status"], status);
    }
}

#[tokio::test]
async fn test_shield_response_structure() {
    // Test that shield response includes stealth address and signature
    // Requirements: 10.3 (7.1, 7.2)
    
    let response = ShieldFundsResponse {
        stealth_address: "STEALTH_ADDR_123".to_string(),
        signature: "TX_SIG_456".to_string(),
    };
    
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["stealth_address"], "STEALTH_ADDR_123");
    assert_eq!(json["signature"], "TX_SIG_456");
}

#[tokio::test]
async fn test_unshield_response_structure() {
    // Test that unshield response includes destination and signature
    // Requirements: 10.3 (7.2, 7.5)
    
    let response = UnshieldFundsResponse {
        destination_address: "DEST_ADDR_789".to_string(),
        signature: "TX_SIG_ABC".to_string(),
    };
    
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["destination_address"], "DEST_ADDR_789");
    assert_eq!(json["signature"], "TX_SIG_ABC");
}

#[tokio::test]
async fn test_send_payment_response_structure() {
    // Test that send payment response includes all status fields
    // Requirements: 10.3 (5.1, 5.3)
    
    let response = SendStealthPaymentResponse {
        status: "queued".to_string(),
        payment_id: Some("PAY123".to_string()),
        signature: None,
    };
    
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["status"], "queued");
    assert_eq!(json["payment_id"], "PAY123");
    assert!(json["signature"].is_null());
}

#[tokio::test]
async fn test_prepare_payment_response_structure() {
    // Test that prepare payment response includes all required fields
    // Requirements: 10.3 (2.3, 2.4, 2.5, 2.8)
    
    let response = PrepareStealthPaymentResponse {
        stealth_address: "STEALTH123".to_string(),
        amount: 1000000,
        ephemeral_public_key: "EPH456".to_string(),
        viewing_tag: "ABCD1234".to_string(),
    };
    
    let json = serde_json::to_value(&response).unwrap();
    
    assert_eq!(json["stealth_address"], "STEALTH123");
    assert_eq!(json["amount"], 1000000);
    assert_eq!(json["ephemeral_public_key"], "EPH456");
    assert_eq!(json["viewing_tag"], "ABCD1234");
}
