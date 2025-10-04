//! Binary parity tests for Accumulate Rust SDK
//!
//! ⚠️  WARNING: These tests use potentially buggy TypeScript test vectors.
//! See TYPESCRIPT_DEPENDENCIES_AUDIT.md for details.
//! Use canonical tests in /conformance/ directory for reliable validation.
//!
//! This module tests bit-for-bit compatibility with the TypeScript SDK
//! using golden test vectors exported from the TS implementation.

use accumulate_client::codec::{
    BinaryReader, BinaryWriter, DecodingError, EncodingError, TransactionCodec,
    TransactionEnvelope, TransactionHeader, TransactionSignature, AccumulateHash,
};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Load TypeScript SDK test vectors
fn load_typescript_vectors() -> Result<Value, Box<dyn std::error::Error>> {
    let vectors_path = "tests/golden/typescript_sdk_vectors.json";
    let vectors_content = std::fs::read_to_string(vectors_path)?;
    let vectors: Value = serde_json::from_str(&vectors_content)?;
    Ok(vectors)
}

/// Print first differing offset and hex row diff on binary mismatch
fn print_binary_diff(rust_bytes: &[u8], expected_bytes: &[u8], test_name: &str) {
    println!("❌ Binary parity FAILED for: {}", test_name);

    if rust_bytes.len() != expected_bytes.len() {
        println!("  Length mismatch: Rust={}, Expected={}", rust_bytes.len(), expected_bytes.len());
    }

    // Find first differing offset
    let mut first_diff_offset = None;
    for (i, (&rust_byte, &expected_byte)) in rust_bytes.iter().zip(expected_bytes.iter()).enumerate() {
        if rust_byte != expected_byte {
            first_diff_offset = Some(i);
            break;
        }
    }

    if let Some(offset) = first_diff_offset {
        println!("  First difference at offset: {}", offset);

        // Print hex rows around the difference
        let start = offset.saturating_sub(8);
        let end = std::cmp::min(offset + 16, std::cmp::max(rust_bytes.len(), expected_bytes.len()));

        print!("  Rust:     ");
        for i in start..end {
            if i < rust_bytes.len() {
                if i == offset {
                    print!("[{:02x}] ", rust_bytes[i]);
                } else {
                    print!("{:02x} ", rust_bytes[i]);
                }
            } else {
                print!("-- ");
            }
        }
        println!();

        print!("  Expected: ");
        for i in start..end {
            if i < expected_bytes.len() {
                if i == offset {
                    print!("[{:02x}] ", expected_bytes[i]);
                } else {
                    print!("{:02x} ", expected_bytes[i]);
                }
            } else {
                print!("-- ");
            }
        }
        println!();
    }
}

/// Test uvarint encoding/decoding parity with TypeScript SDK
#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_uvarint_binary_parity() {
    let vectors = load_typescript_vectors().expect("Failed to load test vectors");
    let uvarint_tests = vectors["uvarint"].as_array().expect("uvarint tests not found");

    for test in uvarint_tests {
        let input = test["input"].as_u64().expect("Invalid input");
        let expected_bytes: Vec<u8> = test["encoded"].as_array()
            .expect("encoded array not found")
            .iter()
            .map(|v| v.as_u64().expect("Invalid byte") as u8)
            .collect();
        let expected_hex = test["hex"].as_str().expect("hex not found");

        // Test encoding
        let rust_encoded = BinaryWriter::encode_uvarint(input);

        if rust_encoded != expected_bytes {
            print_binary_diff(&rust_encoded, &expected_bytes, &format!("uvarint encode {}", input));
            panic!("UVarint encoding failed for input: {}", input);
        }

        // Verify hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(rust_hex, expected_hex, "UVarint hex mismatch for input: {}", input);

        // Test decoding roundtrip
        let mut reader = BinaryReader::new(&rust_encoded);
        let decoded = reader.read_uvarint().expect("Failed to decode uvarint");
        assert_eq!(decoded, input, "UVarint roundtrip failed for input: {}", input);

        // Re-encode to ensure identical
        let re_encoded = BinaryWriter::encode_uvarint(decoded);
        assert_eq!(re_encoded, rust_encoded, "Re-encoding not identical for input: {}", input);
    }

    println!("✅ UVarint binary parity: {} tests passed", uvarint_tests.len());
}

/// Test varint (zigzag) encoding/decoding parity with TypeScript SDK
#[test]
fn test_varint_binary_parity() {
    let vectors = load_typescript_vectors().expect("Failed to load test vectors");
    let varint_tests = vectors["varint"].as_array().expect("varint tests not found");

    for test in varint_tests {
        let input = test["input"].as_i64().expect("Invalid input");
        let expected_bytes: Vec<u8> = test["encoded"].as_array()
            .expect("encoded array not found")
            .iter()
            .map(|v| v.as_u64().expect("Invalid byte") as u8)
            .collect();
        let expected_hex = test["hex"].as_str().expect("hex not found");

        // Test encoding
        let rust_encoded = BinaryWriter::encode_varint(input);

        if rust_encoded != expected_bytes {
            print_binary_diff(&rust_encoded, &expected_bytes, &format!("varint encode {}", input));
            panic!("Varint encoding failed for input: {}", input);
        }

        // Verify hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(rust_hex, expected_hex, "Varint hex mismatch for input: {}", input);

        // Test decoding roundtrip
        let mut reader = BinaryReader::new(&rust_encoded);
        let decoded = reader.read_varint().expect("Failed to decode varint");
        assert_eq!(decoded, input, "Varint roundtrip failed for input: {}", input);

        // Re-encode to ensure identical
        let re_encoded = BinaryWriter::encode_varint(decoded);
        assert_eq!(re_encoded, rust_encoded, "Re-encoding not identical for input: {}", input);
    }

    println!("✅ Varint binary parity: {} tests passed", varint_tests.len());
}

/// Test string encoding/decoding parity with TypeScript SDK
#[test]
fn test_string_binary_parity() {
    let vectors = load_typescript_vectors().expect("Failed to load test vectors");
    let string_tests = vectors["strings"].as_array().expect("strings tests not found");

    for test in string_tests {
        let input = test["input"].as_str().expect("Invalid input");
        let expected_bytes: Vec<u8> = test["encoded"].as_array()
            .expect("encoded array not found")
            .iter()
            .map(|v| v.as_u64().expect("Invalid byte") as u8)
            .collect();
        let expected_hex = test["hex"].as_str().expect("hex not found");

        // Test encoding
        let rust_encoded = BinaryWriter::encode_string(input);

        if rust_encoded != expected_bytes {
            print_binary_diff(&rust_encoded, &expected_bytes, &format!("string encode '{}'", input));
            panic!("String encoding failed for input: '{}'", input);
        }

        // Verify hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(rust_hex, expected_hex, "String hex mismatch for input: '{}'", input);

        // Test decoding roundtrip
        let mut reader = BinaryReader::new(&rust_encoded);
        let decoded = reader.read_string().expect("Failed to decode string");
        assert_eq!(decoded, input, "String roundtrip failed for input: '{}'", input);

        // Re-encode to ensure identical
        let re_encoded = BinaryWriter::encode_string(&decoded);
        assert_eq!(re_encoded, rust_encoded, "Re-encoding not identical for input: '{}'", input);
    }

    println!("✅ String binary parity: {} tests passed", string_tests.len());
}

/// Test bytes encoding/decoding parity with TypeScript SDK
#[test]
fn test_bytes_binary_parity() {
    let vectors = load_typescript_vectors().expect("Failed to load test vectors");
    let bytes_tests = vectors["bytes"].as_array().expect("bytes tests not found");

    for test in bytes_tests {
        let input_bytes: Vec<u8> = test["input"].as_array()
            .expect("input array not found")
            .iter()
            .map(|v| v.as_u64().expect("Invalid byte") as u8)
            .collect();
        let expected_bytes: Vec<u8> = test["encoded"].as_array()
            .expect("encoded array not found")
            .iter()
            .map(|v| v.as_u64().expect("Invalid byte") as u8)
            .collect();
        let expected_hex = test["hex"].as_str().expect("hex not found");

        // Test encoding
        let rust_encoded = BinaryWriter::encode_bytes(&input_bytes);

        if rust_encoded != expected_bytes {
            print_binary_diff(&rust_encoded, &expected_bytes, &format!("bytes encode {:?}", input_bytes));
            panic!("Bytes encoding failed for input: {:?}", input_bytes);
        }

        // Verify hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(rust_hex, expected_hex, "Bytes hex mismatch for input: {:?}", input_bytes);

        // Test decoding roundtrip
        let mut reader = BinaryReader::new(&rust_encoded);
        let decoded = reader.read_bytes_with_length().expect("Failed to decode bytes");
        assert_eq!(decoded, input_bytes.as_slice(), "Bytes roundtrip failed for input: {:?}", input_bytes);

        // Re-encode to ensure identical
        let re_encoded = BinaryWriter::encode_bytes(&decoded);
        assert_eq!(re_encoded, rust_encoded, "Re-encoding not identical for input: {:?}", input_bytes);
    }

    println!("✅ Bytes binary parity: {} tests passed", bytes_tests.len());
}

/// Test boolean encoding/decoding parity with TypeScript SDK
#[test]
fn test_boolean_binary_parity() {
    let vectors = load_typescript_vectors().expect("Failed to load test vectors");
    let boolean_tests = vectors["booleans"].as_array().expect("booleans tests not found");

    for test in boolean_tests {
        let input = test["input"].as_bool().expect("Invalid input");
        let expected_bytes: Vec<u8> = test["encoded"].as_array()
            .expect("encoded array not found")
            .iter()
            .map(|v| v.as_u64().expect("Invalid byte") as u8)
            .collect();
        let expected_hex = test["hex"].as_str().expect("hex not found");

        // Test encoding
        let rust_encoded = BinaryWriter::encode_bool(input);

        if rust_encoded != expected_bytes {
            print_binary_diff(&rust_encoded, &expected_bytes, &format!("boolean encode {}", input));
            panic!("Boolean encoding failed for input: {}", input);
        }

        // Verify hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(rust_hex, expected_hex, "Boolean hex mismatch for input: {}", input);

        // Test decoding roundtrip
        let mut reader = BinaryReader::new(&rust_encoded);
        let decoded = reader.read_bool().expect("Failed to decode boolean");
        assert_eq!(decoded, input, "Boolean roundtrip failed for input: {}", input);

        // Re-encode to ensure identical
        let re_encoded = BinaryWriter::encode_bool(decoded);
        assert_eq!(re_encoded, rust_encoded, "Re-encoding not identical for input: {}", input);
    }

    println!("✅ Boolean binary parity: {} tests passed", boolean_tests.len());
}

/// Test hash computation parity with TypeScript SDK
#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_hash_binary_parity() {
    let vectors = load_typescript_vectors().expect("Failed to load test vectors");
    let hash_tests = vectors["hashes"].as_array().expect("hashes tests not found");

    for test in hash_tests {
        let input_value = &test["input"];
        let expected_hash = test["hash"].as_str().expect("hash not found");
        let expected_canonical = test["canonical"].as_str().expect("canonical not found");

        // Test canonical JSON serialization
        let rust_canonical = accumulate_client::codec::canonical_json(input_value);
        assert_eq!(rust_canonical, expected_canonical, "Canonical JSON mismatch for input: {}", input_value);

        // Test hash computation
        let rust_hash = AccumulateHash::sha256_hex(input_value);
        assert_eq!(rust_hash, expected_hash, "Hash mismatch for input: {}", input_value);

        // Test byte-level hash
        let canonical_bytes = rust_canonical.as_bytes();
        let rust_hash_bytes = accumulate_client::codec::sha256_bytes(canonical_bytes);
        let rust_hash_hex = hex::encode(rust_hash_bytes);
        assert_eq!(rust_hash_hex, expected_hash, "Byte-level hash mismatch for input: {}", input_value);
    }

    println!("✅ Hash binary parity: {} tests passed", hash_tests.len());
}

/// Test transaction envelope encoding/decoding parity with TypeScript SDK
#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_transaction_envelope_binary_parity() {
    let vectors = load_typescript_vectors().expect("Failed to load test vectors");

    if let Some(envelope_tests) = vectors["envelopes"].as_array() {
        for test in envelope_tests {
            let input_envelope = &test["input"];
            let expected_bytes: Vec<u8> = test["encoded"].as_array()
                .expect("encoded array not found")
                .iter()
                .map(|v| v.as_u64().expect("Invalid byte") as u8)
                .collect();
            let expected_hex = test["hex"].as_str().expect("hex not found");

            // Build Rust struct from fixture
            let envelope = build_envelope_from_fixture(input_envelope)
                .expect("Failed to build envelope from fixture");

            // Test encoding
            let rust_encoded = TransactionCodec::encode_envelope(&envelope)
                .expect("Failed to encode envelope");

            if rust_encoded != expected_bytes {
                print_binary_diff(&rust_encoded, &expected_bytes, "transaction envelope");
                panic!("Transaction envelope encoding failed");
            }

            // Verify hex representation
            let rust_hex = hex::encode(&rust_encoded);
            assert_eq!(rust_hex, expected_hex, "Transaction envelope hex mismatch");

            // Test decoding roundtrip
            let decoded = TransactionCodec::decode_envelope(&rust_encoded)
                .expect("Failed to decode envelope");

            // Deep equality check
            assert_envelope_deep_equal(&envelope, &decoded);

            // Re-encode to ensure identical
            let re_encoded = TransactionCodec::encode_envelope(&decoded)
                .expect("Failed to re-encode envelope");
            assert_eq!(re_encoded, rust_encoded, "Re-encoding not identical");
        }

        println!("✅ Transaction envelope binary parity: {} tests passed", envelope_tests.len());
    }
}

/// Build transaction envelope from test fixture
fn build_envelope_from_fixture(fixture: &Value) -> Result<TransactionEnvelope, Box<dyn std::error::Error>> {
    let header = TransactionHeader {
        principal: fixture["header"]["principal"].as_str().unwrap().to_string(),
        initiator: fixture["header"]["initiator"].as_str().map(|s| s.to_string()),
        timestamp: fixture["header"]["timestamp"].as_u64().unwrap(),
        nonce: fixture["header"]["nonce"].as_u64(),
        memo: fixture["header"]["memo"].as_str().map(|s| s.to_string()),
        metadata: fixture["header"]["metadata"].clone().into(),
    };

    let body = fixture["body"].clone();

    let signatures: Vec<TransactionSignature> = fixture["signatures"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|sig| TransactionSignature {
            signature: sig["signature"].as_array().unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect(),
            signer: sig["signer"].as_str().unwrap().to_string(),
            timestamp: sig["timestamp"].as_u64().unwrap(),
            vote: sig["vote"].as_str().map(|s| s.to_string()),
            public_key: sig["publicKey"].as_array().map(|arr|
                arr.iter().map(|v| v.as_u64().unwrap() as u8).collect()
            ),
            key_page: None, // Add if needed based on fixtures
        })
        .collect();

    Ok(TransactionEnvelope {
        header,
        body,
        signatures,
    })
}

/// Assert deep equality between transaction envelopes
fn assert_envelope_deep_equal(expected: &TransactionEnvelope, actual: &TransactionEnvelope) {
    // Header comparison
    assert_eq!(expected.header.principal, actual.header.principal);
    assert_eq!(expected.header.initiator, actual.header.initiator);
    assert_eq!(expected.header.timestamp, actual.header.timestamp);
    assert_eq!(expected.header.nonce, actual.header.nonce);
    assert_eq!(expected.header.memo, actual.header.memo);
    assert_eq!(expected.header.metadata, actual.header.metadata);

    // Body comparison
    assert_eq!(expected.body, actual.body);

    // Signatures comparison
    assert_eq!(expected.signatures.len(), actual.signatures.len());
    for (i, (exp_sig, act_sig)) in expected.signatures.iter().zip(actual.signatures.iter()).enumerate() {
        assert_eq!(exp_sig.signature, act_sig.signature, "Signature {} signature mismatch", i);
        assert_eq!(exp_sig.signer, act_sig.signer, "Signature {} signer mismatch", i);
        assert_eq!(exp_sig.timestamp, act_sig.timestamp, "Signature {} timestamp mismatch", i);
        assert_eq!(exp_sig.vote, act_sig.vote, "Signature {} vote mismatch", i);
        assert_eq!(exp_sig.public_key, act_sig.public_key, "Signature {} public_key mismatch", i);
        assert_eq!(exp_sig.key_page, act_sig.key_page, "Signature {} key_page mismatch", i);
    }
}

/// Test that all critical codec components are wired correctly
#[test]
fn test_codec_integration() {
    // Verify BinaryWriter static methods exist and work
    let uvarint_test = BinaryWriter::encode_uvarint(12345);
    assert!(!uvarint_test.is_empty());

    let varint_test = BinaryWriter::encode_varint(-42);
    assert!(!varint_test.is_empty());

    let string_test = BinaryWriter::encode_string("test");
    assert!(!string_test.is_empty());

    let bytes_test = BinaryWriter::encode_bytes(&[1, 2, 3]);
    assert!(!bytes_test.is_empty());

    let bool_test = BinaryWriter::encode_bool(true);
    assert_eq!(bool_test, vec![1]);

    // Verify BinaryReader can decode all types
    let mut reader = BinaryReader::new(&uvarint_test);
    assert_eq!(reader.read_uvarint().unwrap(), 12345);

    let mut reader = BinaryReader::new(&varint_test);
    assert_eq!(reader.read_varint().unwrap(), -42);

    let mut reader = BinaryReader::new(&string_test);
    assert_eq!(reader.read_string().unwrap(), "test");

    let mut reader = BinaryReader::new(&bytes_test);
    assert_eq!(reader.read_bytes_with_length().unwrap(), &[1, 2, 3]);

    let mut reader = BinaryReader::new(&bool_test);
    assert_eq!(reader.read_bool().unwrap(), true);

    println!("✅ Codec integration tests passed");
}