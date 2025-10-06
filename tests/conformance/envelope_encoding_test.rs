use accumulate_client::codec::canonical_json;
use accumulate_client::Ed25519Helper;
use accumulate_client::protocol::{EnvelopeBuilder, TransactionEnvelope, helpers};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Load envelope fixed golden fixture
fn load_envelope_fixed() -> Value {
    let envelope_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("enums")
        .join("envelope_fixed.golden.json");

    let content = fs::read_to_string(envelope_path)
        .expect("Failed to read envelope_fixed.golden.json fixture");

    serde_json::from_str(&content).expect("Failed to parse envelope fixed fixture")
}

/// Load transaction only golden fixture
fn load_transaction_only() -> Value {
    let tx_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("enums")
        .join("tx_only.golden.json");

    let content = fs::read_to_string(tx_path)
        .expect("Failed to read tx_only.golden.json fixture");

    serde_json::from_str(&content).expect("Failed to parse transaction only fixture")
}

#[test]
fn test_envelope_structure_matches_fixture() {
    let expected_envelope = load_envelope_fixed();

    // Verify the structure matches our Rust types
    assert!(expected_envelope["signatures"].is_array());
    assert!(expected_envelope["transaction"].is_array());

    let signatures = expected_envelope["signatures"].as_array().unwrap();
    assert!(!signatures.is_empty());

    let first_sig = &signatures[0];
    assert!(first_sig["type"].is_string());
    assert!(first_sig["publicKey"].is_string());
    assert!(first_sig["signature"].is_string());
    assert!(first_sig["signer"].is_string());
    assert!(first_sig["signerVersion"].is_number());
    assert!(first_sig["timestamp"].is_number());
    assert!(first_sig["transactionHash"].is_string());

    println!("✓ Envelope structure matches expected format");
}

#[test]
fn test_envelope_serialization_format() {
    let envelope_data = load_envelope_fixed();

    // Parse into our Rust type
    let envelope: TransactionEnvelope = serde_json::from_value(envelope_data.clone())
        .expect("Failed to parse envelope into Rust type");

    // Serialize back to JSON
    let serialized = serde_json::to_value(&envelope)
        .expect("Failed to serialize envelope");

    // Convert both to canonical JSON for comparison
    let expected_canonical = canonical_json(&envelope_data);
    let actual_canonical = canonical_json(&serialized);

    assert_eq!(
        actual_canonical, expected_canonical,
        "Serialized envelope doesn't match expected format\nExpected: {}\nActual: {}",
        expected_canonical, actual_canonical
    );

    println!("✓ Envelope serialization matches canonical format");
}

#[test]
fn test_envelope_creation_basic() {
    let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

    // Create a send tokens transaction body
    let body = helpers::create_send_tokens_body("acc://test.acme/staking", "200000", None);

    let envelope = EnvelopeBuilder::create_envelope_with_initiator(
        "acc://test.acme/tokens",
        "cf44e0f55dddb5858481ebb2369c35957b38d228d32db1475b051113755bd965",
        body,
        &keypair,
        "acc://test.acme/book/1",
        51,
    ).expect("Failed to create envelope");

    // Verify structure
    assert_eq!(envelope.signatures.len(), 1);
    assert_eq!(envelope.transaction.len(), 1);

    let signature = &envelope.signatures[0];
    assert_eq!(signature.signature_type, "ed25519");
    assert_eq!(signature.signer, "acc://test.acme/book/1");
    assert_eq!(signature.signer_version, 51);

    let transaction = &envelope.transaction[0];
    assert_eq!(transaction.header.principal, "acc://test.acme/tokens");
    assert_eq!(transaction.header.initiator.as_ref().unwrap(), "cf44e0f55dddb5858481ebb2369c35957b38d228d32db1475b051113755bd965");

    println!("✓ Basic envelope creation works");
}

#[test]
fn test_envelope_verification() {
    let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

    let body = helpers::create_send_tokens_body("acc://bob.acme/tokens", "1000", None);

    let envelope = EnvelopeBuilder::create_envelope_from_json(
        "acc://alice.acme/tokens",
        body,
        &keypair,
        "acc://alice.acme/book/1",
        1,
    ).expect("Failed to create envelope");

    // Verify the envelope
    let verification_result = EnvelopeBuilder::verify_envelope(&envelope);
    assert!(verification_result.is_ok(), "Envelope verification failed: {:?}", verification_result);

    println!("✓ Envelope verification works");
}

#[test]
fn test_envelope_canonical_serialization() {
    let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

    let body = json!({
        "type": "sendTokens",
        "to": [{
            "url": "acc://test.acme/staking",
            "amount": "200000"
        }]
    });

    let envelope = EnvelopeBuilder::create_envelope_with_initiator(
        "acc://test.acme/tokens",
        "cf44e0f55dddb5858481ebb2369c35957b38d228d32db1475b051113755bd965",
        body,
        &keypair,
        "acc://test.acme/book/1",
        51,
    ).expect("Failed to create envelope");

    let canonical = EnvelopeBuilder::serialize_envelope(&envelope)
        .expect("Failed to serialize envelope");

    // Verify it's valid JSON
    let parsed: Value = serde_json::from_str(&canonical)
        .expect("Canonical serialization is not valid JSON");

    // Verify structure
    assert!(parsed["signatures"].is_array());
    assert!(parsed["transaction"].is_array());

    // Verify canonical ordering (signatures should come before transaction)
    let signatures_pos = canonical.find("signatures").unwrap();
    let transaction_pos = canonical.find("transaction").unwrap();
    assert!(signatures_pos < transaction_pos, "Keys not in canonical order");

    println!("✓ Envelope canonical serialization works");
    println!("Canonical envelope: {}", canonical);
}

#[test]
fn test_transaction_hash_consistency() {
    let tx_data = load_transaction_only();

    // Create the same transaction
    let transaction = serde_json::from_value(tx_data.clone())
        .expect("Failed to parse transaction");

    // Calculate hash manually
    let canonical = canonical_json(&tx_data);
    let expected_hash = hex::encode(accumulate_client::codec::sha256_bytes(canonical.as_bytes()));

    // Create envelope and verify hash
    let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

    let envelope = EnvelopeBuilder::create_envelope(
        transaction,
        &keypair,
        "acc://test.acme/book/1",
        51,
    ).expect("Failed to create envelope");

    let computed_hash = &envelope.signatures[0].transaction_hash;

    assert_eq!(
        *computed_hash, expected_hash,
        "Transaction hash mismatch\nExpected: {}\nComputed: {}",
        expected_hash, computed_hash
    );

    println!("✓ Transaction hash consistency verified: {}", computed_hash);
}

#[test]
fn test_multiple_transaction_types() {
    let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

    let test_cases = vec![
        ("send_tokens", helpers::create_send_tokens_body("acc://recipient", "500", None)),
        ("create_identity", helpers::create_identity_body("acc://new-identity", "pubkey123")),
        ("add_credits", helpers::create_add_credits_body("acc://recipient", 1000, None)),
    ];

    for (name, body) in test_cases {
        let envelope = EnvelopeBuilder::create_envelope_from_json(
            "acc://principal.acme",
            body,
            &keypair,
            "acc://principal.acme/book/1",
            1,
        ).expect(&format!("Failed to create envelope for {}", name));

        // Verify envelope
        let verification = EnvelopeBuilder::verify_envelope(&envelope);
        assert!(verification.is_ok(), "Verification failed for {}: {:?}", name, verification);

        // Verify serialization
        let canonical = EnvelopeBuilder::serialize_envelope(&envelope)
            .expect(&format!("Failed to serialize {}", name));
        assert!(!canonical.is_empty());

        println!("✓ {} envelope created and verified", name);
    }
}

#[test]
fn test_envelope_round_trip() {
    let hex_key = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
    let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

    let body = json!({
        "type": "createIdentity",
        "url": "acc://alice.acme",
        "keyBook": {
            "publicKeyHash": "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29"
        }
    });

    // Create envelope
    let original_envelope = EnvelopeBuilder::create_envelope_from_json(
        "acc://alice.acme",
        body,
        &keypair,
        "acc://alice.acme/book/1",
        1,
    ).expect("Failed to create envelope");

    // Serialize to canonical JSON
    let canonical = EnvelopeBuilder::serialize_envelope(&original_envelope)
        .expect("Failed to serialize envelope");

    // Parse back from JSON
    let parsed_envelope: TransactionEnvelope = serde_json::from_str(&canonical)
        .expect("Failed to parse envelope from canonical JSON");

    // Verify they're equivalent
    assert_eq!(original_envelope.signatures.len(), parsed_envelope.signatures.len());
    assert_eq!(original_envelope.transaction.len(), parsed_envelope.transaction.len());

    let orig_sig = &original_envelope.signatures[0];
    let parsed_sig = &parsed_envelope.signatures[0];

    assert_eq!(orig_sig.signature_type, parsed_sig.signature_type);
    assert_eq!(orig_sig.public_key, parsed_sig.public_key);
    assert_eq!(orig_sig.signature, parsed_sig.signature);
    assert_eq!(orig_sig.transaction_hash, parsed_sig.transaction_hash);

    // Verify both envelopes pass verification
    assert!(EnvelopeBuilder::verify_envelope(&original_envelope).is_ok());
    assert!(EnvelopeBuilder::verify_envelope(&parsed_envelope).is_ok());

    println!("✓ Envelope round-trip test passed");
}