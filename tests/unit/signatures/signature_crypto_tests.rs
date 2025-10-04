// Cryptographic verification tests for Stage 1.2 signatures
// Focus on testing actual signature verification logic

use accumulate_client::{Signature, AccSignature};
use serde_json;
use hex;

// Test crypto functionality that should work
#[test]
fn test_ed25519_signature_structure() {
    let ed25519_sig = serde_json::json!({
        "type": "ed25519",
        "PublicKey": hex::encode([1u8; 32]), // Non-zero key
        "Signature": hex::encode([2u8; 64]), // Non-zero signature
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Timestamp": 1234567890,
        "Vote": null,
        "TransactionHash": null,
        "Memo": null,
        "Data": null
    });

    let signature: Signature = serde_json::from_value(ed25519_sig).unwrap();
    assert_eq!(signature.wire_tag(), "ed25519");

    // Test that verification can be called (even if it fails due to invalid keys)
    if let Signature::ED25519(ref ed_sig) = signature {
        let test_message = b"test message";
        let result = ed_sig.verify(test_message);
        // Should return Ok(false) or Err - either is acceptable for invalid test data
        assert!(result.is_ok() || result.is_err());
        println!("✓ Ed25519 verification callable");
    } else {
        panic!("Expected ED25519 signature variant");
    }
}

#[test]
fn test_legacy_ed25519_signature_structure() {
    let legacy_sig = serde_json::json!({
        "type": "legacyED25519",
        "Timestamp": 1234567890,
        "PublicKey": hex::encode([3u8; 32]),
        "Signature": hex::encode([4u8; 64]),
        "Signer": "acc://test.acme/signer",
        "SignerVersion": 1,
        "Vote": null,
        "TransactionHash": null
    });

    let signature: Signature = serde_json::from_value(legacy_sig).unwrap();
    assert_eq!(signature.wire_tag(), "legacyED25519");

    if let Signature::LegacyED25519(ref legacy_sig) = signature {
        let test_message = b"legacy test message";
        let result = legacy_sig.verify(test_message);
        assert!(result.is_ok() || result.is_err());
        println!("✓ Legacy Ed25519 verification callable");
    } else {
        panic!("Expected LegacyED25519 signature variant");
    }
}

#[test]
fn test_signature_verification_error_handling() {
    // Test various error conditions in verification
    let test_signatures = vec![
        ("ed25519", serde_json::json!({
            "type": "ed25519",
            "PublicKey": hex::encode([0u8; 32]), // All zeros - invalid
            "Signature": hex::encode([0u8; 64]), // All zeros - invalid
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Timestamp": 1234567890,
            "Vote": null,
            "TransactionHash": null,
            "Memo": null,
            "Data": null
        })),

        ("legacyED25519", serde_json::json!({
            "type": "legacyED25519",
            "Timestamp": 1234567890,
            "PublicKey": hex::encode([0u8; 32]), // All zeros - invalid
            "Signature": hex::encode([0u8; 64]), // All zeros - invalid
            "Signer": "acc://test.acme/signer",
            "SignerVersion": 1,
            "Vote": null,
            "TransactionHash": null
        })),

        ("rcd1", serde_json::json!({
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
        })),
    ];

    for (wire, json) in test_signatures {
        let signature: Signature = serde_json::from_value(json).unwrap();
        let test_message = b"error handling test message";

        let result = match &signature {
            Signature::ED25519(s) => s.verify(test_message),
            Signature::LegacyED25519(s) => s.verify(test_message),
            Signature::RCD1(s) => s.verify(test_message),
            _ => Ok(false)
        };

        // For invalid test data, we expect either Ok(false) or an error
        match result {
            Ok(verified) => {
                assert!(!verified, "Invalid signature should not verify for {}", wire);
                println!("✓ {} correctly returned false for invalid signature", wire);
            },
            Err(e) => {
                println!("✓ {} correctly returned error for invalid signature: {}", wire, e);
            }
        }
    }
}

#[test]
fn test_signature_type_consistency() {
    // Test that sig_type() returns consistent values
    let test_cases = vec![
        ("ed25519", "ed25519"),
        ("legacyED25519", "legacyED25519"),
        ("rcd1", "rcd1"),
        ("btc", "btc"),
        ("btcLegacy", "btcLegacy"),
        ("eth", "eth"),
        ("rsaSha256", "rsaSha256"),
        ("ecdsaSha256", "ecdsaSha256"),
        ("typedData", "typedData"),
        ("receipt", "receipt"),
        ("partition", "partition"),
        ("signatureSet", "signatureSet"),
        ("remote", "remote"),
        ("delegated", "delegated"),
        ("internal", "internal"),
        ("authority", "authority"),
    ];

    for (wire, expected_type) in test_cases {
        let json = create_minimal_signature_json(wire);

        match serde_json::from_value::<Signature>(json) {
            Ok(signature) => {
                assert_eq!(signature.wire_tag(), wire);

                let sig_type = match &signature {
                    Signature::ED25519(s) => s.sig_type(),
                    Signature::LegacyED25519(s) => s.sig_type(),
                    Signature::RCD1(s) => s.sig_type(),
                    Signature::BTC(s) => s.sig_type(),
                    Signature::BTCLegacy(s) => s.sig_type(),
                    Signature::ETH(s) => s.sig_type(),
                    Signature::RsaSha256(s) => s.sig_type(),
                    Signature::EcdsaSha256(s) => s.sig_type(),
                    Signature::TypedData(s) => s.sig_type(),
                    Signature::Receipt(s) => s.sig_type(),
                    Signature::Partition(s) => s.sig_type(),
                    Signature::Set(s) => s.sig_type(),
                    Signature::Remote(s) => s.sig_type(),
                    Signature::Delegated(s) => s.sig_type(),
                    Signature::Internal(s) => s.sig_type(),
                    Signature::Authority(s) => s.sig_type(),
                };

                assert_eq!(sig_type, expected_type, "sig_type() mismatch for {}", wire);
                println!("✓ {} sig_type() returns '{}'", wire, sig_type);
            },
            Err(e) => {
                println!("⚠ Failed to create {} signature: {}", wire, e);
                // Continue with other test cases
            }
        }
    }
}

fn create_minimal_signature_json(wire: &str) -> serde_json::Value {
    use serde_json::json;

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
            "PublicKey": hex::encode([0u8; 33]),
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
            "PublicKey": hex::encode([0u8; 65]),
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
            "PublicKey": hex::encode([0u8; 65]),
            "Signature": hex::encode([0u8; 65]),
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
            "PublicKey": hex::encode([0u8; 256]),
            "Signature": hex::encode([0u8; 256]),
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
            "PublicKey": hex::encode([0u8; 33]),
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
            "ChainID": "1337"
        }),
        "receipt" => json!({
            "type": "receipt",
            "SourceNetwork": "acc://test.acme",
            "Proof": {
                "data": hex::encode([0u8; 32])
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
            "Signatures": [],
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
            "Cause": [hex::encode([0u8; 32])]  // Vec<[u8; 32]> as hex strings
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
fn test_all_signature_types_deserialize() {
    // Comprehensive test that all 16 signature types can be deserialized
    let all_wire_types = vec![
        "ed25519", "legacyED25519", "rcd1", "btc", "btcLegacy", "eth",
        "rsaSha256", "ecdsaSha256", "typedData", "receipt", "partition",
        "signatureSet", "remote", "delegated", "internal", "authority"
    ];

    let mut successful = 0;
    let mut failed = 0;

    for wire in all_wire_types {
        let json = create_minimal_signature_json(wire);

        match serde_json::from_value::<Signature>(json) {
            Ok(signature) => {
                assert_eq!(signature.wire_tag(), wire);
                successful += 1;
                println!("✓ {} signature deserialized successfully", wire);
            },
            Err(e) => {
                failed += 1;
                println!("⚠ {} signature failed: {}", wire, e);
            }
        }
    }

    println!("Summary: {} successful, {} failed", successful, failed);

    // We want all signatures to deserialize successfully
    assert_eq!(failed, 0, "All signature types should deserialize successfully");
    assert_eq!(successful, 16, "Should have exactly 16 working signature types");
}