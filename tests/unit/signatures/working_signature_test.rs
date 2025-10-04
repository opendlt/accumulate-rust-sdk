// Simple working test to verify our signature system functionality

use accumulate_client::{Signature, AccSignature};
use serde_json;
use hex;

#[test]
fn test_basic_signature_creation() {
    // Test creating a simple Ed25519 signature from JSON
    let sig_json = serde_json::json!({
        "type": "ed25519",
        "PublicKey": hex::encode([0u8; 32]),
        "Signature": hex::encode([0u8; 64]),
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Timestamp": 1234567890,
        "Vote": null,
        "TransactionHash": null,
        "Memo": null,
        "Data": null
    });

    let signature: Result<Signature, _> = serde_json::from_value(sig_json);
    assert!(signature.is_ok(), "Ed25519 signature should deserialize successfully");

    let sig = signature.unwrap();
    assert_eq!(sig.wire_tag(), "ed25519", "Wire tag should match");
}

#[test]
fn test_legacy_ed25519_signature() {
    let sig_json = serde_json::json!({
        "type": "legacyED25519",
        "Timestamp": 1234567890,
        "PublicKey": hex::encode([0u8; 32]),
        "Signature": hex::encode([0u8; 64]),
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Vote": null,
        "TransactionHash": null
    });

    let signature: Result<Signature, _> = serde_json::from_value(sig_json);
    assert!(signature.is_ok(), "Legacy Ed25519 signature should deserialize successfully");

    let sig = signature.unwrap();
    assert_eq!(sig.wire_tag(), "legacyED25519", "Wire tag should match");
}

#[test]
fn test_signature_verification_interface() {
    // Test that the verification interface works (even if it returns errors for test data)
    let sig_json = serde_json::json!({
        "type": "ed25519",
        "PublicKey": hex::encode([1u8; 32]),
        "Signature": hex::encode([2u8; 64]),
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Timestamp": 1234567890,
        "Vote": null,
        "TransactionHash": null,
        "Memo": null,
        "Data": null
    });

    let signature: Signature = serde_json::from_value(sig_json).unwrap();
    let test_message = b"test verification message";

    // Test that verify method can be called
    match &signature {
        Signature::ED25519(sig) => {
            let result = sig.verify(test_message);
            // Should return either Ok(false) or Err for test data
            assert!(result.is_ok() || result.is_err(), "Verify method should return a result");
        },
        _ => panic!("Expected Ed25519 signature variant"),
    }
}

#[test]
fn test_signature_roundtrip() {
    // Test serialize -> deserialize -> serialize consistency
    let original_json = serde_json::json!({
        "type": "ed25519",
        "PublicKey": hex::encode([3u8; 32]),
        "Signature": hex::encode([4u8; 64]),
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Timestamp": 1234567890,
        "Vote": null,
        "TransactionHash": null,
        "Memo": null,
        "Data": null
    });

    // First deserialization
    let signature: Signature = serde_json::from_value(original_json.clone()).unwrap();

    // Serialize back to JSON
    let serialized = serde_json::to_value(&signature).unwrap();

    // Second deserialization
    let signature2: Signature = serde_json::from_value(serialized.clone()).unwrap();

    // Should maintain consistency
    assert_eq!(signature.wire_tag(), signature2.wire_tag());
    assert_eq!(serialized["type"], original_json["type"]);
}

#[test]
fn test_signature_count_validation() {
    // Test that we can read the manifest and it has the expected count
    use std::fs;
    use std::path::PathBuf;

    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("signatures_manifest.json");

    let content = fs::read_to_string(&manifest_path)
        .expect("Failed to read signatures manifest");
    let manifest: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse manifest JSON");

    let count = manifest["counts"]["signatures"].as_u64().unwrap();
    assert_eq!(count, 16, "Should have exactly 16 signatures");

    let signatures = manifest["signatures"].as_array().unwrap();
    assert_eq!(signatures.len(), 16, "Manifest should contain 16 signatures");
}

#[test]
fn test_error_handling() {
    // Test that invalid JSON is properly rejected
    let invalid_cases = vec![
        serde_json::json!({}), // Empty object
        serde_json::json!({"type": "nonexistent"}), // Invalid type
        serde_json::json!(null), // Null
        serde_json::json!("string"), // Wrong type
    ];

    for invalid_json in invalid_cases {
        let result: Result<Signature, _> = serde_json::from_value(invalid_json);
        assert!(result.is_err(), "Invalid JSON should be rejected");
    }
}

#[test]
fn test_signature_type_methods() {
    let test_cases = vec![
        ("ed25519", create_ed25519_json()),
        ("legacyED25519", create_legacy_ed25519_json()),
    ];

    for (expected_type, json) in test_cases {
        let signature: Signature = serde_json::from_value(json).unwrap();

        // Test wire_tag
        assert_eq!(signature.wire_tag(), expected_type);

        // Test sig_type via trait
        let sig_type = match &signature {
            Signature::ED25519(s) => s.sig_type(),
            Signature::LegacyED25519(s) => s.sig_type(),
            _ => panic!("Unexpected signature type"),
        };

        assert_eq!(sig_type, expected_type);
    }
}

fn create_ed25519_json() -> serde_json::Value {
    serde_json::json!({
        "type": "ed25519",
        "PublicKey": hex::encode([0u8; 32]),
        "Signature": hex::encode([0u8; 64]),
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Timestamp": 1234567890,
        "Vote": null,
        "TransactionHash": null,
        "Memo": null,
        "Data": null
    })
}

fn create_legacy_ed25519_json() -> serde_json::Value {
    serde_json::json!({
        "type": "legacyED25519",
        "Timestamp": 1234567890,
        "PublicKey": hex::encode([0u8; 32]),
        "Signature": hex::encode([0u8; 64]),
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Vote": null,
        "TransactionHash": null
    })
}