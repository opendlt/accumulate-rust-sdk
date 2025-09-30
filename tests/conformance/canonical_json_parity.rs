use accumulate_client::canonjson::canonicalize;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Load test fixtures from JSON file
fn load_canonical_fixtures() -> serde_json::Result<Value> {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("canonical_json_tests.json");

    let content = fs::read_to_string(fixture_path)
        .expect("Failed to read canonical JSON test fixtures");

    serde_json::from_str(&content)
}

#[test]
fn test_canonical_json_parity_with_fixtures() {
    let fixtures = load_canonical_fixtures().expect("Failed to parse fixtures");

    let test_cases = fixtures["testCases"].as_array()
        .expect("testCases should be an array");

    for test_case in test_cases {
        let name = test_case["name"].as_str().unwrap();
        let input = &test_case["input"];
        let expected = test_case["expectedCanonical"].as_str().unwrap();

        let actual = canonicalize(input);

        assert_eq!(
            actual,
            expected,
            "Canonical JSON mismatch for test case '{}'\nInput: {}\nExpected: {}\nActual: {}",
            name,
            serde_json::to_string_pretty(input).unwrap(),
            expected,
            actual
        );
    }
}

#[test]
fn test_canonical_json_specific_cases() {
    // Test case 1: Simple object with sorted keys
    let obj1 = json!({ "z": 3, "a": 1, "m": 2 });
    assert_eq!(canonicalize(&obj1), r#"{"a":1,"m":2,"z":3}"#);

    // Test case 2: Nested object with sorted keys
    let obj2 = json!({
        "z": { "y": 2, "x": 1 },
        "a": 1
    });
    assert_eq!(canonicalize(&obj2), r#"{"a":1,"z":{"x":1,"y":2}}"#);

    // Test case 3: Array with objects (arrays preserve order, objects get sorted)
    let obj3 = json!({
        "arr": [{ "b": 2, "a": 1 }, { "d": 4, "c": 3 }]
    });
    assert_eq!(canonicalize(&obj3), r#"{"arr":[{"a":1,"b":2},{"c":3,"d":4}]}"#);

    // Test case 4: Primitives
    assert_eq!(canonicalize(&json!(null)), "null");
    assert_eq!(canonicalize(&json!(true)), "true");
    assert_eq!(canonicalize(&json!(false)), "false");
    assert_eq!(canonicalize(&json!(42)), "42");
    assert_eq!(canonicalize(&json!(3.14)), "3.14");
    assert_eq!(canonicalize(&json!("hello")), r#""hello""#);
    assert_eq!(canonicalize(&json!([])), "[]");
    assert_eq!(canonicalize(&json!({})), "{}");
}

#[test]
fn test_canonical_json_accumulate_structures() {
    // Test Accumulate-specific transaction structures
    let tx = json!({
        "header": {
            "principal": "acc://alice.acme/tokens",
            "timestamp": 1234567890123_u64
        },
        "body": {
            "type": "send-tokens",
            "to": [{
                "url": "acc://bob.acme/tokens",
                "amount": "1000"
            }]
        }
    });

    let canonical = canonicalize(&tx);
    let expected = r#"{"body":{"to":[{"amount":"1000","url":"acc://bob.acme/tokens"}],"type":"send-tokens"},"header":{"principal":"acc://alice.acme/tokens","timestamp":1234567890123}}"#;

    assert_eq!(canonical, expected);
}

#[test]
fn test_canonical_json_deterministic() {
    // Test that canonicalization is deterministic
    let obj = json!({
        "z": { "nested": { "y": 2, "x": 1 } },
        "a": [3, 1, 2],
        "m": { "another": { "b": 2, "a": 1 } }
    });

    let canonical1 = canonicalize(&obj);
    let canonical2 = canonicalize(&obj);

    assert_eq!(canonical1, canonical2);

    // Verify the canonical form is correctly ordered
    let expected = r#"{"a":[3,1,2],"m":{"another":{"a":1,"b":2}},"z":{"nested":{"x":1,"y":2}}}"#;
    assert_eq!(canonical1, expected);
}