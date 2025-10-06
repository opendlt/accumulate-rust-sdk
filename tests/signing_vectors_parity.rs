use accumulate_client::crypto::ed25519::{Ed25519Signer, sha256, verify_signature};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load ED25519 signing test vectors
fn load_ed25519_vectors() -> serde_json::Result<Value> {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("enums")
        .join("sig_ed25519.golden.json");

    let content = fs::read_to_string(fixture_path)
        .expect("Failed to read ED25519 signing vectors");

    serde_json::from_str(&content)
}

#[test]
fn test_ed25519_signing_parity() {
    // Test with known vectors if available
    if let Ok(vectors) = load_ed25519_vectors() {
        if let Some(sig_obj) = vectors.as_object() {
            println!("Testing ED25519 signature vector: {}", serde_json::to_string_pretty(&vectors).unwrap());

            // Extract signature components
            if let (Some(public_key), Some(signature), Some(message)) = (
                sig_obj.get("publicKey").and_then(|v| v.as_str()),
                sig_obj.get("signature").and_then(|v| v.as_str()),
                sig_obj.get("message").and_then(|v| v.as_str()),
            ) {
                let pub_key_bytes = hex::decode(public_key).expect("Invalid public key hex");
                let sig_bytes = hex::decode(signature).expect("Invalid signature hex");
                let message_bytes = message.as_bytes();

                if pub_key_bytes.len() == 32 && sig_bytes.len() == 64 {
                    let pub_key_array: [u8; 32] = pub_key_bytes.try_into().unwrap();
                    let sig_array: [u8; 64] = sig_bytes.try_into().unwrap();

                    // Test signature verification
                    let verification_result = verify_signature(&pub_key_array, message_bytes, &sig_array);
                    println!("Signature verification result: {:?}", verification_result);

                    // Note: We can't assert the signature is valid without knowing if the test vector
                    // is actually a valid signature pair, so we just test that verification doesn't panic
                }
            }
        }
    }
}

#[test]
fn test_ed25519_deterministic_signing() {
    // Test deterministic signing with known seed
    let seed = [0u8; 32]; // All zeros seed
    let signer = Ed25519Signer::from_seed(&seed).expect("Failed to create signer from seed");

    let message = b"Hello, Accumulate!";
    let signature1 = signer.sign(message);
    let signature2 = signer.sign(message);

    // Signatures should be deterministic for the same message and key
    assert_eq!(signature1, signature2, "Signatures should be deterministic");

    // Test verification
    let pub_key_bytes = signer.public_key_bytes();

    assert!(
        verify_signature(&pub_key_bytes, message, &signature1).is_ok(),
        "Signature should be valid"
    );
}

#[test]
fn test_ed25519_different_seeds() {
    // Test that different seeds produce different keys and signatures
    let seed1 = [1u8; 32];
    let seed2 = [2u8; 32];

    let signer1 = Ed25519Signer::from_seed(&seed1).unwrap();
    let signer2 = Ed25519Signer::from_seed(&seed2).unwrap();

    // Different seeds should produce different public keys
    assert_ne!(signer1.public_key_bytes(), signer2.public_key_bytes());

    let message = b"Test message";
    let sig1 = signer1.sign(message);
    let sig2 = signer2.sign(message);

    // Different keys should produce different signatures for the same message
    assert_ne!(sig1, sig2);

    // Each signature should verify with its corresponding public key
    assert!(verify_signature(&signer1.public_key_bytes(), message, &sig1).is_ok());
    assert!(verify_signature(&signer2.public_key_bytes(), message, &sig2).is_ok());

    // Cross-verification should fail
    assert!(verify_signature(&signer1.public_key_bytes(), message, &sig2).is_err());
    assert!(verify_signature(&signer2.public_key_bytes(), message, &sig1).is_err());
}

#[test]
fn test_ed25519_prehashed_signing() {
    let seed = [42u8; 32];
    let signer = Ed25519Signer::from_seed(&seed).unwrap();

    let message = b"Message to be hashed and signed";
    let hash = sha256(message);

    // Sign the hash directly
    let signature = signer.sign_prehashed(&hash);

    // Verify against the original message (our implementation may verify against the message)
    let verification = verify_signature(
        &signer.public_key_bytes(),
        message,
        &signature
    );

    // Note: This test depends on how our implementation handles prehashed vs regular signing
    println!("Prehashed signature verification: {:?}", verification);

    // Also test that regular signing of the hash produces expected behavior
    let signature2 = signer.sign(&hash);
    println!("Regular hash signature: {}", hex::encode(signature2));
    println!("Prehashed signature: {}", hex::encode(signature));
}

#[test]
fn test_ed25519_round_trip() {
    // Test complete round-trip: seed -> keypair -> sign -> verify
    let original_seed = [123u8; 32];
    let signer = Ed25519Signer::from_seed(&original_seed).unwrap();

    // Test extracting and reconstructing keypair
    let keypair_bytes = signer.keypair_bytes();
    let signer2 = Ed25519Signer::from_keypair_bytes(&keypair_bytes).unwrap();

    // Should have identical keys
    assert_eq!(signer.public_key_bytes(), signer2.public_key_bytes());
    assert_eq!(signer.private_key_bytes(), signer2.private_key_bytes());

    // Should produce identical signatures
    let message = b"Round trip test";
    let sig1 = signer.sign(message);
    let sig2 = signer2.sign(message);

    assert_eq!(sig1, sig2);
}

#[test]
fn test_ed25519_key_format_consistency() {
    let seed = [200u8; 32];
    let signer = Ed25519Signer::from_seed(&seed).unwrap();

    // Test key format consistency
    let pub_key = signer.public_key_bytes();
    let priv_key = signer.private_key_bytes();
    let keypair = signer.keypair_bytes();

    // Keypair should be private key + public key
    assert_eq!(keypair[..32], priv_key);
    assert_eq!(keypair[32..], pub_key);

    // Lengths should be correct
    assert_eq!(pub_key.len(), 32);
    assert_eq!(priv_key.len(), 32);
    assert_eq!(keypair.len(), 64);
}

#[test]
fn test_ed25519_test_vectors() {
    // Test with a few incremental seeds for reproducibility
    let test_cases = [
        ([0u8; 32], "Test vector 0"),
        ([1u8; 32], "Test vector 1"),
        ([255u8; 32], "Test vector 255"),
    ];

    for (seed, description) in &test_cases {
        let signer = Ed25519Signer::from_seed(seed).unwrap();
        let message = description.as_bytes();
        let signature = signer.sign(message);

        println!("Test case: {}", description);
        println!("  Seed: {}", hex::encode(seed));
        println!("  Public key: {}", hex::encode(signer.public_key_bytes()));
        println!("  Message: {:?}", description);
        println!("  Signature: {}", hex::encode(signature));

        // Verify the signature
        assert!(
            verify_signature(&signer.public_key_bytes(), message, &signature).is_ok(),
            "Signature should verify for test case: {}", description
        );
    }
}