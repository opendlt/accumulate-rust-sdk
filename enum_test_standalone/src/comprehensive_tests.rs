// Comprehensive enum test suite
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

// Include the generated enums directly
include!("../../src/generated/enums.rs");

fn load_manifest() -> serde_json::Value {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("src")
        .join("generated")
        .join("enums_manifest.json");
    let content = fs::read_to_string(&manifest_path)
        .expect("Failed to read enums manifest");
    serde_json::from_str(&content).expect("Failed to parse manifest JSON")
}

#[test]
fn test_edge_cases() {
    // Test case sensitivity
    let write_data: Result<TransactionType, _> = serde_json::from_str("\"writeData\"");
    assert!(write_data.is_ok());

    let write_data_wrong: Result<TransactionType, _> = serde_json::from_str("\"WriteData\"");
    assert!(write_data_wrong.is_err(), "Should reject incorrect casing");

    // Test whitespace rejection
    let with_space: Result<TransactionType, _> = serde_json::from_str("\" writeData\"");
    assert!(with_space.is_err(), "Should reject leading space");

    // Test empty/null rejection
    let empty_string: Result<TransactionType, _> = serde_json::from_str("\"\"");
    assert!(empty_string.is_err(), "Should reject empty string");

    let null_value: Result<TransactionType, _> = serde_json::from_str("null");
    assert!(null_value.is_err(), "Should reject null value");

    // Test numeric rejection
    let number: Result<TransactionType, _> = serde_json::from_str("1");
    assert!(number.is_err(), "Should reject numeric values");
}

#[test]
fn test_roundtrip_all_variants() {
    // Test every TransactionType variant
    let tx_variants = vec![
        TransactionType::Unknown,
        TransactionType::CreateIdentity,
        TransactionType::CreateTokenAccount,
        TransactionType::SendTokens,
        TransactionType::WriteData,
        TransactionType::SystemGenesis,
        TransactionType::DirectoryAnchor,
    ];

    for variant in tx_variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: TransactionType = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, deserialized, "Roundtrip failed for {:?}", variant);
    }

    // Test every AccountType variant
    let acc_variants = vec![
        AccountType::Unknown,
        AccountType::Identity,
        AccountType::TokenAccount,
        AccountType::DataAccount,
        AccountType::KeyPage,
        AccountType::KeyBook,
    ];

    for variant in acc_variants {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: AccountType = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, deserialized, "Roundtrip failed for {:?}", variant);
    }
}

#[test]
fn test_critical_wire_tags() {
    // Test critical TransactionType wire tags
    let tx_cases = vec![
        (TransactionType::WriteData, "writeData"),
        (TransactionType::CreateIdentity, "createIdentity"),
        (TransactionType::SendTokens, "sendTokens"),
    ];

    for (enum_val, expected_tag) in tx_cases {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        let expected_json = format!("\"{}\"", expected_tag);
        assert_eq!(serialized, expected_json, "Wire tag mismatch for {:?}", enum_val);

        let deserialized: TransactionType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(enum_val, deserialized, "Roundtrip failed for {:?}", enum_val);
    }

    // Test critical AccountType wire tags
    let acc_cases = vec![
        (AccountType::Identity, "identity"),
        (AccountType::TokenAccount, "tokenaccount"),
    ];

    for (enum_val, expected_tag) in acc_cases {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        let expected_json = format!("\"{}\"", expected_tag);
        assert_eq!(serialized, expected_json, "Wire tag mismatch for {:?}", enum_val);

        let deserialized: AccountType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(enum_val, deserialized, "Roundtrip failed for {:?}", enum_val);
    }

    // Test critical SignatureType wire tags
    let sig_cases = vec![
        (SignatureType::ED25519, "ed25519"),
        (SignatureType::Delegated, "delegated"),
    ];

    for (enum_val, expected_tag) in sig_cases {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        let expected_json = format!("\"{}\"", expected_tag);
        assert_eq!(serialized, expected_json, "Wire tag mismatch for {:?}", enum_val);

        let deserialized: SignatureType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(enum_val, deserialized, "Roundtrip failed for {:?}", enum_val);
    }

    // Test critical ExecutorVersion wire tags
    let exec_cases = vec![
        (ExecutorVersion::V1, "v1"),
        (ExecutorVersion::V2, "v2"),
    ];

    for (enum_val, expected_tag) in exec_cases {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        let expected_json = format!("\"{}\"", expected_tag);
        assert_eq!(serialized, expected_json, "Wire tag mismatch for {:?}", enum_val);

        let deserialized: ExecutorVersion = serde_json::from_str(&serialized).unwrap();
        assert_eq!(enum_val, deserialized, "Roundtrip failed for {:?}", enum_val);
    }
}

#[test]
fn test_enum_counts() {
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    // Should have exactly 14 enums
    assert_eq!(enums.len(), 14, "Should have exactly 14 enums");

    // Check variant counts for critical enums
    let mut enum_variant_counts = HashMap::new();
    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        let variants = e["variants"].as_array().unwrap();
        enum_variant_counts.insert(enum_name, variants.len());
    }

    // TransactionType should have 34 variants
    assert_eq!(enum_variant_counts["TransactionType"], 34);

    // AccountType should have 15 variants
    assert_eq!(enum_variant_counts["AccountType"], 15);

    // SignatureType should have 17 variants
    assert_eq!(enum_variant_counts["SignatureType"], 17);
}

#[test]
fn test_memory_layout() {
    use std::mem;

    // All enums should be 1 byte
    assert_eq!(mem::size_of::<TransactionType>(), 1);
    assert_eq!(mem::size_of::<AccountType>(), 1);
    assert_eq!(mem::size_of::<SignatureType>(), 1);
    assert_eq!(mem::size_of::<ExecutorVersion>(), 1);

    // Option optimization should work
    assert_eq!(mem::size_of::<Option<TransactionType>>(), 1);
    assert_eq!(mem::size_of::<Option<AccountType>>(), 1);
}

#[test]
fn test_collections() {
    // Test HashMap usage
    let mut tx_map = HashMap::new();
    tx_map.insert(TransactionType::WriteData, "write operation");
    tx_map.insert(TransactionType::CreateIdentity, "identity creation");

    assert_eq!(tx_map.len(), 2);
    assert_eq!(tx_map[&TransactionType::WriteData], "write operation");

    // Test HashSet usage
    let mut sig_set = HashSet::new();
    sig_set.insert(SignatureType::ED25519);
    sig_set.insert(SignatureType::Delegated);
    sig_set.insert(SignatureType::ED25519); // duplicate

    assert_eq!(sig_set.len(), 2); // Should deduplicate
    assert!(sig_set.contains(&SignatureType::ED25519));
    assert!(sig_set.contains(&SignatureType::Delegated));
}

#[test]
fn test_serialization_stability() {
    // Test that serialization is deterministic
    let tx_type = TransactionType::WriteData;

    let json1 = serde_json::to_string(&tx_type).unwrap();
    let json2 = serde_json::to_string(&tx_type).unwrap();
    let json3 = serde_json::to_string(&tx_type).unwrap();

    assert_eq!(json1, json2);
    assert_eq!(json2, json3);
    assert_eq!(json1, "\"writeData\"");
}

#[test]
fn test_malformed_input_rejection() {
    // Test various malformed inputs
    let malformed_inputs = vec![
        // Wrong types
        "123",
        "true",
        "false",
        "[]",
        "{}",

        // Invalid strings
        "\"invalid_enum_value\"",
        "\"🚀🚀🚀\"",
        "\"writeData extra\"",
        "\"WriteData\"", // wrong case
        "\"WRITEDATA\"", // wrong case
    ];

    for input in malformed_inputs {
        let tx_result: Result<TransactionType, _> = serde_json::from_str(input);
        let acc_result: Result<AccountType, _> = serde_json::from_str(input);

        assert!(tx_result.is_err(), "TransactionType should reject: {}", input);
        assert!(acc_result.is_err(), "AccountType should reject: {}", input);
    }
}

#[test]
fn test_clone_and_equality() {
    let original = TransactionType::CreateIdentity;
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(cloned, original);

    let different = TransactionType::WriteData;
    assert_ne!(original, different);
}

#[test]
fn test_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let tx1 = TransactionType::WriteData;
    let tx2 = TransactionType::WriteData;

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    tx1.hash(&mut hasher1);
    tx2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_debug_format() {
    let tx_type = TransactionType::WriteData;
    let debug_str = format!("{:?}", tx_type);
    assert!(debug_str.contains("WriteData"));
}

