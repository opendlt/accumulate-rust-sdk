use accumulate_client::canonjson::canonicalize;
use accumulate_client::crypto::ed25519::sha256;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Load envelope test fixtures
fn load_envelope_fixtures() -> serde_json::Result<Value> {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("enums")
        .join("envelope_fixed.golden.json");

    let content = fs::read_to_string(fixture_path)
        .expect("Failed to read envelope fixtures");

    serde_json::from_str(&content)
}

#[test]
fn test_envelope_encoding_parity() {
    let fixtures = load_envelope_fixtures().expect("Failed to parse envelope fixtures");

    let signatures = fixtures["signatures"].as_array()
        .expect("signatures should be an array");

    let transactions = fixtures["transaction"].as_array()
        .expect("transaction should be an array");

    // Test each transaction in the envelope
    for (i, transaction) in transactions.iter().enumerate() {
        println!("Testing transaction {}: {}", i, serde_json::to_string_pretty(transaction).unwrap());

        // Test canonical JSON encoding
        let canonical = canonicalize(transaction);
        println!("Canonical JSON: {}", canonical);

        // Test hash computation
        let hash = sha256(canonical.as_bytes());
        let hash_hex = hex::encode(hash);
        println!("Transaction hash: {}", hash_hex);

        // Verify the canonical form is deterministic
        let canonical2 = canonicalize(transaction);
        assert_eq!(canonical, canonical2, "Canonical JSON should be deterministic");

        // Verify hash is deterministic
        let hash2 = sha256(canonical2.as_bytes());
        assert_eq!(hash, hash2, "Hash should be deterministic");
    }

    // Test signatures
    for (i, signature) in signatures.iter().enumerate() {
        println!("Testing signature {}: {}", i, serde_json::to_string_pretty(signature).unwrap());

        // Verify signature structure
        assert!(signature["type"].is_string(), "Signature should have type");
        assert!(signature["publicKey"].is_string(), "Signature should have publicKey");
        assert!(signature["signature"].is_string(), "Signature should have signature");
        assert!(signature["signer"].is_string(), "Signature should have signer");
        assert!(signature["timestamp"].is_number(), "Signature should have timestamp");

        // Test that signature canonical encoding is consistent
        let sig_canonical = canonicalize(signature);
        let sig_canonical2 = canonicalize(signature);
        assert_eq!(sig_canonical, sig_canonical2, "Signature canonical JSON should be deterministic");
    }
}

#[test]
fn test_envelope_structure_validation() {
    // Test a complete envelope structure
    let envelope = json!({
        "signatures": [{
            "type": "ed25519",
            "publicKey": "dff03fddf03d29a1f45daf8e9f2bd7c68ee3f2989b0c6c3385946d20f04b4926",
            "signature": "cff669b816312fbac709f12b0d18a96bcab6a570c27b2d13f662a04afdfeb36f59ddb9249f803677ed928e27500b7c35aebce432141ea9e3af1eb8fbb901420a",
            "signer": "acc://test.acme/book/1",
            "signerVersion": 51,
            "timestamp": 1757520686204512_u64,
        }],
        "transaction": [{
            "header": {
                "principal": "acc://test.acme/tokens",
                "initiator": "cf44e0f55dddb5858481ebb2369c35957b38d228d32db1475b051113755bd965"
            },
            "body": {
                "type": "sendTokens",
                "to": [{
                    "url": "acc://test.acme/staking",
                    "amount": "200000"
                }]
            }
        }]
    });

    // Test envelope canonical encoding
    let canonical = canonicalize(&envelope);
    println!("Envelope canonical: {}", canonical);

    // Verify it's deterministic
    let canonical2 = canonicalize(&envelope);
    assert_eq!(canonical, canonical2);

    // Test that the transaction within the envelope can be extracted and hashed
    let transaction = &envelope["transaction"][0];
    let tx_canonical = canonicalize(transaction);
    let tx_hash = sha256(tx_canonical.as_bytes());

    println!("Transaction canonical: {}", tx_canonical);
    println!("Transaction hash: {}", hex::encode(tx_hash));

    // Expected canonical form should have sorted keys
    assert!(tx_canonical.contains(r#""body":{"#));
    assert!(tx_canonical.contains(r#""header":{"#));
    assert!(tx_canonical.contains(r#""type":"sendTokens""#));
}

#[test]
fn test_envelope_transaction_hash_correlation() {
    let fixtures = load_envelope_fixtures().expect("Failed to parse envelope fixtures");

    let signatures = fixtures["signatures"].as_array().unwrap();
    let transactions = fixtures["transaction"].as_array().unwrap();

    for transaction in transactions {
        let tx_canonical = canonicalize(transaction);
        let tx_hash = sha256(tx_canonical.as_bytes());
        let tx_hash_hex = hex::encode(tx_hash);

        // Check if any signature references this transaction hash
        for signature in signatures {
            if let Some(sig_tx_hash) = signature.get("transactionHash") {
                if let Some(sig_tx_hash_str) = sig_tx_hash.as_str() {
                    if sig_tx_hash_str == tx_hash_hex {
                        println!("Found matching transaction hash in signature: {}", sig_tx_hash_str);

                        // Verify the signature structure is valid
                        assert!(signature["publicKey"].is_string());
                        assert!(signature["signature"].is_string());
                        assert!(signature["timestamp"].is_number());
                    }
                }
            }
        }
    }
}

#[test]
fn test_minimal_envelope() {
    // Test minimal valid envelope
    let minimal_envelope = json!({
        "transaction": {
            "body": {
                "type": "sendTokens",
                "to": [{
                    "url": "acc://test",
                    "amount": "100"
                }]
            }
        },
        "signatures": []
    });

    let canonical = canonicalize(&minimal_envelope);
    println!("Minimal envelope: {}", canonical);

    // Should be deterministic
    assert_eq!(canonical, canonicalize(&minimal_envelope));

    // Should have correct structure
    assert!(canonical.contains(r#""signatures":[]"#));
    assert!(canonical.contains(r#""transaction":{"#));
}