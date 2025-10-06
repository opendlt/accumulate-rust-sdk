use accumulate_client::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_delegated_depth6_fail_golden_vector() {
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("signatures")
        .join("delegated_depth6_fail.json");

    let content = fs::read_to_string(&golden_path)
        .expect("Failed to read delegated_depth6_fail.json golden vector");

    // Deserialize the signature
    let signature: Signature = serde_json::from_str(&content)
        .expect("Failed to deserialize delegated signature");

    // Verify it's a delegated signature
    if let Signature::Delegated(_) = signature {
        // Check that the depth is 6
        assert_eq!(delegated_depth(&signature), 6, "Expected delegation depth of 6");

        // Verify that enforce_delegated_depth fails
        let result = enforce_delegated_depth(&signature);
        assert!(result.is_err(), "Depth 6 should exceed limit");
        assert!(matches!(result.unwrap_err(), SigRuntimeError::DelegationDepthExceeded));
    } else {
        panic!("Expected delegated signature");
    }
}

#[test]
fn test_signature_set_threshold_golden_vector() {
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("signatures")
        .join("signature_set_threshold.json");

    let content = fs::read_to_string(&golden_path)
        .expect("Failed to read signature_set_threshold.json golden vector");

    // Deserialize the signature set
    let signature: Signature = serde_json::from_str(&content)
        .expect("Failed to deserialize signature set");

    // Verify it's a signature set
    if let Signature::Set(sig_set) = signature {
        // Check that it has 3 signatures
        assert_eq!(sig_set.signatures.len(), 3, "Expected 3 signatures in set");

        // Create a threshold wrapper with threshold 2
        let set_with_threshold = SignatureSetWithThreshold::new(sig_set, 2)
            .expect("Should create valid threshold set");

        // Test evaluation - should fail since signatures verify as false
        let message = b"test message";
        let result = evaluate_signature_set(&set_with_threshold, message)
            .expect("Should evaluate without error");

        // Should be false since we have threshold=2 but only signatures that verify false
        assert_eq!(result, false, "Should fail with threshold 2 and all false signatures");

        // Test with threshold 1 - should still fail since all signatures are false
        let set_with_threshold = SignatureSetWithThreshold::new(
            serde_json::from_str(&content).unwrap(),
            1
        ).expect("Should create valid threshold set");

        let result = evaluate_signature_set(&set_with_threshold, message)
            .expect("Should evaluate without error");
        assert_eq!(result, false, "Should fail even with threshold 1 since all signatures are false");
    } else {
        panic!("Expected signature set");
    }
}

#[test]
fn test_golden_vector_roundtrip() {
    // Test that we can serialize and deserialize our golden vectors
    let vectors = [
        "delegated_depth6_fail.json",
        "signature_set_threshold.json",
    ];

    for vector_file in &vectors {
        let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("golden")
            .join("signatures")
            .join(vector_file);

        let content = fs::read_to_string(&golden_path)
            .expect(&format!("Failed to read {}", vector_file));

        // Deserialize
        let signature: Signature = serde_json::from_str(&content)
            .expect(&format!("Failed to deserialize {}", vector_file));

        // Serialize back
        let serialized = serde_json::to_string_pretty(&signature)
            .expect(&format!("Failed to serialize {}", vector_file));

        // Deserialize again to verify roundtrip
        let signature2: Signature = serde_json::from_str(&serialized)
            .expect(&format!("Failed to deserialize roundtrip {}", vector_file));

        // Verify they have the same wire tag
        assert_eq!(signature.wire_tag(), signature2.wire_tag(),
                   "Wire tags should match after roundtrip for {}", vector_file);
    }
}

#[test]
fn test_delegated_depth_with_various_inner_types() {
    // Test depth calculation with different inner signature types
    let inner_types = vec![
        serde_json::json!({
            "type": "internal",
            "Cause": "0000000000000000000000000000000000000000000000000000000000000000",
            "TransactionHash": "0000000000000000000000000000000000000000000000000000000000000000"
        }),
        serde_json::json!({
            "type": "partition",
            "SourceNetwork": "acc://source.acme",
            "DestinationNetwork": "acc://dest.acme",
            "SequenceNumber": 12345,
            "TransactionHash": null
        }),
        serde_json::json!({
            "type": "ed25519",
            "PublicKey": "0000000000000000000000000000000000000000000000000000000000000000",
            "Signature": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        }),
    ];

    for (i, inner) in inner_types.iter().enumerate() {
        let delegated = serde_json::json!({
            "type": "delegated",
            "Signature": inner,
            "Delegator": format!("acc://delegator-{}.acme", i)
        });

        let signature: Signature = serde_json::from_value(delegated)
            .expect(&format!("Should deserialize delegated with inner type {}", i));

        assert_eq!(delegated_depth(&signature), 1,
                   "Delegation depth should be 1 for inner type {}", i);
        assert!(enforce_delegated_depth(&signature).is_ok(),
                "Should pass depth check for inner type {}", i);
    }
}