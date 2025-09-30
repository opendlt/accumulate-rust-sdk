use accumulate_client::{Ed25519Signer, verify, AccumulateHash, canonical_json};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Load ed25519 signature test fixtures
fn load_ed25519_fixtures() -> serde_json::Result<Value> {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("sig_ed25519.golden.json");

    let content = fs::read_to_string(fixture_path)
        .expect("Failed to read sig_ed25519.golden.json fixture");

    serde_json::from_str(&content)
}

/// Load transaction signing vector fixtures
fn load_tx_signing_vectors() -> serde_json::Result<Value> {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("tx_signing_vectors.json");

    let content = fs::read_to_string(fixture_path)
        .expect("Failed to read tx_signing_vectors.json fixture");

    serde_json::from_str(&content)
}

/// Convert hex string to byte array
fn hex_to_bytes<const N: usize>(hex: &str) -> [u8; N] {
    let bytes = hex::decode(hex).expect("Invalid hex string");
    let mut array = [0u8; N];
    array.copy_from_slice(&bytes[..N]);
    array
}

/// Convert byte array to hex string
fn bytes_to_hex<const N: usize>(bytes: &[u8; N]) -> String {
    hex::encode(bytes)
}

#[test]
fn test_ed25519_signature_parity() {
    let fixtures = load_ed25519_fixtures().expect("Failed to parse ed25519 fixtures");

    let public_key_hex = fixtures["publicKey"].as_str().unwrap();
    let signature_hex = fixtures["signature"].as_str().unwrap();
    let message = fixtures["message"].as_str().unwrap();
    let message_hash_hex = fixtures["messageHash"].as_str().unwrap();

    let public_key: [u8; 32] = hex_to_bytes(public_key_hex);
    let signature: [u8; 64] = hex_to_bytes(signature_hex);
    let message_hash: [u8; 32] = hex_to_bytes(message_hash_hex);

    // Test message verification
    let is_valid = verify(&public_key, message.as_bytes(), &signature);
    assert!(is_valid, "Ed25519 signature verification failed for message");

    // Test hash verification (if the signature was generated from the hash)
    let computed_hash = AccumulateHash::sha256_bytes(message.as_bytes());
    assert_eq!(
        bytes_to_hex(&computed_hash),
        message_hash_hex,
        "Message hash doesn't match fixture"
    );

    println!("✓ Ed25519 signature parity test passed");
}

#[test]
fn test_transaction_signing_vectors_parity() {
    let vectors = load_tx_signing_vectors().expect("Failed to parse signing vectors");

    let test_vectors = vectors["vectors"].as_array().unwrap();

    for vector in test_vectors {
        let name = vector["name"].as_str().unwrap();
        let private_key_hex = vector["privateKey"].as_str().unwrap();
        let public_key_hex = vector["publicKey"].as_str().unwrap();
        let transaction = &vector["transaction"];
        let expected_canonical = vector["canonicalJSON"].as_str().unwrap();
        let expected_tx_hash = vector["txHash"].as_str().unwrap();
        let expected_signature_hex = vector["signature"].as_str().unwrap();

        // Parse keys
        let private_key: [u8; 32] = hex_to_bytes(private_key_hex);
        let expected_public_key: [u8; 32] = hex_to_bytes(public_key_hex);
        let expected_signature: [u8; 64] = hex_to_bytes(expected_signature_hex);

        // Test key derivation
        let signer = Ed25519Signer::from_seed(&private_key)
            .expect("Failed to create signer from seed");

        let derived_public_key = signer.public_key_bytes();
        assert_eq!(
            bytes_to_hex(&derived_public_key),
            public_key_hex,
            "Public key derivation mismatch for vector '{}'",
            name
        );

        // Test canonical JSON generation
        let actual_canonical = canonical_json(transaction);
        assert_eq!(
            actual_canonical,
            expected_canonical,
            "Canonical JSON mismatch for vector '{}'\nExpected: {}\nActual: {}",
            name,
            expected_canonical,
            actual_canonical
        );

        // Test transaction hash
        let tx_hash = AccumulateHash::sha256_json(transaction);
        let tx_hash_hex = bytes_to_hex(&tx_hash);
        assert_eq!(
            tx_hash_hex,
            expected_tx_hash,
            "Transaction hash mismatch for vector '{}'\nExpected: {}\nActual: {}",
            name,
            expected_tx_hash,
            tx_hash_hex
        );

        // Test signature generation (if not zero signature)
        if expected_signature_hex != "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" {
            let signature = signer.sign(actual_canonical.as_bytes());
            let signature_hex = bytes_to_hex(&signature);

            assert_eq!(
                signature_hex,
                expected_signature_hex,
                "Signature mismatch for vector '{}'\nExpected: {}\nActual: {}",
                name,
                expected_signature_hex,
                signature_hex
            );

            // Verify the signature
            let is_valid = verify(&derived_public_key, actual_canonical.as_bytes(), &signature);
            assert!(is_valid, "Signature verification failed for vector '{}'", name);
        }

        println!("✓ Transaction signing vector '{}' passed", name);
    }
}

#[test]
fn test_hash_transaction_centralized() {
    // Test that our centralized hash_transaction function works correctly
    let tx = json!({
        "header": {
            "principal": "acc://alice.acme/tokens",
            "timestamp": 1234567890123_u64
        },
        "body": {
            "type": "send-tokens",
            "to": [{
                "url": "acc://bob.acme/tokens",
                "amount": "1000"
            }]
        }
    });

    let hash1 = AccumulateHash::hash_transaction(&tx);
    let hash2 = AccumulateHash::sha256_json(&tx);

    assert_eq!(hash1, hash2, "hash_transaction should match sha256_json");

    let expected_canonical = r#"{"body":{"to":[{"amount":"1000","url":"acc://bob.acme/tokens"}],"type":"send-tokens"},"header":{"principal":"acc://alice.acme/tokens","timestamp":1234567890123}}"#;
    let actual_canonical = canonical_json(&tx);

    assert_eq!(actual_canonical, expected_canonical, "Canonical JSON doesn't match expected");

    let expected_hash = "4be49c59c717f1984646998cecac0e5225378d9bbe2e18928272a85b7dfcb608";
    let actual_hash = bytes_to_hex(&hash1);

    assert_eq!(actual_hash, expected_hash, "Transaction hash doesn't match expected");

    println!("✓ Centralized transaction hash test passed");
}

#[test]
fn test_deterministic_signatures() {
    // Test that signatures are deterministic for the same input
    let seed = [1u8; 32];
    let message = b"deterministic test message";

    let signer1 = Ed25519Signer::from_seed(&seed).unwrap();
    let signer2 = Ed25519Signer::from_seed(&seed).unwrap();

    let sig1 = signer1.sign(message);
    let sig2 = signer2.sign(message);

    assert_eq!(sig1, sig2, "Signatures should be deterministic");
    assert_eq!(signer1.public_key_bytes(), signer2.public_key_bytes(), "Public keys should match");

    // Verify both signatures
    let public_key = signer1.public_key_bytes();
    assert!(verify(&public_key, message, &sig1));
    assert!(verify(&public_key, message, &sig2));

    println!("✓ Deterministic signatures test passed");
}

#[test]
fn test_signature_format_compatibility() {
    // Test that our sign() function returns [u8; 64] as required
    let seed = [2u8; 32];
    let signer = Ed25519Signer::from_seed(&seed).unwrap();
    let message = b"format test";

    let signature = signer.sign(message);

    // Verify it's exactly 64 bytes
    assert_eq!(signature.len(), 64, "Signature should be exactly 64 bytes");

    // Verify it works with our verify function
    let public_key = signer.public_key_bytes();
    assert!(verify(&public_key, message, &signature));

    println!("✓ Signature format compatibility test passed");
}

#[test]
fn test_cross_verification() {
    // Test that signatures generated by one method can be verified by another
    let seed = [3u8; 32];
    let signer = Ed25519Signer::from_seed(&seed).unwrap();
    let message = b"cross verification test";

    let signature = signer.sign(message);
    let public_key = signer.public_key_bytes();

    // Test both verification methods
    assert!(verify(&public_key, message, &signature));
    assert!(accumulate_client::verify_signature(&public_key, message, &signature).is_ok());

    // Test with wrong inputs
    assert!(!verify(&public_key, b"wrong message", &signature));
    assert!(accumulate_client::verify_signature(&public_key, b"wrong message", &signature).is_err());

    let wrong_key = [0u8; 32];
    assert!(!verify(&wrong_key, message, &signature));

    println!("✓ Cross verification test passed");
}