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

fn minimal_body_json(body_info: &Value) -> Value {
    let wire = body_info["wire"].as_str().unwrap();
    let fields = body_info["fields"].as_array().unwrap();

    let mut json_obj = json::json!({
        "type": wire
    });

    // Add minimal required fields
    for field in fields {
        if field["required"].as_bool().unwrap_or(false) {
            let field_name = field["name"].as_str().unwrap();
            let field_type = field["type"].as_str().unwrap();
            let repeatable = field["repeatable"].as_bool().unwrap_or(false);

            let value = if repeatable {
                json::json!([])
            } else {
                match field_type {
                    "url" | "string" => json::json!(""),
                    "uint" | "uvarint" => json::json!(0),
                    "bool" | "boolean" => json::json!(false),
                    "bytes" | "hash" | "txid" => json::json!("00"),
                    "bigint" => json::json!("0"),
                    "time" => json::json!(0),
                    _ => json::json!({}), // Complex types
                }
            };

            json_obj[field_name] = value;
        }
    }

    json_obj
}

fn minimal_body_json_for_wire(wire_tag: &str) -> Value {
    // Create minimal JSON for known wire tags
    match wire_tag {
        "writeData" => json::json!({
            "type": "writeData",
            "Entry": {}
        }),
        "createIdentity" => json::json!({
            "type": "createIdentity",
            "Url": ""
        }),
        "sendTokens" => json::json!({
            "type": "sendTokens",
            "To": []
        }),
        "createToken" => json::json!({
            "type": "createToken",
            "Url": "",
            "Symbol": "",
            "Precision": 0
        }),
        "issueTokens" => json::json!({
            "type": "issueTokens",
            "Recipient": "",
            "Amount": "0",
            "To": []
        }),
        _ => json::json!({"type": wire_tag})
    }
}

#[test]
fn transaction_bodies_roundtrip_and_validate() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    println!("Testing {} transaction bodies", bodies.len());

    for (i, body_info) in bodies.iter().enumerate() {
        let wire = body_info["wire"].as_str().unwrap();
        let body_name = body_info["name"].as_str().unwrap();

        println!("Testing {}/{}: {} ({})", i + 1, bodies.len(), wire, body_name);

        // Create minimal JSON for this body type
        let val = minimal_body_json(body_info);

        // Test deserialization
        let de_result: Result<TransactionBody, _> = serde_json::from_value(val.clone());
        assert!(
            de_result.is_ok(),
            "Failed to deserialize {}: {:?}",
            wire,
            de_result.err()
        );

        let de = de_result.unwrap();

        // Test serialization back
        let back_result = serde_json::to_value(&de);
        assert!(
            back_result.is_ok(),
            "Failed to serialize back {}: {:?}",
            wire,
            back_result.err()
        );

        let back = back_result.unwrap();

        // Test round-trip consistency (ignoring object key order)
        assert_eq!(
            normalize_json(&val),
            normalize_json(&back),
            "Round-trip mismatch for {}: original != serialized",
            wire
        );

        // Test validation
        let validate_result = de.validate();
        assert!(
            validate_result.is_ok(),
            "Validation failed for {}: {:?}",
            wire,
            validate_result.err()
        );

        println!("  ✓ {} roundtrip and validation OK", wire);
    }
}

#[test]
fn unknown_type_fails() {
    let bad = json::json!({ "type": "notARealTx" });
    let result: Result<TransactionBody, _> = serde_json::from_value(bad);
    assert!(result.is_err(), "Unknown transaction type should fail to deserialize");
}

#[test]
fn missing_type_fails() {
    let bad = json::json!({ "someField": "value" });
    let result: Result<TransactionBody, _> = serde_json::from_value(bad);
    assert!(result.is_err(), "Missing type field should fail to deserialize");
}

#[test]
fn test_specific_transaction_types() {
    // Test a few specific transaction types with known structures

    // WriteData transaction
    let write_data = json::json!({
        "type": "writeData",
        "Entry": {},
        "Scratch": false,
        "WriteToState": true
    });

    let parsed: TransactionBody = serde_json::from_value(write_data).unwrap();
    assert!(parsed.validate().is_ok());

    // SendTokens transaction
    let send_tokens = json::json!({
        "type": "sendTokens",
        "To": []
    });

    let parsed: TransactionBody = serde_json::from_value(send_tokens).unwrap();
    assert!(parsed.validate().is_ok());

    // CreateIdentity transaction
    let create_identity = json::json!({
        "type": "createIdentity",
        "Url": "acc://test.acme"
    });

    let parsed: TransactionBody = serde_json::from_value(create_identity).unwrap();
    assert!(parsed.validate().is_ok());
}

#[test]
fn test_transaction_count() {
    let m = manifest();
    let count = m["counts"]["transactions"].as_u64().unwrap();
    assert_eq!(count, 24, "Expected 24 transaction types in manifest");
}

#[test]
fn test_all_wire_tags_unique() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let mut wire_tags = std::collections::HashSet::new();
    for body_info in bodies {
        let wire = body_info["wire"].as_str().unwrap();
        assert!(
            wire_tags.insert(wire),
            "Duplicate wire tag found: {}",
            wire
        );
    }

    assert_eq!(wire_tags.len(), bodies.len(), "All wire tags should be unique");
}

#[test]
fn test_all_struct_names_unique() {
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    let mut struct_names = std::collections::HashSet::new();
    for body_info in bodies {
        let name = body_info["name"].as_str().unwrap();
        assert!(
            struct_names.insert(name),
            "Duplicate struct name found: {}",
            name
        );
    }

    assert_eq!(struct_names.len(), bodies.len(), "All struct names should be unique");
}

#[test]
fn test_codegen_helpers() {
    // Test the generated helper functions
    let wire_tags = [
        "writeData", "createIdentity", "sendTokens", "createToken", "issueTokens"
    ];

    for wire_tag in &wire_tags {
        // Test roundtrip manually since helper functions are cfg(test) in the generated crate
        let original = minimal_body_json_for_wire(wire_tag);
        let body_result: Result<TransactionBody, _> = serde_json::from_value(original.clone());
        assert!(body_result.is_ok(), "Failed to deserialize {}", wire_tag);

        let body = body_result.unwrap();
        let serialized = serde_json::to_value(&body).unwrap();

        assert_eq!(
            normalize_json(&original),
            normalize_json(&serialized),
            "Roundtrip mismatch for {}", wire_tag
        );

        assert!(body.validate().is_ok(), "Validation failed for {}", wire_tag);
    }
}

#[test]
fn test_all_roundtrips() {
    // Test roundtrips for all transaction types
    let m = manifest();
    let bodies = m["bodies"].as_array().unwrap();

    for body_info in bodies {
        let wire = body_info["wire"].as_str().unwrap();
        let original = minimal_body_json_for_wire(wire);
        let body: TransactionBody = serde_json::from_value(original.clone()).unwrap();
        let serialized = serde_json::to_value(&body).unwrap();

        assert_eq!(
            normalize_json(&original),
            normalize_json(&serialized),
            "Roundtrip failed for {}", wire
        );

        assert!(body.validate().is_ok(), "Validation failed for {}", wire);
    }
}

// Helper function to normalize JSON for comparison (sort keys)
fn normalize_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted_map = serde_json::Map::new();
            for (k, v) in map {
                sorted_map.insert(k.clone(), normalize_json(v));
            }
            Value::Object(sorted_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(normalize_json).collect())
        }
        other => other.clone(),
    }
}