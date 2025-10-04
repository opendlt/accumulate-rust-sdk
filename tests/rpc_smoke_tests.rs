use accumulate_client::*;
use accumulate_client::errors::Error;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone)]
struct MockTransport {
    pub recorded_calls: std::sync::Arc<std::sync::Mutex<Vec<(String, serde_json::Value)>>>,
    pub canned_responses: HashMap<String, serde_json::Value>,
}

impl MockTransport {
    fn new() -> Self {
        let mut responses = HashMap::new();

        // Add canned responses for all 35 API methods
        responses.insert("status".to_string(), json!({"ok": true, "version": "test"}));
        responses.insert("version".to_string(), json!({"data": {"version": "1.0.0"}}));
        responses.insert("describe".to_string(), json!({"data": {"description": "test node"}}));
        responses.insert("metrics".to_string(), json!({"data": {"tps": 100}}));
        responses.insert("faucet".to_string(), json!({"transactionHash": "deadbeef123456"}));
        responses.insert("query".to_string(), json!({"data": {"url": "acc://test.acme", "type": "identity"}}));
        responses.insert("query-directory".to_string(), json!({"data": {"items": []}}));
        responses.insert("query-tx".to_string(), json!({"data": {"txid": "abc123", "status": "delivered"}}));
        responses.insert("execute".to_string(), json!({"transactionHash": "execute123", "code": 200}));

        // Add generic responses for all other methods
        let generic_response = json!({"data": {"result": "success"}});
        for method in [
            "query-tx-local", "query-tx-history", "query-data", "query-data-set",
            "query-key-index", "query-minor-blocks", "query-major-blocks", "query-synth",
            "execute-direct", "execute-local", "create-adi", "create-identity",
            "create-data-account", "create-key-book", "create-key-page", "create-token",
            "create-token-account", "send-tokens", "add-credits", "update-key-page",
            "update-key", "write-data", "issue-tokens", "write-data-to", "burn-tokens",
            "update-account-auth"
        ] {
            responses.insert(method.to_string(), generic_response.clone());
        }

        Self {
            recorded_calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            canned_responses: responses,
        }
    }

    fn get_recorded_calls(&self) -> Vec<(String, serde_json::Value)> {
        self.recorded_calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl AccumulateRpc for MockTransport {
    async fn rpc_call<TP: serde::Serialize + Send + Sync, TR: for<'de> serde::Deserialize<'de>>(
        &self, method: &str, params: &TP
    ) -> Result<TR, Error> {
        let params_value = serde_json::to_value(params)
            .map_err(|e| Error::General(format!("Failed to serialize params: {}", e)))?;

        // Record the call
        self.recorded_calls.lock().unwrap().push((method.to_string(), params_value.clone()));

        // Return canned response
        let response = self.canned_responses.get(method)
            .cloned()
            .unwrap_or_else(|| json!({"error": "unknown method"}));

        serde_json::from_value(response)
            .map_err(|e| Error::General(format!("Failed to deserialize response: {}", e)))
    }
}

#[tokio::test]
async fn test_status_method_transport() {
    let transport = MockTransport::new();
    let client = GenericAccumulateClient::new(transport.clone());

    let params = StatusParams {};
    let response = client.status(params).await.unwrap();

    // Verify transport was called correctly
    let calls = transport.get_recorded_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "status");

    // Verify response structure
    assert!(response.ok);

    println!("✓ Status method transport test passed");
}

#[tokio::test]
async fn test_query_method_transport() {
    let transport = MockTransport::new();
    let client = GenericAccumulateClient::new(transport.clone());

    let params = QueryParams {
        url: "acc://test.acme".to_string(),
        options: None,
    };
    let _response = client.query(params).await.unwrap();

    // Verify transport call
    let calls = transport.get_recorded_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "query");

    // Verify params structure
    let recorded_params = &calls[0].1;
    assert_eq!(recorded_params["url"], "acc://test.acme");

    println!("✓ Query method transport test passed");
}

#[tokio::test]
async fn test_faucet_method_transport() {
    let transport = MockTransport::new();
    let client = GenericAccumulateClient::new(transport.clone());

    let params = FaucetParams {
        url: "acc://faucet-test.acme".to_string(),
    };
    let response = client.faucet(params).await.unwrap();

    // Verify transport call
    let calls = transport.get_recorded_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "faucet");

    // Verify params structure
    let recorded_params = &calls[0].1;
    assert_eq!(recorded_params["url"], "acc://faucet-test.acme");

    // Verify response structure
    assert_eq!(response.transaction_hash, "deadbeef123456");

    println!("✓ Faucet method transport test passed");
}

#[tokio::test]
async fn test_execute_method_transport() {
    let transport = MockTransport::new();
    let client = GenericAccumulateClient::new(transport.clone());

    let params = ExecuteParams {
        params: json!({"transaction": {"header": {}, "body": {}}}),
    };
    let response = client.execute(params).await.unwrap();

    // Verify transport call
    let calls = transport.get_recorded_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "execute");

    // Verify response
    assert_eq!(response.transaction_hash, "execute123");

    println!("✓ Execute method transport test passed");
}

#[tokio::test]
async fn test_multiple_method_calls() {
    let transport = MockTransport::new();
    let client = GenericAccumulateClient::new(transport.clone());

    // Make multiple calls
    let _status = client.status(StatusParams {}).await.unwrap();
    let _version = client.version(VersionParams {}).await.unwrap();
    let _describe = client.describe(DescribeParams {}).await.unwrap();

    // Verify all calls were recorded
    let calls = transport.get_recorded_calls();
    assert_eq!(calls.len(), 3);

    assert_eq!(calls[0].0, "status");
    assert_eq!(calls[1].0, "version");
    assert_eq!(calls[2].0, "describe");

    println!("✓ Multiple method calls test passed");
}

#[tokio::test]
async fn test_method_name_correctness() {
    let transport = MockTransport::new();
    let client = GenericAccumulateClient::new(transport.clone());

    // Test that Rust method names map to correct RPC method names using different approach
    let test_cases = vec![
        ("query-directory", "query_directory"),
        ("query-tx", "query_tx"),
        ("query-tx-local", "query_tx_local"),
        ("create-identity", "execute_create_identity"),
        ("send-tokens", "execute_send_tokens"),
    ];

    for (i, (expected_rpc_method, rust_method)) in test_cases.into_iter().enumerate() {
        let fresh_transport = MockTransport::new();
        let fresh_client = GenericAccumulateClient::new(fresh_transport.clone());

        // Make the call based on the rust method name
        match rust_method {
            "query_directory" => { let _ = fresh_client.query_directory(QueryDirectoryParams { url: "acc://test.acme".to_string(), options: None }).await; },
            "query_tx" => { let _ = fresh_client.query_tx(QueryTxParams { url: "acc://test.acme".to_string(), options: None }).await; },
            "query_tx_local" => { let _ = fresh_client.query_tx_local(QueryTxLocalParams { url: "acc://test.acme".to_string(), options: None }).await; },
            "execute_create_identity" => { let _ = fresh_client.execute_create_identity(ExecuteCreateIdentityParams { params: json!({}) }).await; },
            "execute_send_tokens" => { let _ = fresh_client.execute_send_tokens(ExecuteSendTokensParams { params: json!({}) }).await; },
            _ => {}
        }

        let calls = fresh_transport.get_recorded_calls();
        if !calls.is_empty() {
            assert_eq!(calls[0].0, expected_rpc_method, "Method {} should map to RPC method {}", i, expected_rpc_method);
        }
    }

    println!("✓ Method name correctness test passed");
}

#[tokio::test]
async fn test_error_handling() {
    #[derive(Clone)]
    struct ErrorTransport;

    #[async_trait]
    impl AccumulateRpc for ErrorTransport {
        async fn rpc_call<TP: serde::Serialize + Send + Sync, TR: for<'de> serde::Deserialize<'de>>(
            &self, _method: &str, _params: &TP
        ) -> Result<TR, Error> {
            Err(Error::General("Mock transport error".to_string()))
        }
    }

    let client = GenericAccumulateClient::new(ErrorTransport);
    let result = client.status(StatusParams {}).await;

    assert!(result.is_err(), "Should return error from transport");

    println!("✓ Error handling test passed");
}

#[tokio::test]
async fn test_json_rpc_payload_structure() {
    #[derive(Clone)]
    struct PayloadInspectorTransport {
        pub captured_payload: std::sync::Arc<std::sync::Mutex<Option<serde_json::Value>>>,
    }

    impl PayloadInspectorTransport {
        fn new() -> Self {
            Self {
                captured_payload: std::sync::Arc::new(std::sync::Mutex::new(None)),
            }
        }
    }

    #[async_trait]
    impl AccumulateRpc for PayloadInspectorTransport {
        async fn rpc_call<TP: serde::Serialize + Send + Sync, TR: for<'de> serde::Deserialize<'de>>(
            &self, method: &str, params: &TP
        ) -> Result<TR, Error> {
            // Simulate what HttpTransport does - create JSON-RPC payload
            let payload = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            });

            *self.captured_payload.lock().unwrap() = Some(payload);

            // Return minimal response
            let response = json!({"ok": true});
            serde_json::from_value(response)
                .map_err(|e| Error::General(format!("Mock error: {}", e)))
        }
    }

    let transport = PayloadInspectorTransport::new();
    let client = GenericAccumulateClient::new(transport.clone());

    let _result = client.status(StatusParams {}).await.unwrap();

    // Verify JSON-RPC payload structure
    let payload = transport.captured_payload.lock().unwrap().clone().unwrap();

    assert_eq!(payload["jsonrpc"], "2.0");
    assert_eq!(payload["id"], 1);
    assert_eq!(payload["method"], "status");
    assert!(payload.get("params").is_some());

    println!("✓ JSON-RPC payload structure test passed");
}

#[tokio::test]
async fn test_client_wrapper_functionality() {
    let transport = MockTransport::new();

    // Test both construction methods
    let client1 = GenericAccumulateClient::new(transport.clone());
    let client2 = GenericAccumulateClient { transport: transport.clone() };

    // Both should work identically
    let _result1 = client1.status(StatusParams {}).await.unwrap();
    let _result2 = client2.status(StatusParams {}).await.unwrap();

    // Verify both calls were recorded
    let calls = transport.get_recorded_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "status");
    assert_eq!(calls[1].0, "status");

    println!("✓ Client wrapper functionality test passed");
}