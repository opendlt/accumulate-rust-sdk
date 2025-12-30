// Signature dispatch and roundtrip tests for all 16 signature types

use serde_json;
use std::fs;
use std::path::PathBuf;
use accumulate_client::{Signature, AccSignature};

fn load_manifest() -> serde_json::Value {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("signatures_manifest.json");
    let content = fs::read_to_string(&manifest_path)
        .expect("Failed to read signatures manifest");
    serde_json::from_str(&content).expect("Failed to parse manifest JSON")
}

fn minimal_signature_json(wire: &str) -> serde_json::Value {
    use serde_json::json;
    use hex;

    match wire {
        "ed25519" => json!({
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
        }),
        "legacyED25519" => json!({
            "type": "legacyED25519",
            "Timestamp": 1234567890,
            "PublicKey": hex::encode([0u8; 32]),
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Vote": null,
            "TransactionHash": null
        }),
        "rcd1" => json!({
            "type": "rcd1",
            "PublicKey": hex::encode([0u8; 32]),
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        }),
        "btc" => json!({
            "type": "btc",
            "PublicKey": hex::encode([0u8; 33]),  // BTC compressed public key
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        }),
        "btcLegacy" => json!({
            "type": "btcLegacy",
            "PublicKey": hex::encode([0u8; 65]),  // BTC uncompressed public key
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        }),
        "eth" => json!({
            "type": "eth",
            "PublicKey": hex::encode([0u8; 65]),  // ETH uncompressed public key
            "Signature": hex::encode([0u8; 65]),  // ETH signature with recovery ID
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        }),
        "rsaSha256" => json!({
            "type": "rsaSha256",
            "PublicKey": hex::encode([0u8; 256]),  // RSA public key (2048-bit)
            "Signature": hex::encode([0u8; 256]),  // RSA signature
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        }),
        "ecdsaSha256" => json!({
            "type": "ecdsaSha256",
            "PublicKey": hex::encode([0u8; 33]),  // ECDSA compressed public key
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        }),
        "typedData" => json!({
            "type": "typedData",
            "PublicKey": hex::encode([0u8; 65]),
            "Signature": hex::encode([0u8; 65]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null,
            "ChainID": "1337"  // Test chain ID
        }),
        "receipt" => json!({
            "type": "receipt",
            "SourceNetwork": "acc://test.acme",
            "Proof": {
                "start": hex::encode([0u8; 32]),
                "startIndex": 0,
                "end": hex::encode([0u8; 32]),
                "endIndex": 1,
                "anchor": hex::encode([0u8; 32]),
                "entries": []
            },
            "TransactionHash": null
        }),
        "partition" => json!({
            "type": "partition",
            "SourceNetwork": "acc://source.acme",
            "DestinationNetwork": "acc://dest.acme",
            "SequenceNumber": 12345,
            "TransactionHash": null
        }),
        "signatureSet" => json!({
            "type": "signatureSet",
            "Vote": null,
            "Signer": "acc://test.acme/signer",
            "TransactionHash": null,
            "Signatures": [],  // Empty set for minimal case
            "Authority": "acc://test.acme/authority"
        }),
        "remote" => json!({
            "type": "remote",
            "Destination": "acc://remote.acme",
            "Signature": {
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
            },
            "Cause": [hex::encode([0u8; 32])]
        }),
        "delegated" => json!({
            "type": "delegated",
            "Signature": {
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
            },
            "Delegator": "acc://test.acme/delegator"
        }),
        "internal" => json!({
            "type": "internal",
            "Cause": hex::encode([0u8; 32]),
            "TransactionHash": hex::encode([0u8; 32])
        }),
        "authority" => json!({
            "type": "authority",
            "Origin": "acc://test.acme/origin",
            "Authority": "acc://test.acme/authority",
            "Vote": null,
            "TxID": "test-tx-id",
            "Cause": "test-cause",
            "Delegator": [],
            "Memo": null
        }),
        _ => panic!("Unmapped wire tag: {}", wire),
    }
}

#[test]
fn test_signature_dispatch_roundtrips() {
    let manifest = load_manifest();
    let signatures = manifest["signatures"].as_array().unwrap();

    for sig_info in signatures {
        let wire = sig_info["wire"].as_str().unwrap();
        let name = sig_info["name"].as_str().unwrap();

        println!("Testing signature roundtrip: {} ({})", name, wire);

        // Create minimal valid JSON
        let original_json = minimal_signature_json(wire);

        // Test deserialization
        let signature: Signature = match serde_json::from_value(original_json.clone()) {
            Ok(sig) => sig,
            Err(e) => panic!("Failed to deserialize signature {}: {}", wire, e)
        };

        // Verify wire tag matches
        assert_eq!(signature.wire_tag(), wire, "Wire tag mismatch for {}", name);

        // Test serialization back to JSON
        let serialized_json = match serde_json::to_value(&signature) {
            Ok(json) => json,
            Err(e) => panic!("Failed to serialize signature {}: {}", wire, e)
        };

        // Compare structure (allow for field reordering)
        assert_signatures_equivalent(&original_json, &serialized_json, wire);

        // Test second roundtrip
        let signature2: Signature = serde_json::from_value(serialized_json.clone()).unwrap();
        let serialized_json2 = serde_json::to_value(&signature2).unwrap();

        assert_eq!(serialized_json, serialized_json2, "Second roundtrip failed for {}", wire);
    }
}

fn assert_signatures_equivalent(original: &serde_json::Value, serialized: &serde_json::Value, wire: &str) {
    let orig_obj = original.as_object().unwrap();
    let ser_obj = serialized.as_object().unwrap();

    // Check that type matches
    assert_eq!(orig_obj["type"], ser_obj["type"], "Type mismatch for {}", wire);

    // Check all original fields are present
    for (key, orig_value) in orig_obj {
        match ser_obj.get(key) {
            Some(ser_value) => {
                if orig_value.is_null() {
                    // Allow null fields to be omitted entirely (serde skip_serializing_if)
                    if !ser_value.is_null() {
                        // Or check that they serialize to the same value
                        assert_eq!(orig_value, ser_value, "Field {} mismatch for {}", key, wire);
                    }
                } else {
                    assert_eq!(orig_value, ser_value, "Field {} mismatch for {}", key, wire);
                }
            }
            None => {
                // Field missing is OK if original was null
                assert!(orig_value.is_null(), "Non-null field {} missing for {}", key, wire);
            }
        }
    }
}

#[test]
fn test_signature_negative_cases() {
    // Test unknown signature type
    let unknown_sig = serde_json::json!({
        "type": "unknownSignatureType",
        "data": "test"
    });

    let result: Result<Signature, _> = serde_json::from_value(unknown_sig);
    assert!(result.is_err(), "Should reject unknown signature type");

    // Test missing type field
    let no_type = serde_json::json!({
        "PublicKey": "abcd",
        "Signature": "1234"
    });

    let result: Result<Signature, _> = serde_json::from_value(no_type);
    assert!(result.is_err(), "Should reject signature without type");

    // Test invalid JSON types
    let invalid_types = vec![
        serde_json::json!(123),        // number
        serde_json::json!("string"),   // string
        serde_json::json!([]),         // array
        serde_json::json!(true),       // boolean
        serde_json::json!(null),       // null
    ];

    for invalid in invalid_types {
        let result: Result<Signature, _> = serde_json::from_value(invalid);
        assert!(result.is_err(), "Should reject non-object JSON");
    }
}

#[test]
fn test_signature_golden() {
    // This test writes and compares golden vectors for each signature type
    let manifest = load_manifest();
    let signatures = manifest["signatures"].as_array().unwrap();

    let golden_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("signatures");

    // Create directory if it doesn't exist
    fs::create_dir_all(&golden_dir).unwrap();

    for sig_info in signatures {
        let wire = sig_info["wire"].as_str().unwrap();
        let golden_file = golden_dir.join(format!("{}.json", wire));

        // Create minimal signature
        let json_value = minimal_signature_json(wire);

        // Parse and re-serialize to ensure canonical formatting
        let signature: Signature = serde_json::from_value(json_value).unwrap();
        let canonical_json = serde_json::to_value(&signature).unwrap();
        let formatted_json = serde_json::to_string_pretty(&canonical_json).unwrap();

        // Check if we should update golden files (for first run)
        let should_update = std::env::var("UPDATE_GOLDENS").is_ok();

        if should_update || !golden_file.exists() {
            // Write golden file
            fs::write(&golden_file, &formatted_json).unwrap();
            println!("Updated golden file: {}", golden_file.display());
        } else {
            // Compare with existing golden file
            let existing_content = fs::read_to_string(&golden_file).unwrap();
            let existing_json: serde_json::Value = serde_json::from_str(&existing_content).unwrap();

            assert_eq!(canonical_json, existing_json,
                      "Golden vector mismatch for {}. Run with UPDATE_GOLDENS=1 to update.", wire);
        }
    }
}

#[test]
fn test_signature_trait_coverage() {
    // Test that AccSignature trait is implemented for all signature types
    let manifest = load_manifest();
    let signatures = manifest["signatures"].as_array().unwrap();

    for sig_info in signatures {
        let wire = sig_info["wire"].as_str().unwrap();

        // Create signature and test trait methods
        let json_value = minimal_signature_json(wire);
        let signature: Signature = serde_json::from_value(json_value).unwrap();

        // Test wire_tag method
        assert_eq!(signature.wire_tag(), wire);

        // Test that verify method can be called (don't test actual verification here)
        // That will be covered in Stage 1.4 with proper crypto vectors
        let test_message = b"test message for verification";

        match &signature {
            Signature::ED25519(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "ed25519");
            },
            Signature::LegacyED25519(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "legacyED25519");
            },
            Signature::RCD1(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "rcd1");
            },
            Signature::BTC(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "btc");
            },
            Signature::BTCLegacy(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "btcLegacy");
            },
            Signature::ETH(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "eth");
            },
            Signature::RsaSha256(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "rsaSha256");
            },
            Signature::EcdsaSha256(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "ecdsaSha256");
            },
            Signature::TypedData(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "typedData");
            },
            Signature::Receipt(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "receipt");
            },
            Signature::Partition(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "partition");
            },
            Signature::Set(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "signatureSet");
            },
            Signature::Remote(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "remote");
            },
            Signature::Delegated(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "delegated");
            },
            Signature::Internal(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "internal");
            },
            Signature::Authority(sig) => {
                let _result = sig.verify(test_message);
                assert_eq!(sig.sig_type(), "authority");
            },
        }
    }
}

#[test]
fn test_signature_count_validation() {
    let manifest = load_manifest();
    let count = manifest["counts"]["signatures"].as_u64().unwrap();

    assert_eq!(count, 16, "Expected exactly 16 signatures");

    let signatures = manifest["signatures"].as_array().unwrap();
    assert_eq!(signatures.len(), 16, "Manifest should contain 16 signatures");
}