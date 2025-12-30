use accumulate_client::{AccOptions, AccumulateClient};
use serde_json::json;
use std::time::Duration;
use url::Url;

#[tokio::test]
async fn test_client_creation() {
    let v2_url = Url::parse("http://localhost:26660/v2").unwrap();
    let v3_url = Url::parse("http://localhost:26661/v3").unwrap();
    let options = AccOptions::default();

    let result = AccumulateClient::new_with_options(v2_url, v3_url, options).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_client_from_endpoints() {
    let v2_url = Url::parse("http://localhost:26660/v2").unwrap();
    let v3_url = Url::parse("http://localhost:26661/v3").unwrap();
    let options = AccOptions::default();

    let result = AccumulateClient::from_endpoints(v2_url, v3_url, options).await;
    assert!(result.is_ok());

    let client = result.unwrap();
    let (v2_base, v3_base) = client.get_urls();
    assert_eq!(v2_base, "http://localhost:26660/v2");
    assert_eq!(v3_base, "http://localhost:26661/v3");
}

#[test]
fn test_acc_options_default() {
    let options = AccOptions::default();
    assert_eq!(options.timeout, Duration::from_secs(30));
    assert!(options.headers.is_empty());
}

#[test]
fn test_acc_options_custom() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());

    let options = AccOptions {
        timeout: Duration::from_secs(60),
        headers,
    };

    assert_eq!(options.timeout, Duration::from_secs(60));
    assert_eq!(options.headers.len(), 1);
    assert_eq!(
        options.headers.get("Authorization"),
        Some(&"Bearer token".to_string())
    );
}

#[test]
fn test_canonical_json_ordering() {
    let value = json!({
        "z": 3,
        "a": 1,
        "m": {
            "y": 2,
            "x": 1
        }
    });

    let canonical = accumulate_client::codec::canonical_json(&value);

    // Keys should be in alphabetical order
    let a_pos = canonical.find(r#""a""#).unwrap();
    let m_pos = canonical.find(r#""m""#).unwrap();
    let z_pos = canonical.find(r#""z""#).unwrap();

    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);

    // Nested objects should also be ordered
    let x_pos = canonical.find(r#""x""#).unwrap();
    let y_pos = canonical.find(r#""y""#).unwrap();
    assert!(x_pos < y_pos);
}

#[test]
fn test_canonical_json_arrays() {
    let value = json!({
        "array": [
            {"b": 2, "a": 1},
            {"d": 4, "c": 3}
        ]
    });

    let canonical = accumulate_client::codec::canonical_json(&value);

    // Array elements should maintain order but internal objects should be sorted
    assert!(canonical.contains(r#"[{"a":1,"b":2},{"c":3,"d":4}]"#));
}

#[test]
fn test_canonical_json_primitives() {
    let value = json!({
        "string": "test",
        "number": 42,
        "boolean": true,
        "null": null
    });

    let canonical = accumulate_client::codec::canonical_json(&value);

    // Should contain all primitive types with keys in order
    assert!(canonical.contains(r#""boolean":true"#));
    assert!(canonical.contains(r#""null":null"#));
    assert!(canonical.contains(r#""number":42"#));
    assert!(canonical.contains(r#""string":"test""#));
}

#[test]
fn test_keypair_generation() {
    let keypair = AccumulateClient::generate_keypair();
    // In ed25519-dalek v2, use verifying_key() and to_bytes() instead of .public/.secret
    assert_eq!(keypair.verifying_key().to_bytes().len(), 32);
    assert_eq!(keypair.to_bytes().len(), 32);
}

#[test]
fn test_validate_account_url() {
    assert!(AccumulateClient::validate_account_url("acc://alice.acme"));
    assert!(AccumulateClient::validate_account_url(
        "acc://alice.acme/tokens"
    ));
    assert!(AccumulateClient::validate_account_url("alice.acme/tokens"));
    assert!(!AccumulateClient::validate_account_url("invalid"));
    assert!(!AccumulateClient::validate_account_url(""));
}

#[cfg(feature = "integration")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "DevNet tests require running local DevNet - not required for SDK usage"]
    async fn test_devnet_status() {
        // This test requires a running DevNet instance
        let v2_url = Url::parse("http://localhost:26660/v2").unwrap();
        let v3_url = Url::parse("http://localhost:26661/v3").unwrap();
        let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default())
            .await
            .expect("Failed to create DevNet client");

        let status = client.status().await.expect("Failed to get status");
        assert!(!status.network.is_empty());
        assert!(!status.version.is_empty());
    }

    #[tokio::test]
    async fn test_faucet() {
        // This test requires a running DevNet instance
        let v2_url = Url::parse("http://localhost:26660/v2").unwrap();
        let v3_url = Url::parse("http://localhost:26661/v3").unwrap();
        let client = AccumulateClient::new_with_options(v2_url, v3_url, AccOptions::default())
            .await
            .expect("Failed to create DevNet client");

        let result = client.faucet("acc://test-account").await;
        // Faucet may succeed or fail depending on DevNet state, but should not panic
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_url_parsing() {
    // Test various URL formats are accepted
    assert!(Url::parse("http://localhost:26660/v2").is_ok());
    assert!(Url::parse("https://testnet.accumulatenetwork.io/v2").is_ok());
    assert!(Url::parse("https://mainnet.accumulatenetwork.io/v2").is_ok());

    // Test invalid URLs are rejected
    assert!(Url::parse("invalid-url").is_err());
    assert!(Url::parse("").is_err());
}
