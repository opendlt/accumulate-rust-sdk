use accumulate_client::codec::{canonical_json, to_canonical_string};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Load canonical JSON test cases from golden fixtures
fn load_canonical_test_cases() -> serde_json::Value {
    let golden_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("enums")
        .join("canonical_json_tests.json");

    let content = fs::read_to_string(golden_path)
        .expect("Failed to read canonical_json_tests.json fixture");

    serde_json::from_str(&content).expect("Failed to parse canonical JSON test cases")
}

#[test]
fn test_canonical_json_fixtures() {
    let test_data = load_canonical_test_cases();
    let test_cases = test_data["testCases"].as_array().unwrap();

    for test_case in test_cases {
        let name = test_case["name"].as_str().unwrap();
        let input = &test_case["input"];
        let expected = test_case["expectedCanonical"].as_str().unwrap();

        let actual = canonical_json(input);

        assert_eq!(
            actual, expected,
            "Canonical JSON mismatch for test case '{}'\nInput: {}\nExpected: {}\nActual: {}",
            name, input, expected, actual
        );

        println!("✓ Test case '{}' passed", name);
    }
}

#[test]
fn test_canonical_json_byte_for_byte_parity() {
    // Test cases that should exactly match TypeScript SDK output
    let test_cases = vec![
        (
            json!({"z": 3, "a": 1, "m": 2}),
            r#"{"a":1,"m":2,"z":3}"#,
        ),
        (
            json!({"z": {"y": 2, "x": 1}, "a": 1}),
            r#"{"a":1,"z":{"x":1,"y":2}}"#,
        ),
        (
            json!({"arr": [{"b": 2, "a": 1}, {"d": 4, "c": 3}]}),
            r#"{"arr":[{"a":1,"b":2},{"c":3,"d":4}]}"#,
        ),
        (
            json!({
                "string": "test",
                "number": 42,
                "boolean": true,
                "null": null
            }),
            r#"{"boolean":true,"null":null,"number":42,"string":"test"}"#,
        ),
    ];

    for (input, expected) in test_cases {
        let actual = canonical_json(&input);
        assert_eq!(
            actual, expected,
            "Canonical JSON mismatch\nInput: {}\nExpected: {}\nActual: {}",
            input, expected, actual
        );
    }
}

#[test]
fn test_canonical_json_nested_objects() {
    let complex_input = json!({
        "transaction": {
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
        },
        "metadata": {
            "version": "1.0",
            "chain": "accumulate"
        }
    });

    let canonical = canonical_json(&complex_input);

    // Verify that keys are in alphabetical order at all levels
    // Note: "metadata" comes before "transaction" alphabetically
    assert!(canonical.starts_with("{\"metadata\":"));
    assert!(canonical.contains("\"transaction\":{\"body\":"));
    assert!(canonical.contains("\"header\":{\"principal\":"));

    // Verify that array elements maintain order but object keys are sorted
    assert!(canonical.contains("[{\"amount\":\"1000\",\"url\":\"acc://bob.acme/tokens\"}]"));

    println!("Complex canonical JSON: {}", canonical);
}

#[test]
fn test_canonical_json_unicode_and_escaping() {
    let input = json!({
        "unicode": "Hello 世界",
        "emoji": "🚀🌟",
        "escape": "line1\nline2\ttab",
        "quotes": "He said \"hello\"",
        "backslash": "path\\to\\file"
    });

    let canonical = canonical_json(&input);

    // Verify that string escaping is handled correctly
    assert!(canonical.contains(r#""backslash":"path\\to\\file""#));
    assert!(canonical.contains(r#""escape":"line1\nline2\ttab""#));
    assert!(canonical.contains(r#""quotes":"He said \"hello\"""#));

    // Keys should still be sorted
    let keys_order = ["backslash", "emoji", "escape", "quotes", "unicode"];
    let mut last_pos = 0;
    for key in keys_order {
        let key_pattern = format!(r#""{}":"#, key);
        let pos = canonical.find(&key_pattern).expect(&format!("Key '{}' not found", key));
        assert!(pos > last_pos, "Keys not in alphabetical order");
        last_pos = pos;
    }
}

#[test]
fn test_canonical_json_numbers() {
    let input = json!({
        "integer": 42,
        "float": 3.14159,
        "negative": -123,
        "zero": 0,
        "scientific": 1.23e-4
    });

    let canonical = canonical_json(&input);

    // Verify number formatting is preserved
    assert!(canonical.contains(r#""float":3.14159"#));
    assert!(canonical.contains(r#""integer":42"#));
    assert!(canonical.contains(r#""negative":-123"#));
    assert!(canonical.contains(r#""zero":0"#));

    println!("Numbers canonical JSON: {}", canonical);
}

#[test]
fn test_canonical_json_empty_structures() {
    let input = json!({
        "empty_object": {},
        "empty_array": [],
        "null_value": null,
        "nested_empty": {
            "inner_empty": {},
            "inner_array": []
        }
    });

    let canonical = canonical_json(&input);

    assert!(canonical.contains(r#""empty_array":[]"#));
    assert!(canonical.contains(r#""empty_object":{}"#));
    assert!(canonical.contains(r#""null_value":null"#));
    assert!(canonical.contains(r#""nested_empty":{"inner_array":[],"inner_empty":{}}"#));

    println!("Empty structures canonical JSON: {}", canonical);
}

#[test]
fn test_canonical_json_deterministic() {
    let input = json!({
        "field1": "value1",
        "field2": {
            "nested": "data",
            "array": [1, 2, 3]
        }
    });

    // Generate canonical JSON multiple times
    let canonical1 = canonical_json(&input);
    let canonical2 = canonical_json(&input);
    let canonical3 = to_canonical_string(&input);

    assert_eq!(canonical1, canonical2);
    assert_eq!(canonical1, canonical3);

    // Verify the output is exactly what we expect
    let expected = r#"{"field1":"value1","field2":{"array":[1,2,3],"nested":"data"}}"#;
    assert_eq!(canonical1, expected);
}

/// Test canonical JSON against the exact fixture from the transaction signing vectors
#[test]
fn test_canonical_json_transaction_vectors() {
    let tx_vectors_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("enums")
        .join("tx_signing_vectors.json");

    let content = fs::read_to_string(tx_vectors_path)
        .expect("Failed to read tx_signing_vectors.json fixture");

    let vectors: Value = serde_json::from_str(&content)
        .expect("Failed to parse transaction signing vectors");

    let test_vectors = vectors["vectors"].as_array().unwrap();

    for vector in test_vectors {
        let name = vector["name"].as_str().unwrap();
        let transaction = &vector["transaction"];
        let expected_canonical = vector["canonicalJSON"].as_str().unwrap();

        let actual_canonical = canonical_json(transaction);

        assert_eq!(
            actual_canonical, expected_canonical,
            "Transaction canonical JSON mismatch for vector '{}'\nExpected: {}\nActual: {}",
            name, expected_canonical, actual_canonical
        );

        println!("✓ Transaction vector '{}' canonical JSON matches", name);
    }
}