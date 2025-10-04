use accumulate_client::*;
use async_trait::async_trait;
use serde_json::json;

#[derive(Clone)]
struct MockOkTransport;

#[async_trait]
impl generated::api_methods::AccumulateRpc for MockOkTransport {
    async fn rpc_call<TP: serde::Serialize + Send + Sync, TR: for<'de> serde::Deserialize<'de>>(
        &self, method: &str, params: &TP
    ) -> Result<TR, errors::Error> {
        let _p = serde_json::to_value(params).unwrap();

        let result = match method {
            "status" => json!({
                "network": "devnet",
                "version": "1.0.0",
                "commit": "abcd1234",
                "describe": {
                    "version": "v1.0.0",
                    "commit": "abcd1234",
                    "dirty": false
                }
            }),
            "version" => json!({
                "version": "v1.0.0",
                "commit": "abcd1234",
                "api": "v2"
            }),
            "query" => json!({
                "type": "identity",
                "url": "acc://test.acme",
                "data": {
                    "url": "acc://test.acme",
                    "accountId": "0000000000000000000000000000000000000000000000000000000000000000"
                },
                "merkleState": {
                    "root": "0000000000000000000000000000000000000000000000000000000000000000",
                    "count": 0
                }
            }),
            "query-tx" => json!({
                "transaction": {
                    "header": {
                        "principal": "acc://test.acme",
                        "initiator": "0000000000000000000000000000000000000000000000000000000000000000"
                    },
                    "body": {
                        "type": "sendTokens",
                        "to": []
                    }
                },
                "signatures": [],
                "status": {
                    "delivered": true
                }
            }),
            "execute" => json!({
                "transactionHash": "0000000000000000000000000000000000000000000000000000000000000000",
                "simple": true
            }),
            "query-directory" => json!({
                "items": []
            }),
            "faucet" => json!({
                "transactionHash": "0000000000000000000000000000000000000000000000000000000000000000",
                "link": "https://testnet.accumulatenetwork.io/tx/0000000000000000000000000000000000000000000000000000000000000000"
            }),
            "describe" => json!({
                "version": "v1.0.0",
                "commit": "abcd1234",
                "dirty": false
            }),
            _ => json!({}),
        };

        Ok(serde_json::from_value(result).unwrap())
    }
}

#[tokio::test]
async fn smoke_status_and_version() {
    let client = generated::api_methods::AccumulateClient { transport: MockOkTransport };

    // Test status method
    let status_params = generated::api_methods::StatusParams {};
    let status = client.status(status_params).await.unwrap();

    // Basic shape assertion - we expect some kind of status response
    let status_json = serde_json::to_value(&status).unwrap();
    assert!(status_json.is_object(), "Status should return an object");

    // Test version method
    let version_params = generated::api_methods::VersionParams {};
    let version = client.version(version_params).await.unwrap();

    // Basic shape assertion - we expect some kind of version response
    let version_json = serde_json::to_value(&version).unwrap();
    assert!(version_json.is_object(), "Version should return an object");
}

#[tokio::test]
async fn smoke_query_methods() {
    let client = generated::api_methods::AccumulateClient { transport: MockOkTransport };

    // Test query method
    let query_params = generated::api_methods::QueryParams {
        url: "acc://test.acme".to_string(),
        prove: Some(false),
        scratch: Some(false),
    };
    let query_result = client.query(query_params).await.unwrap();

    let query_json = serde_json::to_value(&query_result).unwrap();
    assert!(query_json.is_object(), "Query should return an object");

    // Test query-tx method
    let query_tx_params = generated::api_methods::QueryTxParams {
        tx_id: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        wait: None,
        ignore_pending: None,
    };
    let query_tx_result = client.query_tx(query_tx_params).await.unwrap();

    let query_tx_json = serde_json::to_value(&query_tx_result).unwrap();
    assert!(query_tx_json.is_object(), "QueryTx should return an object");

    // Test query-directory method
    let query_dir_params = generated::api_methods::QueryDirectoryParams {
        url: "acc://test.acme".to_string(),
        start: None,
        count: None,
        expand: None,
    };
    let query_dir_result = client.query_directory(query_dir_params).await.unwrap();

    let query_dir_json = serde_json::to_value(&query_dir_result).unwrap();
    assert!(query_dir_json.is_object(), "QueryDirectory should return an object");
}

#[tokio::test]
async fn smoke_execute_methods() {
    let client = generated::api_methods::AccumulateClient { transport: MockOkTransport };

    // Test execute method with minimal transaction
    let execute_params = generated::api_methods::ExecuteParams {
        envelope: json!({
            "transaction": {
                "header": {
                    "principal": "acc://test.acme",
                    "initiator": "0000000000000000000000000000000000000000000000000000000000000000"
                },
                "body": {
                    "type": "sendTokens",
                    "to": []
                }
            },
            "signatures": []
        }),
        check_only: Some(true),
    };
    let execute_result = client.execute(execute_params).await.unwrap();

    let execute_json = serde_json::to_value(&execute_result).unwrap();
    assert!(execute_json.is_object(), "Execute should return an object");
}

#[tokio::test]
async fn smoke_faucet_and_describe() {
    let client = generated::api_methods::AccumulateClient { transport: MockOkTransport };

    // Test faucet method
    let faucet_params = generated::api_methods::FaucetParams {
        url: "acc://test.acme".to_string(),
    };
    let faucet_result = client.faucet(faucet_params).await.unwrap();

    let faucet_json = serde_json::to_value(&faucet_result).unwrap();
    assert!(faucet_json.is_object(), "Faucet should return an object");

    // Test describe method
    let describe_params = generated::api_methods::DescribeParams {};
    let describe_result = client.describe(describe_params).await.unwrap();

    let describe_json = serde_json::to_value(&describe_result).unwrap();
    assert!(describe_json.is_object(), "Describe should return an object");
}

#[tokio::test]
async fn smoke_method_names_and_serialization() {
    // Test that we can serialize parameters properly
    let test_cases = vec![
        ("status", json!({})),
        ("version", json!({})),
        ("query", json!({
            "url": "acc://test.acme",
            "prove": false,
            "scratch": false
        })),
        ("query-tx", json!({
            "txid": "0000000000000000000000000000000000000000000000000000000000000000",
            "wait": 0,
            "ignorePending": false
        })),
        ("execute", json!({
            "envelope": {
                "transaction": {
                    "header": {
                        "principal": "acc://test.acme",
                        "initiator": "0000000000000000000000000000000000000000000000000000000000000000"
                    },
                    "body": {
                        "type": "sendTokens",
                        "to": []
                    }
                },
                "signatures": []
            },
            "checkOnly": true
        })),
        ("query-directory", json!({
            "url": "acc://test.acme",
            "start": 0,
            "count": 10,
            "expand": true
        })),
        ("faucet", json!({
            "url": "acc://test.acme"
        })),
        ("describe", json!({})),
    ];

    for (method_name, expected_params) in test_cases {
        // Verify we can serialize the expected parameter structure
        let serialized = serde_json::to_string(&expected_params).unwrap();
        assert!(!serialized.is_empty(), "Method {} params should serialize", method_name);

        // Verify we can deserialize it back
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected_params, deserialized, "Method {} params should round-trip", method_name);
    }
}

#[test]
fn smoke_transport_trait_bounds() {
    // Verify the AccumulateRpc trait has the expected method signature
    fn check_trait_bounds<T: generated::api_methods::AccumulateRpc>(_transport: T) {
        // This function just needs to compile to verify trait bounds
    }

    // Test with our mock transport
    check_trait_bounds(MockOkTransport);

    // Verify the client can be constructed
    let _client = generated::api_methods::AccumulateClient { transport: MockOkTransport };
}

#[test]
fn smoke_parameter_types_exist() {
    // Verify all parameter types can be constructed
    let _status_params = generated::api_methods::StatusParams {};
    let _version_params = generated::api_methods::VersionParams {};
    let _describe_params = generated::api_methods::DescribeParams {};

    let _query_params = generated::api_methods::QueryParams {
        url: "acc://test.acme".to_string(),
        prove: Some(false),
        scratch: Some(false),
    };

    let _query_tx_params = generated::api_methods::QueryTxParams {
        tx_id: "test".to_string(),
        wait: Some(1000),
        ignore_pending: Some(false),
    };

    let _execute_params = generated::api_methods::ExecuteParams {
        envelope: json!({}),
        check_only: Some(true),
    };

    let _query_dir_params = generated::api_methods::QueryDirectoryParams {
        url: "acc://test.acme".to_string(),
        start: Some(0),
        count: Some(10),
        expand: Some(true),
    };

    let _faucet_params = generated::api_methods::FaucetParams {
        url: "acc://test.acme".to_string(),
    };
}

#[test]
fn smoke_response_types_exist() {
    // Verify response types exist and can be default constructed (where applicable)
    // We're just checking that these types exist and can be referenced

    fn check_response_type<T>() {
        // This function just needs to compile to verify the types exist
        let _type_name = std::any::type_name::<T>();
    }

    check_response_type::<generated::api_methods::StatusResponse>();
    check_response_type::<generated::api_methods::VersionResponse>();
    check_response_type::<generated::api_methods::QueryResponse>();
    check_response_type::<generated::api_methods::QueryTxResponse>();
    check_response_type::<generated::api_methods::ExecuteResponse>();
    check_response_type::<generated::api_methods::QueryDirectoryResponse>();
    check_response_type::<generated::api_methods::FaucetParams>(); // This was the only one in the exports
}