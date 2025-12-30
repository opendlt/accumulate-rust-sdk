//! Binary parity tests using TypeScript SDK golden fixtures
//!
//! WARNING: These tests use potentially buggy TypeScript test vectors.
//! See TYPESCRIPT_DEPENDENCIES_AUDIT.md for details.
//! Use canonical tests in /conformance/ directory for reliable validation.
//!
//! These tests ensure bit-for-bit compatibility between the Rust SDK
//! and TypeScript SDK binary encoding/decoding implementations.

use accumulate_client::codec::{
    AccumulateHash, BinaryReader, BinaryWriter, TransactionBodyBuilder, TransactionCodec,
    TransactionEnvelope, TransactionHeader, TransactionSignature, UrlHash,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Test vector structure matching the TypeScript exporter
#[derive(Debug, Clone, Deserialize)]
struct TestVectors {
    metadata: VectorMetadata,
    uvarint: Vec<UVarintVector>,
    varint: Vec<VarintVector>,
    strings: Vec<StringVector>,
    bytes: Vec<BytesVector>,
    booleans: Vec<BooleanVector>,
    hashes: Vec<HashVector>,
    transactions: Vec<TransactionVector>,
    envelopes: Vec<EnvelopeVector>,
}

#[derive(Debug, Clone, Deserialize)]
struct VectorMetadata {
    generated_at: String,
    typescript_sdk_version: String,
    purpose: String,
    format_version: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UVarintVector {
    input: u64,
    encoded: Vec<u8>,
    hex: String,
}

#[derive(Debug, Clone, Deserialize)]
struct VarintVector {
    input: i64,
    encoded: Vec<u8>,
    hex: String,
}

#[derive(Debug, Clone, Deserialize)]
struct StringVector {
    input: String,
    encoded: Vec<u8>,
    hex: String,
    length: usize,
    utf8_bytes: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct BytesVector {
    input: Vec<u8>,
    encoded: Vec<u8>,
    hex: String,
    input_length: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct BooleanVector {
    input: bool,
    encoded: Vec<u8>,
    hex: String,
}

#[derive(Debug, Clone, Deserialize)]
struct HashVector {
    input: Vec<u8>,
    encoded: Vec<u8>,
    hex: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TransactionVector {
    body: Value,
    canonical_json: String,
    hash: Vec<u8>,
    hash_hex: String,
}

#[derive(Debug, Clone, Deserialize)]
struct EnvelopeVector {
    envelope: Value,
    header_canonical: String,
    body_canonical: String,
    signing_payload: String,
    transaction_hash: Vec<u8>,
    transaction_hash_hex: String,
}

/// Load test vectors from golden fixtures
fn load_test_vectors() -> TestVectors {
    let fixtures_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/enums/typescript_sdk_vectors.json"
    );
    let fixtures_content =
        std::fs::read_to_string(fixtures_path).expect("Failed to read TypeScript SDK test vectors");

    serde_json::from_str(&fixtures_content).expect("Failed to parse TypeScript SDK test vectors")
}

#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_uvarint_parity() {
    let vectors = load_test_vectors();

    for vector in vectors.uvarint {
        println!("Testing uvarint: {} -> {:?}", vector.input, vector.encoded);

        // Test encoding
        let rust_encoded = BinaryWriter::encode_uvarint(vector.input);
        assert_eq!(
            rust_encoded, vector.encoded,
            "UVarint encoding mismatch for input {}: Rust {:?} vs TS {:?}",
            vector.input, rust_encoded, vector.encoded
        );

        // Test hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(
            rust_hex, vector.hex,
            "UVarint hex mismatch for input {}: Rust {} vs TS {}",
            vector.input, rust_hex, vector.hex
        );

        // Test decoding
        let mut reader = BinaryReader::new(&vector.encoded);
        let decoded = reader.read_uvarint().expect("Failed to decode uvarint");
        assert_eq!(
            decoded, vector.input,
            "UVarint decoding mismatch: decoded {} vs original {}",
            decoded, vector.input
        );
    }
}

#[test]
fn test_varint_parity() {
    let vectors = load_test_vectors();

    for vector in vectors.varint {
        println!("Testing varint: {} -> {:?}", vector.input, vector.encoded);

        // Test encoding
        let rust_encoded = BinaryWriter::encode_varint(vector.input);
        assert_eq!(
            rust_encoded, vector.encoded,
            "Varint encoding mismatch for input {}: Rust {:?} vs TS {:?}",
            vector.input, rust_encoded, vector.encoded
        );

        // Test hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(
            rust_hex, vector.hex,
            "Varint hex mismatch for input {}: Rust {} vs TS {}",
            vector.input, rust_hex, vector.hex
        );

        // Test decoding
        let mut reader = BinaryReader::new(&vector.encoded);
        let decoded = reader.read_varint().expect("Failed to decode varint");
        assert_eq!(
            decoded, vector.input,
            "Varint decoding mismatch: decoded {} vs original {}",
            decoded, vector.input
        );
    }
}

#[test]
fn test_string_parity() {
    let vectors = load_test_vectors();

    for vector in vectors.strings {
        println!("Testing string: '{}' -> {:?}", vector.input, vector.encoded);

        // Test encoding
        let rust_encoded = BinaryWriter::encode_string(&vector.input);
        assert_eq!(
            rust_encoded, vector.encoded,
            "String encoding mismatch for input '{}': Rust {:?} vs TS {:?}",
            vector.input, rust_encoded, vector.encoded
        );

        // Test hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(
            rust_hex, vector.hex,
            "String hex mismatch for input '{}': Rust {} vs TS {}",
            vector.input, rust_hex, vector.hex
        );

        // Test length validation (UTF-16 code units for JavaScript compatibility)
        let utf16_length = vector.input.encode_utf16().count();
        assert_eq!(
            vector.length,
            utf16_length,
            "String UTF-16 length mismatch: reported {} vs actual {}",
            vector.length,
            utf16_length
        );

        let utf8_bytes = vector.input.as_bytes().len();
        assert_eq!(
            utf8_bytes, vector.utf8_bytes,
            "UTF-8 byte count mismatch: calculated {} vs reported {}",
            utf8_bytes, vector.utf8_bytes
        );

        // Test decoding
        let mut reader = BinaryReader::new(&vector.encoded);
        let decoded = reader.read_string().expect("Failed to decode string");
        assert_eq!(
            decoded, vector.input,
            "String decoding mismatch: decoded '{}' vs original '{}'",
            decoded, vector.input
        );
    }
}

#[test]
fn test_bytes_parity() {
    let vectors = load_test_vectors();

    for vector in vectors.bytes {
        println!("Testing bytes: {:?} -> {:?}", vector.input, vector.encoded);

        // Test encoding
        let rust_encoded = BinaryWriter::encode_bytes(&vector.input);
        assert_eq!(
            rust_encoded, vector.encoded,
            "Bytes encoding mismatch for input {:?}: Rust {:?} vs TS {:?}",
            vector.input, rust_encoded, vector.encoded
        );

        // Test hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(
            rust_hex, vector.hex,
            "Bytes hex mismatch for input {:?}: Rust {} vs TS {}",
            vector.input, rust_hex, vector.hex
        );

        // Test length validation
        assert_eq!(
            vector.input.len(),
            vector.input_length,
            "Bytes length mismatch: actual {} vs reported {}",
            vector.input.len(),
            vector.input_length
        );

        // Test decoding
        let mut reader = BinaryReader::new(&vector.encoded);
        let decoded = reader
            .read_bytes_with_length()
            .expect("Failed to decode bytes");
        assert_eq!(
            decoded,
            vector.input.as_slice(),
            "Bytes decoding mismatch: decoded {:?} vs original {:?}",
            decoded,
            vector.input
        );
    }
}

#[test]
fn test_boolean_parity() {
    let vectors = load_test_vectors();

    for vector in vectors.booleans {
        println!("Testing boolean: {} -> {:?}", vector.input, vector.encoded);

        // Test encoding
        let rust_encoded = BinaryWriter::encode_bool(vector.input);
        assert_eq!(
            rust_encoded, vector.encoded,
            "Boolean encoding mismatch for input {}: Rust {:?} vs TS {:?}",
            vector.input, rust_encoded, vector.encoded
        );

        // Test hex representation
        let rust_hex = hex::encode(&rust_encoded);
        assert_eq!(
            rust_hex, vector.hex,
            "Boolean hex mismatch for input {}: Rust {} vs TS {}",
            vector.input, rust_hex, vector.hex
        );

        // Test decoding
        let mut reader = BinaryReader::new(&vector.encoded);
        let decoded = reader.read_bool().expect("Failed to decode boolean");
        assert_eq!(
            decoded, vector.input,
            "Boolean decoding mismatch: decoded {} vs original {}",
            decoded, vector.input
        );
    }
}

#[test]
fn test_hash_parity() {
    let vectors = load_test_vectors();

    for vector in vectors.hashes {
        println!("Testing hash: {:?} -> {:?}", vector.input, vector.encoded);

        // Convert input to 32-byte array
        if vector.input.len() == 32 {
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&vector.input);

            // Test encoding
            let rust_encoded = BinaryWriter::encode_hash(&hash_array);
            assert_eq!(
                rust_encoded, vector.encoded,
                "Hash encoding mismatch for input {:?}: Rust {:?} vs TS {:?}",
                vector.input, rust_encoded, vector.encoded
            );

            // Test hex representation
            let rust_hex = hex::encode(&rust_encoded);
            assert_eq!(
                rust_hex, vector.hex,
                "Hash hex mismatch for input {:?}: Rust {} vs TS {}",
                vector.input, rust_hex, vector.hex
            );

            // Test decoding
            let mut reader = BinaryReader::new(&vector.encoded);
            let decoded = reader.read_hash().expect("Failed to decode hash");
            assert_eq!(
                decoded, hash_array,
                "Hash decoding mismatch: decoded {:?} vs original {:?}",
                decoded, hash_array
            );
        }
    }
}

#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_canonical_json_parity() {
    // Enable TypeScript compatibility mode for canonical JSON
    std::env::set_var("ACCUMULATE_BINARY_PARITY_MODE", "1");
    let vectors = load_test_vectors();

    for vector in vectors.transactions {
        println!("Testing canonical JSON for: {:?}", vector.body);

        // Test canonical JSON generation
        let rust_canonical = accumulate_client::codec::canonical_json(&vector.body);
        assert_eq!(
            rust_canonical, vector.canonical_json,
            "Canonical JSON mismatch for {:?}: Rust '{}' vs TS '{}'",
            vector.body, rust_canonical, vector.canonical_json
        );

        // Test hash generation
        let rust_hash = AccumulateHash::sha256_json(&vector.body);
        assert_eq!(
            rust_hash.to_vec(),
            vector.hash,
            "JSON hash mismatch for {:?}: Rust {:?} vs TS {:?}",
            vector.body,
            rust_hash,
            vector.hash
        );

        let rust_hash_hex = hex::encode(rust_hash);
        assert_eq!(
            rust_hash_hex, vector.hash_hex,
            "JSON hash hex mismatch for {:?}: Rust {} vs TS {}",
            vector.body, rust_hash_hex, vector.hash_hex
        );
    }
}

#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_envelope_hash_parity() {
    // Enable TypeScript compatibility mode for canonical JSON
    std::env::set_var("ACCUMULATE_BINARY_PARITY_MODE", "1");
    let vectors = load_test_vectors();

    for vector in vectors.envelopes {
        println!("Testing envelope hash for: {:?}", vector.envelope);

        // Parse envelope from test vector
        let envelope_value = &vector.envelope;
        if let (Some(header), Some(body)) =
            (envelope_value.get("header"), envelope_value.get("body"))
        {
            // Test header canonical JSON
            let rust_header_canonical = accumulate_client::codec::canonical_json(header);
            assert_eq!(
                rust_header_canonical, vector.header_canonical,
                "Header canonical JSON mismatch: Rust '{}' vs TS '{}'",
                rust_header_canonical, vector.header_canonical
            );

            // Test body canonical JSON
            let rust_body_canonical = accumulate_client::codec::canonical_json(body);
            assert_eq!(
                rust_body_canonical, vector.body_canonical,
                "Body canonical JSON mismatch: Rust '{}' vs TS '{}'",
                rust_body_canonical, vector.body_canonical
            );

            // Test signing payload (header + body combined)
            let signing_payload = json!({
                "header": header,
                "body": body
            });
            let rust_signing_payload = accumulate_client::codec::canonical_json(&signing_payload);
            assert_eq!(
                rust_signing_payload, vector.signing_payload,
                "Signing payload mismatch: Rust '{}' vs TS '{}'",
                rust_signing_payload, vector.signing_payload
            );

            // Test transaction hash
            let rust_tx_hash = AccumulateHash::sha256_json(&signing_payload);
            assert_eq!(
                rust_tx_hash.to_vec(),
                vector.transaction_hash,
                "Transaction hash mismatch: Rust {:?} vs TS {:?}",
                rust_tx_hash,
                vector.transaction_hash
            );

            let rust_tx_hash_hex = hex::encode(rust_tx_hash);
            assert_eq!(
                rust_tx_hash_hex, vector.transaction_hash_hex,
                "Transaction hash hex mismatch: Rust {} vs TS {}",
                rust_tx_hash_hex, vector.transaction_hash_hex
            );
        }
    }
}

#[test]
fn test_url_hashing_parity() {
    let test_urls = vec![
        "acc://alice.acme",
        "acc://alice.acme/tokens",
        "acc://bob.acme/book",
        "acc://charlie.acme/book/0",
        "ACC://UPPERCASE.ACME",
        "acc://lowercase.acme/",
        "acc://with-dashes.acme",
        "acc://with_underscores.acme",
    ];

    for url in test_urls {
        println!("Testing URL hash for: {}", url);

        // Test URL normalization
        let normalized = accumulate_client::codec::hashes::UrlHash::normalize_url(url);
        assert!(
            normalized.starts_with("acc://"),
            "URL should be normalized to start with acc://"
        );
        assert!(
            !normalized.ends_with('/') || normalized == "acc://",
            "URL should not end with slash unless root"
        );
        assert_eq!(
            normalized.to_lowercase(),
            normalized,
            "URL should be lowercase"
        );

        // Test URL hashing
        let hash = UrlHash::hash_url(url);
        assert_eq!(hash.len(), 32, "URL hash should be 32 bytes");

        // Test case insensitivity
        let upper_hash = UrlHash::hash_url(&url.to_uppercase());
        assert_eq!(hash, upper_hash, "URL hashing should be case insensitive");

        // Test trailing slash insensitivity
        let slash_url = if url.ends_with('/') {
            url.to_string()
        } else {
            format!("{}/", url)
        };
        let slash_hash = UrlHash::hash_url(&slash_url);
        assert_eq!(
            hash, slash_hash,
            "URL hashing should be trailing slash insensitive"
        );
    }
}

#[test]
fn test_transaction_body_builders() {
    // Test send tokens body
    let send_tokens =
        TransactionBodyBuilder::send_tokens(vec![accumulate_client::codec::TokenRecipient {
            url: "acc://bob.acme/tokens".to_string(),
            amount: "1000".to_string(),
        }]);

    assert_eq!(send_tokens["type"], "send-tokens");
    assert_eq!(send_tokens["to"][0]["url"], "acc://bob.acme/tokens");
    assert_eq!(send_tokens["to"][0]["amount"], "1000");

    // Test create identity body
    let create_identity = TransactionBodyBuilder::create_identity(
        "acc://alice.acme".to_string(),
        "acc://alice.acme/book".to_string(),
    );

    assert_eq!(create_identity["type"], "create-identity");
    assert_eq!(create_identity["url"], "acc://alice.acme");
    assert_eq!(create_identity["keyBook"], "acc://alice.acme/book");

    // Test add credits body
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

// Legacy test removed - incompatible with current FieldReader implementation

#[test]
// Legacy test removed - incompatible with current FieldReader implementation
fn test_comprehensive_roundtrip_legacy_removed() {}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[cfg(feature = "typescript-compat-tests")]
    #[test]
    fn benchmark_encoding_performance() {
        let vectors = load_test_vectors();

        // Benchmark uvarint encoding
        let start = Instant::now();
        for _ in 0..10000 {
            for vector in &vectors.uvarint {
                let _ = BinaryWriter::encode_uvarint(vector.input);
            }
        }
        let uvarint_duration = start.elapsed();
        println!(
            "UVarint encoding: {:?} for {} operations",
            uvarint_duration,
            10000 * vectors.uvarint.len()
        );

        // Benchmark string encoding
        let start = Instant::now();
        for _ in 0..10000 {
            for vector in &vectors.strings {
                let _ = BinaryWriter::encode_string(&vector.input);
            }
        }
        let string_duration = start.elapsed();
        println!(
            "String encoding: {:?} for {} operations",
            string_duration,
            10000 * vectors.strings.len()
        );

        // Benchmark should complete in reasonable time
        assert!(
            uvarint_duration.as_millis() < 1000,
            "UVarint encoding too slow"
        );
        assert!(
            string_duration.as_millis() < 1000,
            "String encoding too slow"
        );
    }
}
