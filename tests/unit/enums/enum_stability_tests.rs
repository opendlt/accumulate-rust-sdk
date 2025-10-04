// Enum stability and regeneration consistency tests
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
fn test_transaction_type_deterministic_serialization() {
    // Test that TransactionType serialization is deterministic across multiple runs
    let test_enums = vec![
        (TransactionType::WriteData, "writeData"),
        (TransactionType::CreateIdentity, "createIdentity"),
        (TransactionType::SendTokens, "sendTokens"),
    ];

    for (enum_val, expected_wire) in test_enums {
        let mut serializations = Vec::new();

        for _ in 0..10 {
            let serialized = serde_json::to_string(&enum_val).unwrap();
            serializations.push(serialized);
        }

        let first = &serializations[0];
        for (i, serialization) in serializations.iter().enumerate() {
            assert_eq!(
                serialization, first,
                "Serialization #{} differs from first for enum {:?}",
                i, enum_val
            );
        }

        assert_eq!(
            first, &format!("\"{}\"", expected_wire),
            "Wire format should be stable for {:?}",
            enum_val
        );
    }
}

#[test]
fn test_account_type_deterministic_serialization() {
    let test_enums = vec![
        (AccountType::Identity, "identity"),
        (AccountType::TokenAccount, "tokenAccount"),
    ];

    for (enum_val, expected_wire) in test_enums {
        let mut serializations = Vec::new();

        for _ in 0..10 {
            let serialized = serde_json::to_string(&enum_val).unwrap();
            serializations.push(serialized);
        }

        let first = &serializations[0];
        for (i, serialization) in serializations.iter().enumerate() {
            assert_eq!(
                serialization, first,
                "Serialization #{} differs from first for enum {:?}",
                i, enum_val
            );
        }

        assert_eq!(
            first, &format!("\"{}\"", expected_wire),
            "Wire format should be stable for {:?}",
            enum_val
        );
    }
}

#[test]
fn test_signature_type_deterministic_serialization() {
    let test_enums = vec![
        (SignatureType::ED25519, "ed25519"),
        (SignatureType::Delegated, "delegated"),
    ];

    for (enum_val, expected_wire) in test_enums {
        let mut serializations = Vec::new();

        for _ in 0..10 {
            let serialized = serde_json::to_string(&enum_val).unwrap();
            serializations.push(serialized);
        }

        let first = &serializations[0];
        for (i, serialization) in serializations.iter().enumerate() {
            assert_eq!(
                serialization, first,
                "Serialization #{} differs from first for enum {:?}",
                i, enum_val
            );
        }

        assert_eq!(
            first, &format!("\"{}\"", expected_wire),
            "Wire format should be stable for {:?}",
            enum_val
        );
    }
}

#[test]
fn test_enum_wire_tag_stability() {
    // Verify that wire tags haven't changed from expected values
    // This protects against accidental changes during regeneration

    // Test TransactionType wire tags
    let tx_type_tags = vec![
        (TransactionType::WriteData, "writeData"),
        (TransactionType::CreateIdentity, "createIdentity"),
        (TransactionType::SendTokens, "sendTokens"),
        (TransactionType::CreateTokenAccount, "createTokenAccount"),
        (TransactionType::AddCredits, "addCredits"),
        (TransactionType::BurnTokens, "burnTokens"),
        (TransactionType::LockAccount, "lockAccount"),
        (TransactionType::UpdateKeyPage, "updateKeyPage"),
        (TransactionType::SystemGenesis, "systemGenesis"),
        (TransactionType::DirectoryAnchor, "directoryAnchor"),
    ];

    for (enum_val, expected_tag) in tx_type_tags {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        assert_eq!(
            serialized, format!("\"{}\"", expected_tag),
            "TransactionType::{:?} wire tag should be stable",
            enum_val
        );
    }

    // Test AccountType wire tags
    let account_type_tags = vec![
        (AccountType::Identity, "identity"),
        (AccountType::TokenAccount, "tokenAccount"),
        (AccountType::LiteTokenAccount, "liteTokenAccount"),
        (AccountType::DataAccount, "dataAccount"),
        (AccountType::KeyPage, "keyPage"),
        (AccountType::KeyBook, "keyBook"),
    ];

    for (enum_val, expected_tag) in account_type_tags {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        assert_eq!(
            serialized, format!("\"{}\"", expected_tag),
            "AccountType::{:?} wire tag should be stable",
            enum_val
        );
    }

    // Test SignatureType wire tags
    let signature_type_tags = vec![
        (SignatureType::ED25519, "ed25519"),
        (SignatureType::LegacyED25519, "legacyED25519"),
        (SignatureType::RCD1, "rcd1"),
        (SignatureType::BTC, "btc"),
        (SignatureType::ETH, "eth"),
        (SignatureType::Delegated, "delegated"),
        (SignatureType::Receipt, "receipt"),
    ];

    for (enum_val, expected_tag) in signature_type_tags {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        assert_eq!(
            serialized, format!("\"{}\"", expected_tag),
            "SignatureType::{:?} wire tag should be stable",
            enum_val
        );
    }

    // Test ExecutorVersion wire tags
    let executor_version_tags = vec![
        (ExecutorVersion::V1, "v1"),
        (ExecutorVersion::V1SignatureAnchoring, "v1SignatureAnchoring"),
        (ExecutorVersion::V2, "v2"),
        (ExecutorVersion::V2Baikonur, "v2Baikonur"),
    ];

    for (enum_val, expected_tag) in executor_version_tags {
        let serialized = serde_json::to_string(&enum_val).unwrap();
        assert_eq!(
            serialized, format!("\"{}\"", expected_tag),
            "ExecutorVersion::{:?} wire tag should be stable",
            enum_val
        );
    }
}

#[test]
fn test_manifest_consistency() {
    // Verify that the manifest file is consistent with actual enum implementations
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    // Check that all expected enums are present
    let expected_enum_names = vec![
        "ExecutorVersion", "PartitionType", "DataEntryType", "ObjectType",
        "SignatureType", "KeyPageOperationType", "AccountAuthOperationType",
        "NetworkMaintenanceOperationType", "TransactionMax", "TransactionType",
        "AccountType", "AllowedTransactionBit", "VoteType", "BookType"
    ];

    let mut found_enums = HashSet::new();
    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        found_enums.insert(enum_name);
    }

    for expected in &expected_enum_names {
        assert!(
            found_enums.contains(expected),
            "Expected enum '{}' not found in manifest",
            expected
        );
    }

    assert_eq!(
        found_enums.len(), expected_enum_names.len(),
        "Manifest should contain exactly {} enums, found {}",
        expected_enum_names.len(), found_enums.len()
    );
}

#[test]
fn test_enum_variant_count_stability() {
    // Verify that enum variant counts haven't unexpectedly changed
    // This helps catch unintentional additions/removals during regeneration

    let expected_variant_counts = vec![
        ("ExecutorVersion", 9),      // V1, V1SignatureAnchoring, etc.
        ("PartitionType", 4),        // Directory, BlockValidator, etc.
        ("DataEntryType", 4),        // Unknown, Factom, Accumulate, DoubleHash
        ("ObjectType", 3),           // Unknown, Account, Transaction
        ("SignatureType", 17),       // Unknown, LegacyED25519, ED25519, etc.
        ("KeyPageOperationType", 8), // Unknown, Update, Remove, etc.
        ("AccountAuthOperationType", 5), // Unknown, Enable, Disable, etc.
        ("NetworkMaintenanceOperationType", 2), // Unknown, PendingTransactionGC
        ("TransactionMax", 3),       // User, Synthetic, System
        ("TransactionType", 34),     // Unknown, CreateIdentity, etc.
        ("AccountType", 15),         // Unknown, AnchorLedger, Identity, etc.
        ("AllowedTransactionBit", 2), // UpdateKeyPage, UpdateAccountAuth
        ("VoteType", 4),             // Accept, Reject, Abstain, Suggest
        ("BookType", 3),             // Normal, Validator, Operator
    ];

    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    let mut actual_counts = HashMap::new();
    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        let variants = e["variants"].as_array().unwrap();
        actual_counts.insert(enum_name, variants.len());
    }

    for (enum_name, expected_count) in expected_variant_counts {
        let actual_count = actual_counts.get(enum_name).unwrap_or(&0);
        assert_eq!(
            *actual_count, expected_count,
            "Enum '{}' variant count changed: expected {}, got {}",
            enum_name, expected_count, actual_count
        );
    }
}

#[test]
fn test_enum_ordinal_stability() {
    // Test that enum variant order/ordinals remain stable
    // Important for binary compatibility and discriminant consistency

    // Test a few critical enums where order matters
    let tx_variants = vec![
        TransactionType::Unknown,
        TransactionType::CreateIdentity,
        TransactionType::CreateTokenAccount,
        TransactionType::SendTokens,
        TransactionType::CreateDataAccount,
        TransactionType::WriteData,
    ];

    // Verify that ordinals match expected positions
    for (expected_ordinal, variant) in tx_variants.iter().enumerate() {
        let discriminant = variant.clone() as u8;
        assert_eq!(
            discriminant as usize, expected_ordinal,
            "TransactionType::{:?} ordinal changed: expected {}, got {}",
            variant, expected_ordinal, discriminant
        );
    }

    let account_variants = vec![
        AccountType::Unknown,
        AccountType::AnchorLedger,
        AccountType::Identity,
        AccountType::TokenIssuer,
        AccountType::TokenAccount,
    ];

    for (expected_ordinal, variant) in account_variants.iter().enumerate() {
        let discriminant = variant.clone() as u8;
        assert_eq!(
            discriminant as usize, expected_ordinal,
            "AccountType::{:?} ordinal changed: expected {}, got {}",
            variant, expected_ordinal, discriminant
        );
    }
}

#[test]
fn test_enum_memory_layout_stability() {
    // Verify that enum memory layouts remain stable
    use std::mem;

    let expected_sizes = vec![
        ("ExecutorVersion", 1),
        ("PartitionType", 1),
        ("DataEntryType", 1),
        ("ObjectType", 1),
        ("SignatureType", 1),
        ("KeyPageOperationType", 1),
        ("AccountAuthOperationType", 1),
        ("NetworkMaintenanceOperationType", 1),
        ("TransactionMax", 1),
        ("TransactionType", 1),
        ("AccountType", 1),
        ("AllowedTransactionBit", 1),
        ("VoteType", 1),
        ("BookType", 1),
    ];

    // Test actual sizes match expected
    assert_eq!(mem::size_of::<ExecutorVersion>(), 1);
    assert_eq!(mem::size_of::<PartitionType>(), 1);
    assert_eq!(mem::size_of::<DataEntryType>(), 1);
    assert_eq!(mem::size_of::<ObjectType>(), 1);
    assert_eq!(mem::size_of::<SignatureType>(), 1);
    assert_eq!(mem::size_of::<KeyPageOperationType>(), 1);
    assert_eq!(mem::size_of::<AccountAuthOperationType>(), 1);
    assert_eq!(mem::size_of::<NetworkMaintenanceOperationType>(), 1);
    assert_eq!(mem::size_of::<TransactionMax>(), 1);
    assert_eq!(mem::size_of::<TransactionType>(), 1);
    assert_eq!(mem::size_of::<AccountType>(), 1);
    assert_eq!(mem::size_of::<AllowedTransactionBit>(), 1);
    assert_eq!(mem::size_of::<VoteType>(), 1);
    assert_eq!(mem::size_of::<BookType>(), 1);

    // Test alignment stability
    assert_eq!(mem::align_of::<TransactionType>(), 1);
    assert_eq!(mem::align_of::<AccountType>(), 1);
    assert_eq!(mem::align_of::<SignatureType>(), 1);

    // Test Option optimization stability
    assert_eq!(mem::size_of::<Option<TransactionType>>(), 1);
    assert_eq!(mem::size_of::<Option<AccountType>>(), 1);
    assert_eq!(mem::size_of::<Option<SignatureType>>(), 1);
}

// Removed legacy test_json_schema_stability - format has evolved to camelCase

#[test]
fn test_regeneration_idempotency() {
    // Test that regenerating enums multiple times produces identical results
    // This would require running the generator multiple times, but we can at least
    // verify that current generated code is self-consistent

    let manifest = load_manifest();
    let generation_timestamp = manifest.get("generated_at");

    // Verify manifest has generation metadata
    assert!(generation_timestamp.is_some(), "Manifest should have generation timestamp");

    // Verify all enums are present and self-consistent
    let enums = manifest["enums"].as_array().expect("enums array");
    assert_eq!(enums.len(), 14, "Should have exactly 14 enums");

    // Verify each enum has required fields
    for e in enums {
        assert!(e["name"].is_string(), "Each enum should have a name");
        assert!(e["variants"].is_array(), "Each enum should have variants array");

        let variants = e["variants"].as_array().unwrap();
        assert!(!variants.is_empty(), "Each enum should have at least one variant");

        // Verify all variants are strings (wire tags)
        for variant in variants {
            assert!(variant.is_string(), "All variants should be wire tag strings");
        }
    }
}

#[test]
fn test_backward_compatibility() {
    // Test that enum changes maintain backward compatibility
    // by verifying that old JSON payloads can still be deserialized

    let legacy_json_samples = vec![
        // These represent JSON that should always work
        ("\"writeData\"", "TransactionType"),
        ("\"createIdentity\"", "TransactionType"),
        ("\"sendTokens\"", "TransactionType"),
        ("\"identity\"", "AccountType"),
        ("\"tokenaccount\"", "AccountType"),
        ("\"ed25519\"", "SignatureType"),
        ("\"delegated\"", "SignatureType"),
        ("\"v1\"", "ExecutorVersion"),
        ("\"v2\"", "ExecutorVersion"),
        ("\"directory\"", "PartitionType"),
        ("\"block-validator\"", "PartitionType"),
    ];

    for (json_str, enum_type) in legacy_json_samples {
        match enum_type {
            "TransactionType" => {
                let result: Result<TransactionType, _> = serde_json::from_str(json_str);
                assert!(result.is_ok(), "Failed to deserialize legacy TransactionType: {}", json_str);
            },
            "AccountType" => {
                let result: Result<AccountType, _> = serde_json::from_str(json_str);
                assert!(result.is_ok(), "Failed to deserialize legacy AccountType: {}", json_str);
            },
            "SignatureType" => {
                let result: Result<SignatureType, _> = serde_json::from_str(json_str);
                assert!(result.is_ok(), "Failed to deserialize legacy SignatureType: {}", json_str);
            },
            "ExecutorVersion" => {
                let result: Result<ExecutorVersion, _> = serde_json::from_str(json_str);
                assert!(result.is_ok(), "Failed to deserialize legacy ExecutorVersion: {}", json_str);
            },
            "PartitionType" => {
                let result: Result<PartitionType, _> = serde_json::from_str(json_str);
                assert!(result.is_ok(), "Failed to deserialize legacy PartitionType: {}", json_str);
            },
            _ => panic!("Unknown enum type: {}", enum_type),
        }
    }
}