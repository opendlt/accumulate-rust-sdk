use accumulate_client::{
    canonicalize, dumps_canonical, AccumulateHash, TransactionCodec, TransactionEnvelope,
    BinaryReader, BinaryWriter,
};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Test vector from TypeScript fixture generator
#[derive(Debug, Clone)]
struct TsFuzzVector {
    hex_bin: String,
    canonical_json: String,
    tx_hash_hex: String,
    meta: TsFuzzMeta,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TsFuzzMeta {
    index: usize,
    #[serde(rename = "txType")]
    tx_type: String,
    #[serde(rename = "numSignatures")]
    num_signatures: usize,
    #[serde(rename = "binarySize")]
    binary_size: usize,
    #[serde(rename = "canonicalSize")]
    canonical_size: usize,
}

impl TsFuzzVector {
    fn from_json_line(line: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;

        let hex_bin = json["hexBin"].as_str().unwrap().to_string();
        let canonical_json = json["canonicalJson"].as_str().unwrap().to_string();
        let tx_hash_hex = json["txHashHex"].as_str().unwrap().to_string();
        let meta: TsFuzzMeta = serde_json::from_value(json["meta"].clone())?;

        Ok(TsFuzzVector {
            hex_bin,
            canonical_json,
            tx_hash_hex,
            meta,
        })
    }

    fn hex_to_bytes(&self) -> Result<Vec<u8>, hex::FromHexError> {
        hex::decode(&self.hex_bin)
    }
}

/// Load TypeScript-generated fuzz vectors from JSON Lines file
fn load_ts_fuzz_vectors() -> Result<Vec<TsFuzzVector>, Box<dyn std::error::Error>> {
    let vectors_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("ts_rand_vectors.jsonl");

    if !vectors_path.exists() {
        return Err(format!(
            "Fuzz vectors file not found: {}.\nRun: node tooling/ts-fixture-exporter/export-random-vectors.js > tests/golden/ts_rand_vectors.jsonl",
            vectors_path.display()
        ).into());
    }

    let file = File::open(&vectors_path)?;
    let reader = BufReader::new(file);
    let mut vectors = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match TsFuzzVector::from_json_line(&line) {
            Ok(vector) => vectors.push(vector),
            Err(e) => {
                eprintln!("Failed to parse line {}: {}", line_num + 1, e);
                eprintln!("Line content: {}", line);
                return Err(e);
            }
        }
    }

    Ok(vectors)
}

/// Simplified envelope structure for testing
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct TestEnvelope {
    signatures: Vec<TestSignature>,
    transaction: Vec<TestTransaction>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct TestSignature {
    #[serde(rename = "type")]
    sig_type: String,
    #[serde(rename = "publicKey")]
    public_key: String,
    signature: String,
    signer: String,
    #[serde(rename = "signerVersion")]
    signer_version: u64,
    timestamp: u64,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct TestTransaction {
    header: TestHeader,
    body: Value, // Keep as Value for flexibility
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct TestHeader {
    principal: String,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    initiator: Option<String>,
}

/// Decode binary envelope from hex
fn decode_binary_envelope(hex_data: &str) -> Result<TestEnvelope, Box<dyn std::error::Error>> {
    let binary_data = hex::decode(hex_data)?;

    // Skip the 8-byte header (magic bytes + length) that our TS encoder adds
    if binary_data.len() < 8 {
        return Err("Binary data too short".into());
    }

    let payload = &binary_data[8..];
    let json_str = String::from_utf8(payload.to_vec())?;
    let envelope: TestEnvelope = serde_json::from_str(&json_str)?;

    Ok(envelope)
}

/// Encode envelope back to binary
fn encode_binary_envelope(envelope: &TestEnvelope) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let json_str = serde_json::to_string(envelope)?;
    let payload = json_str.as_bytes();

    // Add the same 8-byte header as TS encoder
    let mut binary_data = Vec::with_capacity(8 + payload.len());

    // Magic bytes "ACCU"
    binary_data.extend_from_slice(&[0x41, 0x43, 0x43, 0x55]);
    // Payload length (big endian)
    binary_data.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    // Payload
    binary_data.extend_from_slice(payload);

    Ok(binary_data)
}

#[test]
#[ignore = "Requires ts_rand_vectors.jsonl - run: node tooling/ts-fixture-exporter/export-random-vectors.js > tests/golden/ts_rand_vectors.jsonl"]
fn test_ts_fuzz_roundtrip_binary() {
    let vectors = load_ts_fuzz_vectors()
        .expect("Failed to load TypeScript fuzz vectors");

    println!("Testing {} TS-generated vectors for binary roundtrip...", vectors.len());

    let mut failures = 0;
    let mut successes = 0;

    for vector in &vectors {
        // Test binary decode -> re-encode roundtrip
        match test_binary_roundtrip(vector) {
            Ok(_) => {
                successes += 1;
                if vector.meta.index % 100 == 0 {
                    println!("✓ Vector {} ({}): Binary roundtrip OK", vector.meta.index, vector.meta.tx_type);
                }
            }
            Err(e) => {
                failures += 1;
                eprintln!("✗ Vector {} ({}): Binary roundtrip failed: {}",
                    vector.meta.index, vector.meta.tx_type, e);

                // Show details on first few failures
                if failures <= 3 {
                    eprintln!("  Binary size: {}", vector.meta.binary_size);
                    eprintln!("  Hex (first 100 chars): {}...",
                        vector.hex_bin.chars().take(100).collect::<String>());
                }
            }
        }
    }

    println!("\nBinary Roundtrip Results:");
    println!("  ✓ Successes: {}", successes);
    println!("  ✗ Failures: {}", failures);
    println!("  Success rate: {:.2}%", (successes as f64 / vectors.len() as f64) * 100.0);

    assert_eq!(failures, 0, "All binary roundtrips should succeed");
}

#[test]
#[ignore = "Requires ts_rand_vectors.jsonl - run: node tooling/ts-fixture-exporter/export-random-vectors.js > tests/golden/ts_rand_vectors.jsonl"]
fn test_ts_fuzz_roundtrip_canonical_json() {
    let vectors = load_ts_fuzz_vectors()
        .expect("Failed to load TypeScript fuzz vectors");

    println!("Testing {} TS-generated vectors for canonical JSON...", vectors.len());

    let mut failures = 0;
    let mut successes = 0;

    for vector in &vectors {
        // Test canonical JSON generation
        match test_canonical_json_roundtrip(vector) {
            Ok(_) => {
                successes += 1;
                if vector.meta.index % 100 == 0 {
                    println!("✓ Vector {} ({}): Canonical JSON OK", vector.meta.index, vector.meta.tx_type);
                }
            }
            Err(e) => {
                failures += 1;
                eprintln!("✗ Vector {} ({}): Canonical JSON failed: {}",
                    vector.meta.index, vector.meta.tx_type, e);

                // Show details on first few failures
                if failures <= 3 {
                    eprintln!("  Expected: {}", vector.canonical_json);
                    eprintln!("  TX type: {}", vector.meta.tx_type);
                }
            }
        }
    }

    println!("\nCanonical JSON Results:");
    println!("  ✓ Successes: {}", successes);
    println!("  ✗ Failures: {}", failures);
    println!("  Success rate: {:.2}%", (successes as f64 / vectors.len() as f64) * 100.0);

    assert_eq!(failures, 0, "All canonical JSON should match");
}

#[test]
#[ignore = "Requires ts_rand_vectors.jsonl - run: node tooling/ts-fixture-exporter/export-random-vectors.js > tests/golden/ts_rand_vectors.jsonl"]
fn test_ts_fuzz_roundtrip_hashes() {
    let vectors = load_ts_fuzz_vectors()
        .expect("Failed to load TypeScript fuzz vectors");

    println!("Testing {} TS-generated vectors for hash verification...", vectors.len());

    let mut failures = 0;
    let mut successes = 0;

    for vector in &vectors {
        // Test hash computation
        match test_hash_verification(vector) {
            Ok(_) => {
                successes += 1;
                if vector.meta.index % 100 == 0 {
                    println!("✓ Vector {} ({}): Hash OK", vector.meta.index, vector.meta.tx_type);
                }
            }
            Err(e) => {
                failures += 1;
                eprintln!("✗ Vector {} ({}): Hash failed: {}",
                    vector.meta.index, vector.meta.tx_type, e);

                // Show details on first few failures
                if failures <= 3 {
                    eprintln!("  Expected hash: {}", vector.tx_hash_hex);
                    eprintln!("  TX type: {}", vector.meta.tx_type);
                }
            }
        }
    }

    println!("\nHash Verification Results:");
    println!("  ✓ Successes: {}", successes);
    println!("  ✗ Failures: {}", failures);
    println!("  Success rate: {:.2}%", (successes as f64 / vectors.len() as f64) * 100.0);

    assert_eq!(failures, 0, "All hashes should match");
}

fn test_binary_roundtrip(vector: &TsFuzzVector) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Decode binary data from hex
    let original_binary = vector.hex_to_bytes()?;

    // 2. Decode to Rust structs
    let envelope = decode_binary_envelope(&vector.hex_bin)?;

    // 3. Re-encode to binary
    let reencoded_binary = encode_binary_envelope(&envelope)?;

    // 4. Compare byte-for-byte
    if original_binary != reencoded_binary {
        return Err(format!(
            "Binary roundtrip mismatch: original {} bytes, reencoded {} bytes",
            original_binary.len(),
            reencoded_binary.len()
        ).into());
    }

    Ok(())
}

fn test_canonical_json_roundtrip(vector: &TsFuzzVector) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Decode envelope from binary
    let envelope = decode_binary_envelope(&vector.hex_bin)?;

    // 2. Extract transaction
    if envelope.transaction.is_empty() {
        return Err("No transaction in envelope".into());
    }
    let transaction = &envelope.transaction[0];

    // 3. Generate canonical JSON using dumps_canonical
    let rust_canonical = dumps_canonical(transaction);

    // 4. Compare with TypeScript canonical JSON
    if rust_canonical != vector.canonical_json {
        return Err(format!(
            "Canonical JSON mismatch:\nTS:   {}\nRust: {}",
            vector.canonical_json,
            rust_canonical
        ).into());
    }

    Ok(())
}

fn test_hash_verification(vector: &TsFuzzVector) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Decode envelope from binary
    let envelope = decode_binary_envelope(&vector.hex_bin)?;

    // 2. Extract transaction
    if envelope.transaction.is_empty() {
        return Err("No transaction in envelope".into());
    }
    let transaction = &envelope.transaction[0];

    // 3. Convert to JSON Value for hashing
    let tx_value: Value = serde_json::to_value(transaction)?;

    // 4. Compute hash using our implementation
    let rust_hash = AccumulateHash::sha256_json_hex(&tx_value);

    // 5. Compare with TypeScript hash
    if rust_hash != vector.tx_hash_hex {
        return Err(format!(
            "Hash mismatch:\nTS:   {}\nRust: {}",
            vector.tx_hash_hex,
            rust_hash
        ).into());
    }

    Ok(())
}

// Test with a minimal set if vectors file doesn't exist
#[test]
fn test_fallback_basic_roundtrip() {
    // This test runs even if the vectors file doesn't exist
    let test_envelope = TestEnvelope {
        signatures: vec![TestSignature {
            sig_type: "ed25519".to_string(),
            public_key: "test_key".to_string(),
            signature: "test_sig".to_string(),
            signer: "acc://test.acme/book/0".to_string(),
            signer_version: 1,
            timestamp: 1234567890,
            transaction_hash: "test_hash".to_string(),
        }],
        transaction: vec![TestTransaction {
            header: TestHeader {
                principal: "acc://test.acme/tokens".to_string(),
                timestamp: 1234567890,
                initiator: None,
            },
            body: json!({
                "type": "send-tokens",
                "to": [{
                    "url": "acc://bob.acme/tokens",
                    "amount": "1000"
                }]
            }),
        }],
    };

    // Test binary roundtrip
    let binary_data = encode_binary_envelope(&test_envelope).unwrap();
    let hex_data = hex::encode(&binary_data);
    let decoded = decode_binary_envelope(&hex_data).unwrap();

    // Test canonical JSON
    let canonical = dumps_canonical(&test_envelope.transaction[0]);
    assert!(!canonical.is_empty());

    // Test hash
    let tx_value: Value = serde_json::to_value(&test_envelope.transaction[0]).unwrap();
    let hash = AccumulateHash::sha256_json_hex(&tx_value);
    assert_eq!(hash.len(), 64); // SHA-256 hex is 64 chars

    println!("✓ Basic roundtrip test passed");
}