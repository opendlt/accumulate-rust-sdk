// Integration tests with real JSON payloads that would be seen in production
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

// Include the generated enums directly
include!("../src/generated/enums.rs");

// Mock structures that would use our enums in real scenarios
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MockTransaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub nonce: u64,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MockAccount {
    pub url: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub balance: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MockSignature {
    #[serde(rename = "type")]
    pub sig_type: SignatureType,
    pub data: String,
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MockNetworkStatus {
    pub version: ExecutorVersion,
    pub partition: PartitionType,
    pub vote: VoteType,
}

#[test]
fn test_transaction_json_payloads() {
    // Test realistic transaction JSON payloads

    let write_data_json = r#"{
        "type": "writeData",
        "nonce": 123,
        "memo": "Writing some data"
    }"#;

    let tx: MockTransaction = serde_json::from_str(write_data_json).unwrap();
    assert_eq!(tx.tx_type, TransactionType::WriteData);
    assert_eq!(tx.nonce, 123);
    assert_eq!(tx.memo, Some("Writing some data".to_string()));

    // Test serialization back to JSON
    let serialized = serde_json::to_string(&tx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(parsed["type"], "writeData");

    // Test create identity transaction
    let create_identity_json = r#"{
        "type": "createIdentity",
        "nonce": 456,
        "memo": null
    }"#;

    let identity_tx: MockTransaction = serde_json::from_str(create_identity_json).unwrap();
    assert_eq!(identity_tx.tx_type, TransactionType::CreateIdentity);
    assert_eq!(identity_tx.memo, None);

    // Test send tokens transaction
    let send_tokens_json = r#"{
        "type": "sendTokens",
        "nonce": 789
    }"#;

    let send_tx: MockTransaction = serde_json::from_str(send_tokens_json).unwrap();
    assert_eq!(send_tx.tx_type, TransactionType::SendTokens);
    assert_eq!(send_tx.memo, None); // Should default to None when not present
}

#[test]
fn test_account_json_payloads() {
    // Test realistic account JSON payloads

    let identity_account_json = r#"{
        "url": "acc://alice.acme",
        "type": "identity",
        "balance": 1000000
    }"#;

    let account: MockAccount = serde_json::from_str(identity_account_json).unwrap();
    assert_eq!(account.account_type, AccountType::Identity);
    assert_eq!(account.url, "acc://alice.acme");
    assert_eq!(account.balance, 1000000);

    // Test token account
    let token_account_json = r#"{
        "url": "acc://alice.acme/tokens",
        "type": "tokenaccount",
        "balance": 500000
    }"#;

    let token_account: MockAccount = serde_json::from_str(token_account_json).unwrap();
    assert_eq!(token_account.account_type, AccountType::TokenAccount);

    // Test lite token account
    let lite_account_json = r#"{
        "url": "acc://123abc456def",
        "type": "litetokenaccount",
        "balance": 250000
    }"#;

    let lite_account: MockAccount = serde_json::from_str(lite_account_json).unwrap();
    assert_eq!(lite_account.account_type, AccountType::LiteTokenAccount);

    // Test data account
    let data_account_json = r#"{
        "url": "acc://alice.acme/data",
        "type": "dataaccount",
        "balance": 0
    }"#;

    let data_account: MockAccount = serde_json::from_str(data_account_json).unwrap();
    assert_eq!(data_account.account_type, AccountType::DataAccount);
}

#[test]
fn test_signature_json_payloads() {
    // Test realistic signature JSON payloads

    let ed25519_sig_json = r#"{
        "type": "ed25519",
        "data": "a1b2c3d4e5f6...",
        "public_key": "1234567890abcdef..."
    }"#;

    let sig: MockSignature = serde_json::from_str(ed25519_sig_json).unwrap();
    assert_eq!(sig.sig_type, SignatureType::ED25519);
    assert_eq!(sig.data, "a1b2c3d4e5f6...");
    assert_eq!(sig.public_key, "1234567890abcdef...");

    // Test delegated signature
    let delegated_sig_json = r#"{
        "type": "delegated",
        "data": "delegated_signature_data",
        "public_key": "delegated_public_key"
    }"#;

    let delegated_sig: MockSignature = serde_json::from_str(delegated_sig_json).unwrap();
    assert_eq!(delegated_sig.sig_type, SignatureType::Delegated);

    // Test RCD1 signature
    let rcd1_sig_json = r#"{
        "type": "rcd1",
        "data": "rcd1_signature_data",
        "public_key": "rcd1_public_key"
    }"#;

    let rcd1_sig: MockSignature = serde_json::from_str(rcd1_sig_json).unwrap();
    assert_eq!(rcd1_sig.sig_type, SignatureType::RCD1);

    // Test BTC signature
    let btc_sig_json = r#"{
        "type": "btc",
        "data": "btc_signature_data",
        "public_key": "btc_public_key"
    }"#;

    let btc_sig: MockSignature = serde_json::from_str(btc_sig_json).unwrap();
    assert_eq!(btc_sig.sig_type, SignatureType::BTC);
}

#[test]
fn test_network_status_json_payloads() {
    // Test realistic network status JSON payloads

    let network_status_json = r#"{
        "version": "v2",
        "partition": "directory",
        "vote": "accept"
    }"#;

    let status: MockNetworkStatus = serde_json::from_str(network_status_json).unwrap();
    assert_eq!(status.version, ExecutorVersion::V2);
    assert_eq!(status.partition, PartitionType::Directory);
    assert_eq!(status.vote, VoteType::Accept);

    // Test block validator partition
    let validator_status_json = r#"{
        "version": "v1",
        "partition": "block-validator",
        "vote": "reject"
    }"#;

    let validator_status: MockNetworkStatus = serde_json::from_str(validator_status_json).unwrap();
    assert_eq!(validator_status.version, ExecutorVersion::V1);
    assert_eq!(validator_status.partition, PartitionType::BlockValidator);
    assert_eq!(validator_status.vote, VoteType::Reject);

    // Test advanced version with special naming
    let advanced_status_json = r#"{
        "version": "v2-baikonur",
        "partition": "block-summary",
        "vote": "abstain"
    }"#;

    let advanced_status: MockNetworkStatus = serde_json::from_str(advanced_status_json).unwrap();
    assert_eq!(advanced_status.version, ExecutorVersion::V2Baikonur);
    assert_eq!(advanced_status.partition, PartitionType::BlockSummary);
    assert_eq!(advanced_status.vote, VoteType::Abstain);
}

#[test]
fn test_complex_nested_json() {
    // Test complex JSON structures with multiple enum types

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ComplexPayload {
        pub transaction: MockTransaction,
        pub account: MockAccount,
        pub signature: MockSignature,
        pub metadata: MockNetworkStatus,
    }

    let complex_json = r#"{
        "transaction": {
            "type": "createTokenAccount",
            "nonce": 999,
            "memo": "Creating new token account"
        },
        "account": {
            "url": "acc://company.acme/tokens",
            "type": "tokenaccount",
            "balance": 1000000000
        },
        "signature": {
            "type": "ed25519",
            "data": "signature_data_here",
            "public_key": "ed25519_public_key_here"
        },
        "metadata": {
            "version": "v2-vandenberg",
            "partition": "directory",
            "vote": "suggest"
        }
    }"#;

    let complex: ComplexPayload = serde_json::from_str(complex_json).unwrap();

    assert_eq!(complex.transaction.tx_type, TransactionType::CreateTokenAccount);
    assert_eq!(complex.account.account_type, AccountType::TokenAccount);
    assert_eq!(complex.signature.sig_type, SignatureType::ED25519);
    assert_eq!(complex.metadata.version, ExecutorVersion::V2Vandenberg);
    assert_eq!(complex.metadata.partition, PartitionType::Directory);
    assert_eq!(complex.metadata.vote, VoteType::Suggest);

    // Test that it can be serialized back correctly
    let serialized = serde_json::to_string_pretty(&complex).unwrap();
    let reparsed: ComplexPayload = serde_json::from_str(&serialized).unwrap();
    assert_eq!(complex, reparsed);
}

#[test]
fn test_array_of_enums() {
    // Test JSON arrays containing enum values

    let transaction_types_json = r#"[
        "writeData",
        "createIdentity",
        "sendTokens",
        "createTokenAccount",
        "addCredits"
    ]"#;

    let tx_types: Vec<TransactionType> = serde_json::from_str(transaction_types_json).unwrap();
    assert_eq!(tx_types.len(), 5);
    assert_eq!(tx_types[0], TransactionType::WriteData);
    assert_eq!(tx_types[1], TransactionType::CreateIdentity);
    assert_eq!(tx_types[2], TransactionType::SendTokens);
    assert_eq!(tx_types[3], TransactionType::CreateTokenAccount);
    assert_eq!(tx_types[4], TransactionType::AddCredits);

    // Test signature types array
    let signature_types_json = r#"[
        "ed25519",
        "rcd1",
        "delegated",
        "btc",
        "eth"
    ]"#;

    let sig_types: Vec<SignatureType> = serde_json::from_str(signature_types_json).unwrap();
    assert_eq!(sig_types.len(), 5);
    assert_eq!(sig_types[0], SignatureType::ED25519);
    assert_eq!(sig_types[1], SignatureType::RCD1);
    assert_eq!(sig_types[2], SignatureType::Delegated);
    assert_eq!(sig_types[3], SignatureType::BTC);
    assert_eq!(sig_types[4], SignatureType::ETH);
}

#[test]
fn test_optional_enum_fields() {
    // Test structures with optional enum fields

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct OptionalEnumStruct {
        pub required_type: TransactionType,
        pub optional_account: Option<AccountType>,
        pub optional_signature: Option<SignatureType>,
    }

    // Test with all fields present
    let full_json = r#"{
        "required_type": "writeData",
        "optional_account": "identity",
        "optional_signature": "ed25519"
    }"#;

    let full: OptionalEnumStruct = serde_json::from_str(full_json).unwrap();
    assert_eq!(full.required_type, TransactionType::WriteData);
    assert_eq!(full.optional_account, Some(AccountType::Identity));
    assert_eq!(full.optional_signature, Some(SignatureType::ED25519));

    // Test with optional fields missing
    let minimal_json = r#"{
        "required_type": "sendTokens"
    }"#;

    let minimal: OptionalEnumStruct = serde_json::from_str(minimal_json).unwrap();
    assert_eq!(minimal.required_type, TransactionType::SendTokens);
    assert_eq!(minimal.optional_account, None);
    assert_eq!(minimal.optional_signature, None);

    // Test with null values
    let null_json = r#"{
        "required_type": "createIdentity",
        "optional_account": null,
        "optional_signature": null
    }"#;

    let null_opts: OptionalEnumStruct = serde_json::from_str(null_json).unwrap();
    assert_eq!(null_opts.required_type, TransactionType::CreateIdentity);
    assert_eq!(null_opts.optional_account, None);
    assert_eq!(null_opts.optional_signature, None);
}

#[test]
fn test_enum_in_hashmap() {
    // Test enum usage in HashMap structures (realistic for configs, etc.)

    let config_json = r#"{
        "writeData": {"fee": 100, "enabled": true},
        "createIdentity": {"fee": 500, "enabled": true},
        "sendTokens": {"fee": 50, "enabled": false},
        "addCredits": {"fee": 25, "enabled": true}
    }"#;

    #[derive(Serialize, Deserialize, Debug)]
    struct TransactionConfig {
        pub fee: u64,
        pub enabled: bool,
    }

    let config: HashMap<TransactionType, TransactionConfig> = serde_json::from_str(config_json).unwrap();

    assert_eq!(config.len(), 4);
    assert_eq!(config.get(&TransactionType::WriteData).unwrap().fee, 100);
    assert_eq!(config.get(&TransactionType::CreateIdentity).unwrap().fee, 500);
    assert_eq!(config.get(&TransactionType::SendTokens).unwrap().enabled, false);
    assert_eq!(config.get(&TransactionType::AddCredits).unwrap().enabled, true);

    // Test serialization back to JSON
    let serialized = serde_json::to_string(&config).unwrap();
    let reparsed: HashMap<TransactionType, TransactionConfig> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(config.len(), reparsed.len());
}

#[test]
fn test_enum_error_messages() {
    // Test that error messages are helpful when deserialization fails

    let invalid_tx_json = r#"{"type": "invalidTransactionType"}"#;
    let result: Result<MockTransaction, _> = serde_json::from_str(invalid_tx_json);

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Error message should mention the invalid value
    assert!(error_msg.contains("invalidTransactionType") || error_msg.contains("unknown variant"));

    // Test invalid account type
    let invalid_account_json = r#"{"url": "test", "type": "invalidAccountType", "balance": 0}"#;
    let result: Result<MockAccount, _> = serde_json::from_str(invalid_account_json);

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("invalidAccountType") || error_msg.contains("unknown variant"));
}

#[test]
fn test_enum_with_different_json_formatters() {
    // Test that enums work with different JSON formatting options

    let tx = MockTransaction {
        tx_type: TransactionType::CreateTokenAccount,
        nonce: 12345,
        memo: Some("Test memo".to_string()),
    };

    // Test compact JSON
    let compact = serde_json::to_string(&tx).unwrap();
    assert!(compact.contains("createTokenAccount"));
    assert!(!compact.contains("  ")); // No pretty formatting

    // Test pretty JSON
    let pretty = serde_json::to_string_pretty(&tx).unwrap();
    assert!(pretty.contains("createTokenAccount"));
    assert!(pretty.contains("  ")); // Has indentation

    // Both should deserialize correctly
    let from_compact: MockTransaction = serde_json::from_str(&compact).unwrap();
    let from_pretty: MockTransaction = serde_json::from_str(&pretty).unwrap();

    assert_eq!(tx, from_compact);
    assert_eq!(tx, from_pretty);
    assert_eq!(from_compact, from_pretty);
}

#[test]
fn test_real_world_json_examples() {
    // Test with JSON that might actually come from the Accumulate network

    // Example of a real transaction response
    let real_tx_json = r#"{
        "type": "writeData",
        "nonce": 42,
        "memo": "Storing document hash: abc123def456"
    }"#;

    let tx: MockTransaction = serde_json::from_str(real_tx_json).unwrap();
    assert_eq!(tx.tx_type, TransactionType::WriteData);

    // Example of account query response
    let real_account_json = r#"{
        "url": "acc://alice.acme",
        "type": "identity",
        "balance": 999999999
    }"#;

    let account: MockAccount = serde_json::from_str(real_account_json).unwrap();
    assert_eq!(account.account_type, AccountType::Identity);

    // Example of network status
    let real_status_json = r#"{
        "version": "v2-jiuquan",
        "partition": "block-validator",
        "vote": "accept"
    }"#;

    let status: MockNetworkStatus = serde_json::from_str(real_status_json).unwrap();
    assert_eq!(status.version, ExecutorVersion::V2Jiuquan);
    assert_eq!(status.partition, PartitionType::BlockValidator);
    assert_eq!(status.vote, VoteType::Accept);
}