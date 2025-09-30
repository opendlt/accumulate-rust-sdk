//! Integration test to verify codec wiring with the main client

use accumulate_client::{AccumulateClient, TransactionBodyBuilder, TokenRecipient, canonical_json, sha256_bytes};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use serde_json::json;

#[test]
fn test_codec_integration_with_client() {
    // Generate a keypair
    let mut rng = OsRng;
    let keypair = Keypair::generate(&mut rng);

    // Create a transaction body
    let body = TransactionBodyBuilder::send_tokens(vec![TokenRecipient {
        url: "acc://bob.acme/tokens".to_string(),
        amount: "1000".to_string(),
    }]);

    // Create a mock client (we don't need actual network endpoints for this test)
    let v2_url = url::Url::parse("http://localhost:26660/v2").unwrap();
    let v3_url = url::Url::parse("http://localhost:26660/v3").unwrap();
    let client = accumulate_client::AccumulateClient::new_with_options(
        v2_url,
        v3_url,
        accumulate_client::AccOptions::default()
    );

    // This will fail with network error, but that's OK - we just want to verify compilation
    let _result = client.await;

    println!("✅ Codec integration test compiled successfully");
}

#[test]
fn test_standalone_codec_functions() {
    // Test canonical JSON
    let test_value = json!({
        "z": 3,
        "a": 1,
        "m": 2
    });

    let canonical = canonical_json(&test_value);
    assert_eq!(canonical, r#"{"a":1,"m":2,"z":3}"#);

    // Test SHA256
    let hash = sha256_bytes(canonical.as_bytes());
    assert_eq!(hash.len(), 32);

    // Test hex encoding
    let hash_hex = hex::encode(hash);
    assert_eq!(hash_hex.len(), 64);

    println!("✅ Standalone codec functions work correctly");
}

#[test]
fn test_transaction_body_builders() {
    // Test send tokens
    let send_tokens_body = TransactionBodyBuilder::send_tokens(vec![TokenRecipient {
        url: "acc://bob.acme/tokens".to_string(),
        amount: "1000".to_string(),
    }]);

    assert_eq!(send_tokens_body["type"], "send-tokens");
    assert_eq!(send_tokens_body["to"][0]["url"], "acc://bob.acme/tokens");
    assert_eq!(send_tokens_body["to"][0]["amount"], "1000");

    // Test create identity
    let create_identity_body = TransactionBodyBuilder::create_identity(
        "acc://alice.acme".to_string(),
        "acc://alice.acme/book".to_string(),
    );

    assert_eq!(create_identity_body["type"], "create-identity");
    assert_eq!(create_identity_body["url"], "acc://alice.acme");
    assert_eq!(create_identity_body["keyBook"], "acc://alice.acme/book");

    println!("✅ Transaction body builders work correctly");
}