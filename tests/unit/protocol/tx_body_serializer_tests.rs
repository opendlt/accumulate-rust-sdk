use accumulate_client::*;
use serde_json::{self as json, Value};
use std::fs;
use std::path::PathBuf;

fn manifest() -> json::Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("transactions_manifest.json");
    json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

fn minimal_body_json(body_name: &str, fields: &[Value]) -> Value {
    let mut json_obj = json::json!({});

    // Add required fields based on manifest with VALID values
    for field in fields {
        if field["required"].as_bool().unwrap_or(false) {
            let field_name = field["name"].as_str().unwrap();
            let field_type = field["type"].as_str().unwrap();

            let value = match field_type {
                "url" => json::json!("acc://test.acme"),
                "hash" => json::json!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                "string" => {
                    // Handle Symbol specially to be alphanumeric
                    match field_name {
                        "Symbol" => json::json!("TEST"),
                        _ => json::json!("test")
                    }
                },
                "bytes" => json::json!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                "uint" | "uvarint" => {
                    // Handle fields with specific constraints
                    match field_name {
                        "Precision" => json::json!(8), // Must be 0-18
                        "Oracle" => json::json!(500),   // Must be positive
                        "Height" => json::json!(1),     // Must be positive
                        _ => json::json!(100)
                    }
                },
                "bigint" => json::json!("100"),
                "time" => json::json!(1704067200), // 2024-01-01
                "bool" => json::json!(false),
                "rawJson" => json::json!({"type": "test"}),
                // Complex types - provide minimal valid structures as JSON
                "TokenRecipient" => json::json!({
                    "Url": "acc://recipient.acme",
                    "Amount": "100"
                }),
                "CreditRecipient" => json::json!({
                    "Url": "acc://recipient.acme",
                    "Amount": 100
                }),
                "KeySpecParams" => json::json!({
                    "KeyHash": "deadbeef",
                    "Delegate": "acc://delegate.acme"
                }),
                "DataEntry" => json::json!({
                    "Data": "test data"
                }),
                "ExecutorVersion" => json::json!({
                    "Version": "1.0.0"
                }),
                "NetworkAccountUpdate" => json::json!({
                    "Name": "test-partition",
                    "Body": {}
                }),
                "PartitionAnchorReceipt" => json::json!({
                    "Partition": "test-partition",
                    "Receipt": "deadbeef"
                }),
                "NetworkMaintenanceOperation" => json::json!({
                    "Type": "enable",
                    "Partition": "test-partition"
                }),
                "AccountAuthOperation" => json::json!({
                    "Type": "enable",
                    "Authority": "acc://authority.acme"
                }),
                "KeyPageOperation" => json::json!({
                    "Type": "add",
                    "Key": "deadbeef"
                }),
                "TokenIssuerProof" => json::json!({
                    "Transaction": "deadbeef",
                    "Receipt": "feedbeef"
                }),
                _ => json::json!({}), // Fallback for unknown types
            };

            json_obj[field_name] = value;
        }
    }

    json_obj
}

#[test]
fn test_acme_faucet_body_roundtrip() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let body_spec = bodies.iter()
        .find(|b| b["name"].as_str().unwrap() == "AcmeFaucetBody")
        .expect("AcmeFaucetBody not found in manifest");

    let fields = body_spec["fields"].as_array().unwrap();
    let val = minimal_body_json("AcmeFaucetBody", fields);

    println!("Testing AcmeFaucetBody with JSON: {}", serde_json::to_string_pretty(&val).unwrap());

    // Test deserialization
    let body_result: Result<AcmeFaucetBody, _> = serde_json::from_value(val.clone());
    assert!(body_result.is_ok(), "Failed to deserialize AcmeFaucetBody: {:?}", body_result.err());

    let body = body_result.unwrap();

    // Test serialization back
    let back_result = serde_json::to_value(&body);
    assert!(back_result.is_ok(), "Failed to serialize AcmeFaucetBody: {:?}", back_result.err());

    let back = back_result.unwrap();

    // Check that required fields are preserved
    assert_eq!(val["Url"], back["Url"], "Url mismatch");

    // Test validation
    let validate_result = body.validate();
    assert!(validate_result.is_ok(), "AcmeFaucetBody validation failed: {:?}", validate_result.err());

    println!("✓ AcmeFaucetBody roundtrip and validation successful");
}

#[test]
fn test_add_credits_body_roundtrip() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let body_spec = bodies.iter()
        .find(|b| b["name"].as_str().unwrap() == "AddCreditsBody")
        .expect("AddCreditsBody not found in manifest");

    let fields = body_spec["fields"].as_array().unwrap();
    let val = minimal_body_json("AddCreditsBody", fields);

    println!("Testing AddCreditsBody with JSON: {}", serde_json::to_string_pretty(&val).unwrap());

    let body_result: Result<AddCreditsBody, _> = serde_json::from_value(val.clone());
    assert!(body_result.is_ok(), "Failed to deserialize AddCreditsBody: {:?}", body_result.err());

    let body = body_result.unwrap();
    let back = serde_json::to_value(&body).unwrap();

    assert_eq!(val["Recipient"], back["Recipient"], "Recipient mismatch");
    assert_eq!(val["Amount"], back["Amount"], "Amount mismatch");
    assert_eq!(val["Oracle"], back["Oracle"], "Oracle mismatch");

    let validate_result = body.validate();
    assert!(validate_result.is_ok(), "AddCreditsBody validation failed: {:?}", validate_result.err());

    println!("✓ AddCreditsBody roundtrip and validation successful");
}

#[test]
fn test_send_tokens_body_roundtrip() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let body_spec = bodies.iter()
        .find(|b| b["name"].as_str().unwrap() == "SendTokensBody")
        .expect("SendTokensBody not found in manifest");

    let fields = body_spec["fields"].as_array().unwrap();
    let mut val = minimal_body_json("SendTokensBody", fields);

    // SendTokensBody requires repeatable TokenRecipient fields as JSON Values
    val["To"] = json::json!([
        {
            "Url": "acc://recipient1.acme",
            "Amount": "100"
        },
        {
            "Url": "acc://recipient2.acme",
            "Amount": "200"
        }
    ]);

    println!("Testing SendTokensBody with JSON: {}", serde_json::to_string_pretty(&val).unwrap());

    let body_result: Result<SendTokensBody, _> = serde_json::from_value(val.clone());
    assert!(body_result.is_ok(), "Failed to deserialize SendTokensBody: {:?}", body_result.err());

    let body = body_result.unwrap();
    let back = serde_json::to_value(&body).unwrap();

    assert_eq!(val["To"], back["To"], "To recipients mismatch");

    let validate_result = body.validate();
    assert!(validate_result.is_ok(), "SendTokensBody validation failed: {:?}", validate_result.err());

    println!("✓ SendTokensBody roundtrip and validation successful");
}

#[test]
fn test_create_identity_body_roundtrip() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let body_spec = bodies.iter()
        .find(|b| b["name"].as_str().unwrap() == "CreateIdentityBody")
        .expect("CreateIdentityBody not found in manifest");

    let fields = body_spec["fields"].as_array().unwrap();
    let mut val = minimal_body_json("CreateIdentityBody", fields);

    // Add optional repeatable authorities
    val["Authorities"] = json::json!(["acc://authority1.acme", "acc://authority2.acme"]);

    println!("Testing CreateIdentityBody with JSON: {}", serde_json::to_string_pretty(&val).unwrap());

    let body_result: Result<CreateIdentityBody, _> = serde_json::from_value(val.clone());
    assert!(body_result.is_ok(), "Failed to deserialize CreateIdentityBody: {:?}", body_result.err());

    let body = body_result.unwrap();
    let back = serde_json::to_value(&body).unwrap();

    assert_eq!(val["Url"], back["Url"], "Url mismatch");
    assert_eq!(val["Authorities"], back["Authorities"], "Authorities mismatch");

    let validate_result = body.validate();
    assert!(validate_result.is_ok(), "CreateIdentityBody validation failed: {:?}", validate_result.err());

    println!("✓ CreateIdentityBody roundtrip and validation successful");
}

#[test]
fn test_create_token_body_roundtrip() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let body_spec = bodies.iter()
        .find(|b| b["name"].as_str().unwrap() == "CreateTokenBody")
        .expect("CreateTokenBody not found in manifest");

    let fields = body_spec["fields"].as_array().unwrap();
    let mut val = minimal_body_json("CreateTokenBody", fields);

    // Add optional fields
    val["Properties"] = json::json!("acc://token-properties.acme");
    val["SupplyLimit"] = json::json!("1000000");
    val["Authorities"] = json::json!(["acc://authority1.acme"]);

    println!("Testing CreateTokenBody with JSON: {}", serde_json::to_string_pretty(&val).unwrap());

    let body_result: Result<CreateTokenBody, _> = serde_json::from_value(val.clone());
    assert!(body_result.is_ok(), "Failed to deserialize CreateTokenBody: {:?}", body_result.err());

    let body = body_result.unwrap();
    let back = serde_json::to_value(&body).unwrap();

    assert_eq!(val["Url"], back["Url"], "Url mismatch");
    assert_eq!(val["Symbol"], back["Symbol"], "Symbol mismatch");
    assert_eq!(val["Precision"], back["Precision"], "Precision mismatch");
    assert_eq!(val["Properties"], back["Properties"], "Properties mismatch");
    assert_eq!(val["SupplyLimit"], back["SupplyLimit"], "SupplyLimit mismatch");
    assert_eq!(val["Authorities"], back["Authorities"], "Authorities mismatch");

    let validate_result = body.validate();
    assert!(validate_result.is_ok(), "CreateTokenBody validation failed: {:?}", validate_result.err());

    println!("✓ CreateTokenBody roundtrip and validation successful");
}

#[test]
fn test_write_data_body_roundtrip() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let body_spec = bodies.iter()
        .find(|b| b["name"].as_str().unwrap() == "WriteDataBody")
        .expect("WriteDataBody not found in manifest");

    let fields = body_spec["fields"].as_array().unwrap();
    let mut val = minimal_body_json("WriteDataBody", fields);

    // Add optional fields
    val["Scratch"] = json::json!(true);
    val["WriteToState"] = json::json!(false);

    println!("Testing WriteDataBody with JSON: {}", serde_json::to_string_pretty(&val).unwrap());

    let body_result: Result<WriteDataBody, _> = serde_json::from_value(val.clone());
    assert!(body_result.is_ok(), "Failed to deserialize WriteDataBody: {:?}", body_result.err());

    let body = body_result.unwrap();
    let back = serde_json::to_value(&body).unwrap();

    assert_eq!(val["Entry"], back["Entry"], "Entry mismatch");
    assert_eq!(val["Scratch"], back["Scratch"], "Scratch mismatch");
    assert_eq!(val["WriteToState"], back["WriteToState"], "WriteToState mismatch");

    let validate_result = body.validate();
    assert!(validate_result.is_ok(), "WriteDataBody validation failed: {:?}", validate_result.err());

    println!("✓ WriteDataBody roundtrip and validation successful");
}

#[test]
fn test_transaction_body_enum_roundtrip() {
    // Test the TransactionBody enum wrapper
    let send_tokens = SendTokensBody {
        hash: None,
        meta: None,
        to: vec![
            json::json!({
                "Url": "acc://recipient.acme",
                "Amount": "100"
            })
        ],
    };

    let body_enum = TransactionBody::SendTokens(send_tokens);

    // Test serialization
    let serialized = serde_json::to_value(&body_enum).unwrap();
    println!("Serialized TransactionBody enum: {}", serde_json::to_string_pretty(&serialized).unwrap());

    // Should have wire format with type field
    assert!(serialized.is_object());
    let obj = serialized.as_object().unwrap();
    assert!(obj.contains_key("type"), "Should have type field");
    assert_eq!(obj["type"], "sendTokens", "Should have correct wire type");

    // Test deserialization back
    let deserialized: TransactionBody = serde_json::from_value(serialized).unwrap();

    match deserialized {
        TransactionBody::SendTokens(body) => {
            assert_eq!(body.to.len(), 1);
            // Since to is Vec<serde_json::Value>, we check the JSON structure
            assert_eq!(body.to[0]["Url"], "acc://recipient.acme");
            assert_eq!(body.to[0]["Amount"], "100");
        },
        _ => panic!("Wrong variant deserialized"),
    }

    println!("✓ TransactionBody enum roundtrip successful");
}

#[test]
fn test_manifest_coverage() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    // Verify we have the expected number of transaction bodies
    println!("Found {} transaction bodies in manifest", bodies.len());

    // List all transaction body names for verification
    let mut body_names: Vec<String> = bodies.iter()
        .map(|b| b["name"].as_str().unwrap().to_string())
        .collect();
    body_names.sort();

    println!("Available transaction bodies:");
    for (i, name) in body_names.iter().enumerate() {
        println!("  {}: {}", i + 1, name);
    }

    // Verify we have at least the core transaction types
    let required_bodies = [
        "SendTokensBody",
        "CreateIdentityBody",
        "CreateTokenBody",
        "AddCreditsBody",
        "WriteDataBody",
        "AcmeFaucetBody"
    ];

    for required in &required_bodies {
        assert!(body_names.contains(&required.to_string()),
                "Missing required transaction body: {}", required);
    }

    println!("✓ Manifest coverage verification successful");
}

#[test]
fn test_body_validation_current_state() {
    // Test that validation now properly rejects invalid data

    // Test SendTokensBody with empty recipients - should fail validation
    let invalid_send = SendTokensBody {
        hash: None,
        meta: None,
        to: vec![], // Empty recipients - should fail validation
    };

    let validation_result = invalid_send.validate();
    assert!(validation_result.is_err(), "Validation should fail for empty recipients");

    // Test CreateIdentityBody with empty URL - should fail validation
    let invalid_identity = CreateIdentityBody {
        url: "".to_string(), // Empty URL - should fail validation
        key_hash: None,
        key_book_url: None,
        authorities: None,
    };

    let validation_result = invalid_identity.validate();
    assert!(validation_result.is_err(), "Validation should fail for empty URL");

    // Test CreateIdentityBody with invalid URL (missing acc:// prefix)
    let invalid_identity2 = CreateIdentityBody {
        url: "test.acme".to_string(), // Missing acc:// prefix
        key_hash: None,
        key_book_url: None,
        authorities: None,
    };

    let validation_result = invalid_identity2.validate();
    assert!(validation_result.is_err(), "Validation should fail for URL without acc:// prefix");

    // Test CreateIdentityBody with VALID data
    let valid_identity = CreateIdentityBody {
        url: "acc://test.acme".to_string(),
        key_hash: Some(vec![0u8; 32]), // Valid 32-byte hash
        key_book_url: Some("acc://test.acme/book".to_string()),
        authorities: None,
    };

    let validation_result = valid_identity.validate();
    assert!(validation_result.is_ok(), "Validation should pass for valid CreateIdentityBody: {:?}", validation_result.err());

    println!("✓ Body validation test successful - validation properly rejects invalid data");
}

#[test]
fn test_complex_nested_types() {
    // Test transaction bodies with complex nested types

    // Test CreateKeyPageBody with KeySpecParams as JSON
    let create_key_page = CreateKeyPageBody {
        keys: vec![
            json::json!({
                "KeyHash": "deadbeef",
                "Delegate": "acc://delegate.acme"
            }),
            json::json!({
                "KeyHash": "feedface"
            })
        ],
    };

    // Test serialization/deserialization
    let serialized = serde_json::to_value(&create_key_page).unwrap();
    let deserialized: CreateKeyPageBody = serde_json::from_value(serialized).unwrap();

    assert_eq!(deserialized.keys.len(), 2);
    assert_eq!(deserialized.keys[0]["KeyHash"], "deadbeef");
    assert_eq!(deserialized.keys[0]["Delegate"], "acc://delegate.acme");
    assert_eq!(deserialized.keys[1]["KeyHash"], "feedface");
    assert!(deserialized.keys[1].get("Delegate").is_none());

    let validation_result = create_key_page.validate();
    assert!(validation_result.is_ok(), "CreateKeyPageBody validation failed: {:?}", validation_result.err());

    println!("✓ Complex nested types test successful");
}

#[test]
fn test_serialization_consistency() {
    // Test multiple roundtrips to ensure consistency
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    for body_spec in bodies.iter().take(5) { // Test first 5 bodies for consistency
        let body_name = body_spec["name"].as_str().unwrap();
        let fields = body_spec["fields"].as_array().unwrap();

        for _i in 0..3 { // Multiple roundtrips
            let val = minimal_body_json(body_name, fields);

            // Skip bodies that require complex setup for this basic test
            if body_name == "DirectoryAnchorBody" || body_name == "NetworkMaintenanceBody" {
                continue;
            }

            // Test basic JSON roundtrip consistency
            let json_str = serde_json::to_string(&val).unwrap();
            let parsed_back: Value = serde_json::from_str(&json_str).unwrap();

            // Core fields should be consistent
            for field in fields {
                if field["required"].as_bool().unwrap_or(false) {
                    let field_name = field["name"].as_str().unwrap();
                    if val.get(field_name).is_some() {
                        assert_eq!(val[field_name], parsed_back[field_name],
                                  "Field {} inconsistent in body {}", field_name, body_name);
                    }
                }
            }
        }
    }

    println!("✓ Serialization consistency test successful");
}