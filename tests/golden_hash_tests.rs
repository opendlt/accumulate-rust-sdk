use accumulate_client::*;
use serde_json::json;
use std::{fs, path::PathBuf};

fn gpath(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests").join("golden_vectors").join("hash")
      .join(format!("{name}.json"))
}

#[test]
fn transaction_header_hash_golden() {
    // Build a minimal but deterministic header using the established canonical format
    let hdr = generated::header::TransactionHeader {
        principal: "acc://example.acme".to_string(),
        initiator: vec![0u8; 32], // 32-byte hash as required
        memo: None,
        metadata: None,
        expire: None,
        hold_until: None,
        authorities: None,
    };

    // Serialize using the canonical JSON implementation (our established truth)
    let canon_json = canonical_json(&serde_json::to_value(&hdr).unwrap());
    let h = runtime::hashing::sha256(canon_json.as_bytes());
    let actual = hex::encode(h);

    let p = gpath("transaction_header_hash");
    if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !p.exists() {
        fs::create_dir_all(p.parent().unwrap()).ok();
        let golden = json!({
            "input_header": hdr,
            "canonical_json": canon_json,
            "hash": actual
        });
        fs::write(&p, golden.to_string()).unwrap();
        return; // In write mode, just generate and exit
    }

    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    assert_eq!(actual, expected["hash"].as_str().unwrap(), "header hash mismatch");

    // Also verify the canonical JSON matches
    assert_eq!(canon_json, expected["canonical_json"].as_str().unwrap(), "canonical JSON mismatch");
}

#[test]
fn transaction_header_with_optional_fields_hash_golden() {
    // Test with some optional fields populated
    let hdr = generated::header::TransactionHeader {
        principal: "acc://test.acme/tokens".to_string(),
        initiator: vec![0x01; 32], // Different initiator
        memo: Some("test memo".to_string()),
        metadata: Some(vec![0x01, 0x02, 0x03]),
        expire: Some(generated::header::ExpireOptions {
            at_time: Some(1234567890)
        }),
        hold_until: None,
        authorities: Some(vec!["acc://auth.acme".to_string()]),
    };

    let canon_json = canonical_json(&serde_json::to_value(&hdr).unwrap());
    let h = runtime::hashing::sha256(canon_json.as_bytes());
    let actual = hex::encode(h);

    let p = gpath("transaction_header_with_fields_hash");
    if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !p.exists() {
        fs::create_dir_all(p.parent().unwrap()).ok();
        let golden = json!({
            "input_header": hdr,
            "canonical_json": canon_json,
            "hash": actual
        });
        fs::write(&p, golden.to_string()).unwrap();
        return; // In write mode, just generate and exit
    }

    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    assert_eq!(actual, expected["hash"].as_str().unwrap(), "header with fields hash mismatch");
    assert_eq!(canon_json, expected["canonical_json"].as_str().unwrap(), "canonical JSON with fields mismatch");
}

#[test]
fn url_hash_golden() {
    // Test URL hashing with the established UrlHash implementation
    let test_urls = vec![
        "acc://alice.acme",
        "acc://alice.acme/tokens",
        "acc://bob.acme/book/1",
        "ACC://UPPERCASE.ACME", // Should normalize to lowercase
    ];

    for url in test_urls {
        let normalized = UrlHash::normalize_url(url);
        let hash_bytes = UrlHash::hash_url(url);
        let hash_hex = hex::encode(hash_bytes);

        let safe_name = url.replace("://", "_").replace("/", "_").replace(".", "_").to_lowercase();
        let p = gpath(&format!("url_hash_{}", safe_name));

        if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !p.exists() {
            fs::create_dir_all(p.parent().unwrap()).ok();
            let golden = json!({
                "original_url": url,
                "normalized_url": normalized,
                "hash": hash_hex
            });
            fs::write(&p, golden.to_string()).unwrap();
            continue; // In write mode, just generate and continue to next
        }

        let expected: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(hash_hex, expected["hash"].as_str().unwrap(), "URL hash mismatch for {}", url);
        assert_eq!(normalized, expected["normalized_url"].as_str().unwrap(), "URL normalization mismatch for {}", url);
    }
}

#[test]
fn sha256_deterministic_golden() {
    // Test basic SHA-256 determinism
    let test_inputs = vec![
        b"hello world".to_vec(),
        b"".to_vec(), // empty input
        b"The quick brown fox jumps over the lazy dog".to_vec(),
        vec![0u8; 32], // 32 zero bytes
        vec![0xFF; 64], // 64 0xFF bytes
    ];

    for (i, input) in test_inputs.iter().enumerate() {
        let hash_bytes = runtime::hashing::sha256(input);
        let hash_hex = hex::encode(hash_bytes);

        let p = gpath(&format!("sha256_test_{}", i));

        if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !p.exists() {
            fs::create_dir_all(p.parent().unwrap()).ok();
            let golden = json!({
                "input": hex::encode(input),
                "input_length": input.len(),
                "hash": hash_hex
            });
            fs::write(&p, golden.to_string()).unwrap();
            continue; // In write mode, just generate and continue to next
        }

        let expected: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(hash_hex, expected["hash"].as_str().unwrap(), "SHA-256 hash mismatch for input {}", i);
    }
}