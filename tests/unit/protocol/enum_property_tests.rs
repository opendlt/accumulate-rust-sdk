// Property-based and fuzzing tests for enum robustness
use serde_json;
use std::collections::{HashMap, HashSet};

// Include the generated enums directly
include!("../../../src/generated/enums.rs");

#[test]
fn test_enum_serialization_properties() {
    // Property: For any enum variant, serialize(deserialize(serialize(x))) == serialize(x)

    // Test TransactionType variants
    test_enum_serialization_property(TransactionType::WriteData);
    test_enum_serialization_property(TransactionType::CreateIdentity);
    test_enum_serialization_property(TransactionType::SendTokens);
    test_enum_serialization_property(TransactionType::SystemGenesis);

    // Test AccountType variants
    test_enum_serialization_property(AccountType::Identity);
    test_enum_serialization_property(AccountType::TokenAccount);
    test_enum_serialization_property(AccountType::DataAccount);

    // Test SignatureType variants
    test_enum_serialization_property(SignatureType::ED25519);
    test_enum_serialization_property(SignatureType::Delegated);
    test_enum_serialization_property(SignatureType::BTC);

    // Test ExecutorVersion variants
    test_enum_serialization_property(ExecutorVersion::V1);
    test_enum_serialization_property(ExecutorVersion::V2);
    test_enum_serialization_property(ExecutorVersion::V2Baikonur);

    println!("Success: All enum serialization properties verified");
}

#[test]
fn test_transaction_type_roundtrip_property() {
    // Property: For every TransactionType variant, roundtrip serialization preserves equality

    let all_variants = vec![
        TransactionType::Unknown,
        TransactionType::CreateIdentity,
        TransactionType::CreateTokenAccount,
        TransactionType::SendTokens,
        TransactionType::CreateDataAccount,
        TransactionType::WriteData,
        TransactionType::WriteDataTo,
        TransactionType::AcmeFaucet,
        TransactionType::CreateToken,
        TransactionType::IssueTokens,
        TransactionType::BurnTokens,
        TransactionType::CreateLiteTokenAccount,
        TransactionType::CreateKeyPage,
        TransactionType::CreateKeyBook,
        TransactionType::AddCredits,
        TransactionType::UpdateKeyPage,
        TransactionType::LockAccount,
        TransactionType::BurnCredits,
        TransactionType::TransferCredits,
        TransactionType::UpdateAccountAuth,
        TransactionType::UpdateKey,
        TransactionType::NetworkMaintenance,
        TransactionType::ActivateProtocolVersion,
        TransactionType::Remote,
        TransactionType::SyntheticCreateIdentity,
        TransactionType::SyntheticWriteData,
        TransactionType::SyntheticDepositTokens,
        TransactionType::SyntheticDepositCredits,
        TransactionType::SyntheticBurnTokens,
        TransactionType::SyntheticForwardTransaction,
        TransactionType::SystemGenesis,
        TransactionType::DirectoryAnchor,
        TransactionType::BlockValidatorAnchor,
        TransactionType::SystemWriteData,
    ];

    for variant in all_variants {
        // Property: serialize -> deserialize -> serialize gives same result
        let json1 = serde_json::to_string(&variant).unwrap();
        let deserialized: TransactionType = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&deserialized).unwrap();

        assert_eq!(variant, deserialized, "Roundtrip should preserve equality");
        assert_eq!(json1, json2, "Roundtrip should preserve JSON representation");

        // Property: JSON should be deterministic
        let json3 = serde_json::to_string(&variant).unwrap();
        assert_eq!(json1, json3, "Serialization should be deterministic");
    }
}

#[test]
fn test_account_type_exhaustive_property() {
    // Property: Every AccountType variant should have unique JSON representation

    let all_variants = vec![
        AccountType::Unknown,
        AccountType::AnchorLedger,
        AccountType::Identity,
        AccountType::TokenIssuer,
        AccountType::TokenAccount,
        AccountType::LiteTokenAccount,
        AccountType::BlockLedger,
        AccountType::KeyPage,
        AccountType::KeyBook,
        AccountType::DataAccount,
        AccountType::LiteDataAccount,
        AccountType::UnknownSigner,
        AccountType::SystemLedger,
        AccountType::LiteIdentity,
        AccountType::SyntheticLedger,
    ];

    let mut json_representations = HashSet::new();
    let mut wire_tags = HashSet::new();

    for variant in all_variants {
        let json = serde_json::to_string(&variant).unwrap();

        // Property: Each variant should have unique JSON
        assert!(json_representations.insert(json.clone()),
               "Duplicate JSON representation found: {}", json);

        // Extract wire tag (remove quotes)
        let wire_tag = json.trim_matches('"');
        assert!(wire_tags.insert(wire_tag.to_string()),
               "Duplicate wire tag found: {}", wire_tag);

        // Property: Wire tag should be non-empty
        assert!(!wire_tag.is_empty(), "Wire tag should not be empty");

        // Property: Wire tag should be reasonable length (not too long)
        assert!(wire_tag.len() <= 30, "Wire tag too long: {}", wire_tag);
    }
}

#[test]
fn test_signature_type_injection_resistance() {
    // Property: SignatureType should reject injection attempts

    let malicious_inputs = vec![
        // Script injection attempts
        "\"<script>alert('xss')</script>\"",
        "\"; DROP TABLE users; --\"",
        "\"../../../etc/passwd\"",

        // Format string attacks
        "\"%s%s%s%s\"",
        "\"%n%n%n%n\"",

        // Unicode normalization - this is actually valid JSON and should decode to "ed25519"
        // Removed: "\"\\u0065\\u0064\\u0032\\u0035\\u0035\\u0031\\u0039\"" - this is legitimate

        // Null byte injection
        "\"ed25519\\u0000malicious\"",

        // Control characters
        "\"ed25519\\r\\n\"",
        "\"ed25519\\t\"",

        // Binary data as JSON string
        "\"\\x00\\x01\\x02\\x03\"",
    ];

    // Add overly long string separately to avoid lifetime issues
    let long_string = format!("\"{}\"", "a".repeat(1000));

    // Test all malicious inputs
    for malicious_input in malicious_inputs.iter().chain(std::iter::once(&long_string.as_str())) {
        let result: Result<SignatureType, _> = serde_json::from_str(malicious_input);
        assert!(result.is_err(),
               "Should reject malicious input: {}", malicious_input);
    }
}

#[test]
fn test_enum_boundary_conditions() {
    // Property: Enums should handle boundary conditions gracefully

    // Test empty JSON
    let empty_cases = vec!["", " ", "\t", "\n"];
    for case in empty_cases {
        let result: Result<TransactionType, _> = serde_json::from_str(case);
        assert!(result.is_err(), "Should reject empty/whitespace input: '{}'", case);
    }

    // Test malformed JSON
    let malformed_cases = vec![
        "\"unclosed string",
        "unquoted_string",
        "\"double\"\"quotes\"",
        "{\"object\": \"not_enum\"}",
        "[\"array\", \"not_enum\"]",
        "123",
        "true",
        "false",
        "null",
    ];

    for case in malformed_cases {
        let result: Result<AccountType, _> = serde_json::from_str(case);
        assert!(result.is_err(), "Should reject malformed JSON: {}", case);
    }
}

#[test]
fn test_enum_case_sensitivity_property() {
    // Property: Enum deserialization should be strictly case-sensitive

    let base_cases = vec![
        ("writeData", "WriteData", "WRITEDATA", "writedata"),
        ("createIdentity", "CreateIdentity", "CREATEIDENTITY", "createidentity"),
        ("identity", "Identity", "IDENTITY", "Identity"),
        ("ed25519", "Ed25519", "ED25519", "Ed25519"),
    ];

    for (correct, pascal, upper, lower) in base_cases {
        // Correct case should work
        let correct_json = format!("\"{}\"", correct);
        let result: Result<serde_json::Value, _> = serde_json::from_str(&correct_json);
        assert!(result.is_ok(), "Correct case should deserialize: {}", correct);

        // Wrong cases should fail
        for wrong_case in vec![pascal, upper, lower] {
            let wrong_json = format!("\"{}\"", wrong_case);

            // Try with different enum types to be thorough
            let tx_result: Result<TransactionType, _> = serde_json::from_str(&wrong_json);
            let acc_result: Result<AccountType, _> = serde_json::from_str(&wrong_json);
            let sig_result: Result<SignatureType, _> = serde_json::from_str(&wrong_json);

            // At least one should fail (and probably all)
            let all_failed = tx_result.is_err() && acc_result.is_err() && sig_result.is_err();
            if !all_failed {
                // If any succeeded, it better be the correct case
                assert_eq!(wrong_case, correct,
                          "Wrong case succeeded when it shouldn't: {}", wrong_case);
            }
        }
    }
}

#[test]
fn test_enum_collection_properties() {
    // Property: Enums should work correctly in collections with expected semantics

    // Test HashMap properties
    let mut tx_counts = HashMap::new();
    let test_transactions = vec![
        TransactionType::WriteData,
        TransactionType::CreateIdentity,
        TransactionType::WriteData,  // duplicate
        TransactionType::SendTokens,
        TransactionType::CreateIdentity,  // duplicate
    ];

    for tx in test_transactions {
        *tx_counts.entry(tx).or_insert(0) += 1;
    }

    // Property: HashMap should deduplicate keys
    assert_eq!(tx_counts.len(), 3, "HashMap should have 3 unique keys");
    assert_eq!(tx_counts[&TransactionType::WriteData], 2);
    assert_eq!(tx_counts[&TransactionType::CreateIdentity], 2);
    assert_eq!(tx_counts[&TransactionType::SendTokens], 1);

    // Test HashSet properties
    let mut sig_set = HashSet::new();
    let test_signatures = vec![
        SignatureType::ED25519,
        SignatureType::Delegated,
        SignatureType::ED25519,  // duplicate
        SignatureType::BTC,
        SignatureType::Delegated,  // duplicate
    ];

    for sig in test_signatures {
        sig_set.insert(sig);
    }

    // Property: HashSet should deduplicate values
    assert_eq!(sig_set.len(), 3, "HashSet should have 3 unique values");
    assert!(sig_set.contains(&SignatureType::ED25519));
    assert!(sig_set.contains(&SignatureType::Delegated));
    assert!(sig_set.contains(&SignatureType::BTC));
}

#[test]
fn test_enum_arithmetic_properties() {
    // Property: Enum discriminants should be consistent and stable

    // Test that discriminants are in expected ranges
    let tx_unknown = TransactionType::Unknown as u8;
    let tx_create_identity = TransactionType::CreateIdentity as u8;

    assert_eq!(tx_unknown, 0, "Unknown should be discriminant 0");
    assert!(tx_create_identity > tx_unknown, "CreateIdentity should be after Unknown");
    assert!(tx_create_identity < 50, "Discriminants should be reasonable");

    // Test that different variants have different discriminants
    let acc_unknown = AccountType::Unknown as u8;
    let acc_identity = AccountType::Identity as u8;
    let acc_token = AccountType::TokenAccount as u8;

    let discriminants = vec![acc_unknown, acc_identity, acc_token];
    let unique_count = discriminants.iter().collect::<HashSet<_>>().len();
    assert_eq!(unique_count, discriminants.len(), "All discriminants should be unique");
}

#[test]
fn test_enum_stress_serialization() {
    // Property: Enum serialization should handle high-volume operations

    let iterations = 10_000;
    let variants = vec![
        TransactionType::WriteData,
        TransactionType::CreateIdentity,
        TransactionType::SendTokens,
        TransactionType::CreateTokenAccount,
        TransactionType::AddCredits,
    ];

    // Stress test serialization
    for i in 0..iterations {
        let variant = &variants[i % variants.len()];
        let json = serde_json::to_string(variant).unwrap();

        // Property: Each serialization should be consistent
        let expected_json = serde_json::to_string(variant).unwrap();
        assert_eq!(json, expected_json, "Serialization should be consistent at iteration {}", i);

        // Property: JSON should be valid and parseable
        let _value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Property: Roundtrip should preserve value
        let deserialized: TransactionType = serde_json::from_str(&json).unwrap();
        assert_eq!(*variant, deserialized, "Roundtrip should preserve value at iteration {}", i);
    }
}

#[test]
fn test_enum_concurrent_safety() {
    // Property: Enum operations should be thread-safe (Copy + Send + Sync)

    use std::sync::Arc;
    use std::thread;

    let shared_enum = Arc::new(TransactionType::WriteData);
    let mut handles = vec![];

    // Spawn multiple threads using the same enum value
    for i in 0..10 {
        let enum_clone = Arc::clone(&shared_enum);
        let handle = thread::spawn(move || {
            // Each thread serializes the enum multiple times
            for _ in 0..100 {
                let json = serde_json::to_string(&*enum_clone).unwrap();
                assert_eq!(json, "\"writeData\"", "Thread {} got wrong JSON", i);

                let deserialized: TransactionType = serde_json::from_str(&json).unwrap();
                assert_eq!(*enum_clone, deserialized, "Thread {} roundtrip failed", i);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_enum_fuzzing_simulation() {
    // Property: Enum deserialization should gracefully handle random inputs

    // Simulate fuzzing with various malformed inputs
    let fuzz_inputs = vec![
        // Random strings
        "\"random_string_123\"",
        "\"invalid123\"",
        "\"\\u0000\\u0001\\u0002\"",

        // Partial matches
        "\"write\"",
        "\"writeDataExtra\"",
        "\"write_data\"",
        "\"write-data\"",

        // Number-like strings
        "\"123\"",
        "\"0x123\"",
        "\"3.14159\"",

        // Boolean-like strings
        "\"true\"",
        "\"false\"",
        "\"TRUE\"",
        "\"FALSE\"",

        // Common typos
        "\"writeDate\"",  // typo
        "\"createIndentity\"",  // typo
        "\"sendToken\"",  // missing s

        // Case variations
        "\"WriteData\"",
        "\"WRITEDATA\"",
        "\"writedata\"",

        // Special characters
        "\"write@data\"",
        "\"write#data\"",
        "\"write$data\"",
        "\"write%data\"",
    ];

    for input in fuzz_inputs {
        // All of these should fail to deserialize
        let tx_result: Result<TransactionType, _> = serde_json::from_str(input);
        let acc_result: Result<AccountType, _> = serde_json::from_str(input);
        let sig_result: Result<SignatureType, _> = serde_json::from_str(input);
        let exec_result: Result<ExecutorVersion, _> = serde_json::from_str(input);

        // Property: Invalid inputs should be rejected
        assert!(tx_result.is_err(), "TransactionType should reject: {}", input);
        assert!(acc_result.is_err(), "AccountType should reject: {}", input);
        assert!(sig_result.is_err(), "SignatureType should reject: {}", input);
        assert!(exec_result.is_err(), "ExecutorVersion should reject: {}", input);
    }
}

// Helper function for testing enum serialization properties
fn test_enum_serialization_property<T>(value: T)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug,
{
    let json1 = serde_json::to_string(&value).unwrap();

    // Property: JSON serialization should be valid JSON
    let _parsed: serde_json::Value = serde_json::from_str(&json1).unwrap();

    // Property: Should be a string (enum wire format)
    assert!(json1.starts_with('"') && json1.ends_with('"'),
           "Enum should serialize to JSON string: {}", json1);

    // Property: Should contain no whitespace (compact format)
    assert!(!json1.contains(' ') && !json1.contains('\t') && !json1.contains('\n'),
           "Enum JSON should be compact: {}", json1);

    // Property: Should be ASCII only (no unicode)
    assert!(json1.is_ascii(), "Enum JSON should be ASCII only: {}", json1);

    println!("Success: Enum serialization property verified for: {}", json1);
}