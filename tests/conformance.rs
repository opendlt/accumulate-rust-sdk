//! Conformance tests for Accumulate Rust SDK
//!
//! These tests verify that the Rust SDK implementation conforms to the
//! TypeScript SDK behavior for critical functionality.

use accumulate_client::codec::{
    BinaryWriter, BinaryReader, TransactionCodec, AccumulateHash, UrlHash,
    TransactionEnvelope, TransactionHeader, TransactionSignature, TransactionBodyBuilder
};
use serde_json::{json, Value};

/// Test that verifies the canonical JSON implementation produces
/// identical output to the TypeScript SDK
#[test]
fn test_canonical_json_conformance() {
    // Test case 1: Simple object with reordered keys
    let test1 = json!({
        "z": 3,
        "a": 1,
        "m": 2
    });

    let canonical1 = accumulate_client::codec::canonical_json(&test1);
    assert_eq!(canonical1, r#"{"a":1,"m":2,"z":3}"#);

    // Test case 2: Nested objects
    let test2 = json!({
        "z": { "y": 2, "x": 1 },
        "a": 1
    });

    let canonical2 = accumulate_client::codec::canonical_json(&test2);
    assert_eq!(canonical2, r#"{"a":1,"z":{"x":1,"y":2}}"#);

    // Test case 3: Arrays with objects
    let test3 = json!({
        "arr": [{ "b": 2, "a": 1 }, { "d": 4, "c": 3 }]
    });

    let canonical3 = accumulate_client::codec::canonical_json(&test3);
    assert_eq!(canonical3, r#"{"arr":[{"a":1,"b":2},{"c":3,"d":4}]}"#);

    // Test case 4: All primitive types
    let test4 = json!({
        "string": "test",
        "number": 42,
        "boolean": true,
        "null": null
    });

    let canonical4 = accumulate_client::codec::canonical_json(&test4);
    assert_eq!(canonical4, r#"{"boolean":true,"null":null,"number":42,"string":"test"}"#);
}

/// Test that verifies SHA-256 hashing produces identical results
/// to the TypeScript SDK for the same inputs
#[test]
fn test_sha256_conformance() {
    // Test case from TypeScript SDK fixtures
    let value = json!({
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

    let canonical = accumulate_client::codec::canonical_json(&value);
    let hash = AccumulateHash::sha256_hex(&value);

    // These values should match the TypeScript SDK exactly
    let expected_canonical = r#"{"body":{"to":[{"amount":"1000","url":"acc://bob.acme/tokens"}],"type":"send-tokens"},"header":{"principal":"acc://alice.acme/tokens","timestamp":1234567890123}}"#;
    let expected_hash = "4be49c59c717f1984646998cecac0e5225378d9bbe2e18928272a85b7dfcb608";

    assert_eq!(canonical, expected_canonical);
    assert_eq!(hash, expected_hash);
}

/// Test that verifies URL normalization and hashing matches TypeScript SDK
#[test]
fn test_url_conformance() {
    let test_cases = vec![
        ("acc://alice.acme", "acc://alice.acme"),
        ("ACC://ALICE.ACME", "acc://alice.acme"),
        ("acc://alice.acme/", "acc://alice.acme"),
        ("acc://alice.acme///", "acc://alice.acme"),
        ("//alice.acme", "acc://alice.acme"),
        ("alice.acme", "acc://alice.acme"),
        ("/alice.acme", "acc://alice.acme"),
    ];

    for (input, expected) in test_cases {
        let normalized = accumulate_client::codec::hashes::UrlHash::normalize_url(input);
        assert_eq!(normalized, expected, "URL normalization failed for: {}", input);
    }

    // Test URL derivation
    let identity_url = "acc://alice.acme";
    let key_book_url = UrlHash::derive_key_book_url(identity_url);
    assert_eq!(key_book_url, "acc://alice.acme/book");

    let key_page_url = UrlHash::derive_key_page_url(&key_book_url, 0);
    assert_eq!(key_page_url, "acc://alice.acme/book/0");

    // Test URL components extraction
    let url = "acc://alice.acme/tokens";
    let authority = UrlHash::extract_authority(url).unwrap();
    assert_eq!(authority, "alice.acme");

    let path = UrlHash::extract_path(url);
    assert_eq!(path, "/tokens");
}

/// Test that verifies binary encoding/decoding roundtrips match TypeScript SDK
#[test]
fn test_binary_encoding_conformance() {
    // Test uvarint encoding for critical values
    let uvarint_tests = vec![
        (0u64, vec![0]),
        (1u64, vec![1]),
        (127u64, vec![127]),
        (128u64, vec![128, 1]),
        (256u64, vec![128, 2]),
        (16384u64, vec![128, 128, 1]),
    ];

    for (input, expected) in uvarint_tests {
        let encoded = BinaryWriter::encode_uvarint(input);
        assert_eq!(encoded, expected, "UVarint encoding failed for: {}", input);

        let mut reader = BinaryReader::new(&encoded);
        let decoded = reader.read_uvarint().unwrap();
        assert_eq!(decoded, input, "UVarint decoding failed for: {}", input);
    }

    // Test varint encoding with zigzag
    let varint_tests = vec![
        (0i64, vec![0]),
        (-1i64, vec![1]),
        (1i64, vec![2]),
        (-2i64, vec![3]),
        (2i64, vec![4]),
    ];

    for (input, expected) in varint_tests {
        let encoded = BinaryWriter::encode_varint(input);
        assert_eq!(encoded, expected, "Varint encoding failed for: {}", input);

        let mut reader = BinaryReader::new(&encoded);
        let decoded = reader.read_varint().unwrap();
        assert_eq!(decoded, input, "Varint decoding failed for: {}", input);
    }

    // Test string encoding
    let string_encoded = BinaryWriter::encode_string("hello");
    let expected_string = vec![5, b'h', b'e', b'l', b'l', b'o'];
    assert_eq!(string_encoded, expected_string);

    let mut reader = BinaryReader::new(&string_encoded);
    let decoded_string = reader.read_string().unwrap();
    assert_eq!(decoded_string, "hello");

    // Test bytes encoding
    let bytes_input = vec![1, 2, 3, 4];
    let bytes_encoded = BinaryWriter::encode_bytes(&bytes_input);
    let expected_bytes = vec![4, 1, 2, 3, 4];
    assert_eq!(bytes_encoded, expected_bytes);

    let mut reader = BinaryReader::new(&bytes_encoded);
    let decoded_bytes = reader.read_bytes_with_length().unwrap();
    assert_eq!(decoded_bytes, bytes_input.as_slice());
}

/// Test that verifies transaction body builders produce TypeScript SDK compatible structures
#[test]
fn test_transaction_body_conformance() {
    // Test send tokens transaction
    let send_tokens = TransactionBodyBuilder::send_tokens(vec![
        accumulate_client::codec::TokenRecipient {
            url: "acc://bob.acme/tokens".to_string(),
            amount: "1000".to_string(),
        }
    ]);

    assert_eq!(send_tokens["type"], "send-tokens");
    assert!(send_tokens["to"].is_array());
    assert_eq!(send_tokens["to"][0]["url"], "acc://bob.acme/tokens");
    assert_eq!(send_tokens["to"][0]["amount"], "1000");

    // Test create identity transaction
    let create_identity = TransactionBodyBuilder::create_identity(
        "acc://alice.acme".to_string(),
        "acc://alice.acme/book".to_string(),
    );

    assert_eq!(create_identity["type"], "create-identity");
    assert_eq!(create_identity["url"], "acc://alice.acme");
    assert_eq!(create_identity["keyBook"], "acc://alice.acme/book");

    // Test create key book transaction
    let create_key_book = TransactionBodyBuilder::create_key_book(
        "acc://alice.acme/book".to_string(),
        "abcdef1234567890".to_string(),
    );

    assert_eq!(create_key_book["type"], "create-key-book");
    assert_eq!(create_key_book["url"], "acc://alice.acme/book");
    assert_eq!(create_key_book["publicKeyHash"], "abcdef1234567890");

    // Test add credits transaction
    let add_credits = TransactionBodyBuilder::add_credits(
        "acc://alice.acme".to_string(),
        "100000".to_string(),
        Some(0.05),
    );

    assert_eq!(add_credits["type"], "add-credits");
    assert_eq!(add_credits["recipient"], "acc://alice.acme");
    assert_eq!(add_credits["amount"], "100000");
    assert_eq!(add_credits["oracle"], 0.05);
}

/// Test that verifies transaction envelope creation and validation
#[test]
fn test_transaction_envelope_conformance() {
    // Create a transaction envelope
    let body = TransactionBodyBuilder::send_tokens(vec![
        accumulate_client::codec::TokenRecipient {
            url: "acc://bob.acme/tokens".to_string(),
            amount: "1000".to_string(),
        }
    ]);

    let envelope = TransactionCodec::create_envelope(
        "acc://alice.acme/tokens".to_string(),
        body,
        Some(1234567890123),
    );

    // Validate envelope structure
    assert_eq!(envelope.header.principal, "acc://alice.acme/tokens");
    assert_eq!(envelope.header.timestamp, 1234567890123);
    assert!(envelope.header.initiator.is_none());
    assert!(envelope.header.nonce.is_none());
    assert!(envelope.header.memo.is_none());
    assert!(envelope.header.metadata.is_none());

    // Validate body
    assert_eq!(envelope.body["type"], "send-tokens");
    assert_eq!(envelope.body["to"][0]["url"], "acc://bob.acme/tokens");
    assert_eq!(envelope.body["to"][0]["amount"], "1000");

    // Validate signatures (should be empty initially)
    assert!(envelope.signatures.is_empty());

    // Test validation
    TransactionCodec::validate_envelope(&envelope).unwrap();

    // Test transaction hash generation
    let tx_hash = TransactionCodec::get_transaction_hash(&envelope).unwrap();
    assert_eq!(tx_hash.len(), 32);

    // Hash should be deterministic
    let tx_hash2 = TransactionCodec::get_transaction_hash(&envelope).unwrap();
    assert_eq!(tx_hash, tx_hash2);
}

/// Test that verifies binary encoding/decoding of complete transaction envelopes
#[test]
fn test_envelope_binary_conformance() {
    // Create a complete transaction envelope with signature
    let body = TransactionBodyBuilder::send_tokens(vec![
        accumulate_client::codec::TokenRecipient {
            url: "acc://bob.acme/tokens".to_string(),
            amount: "1000".to_string(),
        }
    ]);

    let mut envelope = TransactionCodec::create_envelope(
        "acc://alice.acme/tokens".to_string(),
        body,
        Some(1234567890123),
    );

    // Add a signature
    TransactionCodec::add_signature(
        &mut envelope,
        vec![1, 2, 3, 4], // Mock signature
        "acc://alice.acme/book/1".to_string(),
        Some(vec![5, 6, 7, 8]), // Mock public key
    );

    // Test binary encoding roundtrip
    let encoded = TransactionCodec::encode_envelope(&envelope).unwrap();
    let decoded = TransactionCodec::decode_envelope(&encoded).unwrap();

    // Verify all fields match
    assert_eq!(decoded.header.principal, envelope.header.principal);
    assert_eq!(decoded.header.timestamp, envelope.header.timestamp);
    assert_eq!(decoded.body, envelope.body);
    assert_eq!(decoded.signatures.len(), envelope.signatures.len());

    if !decoded.signatures.is_empty() {
        let orig_sig = &envelope.signatures[0];
        let decoded_sig = &decoded.signatures[0];
        assert_eq!(decoded_sig.signature, orig_sig.signature);
        assert_eq!(decoded_sig.signer, orig_sig.signer);
        assert_eq!(decoded_sig.timestamp, orig_sig.timestamp);
        assert_eq!(decoded_sig.public_key, orig_sig.public_key);
    }

    // Test validation of decoded envelope
    TransactionCodec::validate_envelope(&decoded).unwrap();
}

/// Integration test that verifies end-to-end transaction creation and encoding
#[test]
fn test_end_to_end_conformance() {
    // Create a complete transaction workflow
    let principal = "acc://alice.acme/tokens".to_string();
    let signer_url = "acc://alice.acme/book/1".to_string();

    // Step 1: Create transaction body
    let body = TransactionBodyBuilder::send_tokens(vec![
        accumulate_client::codec::TokenRecipient {
            url: "acc://bob.acme/tokens".to_string(),
            amount: "1000".to_string(),
        },
        accumulate_client::codec::TokenRecipient {
            url: "acc://charlie.acme/tokens".to_string(),
            amount: "500".to_string(),
        }
    ]);

    // Step 2: Create envelope
    let mut envelope = TransactionCodec::create_envelope(
        principal,
        body,
        Some(1234567890123),
    );

    // Step 3: Get transaction hash for signing
    let tx_hash = TransactionCodec::get_transaction_hash(&envelope).unwrap();

    // Step 4: Add signature (mock signing)
    let mock_signature = AccumulateHash::sha256_bytes(&tx_hash); // Mock signature
    let mock_public_key = AccumulateHash::sha256_bytes(b"mock_public_key"); // Mock public key

    TransactionCodec::add_signature(
        &mut envelope,
        mock_signature.to_vec(),
        signer_url,
        Some(mock_public_key.to_vec()),
    );

    // Step 5: Validate complete envelope
    TransactionCodec::validate_envelope(&envelope).unwrap();

    // Step 6: Binary encoding roundtrip
    let encoded = TransactionCodec::encode_envelope(&envelope).unwrap();
    let decoded = TransactionCodec::decode_envelope(&encoded).unwrap();

    // Step 7: Verify decoded envelope is identical
    assert_eq!(decoded.header.principal, envelope.header.principal);
    assert_eq!(decoded.header.timestamp, envelope.header.timestamp);
    assert_eq!(decoded.body, envelope.body);
    assert_eq!(decoded.signatures.len(), 1);
    assert_eq!(decoded.signatures[0].signature, mock_signature.to_vec());
    assert_eq!(decoded.signatures[0].signer, "acc://alice.acme/book/1");
    assert_eq!(decoded.signatures[0].public_key, Some(mock_public_key.to_vec()));

    // Step 8: Verify hash consistency
    let decoded_tx_hash = TransactionCodec::get_transaction_hash(&decoded).unwrap();
    assert_eq!(tx_hash, decoded_tx_hash);

    println!("End-to-end conformance test passed!");
    println!("Envelope size: {} bytes", encoded.len());
    println!("Transaction hash: {}", hex::encode(tx_hash));
    println!("Principal: {}", decoded.header.principal);
    println!("Recipients: {}", decoded.body["to"].as_array().unwrap().len());
}