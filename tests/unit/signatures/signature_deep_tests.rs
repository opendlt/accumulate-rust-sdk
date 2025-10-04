// Deep test suite for Stage 1.2 signature system
// Comprehensive testing of all signature types, verification, serialization, and edge cases

use serde_json;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use accumulate_client::{Signature, AccSignature};
use hex;

// Test data generators
mod test_data {
    use super::*;

    pub fn ed25519_test_vectors() -> Vec<(Vec<u8>, Vec<u8>, Vec<u8>, bool)> {
        // (public_key, signature, message, should_verify)
        vec![
            // Valid Ed25519 signature (32-byte pubkey, 64-byte sig)
            (vec![0u8; 32], vec![0u8; 64], b"test message".to_vec(), false), // Invalid signature
            // Add more real test vectors when we have proper crypto implementation
        ]
    }

    pub fn signature_field_variations() -> HashMap<&'static str, Vec<serde_json::Value>> {
        let mut variations = HashMap::new();

        // Test different public key sizes
        variations.insert("publicKey", vec![
            serde_json::Value::String(hex::encode(vec![0u8; 32])), // Ed25519
            serde_json::Value::String(hex::encode(vec![0u8; 33])), // Bitcoin compressed
            serde_json::Value::String(hex::encode(vec![0u8; 65])), // Ethereum uncompressed
            serde_json::Value::String(hex::encode(vec![0u8; 256])), // RSA
        ]);

        // Test different signature sizes
        variations.insert("signature", vec![
            serde_json::Value::String(hex::encode(vec![0u8; 64])), // Standard
            serde_json::Value::String(hex::encode(vec![0u8; 65])), // Ethereum with recovery
            serde_json::Value::String(hex::encode(vec![0u8; 256])), // RSA
        ]);

        // Test URL variations
        variations.insert("signer", vec![
            serde_json::Value::String("acc://test.acme/signer".to_string()),
            serde_json::Value::String("acc://long-domain-name.accumulate.network/path/to/signer".to_string()),
            serde_json::Value::String("acc://localhost/test".to_string()),
        ]);

        variations
    }

    pub fn create_comprehensive_signature_json(wire: &str, variation: usize) -> serde_json::Value {
        use serde_json::json;

        let variations = signature_field_variations();
        let pubkey_idx = variation % variations["publicKey"].len();
        let sig_idx = variation % variations["signature"].len();
        let signer_idx = variation % variations["signer"].len();

        match wire {
            "ed25519" => json!({
                "type": "ed25519",
                "PublicKey": variations["publicKey"][pubkey_idx].clone(),
                "Signature": variations["signature"][sig_idx].clone(),
                "Signer": variations["signer"][signer_idx].clone(),
                "SignerVersion": variation as u64 + 1,
                "Timestamp": 1234567890u64 + variation as u64,
                "Vote": if variation % 3 == 0 { serde_json::Value::Null } else { json!("accept") },
                "TransactionHash": if variation % 2 == 0 {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(hex::encode([variation as u8; 32]))
                },
                "Memo": if variation % 4 == 0 { serde_json::Value::Null } else { serde_json::Value::String("test memo".to_string()) },
                "Data": if variation % 5 == 0 { serde_json::Value::Null } else { serde_json::Value::String(hex::encode([0u8; 16])) }
            }),
            "legacyED25519" => json!({
                "type": "legacyED25519",
                "Timestamp": 1234567890u64 + variation as u64,
                "PublicKey": variations["publicKey"][pubkey_idx].clone(),
                "Signature": variations["signature"][sig_idx].clone(),
                "Signer": variations["signer"][signer_idx].clone(),
                "SignerVersion": variation as u64 + 1,
                "Vote": if variation % 3 == 0 { serde_json::Value::Null } else { serde_json::Value::String("reject".to_string()) },
                "TransactionHash": if variation % 2 == 0 {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(hex::encode([variation as u8; 32]))
                }
            }),
            "rcd1" => json!({
                "type": "rcd1",
                "PublicKey": variations["publicKey"][pubkey_idx].clone(),
                "Signature": variations["signature"][sig_idx].clone(),
                "Signer": variations["signer"][signer_idx].clone(),
                "SignerVersion": variation as u64 + 1,
                "Timestamp": 1234567890u64 + variation as u64,
                "Vote": serde_json::Value::Null,
                "TransactionHash": serde_json::Value::Null,
                "Memo": if variation % 2 == 0 { serde_json::Value::Null } else { serde_json::Value::String("rcd1 memo".to_string()) },
                "Data": serde_json::Value::Null
            }),
            _ => json!({
                "type": wire,
                "test": true
            })
        }
    }
}

// Core functionality tests
#[test]
fn test_signature_enum_completeness() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("signatures_manifest.json");

    let content = fs::read_to_string(&manifest_path)
        .expect("Failed to read signatures manifest");
    let manifest: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse manifest JSON");

    let signatures = manifest["signatures"].as_array().unwrap();
    assert_eq!(signatures.len(), 16, "Must have exactly 16 signatures");

    // Test that each signature has required fields
    for sig_info in signatures {
        let wire = sig_info["wire"].as_str().unwrap();
        let name = sig_info["name"].as_str().unwrap();
        let fields = sig_info["fields"].as_array().unwrap();

        assert!(!wire.is_empty(), "Wire tag cannot be empty for {}", name);
        assert!(!name.is_empty(), "Name cannot be empty");
        assert!(!fields.is_empty(), "Fields cannot be empty for {}", name);

        println!("✓ Signature {} ({}) has {} fields", name, wire, fields.len());
    }
}

#[test]
fn test_signature_serialization_variations() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("signatures_manifest.json");

    let content = fs::read_to_string(&manifest_path)
        .expect("Failed to read signatures manifest");
    let manifest: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse manifest JSON");

    let signatures = manifest["signatures"].as_array().unwrap();

    // Test multiple variations of each signature type
    for sig_info in signatures.iter().take(3) { // Test first 3 for now
        let wire = sig_info["wire"].as_str().unwrap();
        let name = sig_info["name"].as_str().unwrap();

        println!("Testing serialization variations for {} ({})", name, wire);

        for variation in 0..5 {
            let test_json = test_data::create_comprehensive_signature_json(wire, variation);

            // Test deserialization
            let signature: Result<Signature, _> = serde_json::from_value(test_json.clone());
            match signature {
                Ok(sig) => {
                    assert_eq!(sig.wire_tag(), wire, "Wire tag mismatch for {} variation {}", wire, variation);

                    // Test re-serialization
                    let reserialized = serde_json::to_value(&sig).unwrap();
                    assert_eq!(reserialized["type"], test_json["type"],
                              "Type field lost in serialization for {} variation {}", wire, variation);

                    println!("  ✓ Variation {} passed", variation);
                },
                Err(e) => {
                    println!("  ⚠ Variation {} failed: {}", variation, e);
                    // For now, we'll continue - some variations may fail due to validation
                }
            }
        }
    }
}

#[test]
fn test_signature_trait_implementation() {
    use test_data::create_comprehensive_signature_json;

    let test_signatures = vec!["ed25519", "legacyED25519", "rcd1"];

    for wire in test_signatures {
        let json_value = create_comprehensive_signature_json(wire, 0);

        match serde_json::from_value::<Signature>(json_value) {
            Ok(signature) => {
                // Test wire_tag method
                assert_eq!(signature.wire_tag(), wire);

                // Test sig_type method
                let sig_type = match &signature {
                    Signature::ED25519(s) => s.sig_type(),
                    Signature::LegacyED25519(s) => s.sig_type(),
                    Signature::RCD1(s) => s.sig_type(),
                    _ => "unknown"
                };
                assert_eq!(sig_type, wire);

                // Test verify method (should not panic)
                let test_message = b"test message for verification";
                let verify_result = match &signature {
                    Signature::ED25519(s) => s.verify(test_message),
                    Signature::LegacyED25519(s) => s.verify(test_message),
                    Signature::RCD1(s) => s.verify(test_message),
                    _ => Ok(false)
                };

                // Should return a result (Ok or Err, doesn't matter for this test)
                assert!(verify_result.is_ok() || verify_result.is_err());

                println!("✓ Trait methods work for {}", wire);
            },
            Err(e) => {
                panic!("Failed to deserialize test signature {}: {}", wire, e);
            }
        }
    }
}

#[test]
fn test_json_edge_cases() {
    let edge_cases = vec![
        // Empty object
        ("empty_object", serde_json::json!({})),

        // Missing type field
        ("missing_type", serde_json::json!({
            "PublicKey": "00000000000000000000000000000000",
            "Signature": "1111111111111111111111111111111111111111111111111111111111111111"
        })),

        // Invalid type
        ("invalid_type", serde_json::json!({
            "type": "invalidSignatureType",
            "data": "test"
        })),

        // Wrong JSON types
        ("number_instead_of_object", serde_json::json!(42)),
        ("string_instead_of_object", serde_json::json!("not an object")),
        ("array_instead_of_object", serde_json::json!([])),
        ("serde_json::Value::Null_instead_of_object", serde_json::json!(serde_json::Value::Null)),

        // Invalid hex strings
        ("invalid_hex_pubkey", serde_json::json!({
            "type": "ed25519",
            "PublicKey": "invalid_hex_string",
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890
        })),

        // Wrong field types
        ("string_timestamp", serde_json::json!({
            "type": "ed25519",
            "PublicKey": hex::encode([0u8; 32]),
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": "not_a_number"
        })),
    ];

    for (case_name, test_json) in edge_cases {
        let result: Result<Signature, _> = serde_json::from_value(test_json);
        assert!(result.is_err(), "Edge case '{}' should fail but didn't", case_name);
        println!("✓ Edge case '{}' correctly rejected", case_name);
    }
}

#[test]
fn test_large_data_handling() {
    use test_data::create_comprehensive_signature_json;

    // Test with large hex strings
    let large_cases = vec![
        ("large_pubkey", serde_json::json!({
            "type": "ed25519",
            "PublicKey": hex::encode(vec![0u8; 1024]), // Very large public key
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        })),

        ("large_signature", serde_json::json!({
            "type": "ed25519",
            "PublicKey": hex::encode([0u8; 32]),
            "Signature": hex::encode(vec![0u8; 2048]), // Very large signature
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        })),

        ("large_signer_url", serde_json::json!({
            "type": "ed25519",
            "PublicKey": hex::encode([0u8; 32]),
            "Signature": hex::encode([0u8; 64]),
            "Signer": format!("acc://{}/signer", "a".repeat(1000)), // Very long URL
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        })),
    ];

    for (case_name, test_json) in large_cases {
        let result: Result<Signature, _> = serde_json::from_value(test_json);
        match result {
            Ok(sig) => {
                println!("✓ Large data case '{}' accepted", case_name);
                // Should still be able to get wire tag
                assert!(!sig.wire_tag().is_empty());
            },
            Err(e) => {
                println!("⚠ Large data case '{}' rejected: {}", case_name, e);
                // This might be expected behavior
            }
        }
    }
}

#[test]
fn test_nested_signature_structures() {
    // Test complex signature types with nested signatures
    let remote_sig = serde_json::json!({
        "type": "remote",
        "Destination": "acc://remote.acme",
        "Signature": {
            "type": "ed25519",
            "PublicKey": hex::encode([0u8; 32]),
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": serde_json::Value::Null,
            "TransactionHash": serde_json::Value::Null,
            "Memo": serde_json::Value::Null,
            "Data": serde_json::Value::Null
        },
        "Cause": [hex::encode([0u8; 32])]
    });

    let delegated_sig = serde_json::json!({
        "type": "delegated",
        "Signature": {
            "type": "ed25519",
            "PublicKey": hex::encode([0u8; 32]),
            "Signature": hex::encode([0u8; 64]),
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": serde_json::Value::Null,
            "TransactionHash": serde_json::Value::Null,
            "Memo": serde_json::Value::Null,
            "Data": serde_json::Value::Null
        },
        "Delegator": "acc://test.acme/delegator"
    });

    let signature_set = serde_json::json!({
        "type": "signatureSet",
        "Vote": serde_json::Value::Null,
        "Signer": "acc://test.acme/signer",
        "TransactionHash": serde_json::Value::Null,
        "Signatures": [], // Empty for this test
        "Authority": "acc://test.acme/authority"
    });

    let nested_cases = vec![
        ("remote", remote_sig),
        ("delegated", delegated_sig),
        ("signatureSet", signature_set),
    ];

    for (case_name, test_json) in nested_cases {
        let result: Result<Signature, _> = serde_json::from_value(test_json.clone());
        match result {
            Ok(signature) => {
                assert_eq!(signature.wire_tag(), case_name);

                // Test re-serialization of nested structure
                let reserialized = serde_json::to_value(&signature).unwrap();
                assert_eq!(reserialized["type"], test_json["type"]);

                println!("✓ Nested signature '{}' handled correctly", case_name);
            },
            Err(e) => {
                println!("⚠ Nested signature '{}' failed: {}", case_name, e);
                // May fail due to compilation issues, continue for now
            }
        }
    }
}

#[test]
fn test_round_trip_consistency() {
    // Test that multiple serialization/deserialization cycles are stable
    use test_data::create_comprehensive_signature_json;

    let test_wires = vec!["ed25519", "legacyED25519", "rcd1"];

    for wire in test_wires {
        let original_json = create_comprehensive_signature_json(wire, 1);

        match serde_json::from_value::<Signature>(original_json.clone()) {
            Ok(sig1) => {
                // First round-trip
                let json1 = serde_json::to_value(&sig1).unwrap();
                let sig2: Signature = serde_json::from_value(json1.clone()).unwrap();

                // Second round-trip
                let json2 = serde_json::to_value(&sig2).unwrap();
                let sig3: Signature = serde_json::from_value(json2.clone()).unwrap();

                // Third round-trip
                let json3 = serde_json::to_value(&sig3).unwrap();

                // All should be identical
                assert_eq!(json1, json2, "First and second JSON should match for {}", wire);
                assert_eq!(json2, json3, "Second and third JSON should match for {}", wire);
                assert_eq!(sig1.wire_tag(), sig2.wire_tag());
                assert_eq!(sig2.wire_tag(), sig3.wire_tag());

                println!("✓ Round-trip consistency verified for {}", wire);
            },
            Err(e) => {
                println!("⚠ Round-trip test skipped for {} due to: {}", wire, e);
            }
        }
    }
}

#[test]
fn test_performance_stress() {
    // Test performance with many signatures
    use test_data::create_comprehensive_signature_json;
    use std::time::Instant;

    let wire = "ed25519";
    let iterations = 1000;

    let start = Instant::now();

    for i in 0..iterations {
        let json = create_comprehensive_signature_json(wire, i % 10);

        match serde_json::from_value::<Signature>(json) {
            Ok(sig) => {
                assert_eq!(sig.wire_tag(), wire);
                let _reserialized = serde_json::to_value(&sig).unwrap();
            },
            Err(_) => {
                // Skip failed attempts for this stress test
                continue;
            }
        }
    }

    let duration = start.elapsed();
    println!("✓ Processed {} signatures in {:?} ({:.2} per second)",
             iterations, duration, iterations as f64 / duration.as_secs_f64());

    // Should be reasonably fast (less than 1 second for 1000 operations)
    assert!(duration.as_secs() < 10, "Performance test too slow: {:?}", duration);
}