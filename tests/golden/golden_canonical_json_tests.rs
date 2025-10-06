use accumulate_client::*;
use serde_json::json;
use std::{fs, path::PathBuf};

fn write_or_read(path: &PathBuf, v: &serde_json::Value) -> serde_json::Value {
    if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !path.exists() {
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(path, v.to_string()).unwrap();
        return v.clone(); // In write mode, just return the input
    }
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn canonical_write_data_tx() {
    // Build a minimal writeData transaction body using generated models
    let body = generated::transactions::TransactionBody::WriteData(
        generated::transactions::WriteDataBody {
            entry: json!({"data": "test data", "value": 42}),
            scratch: Some(false),
            write_to_state: Some(true),
        }
    );

    // Convert to JSON value for canonical testing
    let v = serde_json::to_value(&body).unwrap();

    // Test canonical JSON generation
    let canonical = canonical_json(&v);

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/golden/transactions/canonical/writeData.json");

    let golden = json!({
        "transaction_body": v,
        "canonical_json": canonical
    });

    let expected = write_or_read(&p, &golden);
    assert_eq!(golden, expected, "writeData canonical JSON mismatch");
}

#[test]
fn canonical_send_tokens_tx() {
    // Build a sendTokens transaction body
    let body = generated::transactions::TransactionBody::SendTokens(
        generated::transactions::SendTokensBody {
            hash: None,
            meta: None,
            to: vec![
                json!({
                    "Url": "acc://bob.acme/tokens",
                    "Amount": "1000"
                })
            ]
        }
    );

    let v = serde_json::to_value(&body).unwrap();
    let canonical = canonical_json(&v);

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/golden/transactions/canonical/sendTokens.json");

    let golden = json!({
        "transaction_body": v,
        "canonical_json": canonical
    });

    let expected = write_or_read(&p, &golden);
    assert_eq!(golden, expected, "sendTokens canonical JSON mismatch");
}

#[test]
fn canonical_create_identity_tx() {
    // Build a createIdentity transaction body
    let body = generated::transactions::TransactionBody::CreateIdentity(
        generated::transactions::CreateIdentityBody {
            url: "acc://alice.acme".to_string(),
            key_hash: None,
            key_book_url: Some("acc://alice.acme/book".to_string()),
            authorities: None,
        }
    );

    let v = serde_json::to_value(&body).unwrap();
    let canonical = canonical_json(&v);

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/golden/transactions/canonical/createIdentity.json");

    let golden = json!({
        "transaction_body": v,
        "canonical_json": canonical
    });

    let expected = write_or_read(&p, &golden);
    assert_eq!(golden, expected, "createIdentity canonical JSON mismatch");
}

#[test]
fn canonical_add_credits_tx() {
    // Build an addCredits transaction body
    let body = generated::transactions::TransactionBody::AddCredits(
        generated::transactions::AddCreditsBody {
            recipient: "acc://alice.acme".to_string(),
            amount: "100000".to_string(),
            oracle: 500000,
        }
    );

    let v = serde_json::to_value(&body).unwrap();
    let canonical = canonical_json(&v);

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/golden/transactions/canonical/addCredits.json");

    let golden = json!({
        "transaction_body": v,
        "canonical_json": canonical
    });

    let expected = write_or_read(&p, &golden);
    assert_eq!(golden, expected, "addCredits canonical JSON mismatch");
}

#[test]
fn canonical_transaction_header() {
    // Test canonical JSON for transaction headers
    let headers = vec![
        ("minimal", generated::header::TransactionHeader {
            principal: "acc://example.acme".to_string(),
            initiator: vec![0u8; 32],
            memo: None,
            metadata: None,
            expire: None,
            hold_until: None,
            authorities: None,
        }),
        ("with_memo", generated::header::TransactionHeader {
            principal: "acc://test.acme/tokens".to_string(),
            initiator: vec![0x01; 32],
            memo: Some("test memo".to_string()),
            metadata: None,
            expire: None,
            hold_until: None,
            authorities: None,
        }),
        ("with_metadata", generated::header::TransactionHeader {
            principal: "acc://meta.acme".to_string(),
            initiator: vec![0x02; 32],
            memo: None,
            metadata: Some(vec![0x01, 0x02, 0x03]),
            expire: None,
            hold_until: None,
            authorities: None,
        }),
        ("with_expiry", generated::header::TransactionHeader {
            principal: "acc://expire.acme".to_string(),
            initiator: vec![0x03; 32],
            memo: None,
            metadata: None,
            expire: Some(generated::header::ExpireOptions {
                at_time: Some(1234567890)
            }),
            hold_until: None,
            authorities: None,
        }),
        ("with_authorities", generated::header::TransactionHeader {
            principal: "acc://auth.acme".to_string(),
            initiator: vec![0x04; 32],
            memo: None,
            metadata: None,
            expire: None,
            hold_until: None,
            authorities: Some(vec!["acc://authority1.acme".to_string(), "acc://authority2.acme".to_string()]),
        }),
    ];

    for (name, header) in headers {
        let v = serde_json::to_value(&header).unwrap();
        let canonical = canonical_json(&v);

        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
          .join("tests/golden/transactions/canonical")
          .join(format!("header_{}.json", name));

        let golden = json!({
            "header": v,
            "canonical_json": canonical
        });

        let expected = write_or_read(&p, &golden);
        assert_eq!(golden, expected, "header {} canonical JSON mismatch", name);
    }
}

#[test]
fn canonical_signature_types() {
    // Test canonical JSON for different signature types
    let signatures = vec![
        ("ed25519", generated::signatures::Signature::ED25519(
            generated::signatures::ED25519Signature {
                public_key: vec![0u8; 32],
                signature: vec![0u8; 64],
                signer: "acc://signer.acme/book/1".to_string(),
                signer_version: 1,
                timestamp: Some(1234567890),
                vote: None,
                transaction_hash: None,
                memo: Some("signature memo".to_string()),
                data: None,
            }
        )),
        ("legacy_ed25519", generated::signatures::Signature::LegacyED25519(
            generated::signatures::LegacyED25519Signature {
                timestamp: 1234567890,
                public_key: vec![0u8; 32],
                signature: vec![0u8; 64],
                signer: "acc://legacy.acme/book/1".to_string(),
                signer_version: 1,
                vote: None,
                transaction_hash: None,
            }
        )),
        ("delegated", generated::signatures::Signature::Delegated(
            generated::signatures::DelegatedSignature {
                signature: Box::new(generated::signatures::Signature::ED25519(
                    generated::signatures::ED25519Signature {
                        public_key: vec![0u8; 32],
                        signature: vec![0u8; 64],
                        signer: "acc://inner.acme/book/1".to_string(),
                        signer_version: 1,
                        timestamp: None,
                        vote: None,
                        transaction_hash: None,
                        memo: None,
                        data: None,
                    }
                )),
                delegator: "acc://delegator.acme/book/1".to_string(),
            }
        )),
    ];

    for (name, signature) in signatures {
        let v = serde_json::to_value(&signature).unwrap();
        let canonical = canonical_json(&v);

        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
          .join("tests/golden/signatures")
          .join(format!("canonical_{}.json", name));

        let golden = json!({
            "signature": v,
            "canonical_json": canonical
        });

        let expected = write_or_read(&p, &golden);
        assert_eq!(golden, expected, "signature {} canonical JSON mismatch", name);
    }
}

#[test]
fn canonical_complex_nested_objects() {
    // Test canonical JSON with complex nested structures
    let complex_cases = vec![
        ("empty_object", json!({})),
        ("empty_array", json!([])),
        ("mixed_types", json!({
            "string": "value",
            "number": 42,
            "boolean": true,
            "null": null,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        })),
        ("key_ordering", json!({
            "z_last": "should be last",
            "a_first": "should be first",
            "m_middle": "should be middle"
        })),
        ("deep_nesting", json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "value": "deep"
                        }
                    }
                }
            }
        })),
    ];

    for (name, test_value) in complex_cases {
        let canonical = canonical_json(&test_value);

        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
          .join("tests/golden/canonical")
          .join(format!("canonical_{}.json", name));

        let golden = json!({
            "input": test_value,
            "canonical_json": canonical
        });

        let expected = write_or_read(&p, &golden);
        assert_eq!(golden, expected, "canonical {} JSON mismatch", name);
    }
}

#[test]
fn canonical_deterministic_ordering() {
    // Verify that canonical JSON is deterministic regardless of input ordering
    let input1 = json!({
        "zebra": "animal",
        "alpha": "letter",
        "beta": "test"
    });

    let input2 = json!({
        "alpha": "letter",
        "zebra": "animal",
        "beta": "test"
    });

    let canonical1 = canonical_json(&input1);
    let canonical2 = canonical_json(&input2);

    assert_eq!(canonical1, canonical2, "Canonical JSON should be deterministic regardless of input order");

    // Expected order: alpha, beta, zebra (alphabetical)
    let expected_canonical = r#"{"alpha":"letter","beta":"test","zebra":"animal"}"#;
    assert_eq!(canonical1, expected_canonical);

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/golden/canonical/canonical_deterministic.json");

    let golden = json!({
        "input1": input1,
        "input2": input2,
        "canonical_result": canonical1,
        "deterministic": true
    });

    let expected = write_or_read(&p, &golden);
    assert_eq!(golden, expected, "canonical deterministic JSON mismatch");
}