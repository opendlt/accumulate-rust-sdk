//! Protocol Type Conformance Tests
//!
//! Tests for basic protocol type functionality and serialization

use accumulate_client::generated::enums::*;
use serde_json;

#[test]
fn test_account_type_serialization() {
    // Test AccountType enum variants
    let account_types = vec![
        AccountType::Unknown,
        AccountType::Identity,
        AccountType::TokenAccount,
        AccountType::LiteTokenAccount,
        AccountType::KeyPage,
        AccountType::KeyBook,
        AccountType::DataAccount,
    ];

    for account_type in account_types {
        let serialized = serde_json::to_value(&account_type).expect("Failed to serialize AccountType");
        assert!(serialized.is_string() || serialized.is_number());

        // Test round-trip
        let deserialized: AccountType = serde_json::from_value(serialized)
            .expect("Failed to deserialize AccountType");
        assert_eq!(account_type, deserialized);
    }

    println!("✓ AccountType enum serialization works");
}

#[test]
fn test_signature_type_serialization() {
    // Test SignatureType enum variants
    let signature_types = vec![
        SignatureType::Unknown,
        SignatureType::LegacyED25519,
        SignatureType::ED25519,
        SignatureType::RCD1,
        SignatureType::BTC,
        SignatureType::BTCLegacy,
        SignatureType::ETH,
    ];

    for sig_type in signature_types {
        let serialized = serde_json::to_value(&sig_type).expect("Failed to serialize SignatureType");
        assert!(serialized.is_string() || serialized.is_number());

        // Test round-trip
        let deserialized: SignatureType = serde_json::from_value(serialized)
            .expect("Failed to deserialize SignatureType");
        assert_eq!(sig_type, deserialized);
    }

    println!("✓ SignatureType enum serialization works");
}

#[test]
fn test_transaction_type_serialization() {
    // Test TransactionType enum variants
    let transaction_types = vec![
        TransactionType::Unknown,
        TransactionType::SendTokens,
        TransactionType::CreateIdentity,
        TransactionType::CreateTokenAccount,
        TransactionType::CreateDataAccount,
        TransactionType::CreateKeyPage,
        TransactionType::CreateKeyBook,
        TransactionType::AddCredits,
        TransactionType::UpdateKeyPage,
        TransactionType::WriteData,
        TransactionType::WriteDataTo,
    ];

    for tx_type in transaction_types {
        let serialized = serde_json::to_value(&tx_type).expect("Failed to serialize TransactionType");
        assert!(serialized.is_string() || serialized.is_number());

        // Test round-trip
        let deserialized: TransactionType = serde_json::from_value(serialized)
            .expect("Failed to deserialize TransactionType");
        assert_eq!(tx_type, deserialized);
    }

    println!("✓ TransactionType enum serialization works");
}

#[test]
fn test_executor_version_serialization() {
    // Test ExecutorVersion enum variants
    let executor_versions = vec![
        ExecutorVersion::V1,
        ExecutorVersion::V1SignatureAnchoring,
        ExecutorVersion::V1DoubleHashEntries,
        ExecutorVersion::V1Halt,
        ExecutorVersion::V2,
        ExecutorVersion::V2Baikonur,
        ExecutorVersion::V2Vandenberg,
        ExecutorVersion::V2Jiuquan,
        ExecutorVersion::VNext,
    ];

    for exec_version in executor_versions {
        let serialized = serde_json::to_value(&exec_version).expect("Failed to serialize ExecutorVersion");
        assert!(serialized.is_string() || serialized.is_number());

        // Test round-trip
        let deserialized: ExecutorVersion = serde_json::from_value(serialized)
            .expect("Failed to deserialize ExecutorVersion");
        assert_eq!(exec_version, deserialized);
    }

    println!("✓ ExecutorVersion enum serialization works");
}

#[test]
fn test_vote_type_serialization() {
    // Test VoteType enum variants
    let vote_types = vec![
        VoteType::Accept,
        VoteType::Reject,
        VoteType::Abstain,
        VoteType::Suggest,
    ];

    for vote_type in vote_types {
        let serialized = serde_json::to_value(&vote_type).expect("Failed to serialize VoteType");
        assert!(serialized.is_string() || serialized.is_number());

        // Test round-trip
        let deserialized: VoteType = serde_json::from_value(serialized)
            .expect("Failed to deserialize VoteType");
        assert_eq!(vote_type, deserialized);
    }

    println!("✓ VoteType enum serialization works");
}

#[test]
fn test_json_format_consistency() {
    // Test that the JSON format is consistent and readable
    let account_type = AccountType::TokenAccount;
    let serialized = serde_json::to_string(&account_type).expect("Failed to serialize to string");

    // Should be a valid JSON string or number
    assert!(serialized.starts_with('"') || serialized.chars().next().unwrap().is_ascii_digit());

    // Should be parseable back
    let deserialized: AccountType = serde_json::from_str(&serialized)
        .expect("Failed to deserialize from string");
    assert_eq!(account_type, deserialized);

    println!("✓ JSON format consistency verified");
    println!("  Serialized AccountType::TokenAccount as: {}", serialized);
}

#[test]
fn test_error_handling() {
    // Test that invalid JSON fails gracefully
    let invalid_json = serde_json::json!("NotAValidAccountType");

    let result: Result<AccountType, _> = serde_json::from_value(invalid_json);

    // Should fail, but gracefully
    assert!(result.is_err());
    println!("✓ Invalid JSON properly rejected");
}

#[test]
fn test_enum_equality() {
    // Test that enum equality works correctly
    let account1 = AccountType::Identity;
    let account2 = AccountType::Identity;
    let account3 = AccountType::TokenAccount;

    assert_eq!(account1, account2);
    assert_ne!(account1, account3);

    println!("✓ Enum equality comparison works");
}

#[test]
fn test_clone_and_debug() {
    // Test that enums support Clone and Debug
    let original = AccountType::DataAccount;
    let cloned = original.clone();

    assert_eq!(original, cloned);

    // Test Debug formatting
    let debug_string = format!("{:?}", original);
    assert!(debug_string.contains("DataAccount"));

    println!("✓ Clone and Debug traits work");
}

#[test]
fn test_comprehensive_enum_coverage() {
    // Ensure we have good coverage of the generated enums
    println!("Testing comprehensive enum coverage:");

    // Test BookType
    let book_type = BookType::Normal;
    let serialized = serde_json::to_value(&book_type).expect("Failed to serialize BookType");
    let _: BookType = serde_json::from_value(serialized).expect("Failed to deserialize BookType");
    println!("  ✓ BookType serialization works");

    // Test DataEntryType
    let data_entry_type = DataEntryType::Accumulate;
    let serialized = serde_json::to_value(&data_entry_type).expect("Failed to serialize DataEntryType");
    let _: DataEntryType = serde_json::from_value(serialized).expect("Failed to deserialize DataEntryType");
    println!("  ✓ DataEntryType serialization works");

    // Test KeyPageOperationType
    let key_op_type = KeyPageOperationType::Update;
    let serialized = serde_json::to_value(&key_op_type).expect("Failed to serialize KeyPageOperationType");
    let _: KeyPageOperationType = serde_json::from_value(serialized).expect("Failed to deserialize KeyPageOperationType");
    println!("  ✓ KeyPageOperationType serialization works");

    println!("✓ Comprehensive enum coverage test passed");
}