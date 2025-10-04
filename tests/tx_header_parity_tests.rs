// Import the generated TransactionHeader (not the codec one)
use accumulate_client::TransactionHeader;  // Generated header from header.rs
use serde_json::{self as json, Value};
use std::fs;
use std::path::PathBuf;

fn manifest() -> json::Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("header_manifest.json");
    json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

fn minimal_header_json(manifest: &Value) -> Value {
    let fields = manifest["fields"].as_array().unwrap();
    let mut json_obj = json::json!({});

    // Add required fields based on manifest
    for field in fields {
        if field["required"].as_bool().unwrap_or(false) {
            let field_name = field["name"].as_str().unwrap();
            let field_type = field["type"].as_str().unwrap();

            let value = match field_type {
                "url" => json::json!("acc://test.acme"),
                "hash" => json::json!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                "string" => json::json!("test"),
                "bytes" => json::json!("deadbeef"),
                "uint" | "uvarint" => json::json!(123),
                "time" => json::json!(1640995200), // Unix timestamp
                "bool" => json::json!(false),
                _ => json::json!({}), // Complex types
            };

            json_obj[field_name] = value;
        }
    }

    json_obj
}

#[test]
fn header_roundtrip_and_validate() {
    let m = manifest();
    let val = minimal_header_json(&m);

    println!("Testing header with JSON: {}", serde_json::to_string_pretty(&val).unwrap());

    // Test deserialization
    let hdr_result: Result<TransactionHeader, _> = serde_json::from_value(val.clone());
    assert!(hdr_result.is_ok(), "Failed to deserialize header: {:?}", hdr_result.err());

    let hdr = hdr_result.unwrap();

    // Test serialization back
    let back_result = serde_json::to_value(&hdr);
    assert!(back_result.is_ok(), "Failed to serialize header: {:?}", back_result.err());

    let back = back_result.unwrap();

    // Check that required fields are preserved
    assert_eq!(val["Principal"], back["Principal"], "Principal mismatch");
    assert_eq!(val["Initiator"], back["Initiator"], "Initiator mismatch");

    // Test validation
    let validate_result = hdr.validate();
    assert!(validate_result.is_ok(), "Header validation failed: {:?}", validate_result.err());

    println!("✓ Header roundtrip and validation successful");
}

#[test]
fn header_missing_required_fails() {
    // Test with missing Principal (required field)
    let incomplete = json::json!({
        "Initiator": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    });

    let result: Result<TransactionHeader, _> = serde_json::from_value(incomplete);
    assert!(result.is_err(), "Should fail when required field Principal is missing");
}

#[test]
fn header_empty_principal_fails_validation() {
    let m = manifest();
    let mut val = minimal_header_json(&m);

    // Set Principal to empty string
    val["Principal"] = json::json!("");

    let hdr: TransactionHeader = serde_json::from_value(val).unwrap();
    let validation_result = hdr.validate();

    assert!(validation_result.is_err(), "Should fail validation with empty Principal");
}

#[test]
fn header_with_optional_fields() {
    let m = manifest();
    let mut val = minimal_header_json(&m);

    // Add optional fields
    val["Memo"] = json::json!("Test transaction memo");
    val["Metadata"] = json::json!("cafebabe");
    val["Authorities"] = json::json!(["acc://authority1.acme", "acc://authority2.acme"]);

    let hdr_result: Result<TransactionHeader, _> = serde_json::from_value(val.clone());
    assert!(hdr_result.is_ok(), "Failed to deserialize header with optional fields");

    let hdr = hdr_result.unwrap();
    assert!(hdr.validate().is_ok(), "Header with optional fields should validate");

    // Test serialization back
    let back = serde_json::to_value(&hdr).unwrap();
    assert_eq!(val["Memo"], back["Memo"], "Memo should be preserved");
    assert_eq!(val["Metadata"], back["Metadata"], "Metadata should be preserved");
    assert_eq!(val["Authorities"], back["Authorities"], "Authorities should be preserved");
}

#[test]
fn header_with_expire_options() {
    let m = manifest();
    let mut val = minimal_header_json(&m);

    // Add ExpireOptions
    val["Expire"] = json::json!({
        "AtTime": 1640995200
    });

    let hdr_result: Result<TransactionHeader, _> = serde_json::from_value(val.clone());
    assert!(hdr_result.is_ok(), "Failed to deserialize header with ExpireOptions");

    let hdr = hdr_result.unwrap();
    assert!(hdr.validate().is_ok(), "Header with ExpireOptions should validate");

    // Check that nested validation is called
    assert!(hdr.expire.is_some(), "ExpireOptions should be present");
    let expire_opts = hdr.expire.as_ref().unwrap();
    assert!(expire_opts.validate().is_ok(), "ExpireOptions should validate");
}

#[test]
fn header_with_hold_until_options() {
    let m = manifest();
    let mut val = minimal_header_json(&m);

    // Add HoldUntilOptions
    val["HoldUntil"] = json::json!({
        "MinorBlock": 12345
    });

    let hdr_result: Result<TransactionHeader, _> = serde_json::from_value(val.clone());
    assert!(hdr_result.is_ok(), "Failed to deserialize header with HoldUntilOptions");

    let hdr = hdr_result.unwrap();
    assert!(hdr.validate().is_ok(), "Header with HoldUntilOptions should validate");

    // Check that nested validation is called
    assert!(hdr.hold_until.is_some(), "HoldUntilOptions should be present");
    let hold_opts = hdr.hold_until.as_ref().unwrap();
    assert!(hold_opts.validate().is_ok(), "HoldUntilOptions should validate");
}

#[test]
fn header_manifest_validation() {
    let m = manifest();

    // Check that manifest has the expected structure
    assert!(m["struct"].as_str().unwrap() == "TransactionHeader", "Manifest should specify TransactionHeader");
    assert!(m["fields"].is_array(), "Manifest should have fields array");

    let fields = m["fields"].as_array().unwrap();
    assert!(fields.len() >= 2, "Should have at least Principal and Initiator fields");

    // Check for required fields
    let field_names: Vec<&str> = fields.iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();

    assert!(field_names.contains(&"Principal"), "Should have Principal field");
    assert!(field_names.contains(&"Initiator"), "Should have Initiator field");
}

#[test]
fn header_serialization_consistency() {
    // Test multiple roundtrips to ensure consistency
    let m = manifest();

    for _i in 0..5 {
        let val = minimal_header_json(&m);
        let hdr: TransactionHeader = serde_json::from_value(val.clone()).unwrap();
        let back = serde_json::to_value(&hdr).unwrap();
        let hdr2: TransactionHeader = serde_json::from_value(back).unwrap();

        assert_eq!(hdr, hdr2, "Multiple roundtrips should produce identical results");
    }
}