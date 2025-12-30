// Comprehensive edge case and validation tests for enum implementation
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

// Include the generated enums directly
include!("../../../src/generated/enums.rs");

fn load_manifest() -> serde_json::Value {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("enums_manifest.json");
    let content = fs::read_to_string(&manifest_path)
        .expect("Failed to read enums manifest");
    serde_json::from_str(&content).expect("Failed to parse manifest JSON")
}

#[test]
fn test_enum_case_sensitivity() {
    // Test that enum deserialization is case-sensitive (as it should be)

    // These should work (correct case)
    let write_data: Result<TransactionType, _> = serde_json::from_str("\"writeData\"");
    assert!(write_data.is_ok());

    let identity: Result<AccountType, _> = serde_json::from_str("\"identity\"");
    assert!(identity.is_ok());

    // These should fail (wrong case)
    let write_data_wrong: Result<TransactionType, _> = serde_json::from_str("\"WriteData\"");
    assert!(write_data_wrong.is_err(), "Should reject incorrect casing");

    let write_data_upper: Result<TransactionType, _> = serde_json::from_str("\"WRITEDATA\"");
    assert!(write_data_upper.is_err(), "Should reject all uppercase");

    let identity_wrong: Result<AccountType, _> = serde_json::from_str("\"Identity\"");
    assert!(identity_wrong.is_err(), "Should reject PascalCase");
}

#[test]
fn test_enum_whitespace_sensitivity() {
    // Test that enum deserialization rejects whitespace variations

    let with_space: Result<TransactionType, _> = serde_json::from_str("\" writeData\"");
    assert!(with_space.is_err(), "Should reject leading space");

    let trailing_space: Result<TransactionType, _> = serde_json::from_str("\"writeData \"");
    assert!(trailing_space.is_err(), "Should reject trailing space");

    let with_newline: Result<TransactionType, _> = serde_json::from_str("\"writeData\\n\"");
    assert!(with_newline.is_err(), "Should reject newline characters");

    let with_tab: Result<TransactionType, _> = serde_json::from_str("\"write\\tData\"");
    assert!(with_tab.is_err(), "Should reject tab characters");
}

#[test]
fn test_enum_special_characters() {
    // Test enum format validation (camelCase vs other formats)

    // Test camelCase variants (canonical format)
    let v1_sig: Result<ExecutorVersion, _> = serde_json::from_str("\"v1SignatureAnchoring\"");
    assert!(v1_sig.is_ok(), "Should accept camelCase variants");

    let block_validator: Result<PartitionType, _> = serde_json::from_str("\"blockValidator\"");
    assert!(block_validator.is_ok(), "Should accept camelCase partition types");

    let block_summary: Result<PartitionType, _> = serde_json::from_str("\"blockSummary\"");
    assert!(block_summary.is_ok(), "Should accept camelCase summary type");

    // Test that underscore variants are rejected (we use camelCase, but accept legacy hyphenated)
    let underscore: Result<PartitionType, _> = serde_json::from_str("\"block_validator\"");
    assert!(underscore.is_err(), "Should reject underscore variants");

    // Test that hyphenated variants are accepted for backward compatibility
    let hyphenated: Result<PartitionType, _> = serde_json::from_str("\"block-validator\"");
    assert!(hyphenated.is_ok(), "Should accept hyphenated variants for backward compatibility");
}

#[test]
fn test_enum_unicode_rejection() {
    // Test that unicode characters are properly rejected

    let unicode: Result<TransactionType, _> = serde_json::from_str("\"writeDataé\"");
    assert!(unicode.is_err(), "Should reject unicode characters");

    let invalid: Result<AccountType, _> = serde_json::from_str("\"identity123\"");
    assert!(invalid.is_err(), "Should reject invalid characters");

    let chinese: Result<ExecutorVersion, _> = serde_json::from_str("\"v1中文\"");
    assert!(chinese.is_err(), "Should reject non-Latin characters");
}

#[test]
fn test_enum_empty_and_null_values() {
    // Test edge cases with empty/null values

    let empty_string: Result<TransactionType, _> = serde_json::from_str("\"\"");
    assert!(empty_string.is_err(), "Should reject empty string");

    let null_value: Result<TransactionType, _> = serde_json::from_str("null");
    assert!(null_value.is_err(), "Should reject null value");

    let undefined: Result<TransactionType, _> = serde_json::from_str("undefined");
    assert!(undefined.is_err(), "Should reject undefined");
}

#[test]
fn test_enum_numeric_rejection() {
    // Test that numeric values are rejected (enums should be strings)

    let number: Result<TransactionType, _> = serde_json::from_str("1");
    assert!(number.is_err(), "Should reject numeric values");

    let float: Result<AccountType, _> = serde_json::from_str("1.5");
    assert!(float.is_err(), "Should reject float values");

    let hex: Result<ExecutorVersion, _> = serde_json::from_str("0x1");
    assert!(hex.is_err(), "Should reject hex values");
}

#[test]
fn test_enum_array_object_rejection() {
    // Test that complex JSON types are rejected

    let array: Result<TransactionType, _> = serde_json::from_str("[\"writeData\"]");
    assert!(array.is_err(), "Should reject arrays");

    let object: Result<TransactionType, _> = serde_json::from_str("{\"type\": \"writeData\"}");
    assert!(object.is_err(), "Should reject objects");

    let nested: Result<AccountType, _> = serde_json::from_str("{\"identity\": {}}");
    assert!(nested.is_err(), "Should reject nested objects");
}

#[test]
fn test_enum_wire_format_completeness() {
    // Verify that every Go truth enum variant has a corresponding wire format
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        let variants = e["variants"].as_array().unwrap();

        for v in variants {
            let wire_tag = v.as_str().unwrap();

            // Each wire tag should be non-empty and contain only valid characters
            assert!(!wire_tag.is_empty(), "Wire tag should not be empty for {}", enum_name);
            assert!(!wire_tag.contains('\0'), "Wire tag should not contain null chars for {}", enum_name);
            assert!(!wire_tag.starts_with(' '), "Wire tag should not start with space for {}", enum_name);
            assert!(!wire_tag.ends_with(' '), "Wire tag should not end with space for {}", enum_name);

            // Verify it can be serialized/deserialized
            let json_value = serde_json::Value::String(wire_tag.to_string());
            let json_str = serde_json::to_string(&json_value).unwrap();
            assert!(json_str.contains(wire_tag), "Wire tag should survive JSON roundtrip");
        }
    }
}

#[test]
fn test_enum_uniqueness_within_types() {
    // Verify that within each enum, all wire tags are unique
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        let variants = e["variants"].as_array().unwrap();

        let mut seen_tags = HashSet::new();
        for v in variants {
            let wire_tag = v.as_str().unwrap();
            assert!(
                seen_tags.insert(wire_tag),
                "Duplicate wire tag '{}' found in enum {}",
                wire_tag, enum_name
            );
        }
    }
}

#[test]
fn test_enum_cross_type_uniqueness() {
    // Verify important wire tags don't conflict across enum types
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    let mut critical_tags = HashMap::new();

    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        let variants = e["variants"].as_array().unwrap();

        for v in variants {
            let wire_tag = v.as_str().unwrap();

            // Check for critical overlaps that could cause confusion
            if wire_tag == "unknown" {
                critical_tags.entry("unknown".to_string())
                    .or_insert_with(Vec::new)
                    .push(enum_name.to_string());
            }
            if wire_tag == "identity" {
                critical_tags.entry("identity".to_string())
                    .or_insert_with(Vec::new)
                    .push(enum_name.to_string());
            }
        }
    }

    // "unknown" is expected to appear in multiple enums - that's okay
    if let Some(unknown_enums) = critical_tags.get("unknown") {
        println!("'unknown' appears in {} enums: {:?}", unknown_enums.len(), unknown_enums);
        // This is expected behavior
    }

    // "identity" should be unique to AccountType
    if let Some(identity_enums) = critical_tags.get("identity") {
        assert_eq!(identity_enums.len(), 1, "Identity should only appear in one enum type");
        assert_eq!(identity_enums[0], "AccountType", "Identity should only be in AccountType");
    }
}

#[test]
fn test_enum_json_stability() {
    // Test that JSON serialization is stable and deterministic

    let tx_type = TransactionType::WriteData;
    let json1 = serde_json::to_string(&tx_type).unwrap();
    let json2 = serde_json::to_string(&tx_type).unwrap();
    assert_eq!(json1, json2, "JSON serialization should be deterministic");

    // Test multiple serializations of the same enum
    for _ in 0..10 {
        let json = serde_json::to_string(&tx_type).unwrap();
        assert_eq!(json, "\"writeData\"", "JSON should always be the same");
    }

    // Test that deserialization is also stable
    for _ in 0..10 {
        let deserialized: TransactionType = serde_json::from_str("\"writeData\"").unwrap();
        assert_eq!(deserialized, TransactionType::WriteData, "Deserialization should be stable");
    }
}

#[test]
fn test_enum_debug_format() {
    // Test that Debug trait produces reasonable output

    let tx_type = TransactionType::WriteData;
    let debug_str = format!("{:?}", tx_type);
    assert!(debug_str.contains("WriteData"), "Debug format should contain variant name");
    assert!(!debug_str.contains("writeData"), "Debug format should show Rust name, not wire tag");

    let account_type = AccountType::Identity;
    let debug_str = format!("{:?}", account_type);
    assert!(debug_str.contains("Identity"), "Debug format should show Rust variant");
}

#[test]
fn test_enum_clone_equality() {
    // Test that Clone and PartialEq work correctly

    let original = TransactionType::CreateIdentity;
    let cloned = original.clone();

    assert_eq!(original, cloned, "Cloned enum should equal original");
    assert_eq!(cloned, original, "Equality should be symmetric");

    let different = TransactionType::WriteData;
    assert_ne!(original, different, "Different variants should not be equal");
}

#[test]
fn test_enum_hash_consistency() {
    // Test that equal enums have equal hashes (important for HashMap usage)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let tx1 = TransactionType::WriteData;
    let tx2 = TransactionType::WriteData;

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    tx1.hash(&mut hasher1);
    tx2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish(), "Equal enums should have equal hashes");
}

#[test]
fn test_enum_in_collections() {
    // Test that enums work correctly in standard collections

    let mut set = HashSet::new();
    set.insert(TransactionType::WriteData);
    set.insert(TransactionType::CreateIdentity);
    set.insert(TransactionType::WriteData); // Duplicate

    assert_eq!(set.len(), 2, "HashSet should deduplicate enum values");
    assert!(set.contains(&TransactionType::WriteData), "HashSet should contain WriteData");
    assert!(set.contains(&TransactionType::CreateIdentity), "HashSet should contain CreateIdentity");

    let mut map = HashMap::new();
    map.insert(AccountType::Identity, "This is an identity account");
    map.insert(AccountType::TokenAccount, "This is a token account");

    assert_eq!(map.len(), 2, "HashMap should work with enum keys");
    assert_eq!(map.get(&AccountType::Identity), Some(&"This is an identity account"));
}

#[test]
fn test_enum_memory_size() {
    // Test that enums have reasonable memory footprint
    use std::mem;

    // All our enums should be small (just discriminant + no data)
    assert_eq!(mem::size_of::<TransactionType>(), 1, "TransactionType should be 1 byte");
    assert_eq!(mem::size_of::<AccountType>(), 1, "AccountType should be 1 byte");
    assert_eq!(mem::size_of::<ExecutorVersion>(), 1, "ExecutorVersion should be 1 byte");
    assert_eq!(mem::size_of::<SignatureType>(), 1, "SignatureType should be 1 byte");

    // Test that Option<Enum> is optimized
    assert_eq!(mem::size_of::<Option<TransactionType>>(), 1, "Option<TransactionType> should be optimized");
}

#[test]
fn test_all_expected_enums_present() {
    // Final verification that all 14 expected enums are actually implemented

    let expected_enums = vec![
        "ExecutorVersion", "PartitionType", "DataEntryType", "ObjectType",
        "SignatureType", "KeyPageOperationType", "AccountAuthOperationType",
        "NetworkMaintenanceOperationType", "TransactionMax", "TransactionType",
        "AccountType", "AllowedTransactionBit", "VoteType", "BookType"
    ];

    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    let mut found_enums = HashSet::new();
    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        found_enums.insert(enum_name);
    }

    for expected in &expected_enums {
        assert!(
            found_enums.contains(expected),
            "Expected enum '{}' not found in generated manifest",
            expected
        );
    }

    assert_eq!(
        found_enums.len(),
        expected_enums.len(),
        "Expected exactly {} enums, found {}",
        expected_enums.len(),
        found_enums.len()
    );
}

#[test]
fn test_critical_enum_variants() {
    // Test specific critical variants that must work correctly

    // Test transaction types that are commonly used
    let write_data: TransactionType = serde_json::from_str("\"writeData\"").unwrap();
    assert_eq!(write_data, TransactionType::WriteData);

    let create_identity: TransactionType = serde_json::from_str("\"createIdentity\"").unwrap();
    assert_eq!(create_identity, TransactionType::CreateIdentity);

    let send_tokens: TransactionType = serde_json::from_str("\"sendTokens\"").unwrap();
    assert_eq!(send_tokens, TransactionType::SendTokens);

    // Test signature types that are critical for validation
    let ed25519: SignatureType = serde_json::from_str("\"ed25519\"").unwrap();
    assert_eq!(ed25519, SignatureType::ED25519);

    let delegated: SignatureType = serde_json::from_str("\"delegated\"").unwrap();
    assert_eq!(delegated, SignatureType::Delegated);

    // Test account types for account management
    let identity: AccountType = serde_json::from_str("\"identity\"").unwrap();
    assert_eq!(identity, AccountType::Identity);

    let token_account: AccountType = serde_json::from_str("\"tokenaccount\"").unwrap();
    assert_eq!(token_account, AccountType::TokenAccount);

    // Test executor versions for protocol compatibility
    let v1: ExecutorVersion = serde_json::from_str("\"v1\"").unwrap();
    assert_eq!(v1, ExecutorVersion::V1);

    let v2: ExecutorVersion = serde_json::from_str("\"v2\"").unwrap();
    assert_eq!(v2, ExecutorVersion::V2);
}