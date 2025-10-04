// Core edge case tests focusing on critical functionality and boundary conditions
use accumulate_client::{TransactionHeader, TransactionType, AccountType, SignatureType};
use accumulate_client::generated::header::{ExpireOptions, HoldUntilOptions};
use accumulate_client::errors::Error;
use serde_json;

/// Test core JSON serialization/deserialization edge cases
#[test]
fn test_json_roundtrip_edge_cases() {
    println!("Testing JSON roundtrip edge cases");

    // Test with all optional fields as None
    let minimal_header = TransactionHeader {
        principal: "acc://test.acme".to_string(),
        initiator: vec![0xde, 0xad, 0xbe, 0xef],
        memo: None,
        metadata: None,
        expire: None,
        hold_until: None,
        authorities: None,
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&minimal_header).unwrap();
    let deserialized: TransactionHeader = serde_json::from_str(&json).unwrap();

    assert_eq!(minimal_header.principal, deserialized.principal);
    assert_eq!(minimal_header.initiator, deserialized.initiator);
    assert_eq!(minimal_header.memo, deserialized.memo);
    assert_eq!(minimal_header.metadata, deserialized.metadata);

    // Test with all optional fields populated
    let complete_header = TransactionHeader {
        principal: "acc://complete.acme".to_string(),
        initiator: vec![0x01, 0x02, 0x03, 0x04],
        memo: Some("Complete test".to_string()),
        metadata: Some(vec![0xaa, 0xbb, 0xcc, 0xdd]),
        expire: Some(ExpireOptions { at_time: Some(9999999999) }),
        hold_until: Some(HoldUntilOptions { minor_block: Some(12345) }),
        authorities: Some(vec!["acc://auth1.acme".to_string(), "acc://auth2.acme".to_string()]),
    };

    let json = serde_json::to_string(&complete_header).unwrap();
    let deserialized: TransactionHeader = serde_json::from_str(&json).unwrap();

    assert_eq!(complete_header, deserialized);

    println!("✓ JSON roundtrip edge cases passed");
}

/// Test enum serialization consistency
#[test]
fn test_enum_serialization_consistency() {
    println!("Testing enum serialization consistency");

    // Test that serialization is deterministic
    let tx_type = TransactionType::WriteData;
    let mut serializations = Vec::new();

    for _ in 0..5 {
        let json = serde_json::to_string(&tx_type).unwrap();
        serializations.push(json);
    }

    // All serializations should be identical
    for i in 1..serializations.len() {
        assert_eq!(serializations[0], serializations[i], "Serialization should be deterministic");
    }

    // Test common transaction types
    let transaction_types = vec![
        TransactionType::WriteData,
        TransactionType::CreateIdentity,
        TransactionType::SendTokens,
        TransactionType::CreateTokenAccount,
        TransactionType::AddCredits,
    ];

    for tx_type in transaction_types {
        let json = serde_json::to_string(&tx_type).unwrap();
        let deserialized: TransactionType = serde_json::from_str(&json).unwrap();
        assert_eq!(tx_type, deserialized, "Transaction type roundtrip should work");
    }

    // Test account types
    let account_types = vec![
        AccountType::Identity,
        AccountType::TokenAccount,
        AccountType::DataAccount,
        AccountType::KeyPage,
        AccountType::KeyBook,
    ];

    for account_type in account_types {
        let json = serde_json::to_string(&account_type).unwrap();
        let deserialized: AccountType = serde_json::from_str(&json).unwrap();
        assert_eq!(account_type, deserialized, "Account type roundtrip should work");
    }

    // Test signature types
    let signature_types = vec![
        SignatureType::ED25519,
        SignatureType::RCD1,
        SignatureType::Delegated,
        SignatureType::Set,
        SignatureType::BTC,
        SignatureType::ETH,
    ];

    for sig_type in signature_types {
        let json = serde_json::to_string(&sig_type).unwrap();
        let deserialized: SignatureType = serde_json::from_str(&json).unwrap();
        assert_eq!(sig_type, deserialized, "Signature type roundtrip should work");
    }

    println!("✓ Enum serialization consistency passed");
}

/// Test malformed JSON handling
#[test]
fn test_malformed_json_handling() {
    println!("Testing malformed JSON handling");

    let malformed_inputs = vec![
        "",                                    // Empty string
        "{",                                   // Incomplete JSON
        "null",                               // Null value
        "[]",                                 // Array instead of object
        "{\"Principal\": }",                  // Missing value
        "{'Principal': 'acc://test'}",        // Single quotes
        "{\"Principal\": \"acc://test\", \"Initiator\": \"not-array\"}", // Wrong type
    ];

    for input in malformed_inputs {
        let result: Result<TransactionHeader, _> = serde_json::from_str(input);
        assert!(result.is_err(), "Should reject malformed input: {}", input);
    }

    // Test enum parsing with invalid values
    let invalid_enum_inputs = vec![
        "\"\"",              // Empty string
        "\"InvalidType\"",   // Invalid enum variant
        "\"writedata\"",     // Wrong case
        "\"WRITEDATA\"",     // Wrong case
    ];

    for input in invalid_enum_inputs {
        let result: Result<TransactionType, _> = serde_json::from_str(input);
        assert!(result.is_err(), "Should reject invalid enum: {}", input);
    }

    println!("✓ Malformed JSON handling passed");
}

/// Test boundary conditions
#[test]
fn test_boundary_conditions() {
    println!("Testing boundary conditions");

    // Test with empty vectors
    let empty_initiator = TransactionHeader {
        principal: "acc://test.acme".to_string(),
        initiator: vec![],
        memo: None,
        metadata: None,
        expire: None,
        hold_until: None,
        authorities: None,
    };

    // Should be able to serialize empty initiator
    let json = serde_json::to_string(&empty_initiator).unwrap();
    let deserialized: TransactionHeader = serde_json::from_str(&json).unwrap();
    assert_eq!(empty_initiator.initiator, deserialized.initiator);

    // Test with large but reasonable data
    let large_initiator = vec![0u8; 256]; // Reasonable size
    let large_metadata = vec![0u8; 1024]; // Reasonable size

    let large_header = TransactionHeader {
        principal: "acc://large.acme".to_string(),
        initiator: large_initiator.clone(),
        memo: Some("A".repeat(1000)), // 1KB memo
        metadata: Some(large_metadata.clone()),
        expire: None,
        hold_until: None,
        authorities: Some(vec!["acc://auth.acme".to_string(); 10]), // Multiple authorities
    };

    // Should handle reasonably large data
    let json = serde_json::to_string(&large_header).unwrap();
    let deserialized: TransactionHeader = serde_json::from_str(&json).unwrap();
    assert_eq!(large_header, deserialized);

    println!("✓ Boundary conditions passed");
}

/// Test error type functionality
#[test]
fn test_error_type_functionality() {
    println!("Testing error type functionality");

    // Test error creation and display
    let general_error = Error::General("test error".to_string());
    let error_display = format!("{}", general_error);
    assert!(error_display.contains("test error"));

    let network_error = Error::Network("network failure".to_string());
    let network_display = format!("{}", network_error);
    assert!(network_display.contains("network failure"));

    let encoding_error = Error::Encoding("encoding failure".to_string());
    let encoding_display = format!("{}", encoding_error);
    assert!(encoding_display.contains("encoding failure"));

    // Test error conversion from string types
    let from_string: Error = "string conversion".to_string().into();
    assert!(matches!(from_string, Error::General(_)));

    let from_str: Error = "str conversion".into();
    assert!(matches!(from_str, Error::General(_)));

    println!("✓ Error type functionality passed");
}

/// Test validation behavior
#[test]
fn test_validation_behavior() {
    println!("Testing validation behavior");

    // Test that basic valid headers pass validation
    let valid_header = TransactionHeader {
        principal: "acc://valid.acme".to_string(),
        initiator: vec![0xde, 0xad, 0xbe, 0xef],
        memo: Some("Valid transaction".to_string()),
        metadata: Some(vec![0x01, 0x02, 0x03]),
        expire: Some(ExpireOptions { at_time: Some(9999999999) }),
        hold_until: Some(HoldUntilOptions { minor_block: Some(12345) }),
        authorities: Some(vec!["acc://authority.acme".to_string()]),
    };

    assert!(valid_header.validate().is_ok(), "Valid header should pass validation");

    // Test minimal valid header
    let minimal_header = TransactionHeader {
        principal: "acc://minimal.acme".to_string(),
        initiator: vec![0x01],
        memo: None,
        metadata: None,
        expire: None,
        hold_until: None,
        authorities: None,
    };

    assert!(minimal_header.validate().is_ok(), "Minimal header should pass validation");

    // Test that ExpireOptions and HoldUntilOptions validate
    let expire_opts = ExpireOptions { at_time: Some(9999999999) };
    assert!(expire_opts.validate().is_ok(), "ExpireOptions should validate");

    let hold_opts = HoldUntilOptions { minor_block: Some(12345) };
    assert!(hold_opts.validate().is_ok(), "HoldUntilOptions should validate");

    println!("✓ Validation behavior passed");
}

/// Test field naming and serialization format
#[test]
fn test_field_naming_format() {
    println!("Testing field naming and serialization format");

    let header = TransactionHeader {
        principal: "acc://naming.acme".to_string(),
        initiator: vec![0xaa, 0xbb, 0xcc, 0xdd],
        memo: Some("Naming test".to_string()),
        metadata: Some(vec![0x11, 0x22]),
        expire: Some(ExpireOptions { at_time: Some(1234567890) }),
        hold_until: Some(HoldUntilOptions { minor_block: Some(5678) }),
        authorities: Some(vec!["acc://auth.acme".to_string()]),
    };

    let json_value = serde_json::to_value(&header).unwrap();

    // Verify field naming follows the expected format
    assert!(json_value.get("Principal").is_some(), "Should have Principal field");
    assert!(json_value.get("Initiator").is_some(), "Should have Initiator field");
    assert!(json_value.get("Memo").is_some(), "Should have Memo field");
    assert!(json_value.get("Metadata").is_some(), "Should have Metadata field");
    assert!(json_value.get("Expire").is_some(), "Should have Expire field");
    assert!(json_value.get("HoldUntil").is_some(), "Should have HoldUntil field");
    assert!(json_value.get("Authorities").is_some(), "Should have Authorities field");

    // Test that the JSON can be roundtripped
    let json_str = serde_json::to_string(&json_value).unwrap();
    let roundtripped: TransactionHeader = serde_json::from_str(&json_str).unwrap();
    assert_eq!(header, roundtripped);

    println!("✓ Field naming format passed");
}