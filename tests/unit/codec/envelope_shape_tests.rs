use accumulate_client::*;
use serde_json::{self as json, Value};
use std::fs;
use std::path::PathBuf;

fn golden_tx_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("transactions")
        .join("envelope")
}

fn ensure_golden_tx_dir() {
    std::fs::create_dir_all(golden_tx_dir()).unwrap();
}

/// Test transaction envelope structure for cross-stage integration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestTransactionEnvelope {
    pub header: TransactionHeader,
    pub body: TransactionBody,
    pub signatures: Vec<serde_json::Value>,
}

fn create_minimal_header() -> TransactionHeader {
    TransactionHeader {
        principal: "acc://test.acme".to_string(),
        initiator: hex::decode("deadbeef").unwrap(),
        memo: None,
        metadata: None,
        expire: None,
        hold_until: None,
        authorities: None,
    }
}

#[test]
fn test_write_data_envelope_shape() {
    println!("Testing WriteData envelope shape...");

    let header = create_minimal_header();

    // Create WriteDataBody using the generated transaction types
    let write_data_body = WriteDataBody {
        entry: json::json!({
            "data": "test data content"
        }),
        scratch: Some(false),
        write_to_state: Some(true),
    };

    let body = TransactionBody::WriteData(write_data_body);

    let envelope = TestTransactionEnvelope {
        header,
        body,
        signatures: vec![],
    };

    // Serialize to JSON and verify canonical shape
    let serialized = serde_json::to_value(&envelope).unwrap();
    println!("WriteData envelope JSON: {}", serde_json::to_string_pretty(&serialized).unwrap());

    // Verify camelCase structure
    assert!(serialized.get("header").is_some());
    assert!(serialized.get("body").is_some());
    assert!(serialized.get("signatures").is_some());

    // Verify header structure
    let header_json = serialized.get("header").unwrap();
    assert!(header_json.get("principal").is_some());
    assert!(header_json.get("initiator").is_some());

    // Verify body structure (should have type discriminant)
    let body_json = serialized.get("body").unwrap();
    assert!(body_json.is_object());
    let body_obj = body_json.as_object().unwrap();
    assert!(body_obj.contains_key("type"), "Body should have type field");
    assert_eq!(body_obj["type"], "writeData", "Should have correct wire type");

    // Write golden vector
    ensure_golden_tx_dir();
    let golden_file = golden_tx_dir().join("writeData.json");
    fs::write(&golden_file, serde_json::to_string_pretty(&serialized).unwrap()).unwrap();

    println!("✓ WriteData envelope shape test passed");
}

#[test]
fn test_create_token_envelope_shape() {
    println!("Testing CreateToken envelope shape...");

    let header = create_minimal_header();

    // Create CreateTokenBody
    let create_token_body = CreateTokenBody {
        url: "acc://token.acme".to_string(),
        symbol: "TEST".to_string(),
        precision: 8,
        properties: Some("acc://token-props.acme".to_string()),
        supply_limit: Some("1000000".to_string()),
        authorities: Some(vec!["acc://authority.acme".to_string()]),
    };

    let body = TransactionBody::CreateToken(create_token_body);

    let envelope = TestTransactionEnvelope {
        header,
        body,
        signatures: vec![],
    };

    // Serialize to JSON and verify canonical shape
    let serialized = serde_json::to_value(&envelope).unwrap();
    println!("CreateToken envelope JSON: {}", serde_json::to_string_pretty(&serialized).unwrap());

    // Verify structure
    assert!(serialized.get("header").is_some());
    assert!(serialized.get("body").is_some());

    let body_json = serialized.get("body").unwrap();
    let body_obj = body_json.as_object().unwrap();
    assert_eq!(body_obj["type"], "createToken", "Should have correct wire type");

    // Verify token-specific fields
    assert!(body_obj.get("Url").is_some());
    assert!(body_obj.get("Symbol").is_some());
    assert!(body_obj.get("Precision").is_some());

    // Write golden vector
    ensure_golden_tx_dir();
    let golden_file = golden_tx_dir().join("createToken.json");
    fs::write(&golden_file, serde_json::to_string_pretty(&serialized).unwrap()).unwrap();

    println!("✓ CreateToken envelope shape test passed");
}

#[test]
fn test_send_tokens_envelope_shape() {
    println!("Testing SendTokens envelope shape...");

    let header = create_minimal_header();

    // Create SendTokensBody with recipients
    let send_tokens_body = SendTokensBody {
        hash: None,
        meta: None,
        to: vec![
            json::json!({
                "Url": "acc://recipient1.acme",
                "Amount": "100"
            }),
            json::json!({
                "Url": "acc://recipient2.acme",
                "Amount": "200"
            }),
        ],
    };

    let body = TransactionBody::SendTokens(send_tokens_body);

    let envelope = TestTransactionEnvelope {
        header,
        body,
        signatures: vec![],
    };

    // Serialize to JSON and verify canonical shape
    let serialized = serde_json::to_value(&envelope).unwrap();
    println!("SendTokens envelope JSON: {}", serde_json::to_string_pretty(&serialized).unwrap());

    // Verify structure
    assert!(serialized.get("header").is_some());
    assert!(serialized.get("body").is_some());

    let body_json = serialized.get("body").unwrap();
    let body_obj = body_json.as_object().unwrap();
    assert_eq!(body_obj["type"], "sendTokens", "Should have correct wire type");

    // Verify recipients array
    assert!(body_obj.get("To").is_some());
    let recipients = body_obj["To"].as_array().unwrap();
    assert_eq!(recipients.len(), 2);

    // Verify recipient structure
    assert_eq!(recipients[0]["Url"], "acc://recipient1.acme");
    assert_eq!(recipients[0]["Amount"], "100");
    assert_eq!(recipients[1]["Url"], "acc://recipient2.acme");
    assert_eq!(recipients[1]["Amount"], "200");

    // Write golden vector
    ensure_golden_tx_dir();
    let golden_file = golden_tx_dir().join("sendTokens.json");
    fs::write(&golden_file, serde_json::to_string_pretty(&serialized).unwrap()).unwrap();

    println!("✓ SendTokens envelope shape test passed");
}

#[test]
fn test_envelope_roundtrip_consistency() {
    println!("Testing envelope roundtrip consistency...");

    let header = create_minimal_header();

    // Test multiple transaction types for roundtrip consistency
    let test_bodies = vec![
        ("WriteData", TransactionBody::WriteData(WriteDataBody {
            entry: json::json!({"data": "test"}),
            scratch: None,
            write_to_state: None,
        })),
        ("CreateIdentity", TransactionBody::CreateIdentity(CreateIdentityBody {
            url: "acc://identity.acme".to_string(),
            key_hash: None,
            key_book_url: None,
            authorities: None,
        })),
        ("AcmeFaucet", TransactionBody::AcmeFaucet(AcmeFaucetBody {
            url: "acc://test.acme".to_string(),
        })),
    ];

    for (name, body) in test_bodies {
        let envelope = TestTransactionEnvelope {
            header: header.clone(),
            body,
            signatures: vec![],
        };

        // Serialize
        let serialized = serde_json::to_value(&envelope).unwrap();

        // Deserialize back
        let deserialized: TestTransactionEnvelope = serde_json::from_value(serialized.clone()).unwrap();

        // Serialize again
        let reserialized = serde_json::to_value(&deserialized).unwrap();

        // Should be identical
        assert_eq!(serialized, reserialized, "{} envelope roundtrip should be consistent", name);

        println!("✓ {} envelope roundtrip consistent", name);
    }
}

#[test]
fn test_envelope_with_signatures() {
    println!("Testing envelope with signatures...");

    let header = create_minimal_header();

    let body = TransactionBody::WriteData(WriteDataBody {
        entry: json::json!({"data": "test"}),
        scratch: None,
        write_to_state: None,
    });

    // Add mock signatures
    let envelope = TestTransactionEnvelope {
        header,
        body,
        signatures: vec![
            json::json!({
                "type": "ed25519",
                "publicKey": "deadbeef",
                "signature": "feedface",
                "signer": "acc://signer.acme",
                "timestamp": 1640995200
            }),
            json::json!({
                "type": "rsa",
                "publicKey": "cafebabe",
                "signature": "beefdead",
                "signer": "acc://signer2.acme",
                "timestamp": 1640995300
            }),
        ],
    };

    // Serialize and verify structure
    let serialized = serde_json::to_value(&envelope).unwrap();

    // Verify signatures array
    let signatures = serialized.get("signatures").unwrap().as_array().unwrap();
    assert_eq!(signatures.len(), 2);

    // Verify signature structure
    assert_eq!(signatures[0]["type"], "ed25519");
    assert_eq!(signatures[0]["signer"], "acc://signer.acme");
    assert_eq!(signatures[1]["type"], "rsa");
    assert_eq!(signatures[1]["signer"], "acc://signer2.acme");

    println!("✓ Envelope with signatures test passed");
}

#[test]
fn test_header_validation_in_envelope() {
    println!("Testing header validation in envelope context...");

    let header = create_minimal_header();

    // Validate header
    assert!(header.validate().is_ok(), "Header should validate successfully");

    // Test invalid header (empty principal)
    let mut invalid_header = header.clone();
    invalid_header.principal = "".to_string();

    let validation_result = invalid_header.validate();
    assert!(validation_result.is_err(), "Empty principal should fail validation");

    println!("✓ Header validation in envelope context passed");
}

#[test]
fn test_canonical_json_properties() {
    println!("Testing canonical JSON properties...");

    let header = create_minimal_header();
    let body = TransactionBody::WriteData(WriteDataBody {
        entry: json::json!({"data": "test"}),
        scratch: Some(true),
        write_to_state: Some(false),
    });

    let envelope = TestTransactionEnvelope {
        header,
        body,
        signatures: vec![],
    };

    let serialized = serde_json::to_value(&envelope).unwrap();
    let json_str = serde_json::to_string(&serialized).unwrap();

    // Verify camelCase field naming
    assert!(json_str.contains("\"header\""));
    assert!(json_str.contains("\"body\""));
    assert!(json_str.contains("\"signatures\""));

    // Verify header fields use camelCase (per Go source truth)
    assert!(json_str.contains("\"principal\""));
    assert!(json_str.contains("\"initiator\""));

    // Verify body fields use PascalCase
    assert!(json_str.contains("\"type\""));
    assert!(json_str.contains("\"Entry\""));

    println!("✓ Canonical JSON properties test passed");
}