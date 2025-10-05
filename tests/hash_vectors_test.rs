use accumulate_client::codec::{canonical_json, sha256_bytes, sha256_hex};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load transaction signing vectors from golden fixtures
fn load_transaction_vectors() -> Value {
    let vectors_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("tx_signing_vectors.json");

    let content = fs::read_to_string(vectors_path)
        .expect("Failed to read tx_signing_vectors.json fixture");

    serde_json::from_str(&content).expect("Failed to parse transaction signing vectors")
}

#[test]
fn test_sha256_hash_vectors() {
    let vectors_data = load_transaction_vectors();
    let vectors = vectors_data["vectors"].as_array().unwrap();

    for vector in vectors {
        let name = vector["name"].as_str().unwrap();
        let transaction = &vector["transaction"];
        let expected_hash = vector["txHash"].as_str().unwrap();

        // Generate canonical JSON and hash it
        let canonical = canonical_json(transaction);
        let computed_hash = hex::encode(sha256_bytes(canonical.as_bytes()));

        assert_eq!(
            computed_hash, expected_hash,
            "Hash mismatch for vector '{}'\nCanonical: {}\nExpected: {}\nComputed: {}",
            name, canonical, expected_hash, computed_hash
        );

        println!("✓ Hash vector '{}' matches: {}", name, computed_hash);
    }
}

#[test]
fn test_sha256_json_helper() {
    let vectors_data = load_transaction_vectors();
    let vectors = vectors_data["vectors"].as_array().unwrap();

    for vector in vectors {
        let name = vector["name"].as_str().unwrap();
        let transaction = &vector["transaction"];
        let expected_hash = vector["txHash"].as_str().unwrap();

        // Use the helper function
        let computed_hash = sha256_hex(transaction);

        assert_eq!(
            computed_hash, expected_hash,
            "Helper hash mismatch for vector '{}'\nExpected: {}\nComputed: {}",
            name, expected_hash, computed_hash
        );

        // Verify consistency with direct sha256_hex call
        let direct_hash = sha256_hex(transaction);
        assert_eq!(direct_hash, expected_hash);

        println!("✓ Helper hash vector '{}' matches: {}", name, computed_hash);
    }
}

#[test]
fn test_sha256_consistency() {
    // Test actual SHA-256 of raw strings (not JSON)
    let test_data = [
        ("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        ("test", "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"),
    ];

    for (input, expected) in test_data {
        let computed = hex::encode(sha256_bytes(input.as_bytes()));
        assert_eq!(computed, expected);

        // Verify byte-level consistency
        let direct_bytes_hash = hex::encode(sha256_bytes(input.as_bytes()));
        assert_eq!(direct_bytes_hash, expected);

        println!("✓ SHA-256('{}') = {}", input, computed);
    }
}

#[test]
fn test_hash_deterministic() {
    let input_json = serde_json::json!({
        "header": {
            "principal": "acc://alice.acme/tokens",
            "timestamp": 1234567890123u64
        },
        "body": {
            "type": "send-tokens",
            "to": [{
                "url": "acc://bob.acme/tokens",
                "amount": "1000"
            }]
        }
    });

    // Generate hash multiple times
    let hash1 = sha256_hex(&input_json);
    let hash2 = sha256_hex(&input_json);
    let hash3 = sha256_hex(&input_json);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1, hash3);

    // Should match the expected hash from fixture
    let expected = "4be49c59c717f1984646998cecac0e5225378d9bbe2e18928272a85b7dfcb608";
    assert_eq!(hash1, expected);

    println!("✓ Deterministic hash: {}", hash1);
}

#[test]
fn test_empty_and_null_hashing() {
    // Test hashing of empty JSON structures (using actual computed values)
    let test_cases = [
        serde_json::json!({}),
        serde_json::json!([]),
        serde_json::json!(null),
        serde_json::json!(""),
    ];

    for input in test_cases {
        let computed = sha256_hex(&input);

        // Verify deterministic behavior
        let computed2 = sha256_hex(&input);
        assert_eq!(computed, computed2, "Hash should be deterministic for input: {}", input);

        // Verify proper length (64 hex chars = 32 bytes)
        assert_eq!(computed.len(), 64, "Hash should be 64 hex characters");

        println!("✓ Hash of {} = {}", input, computed);
    }
}

#[test]
fn test_hash_ordering_independence() {
    // These should produce the same hash due to canonical ordering
    let input1 = serde_json::json!({"z": 3, "a": 1, "m": 2});
    let input2 = serde_json::json!({"a": 1, "m": 2, "z": 3});
    let input3 = serde_json::json!({"m": 2, "z": 3, "a": 1});

    let hash1 = sha256_hex(&input1);
    let hash2 = sha256_hex(&input2);
    let hash3 = sha256_hex(&input3);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1, hash3);

    println!("✓ Order-independent hashing: {}", hash1);
}

#[test]
fn test_nested_object_hashing() {
    let complex_input = serde_json::json!({
        "envelope": {
            "signatures": [{
                "type": "ed25519",
                "publicKey": "abcd1234",
                "signature": "ef567890"
            }],
            "transaction": [{
                "header": {
                    "principal": "acc://test.acme"
                },
                "body": {
                    "type": "sendTokens",
                    "amount": "100"
                }
            }]
        }
    });

    let hash = sha256_hex(&complex_input);
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars

    // Verify deterministic
    let hash2 = sha256_hex(&complex_input);
    assert_eq!(hash, hash2);

    println!("✓ Complex nested object hash: {}", hash);
}

#[test]
fn test_hash_bytes_vs_hex() {
    let input = serde_json::json!({"test": "value"});

    let canonical = canonical_json(&input);
    let hash_bytes = sha256_bytes(canonical.as_bytes());
    let hash_hex = hex::encode(hash_bytes);

    let direct_hex = sha256_hex(&input);

    assert_eq!(hash_hex, direct_hex);
    assert_eq!(hash_bytes.len(), 32);
    assert_eq!(hash_hex.len(), 64);

    println!("✓ Bytes vs hex consistency: {}", hash_hex);
}