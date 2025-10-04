use accumulate_client::*;
use async_trait::async_trait;
use serde_json::json;
use std::{fs, path::PathBuf};

#[derive(Clone)]
struct MockErrorTransport {
    error_code: i32,
    error_message: String,
}

impl MockErrorTransport {
    fn new(code: i32, message: &str) -> Self {
        Self {
            error_code: code,
            error_message: message.to_string(),
        }
    }
}

#[async_trait]
impl generated::api_methods::AccumulateRpc for MockErrorTransport {
    async fn rpc_call<TP: serde::Serialize + Send + Sync, TR: for<'de> serde::Deserialize<'de>>(
        &self, _method: &str, _params: &TP
    ) -> Result<TR, errors::Error> {
        // Always return an RPC error for testing
        Err(errors::Error::rpc(self.error_code, self.error_message.clone()))
    }
}

fn write_or_read_golden(path: &PathBuf, golden: &serde_json::Value) -> serde_json::Value {
    if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !path.exists() {
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(path, golden.to_string()).unwrap();
        return golden.clone(); // In write mode, just return the input
    }
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[tokio::test]
async fn api_error_shape_parity() {
    let test_cases = vec![
        (404, "account not found"),
        (400, "invalid request"),
        (500, "internal server error"),
        (403, "forbidden"),
        (401, "unauthorized"),
    ];

    for (code, message) in test_cases {
        let client = generated::api_methods::AccumulateClient {
            transport: MockErrorTransport::new(code, message)
        };

        // Test with a simple query call (using minimal params)
        let params = generated::api_methods::QueryParams {
            url: "acc://test.acme".to_string(),
            options: Some(json!({"prove": false})),
        };

        let err = client.query(params).await.unwrap_err();

        // Extract error details
        let (actual_code, actual_message) = match &err {
            errors::Error::Rpc { code, message } => (*code, message.clone()),
            _ => panic!("Expected RPC error, got: {:?}", err),
        };

        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden_vectors/api")
            .join(format!("error_{}.json", code));

        let golden = json!({
            "input": {
                "code": code,
                "message": message
            },
            "error": {
                "code": actual_code,
                "message": actual_message,
                "type": "rpc"
            },
            "formatted": format!("{}", err)
        });

        let expected = write_or_read_golden(&p, &golden);
        assert_eq!(golden, expected, "API error shape mismatch for code {}", code);

        // Verify the error components match
        assert_eq!(actual_code, code, "Error code mismatch");
        assert_eq!(actual_message, message, "Error message mismatch");
    }
}

#[tokio::test]
async fn api_error_serialization_golden() {
    // Test error serialization and display formats
    let errors = vec![
        errors::Error::rpc(404, "not found".to_string()),
        errors::Error::rpc(500, "server error".to_string()),
        errors::Error::General("general error".to_string()),
        errors::Error::Network("network timeout".to_string()),
        errors::Error::Encoding("encoding failed".to_string()),
    ];

    let mut results = Vec::new();

    for err in errors {
        let error_info = match &err {
            errors::Error::Rpc { code, message } => json!({
                "type": "rpc",
                "code": code,
                "message": message
            }),
            errors::Error::General(msg) => json!({
                "type": "general",
                "message": msg
            }),
            errors::Error::Network(msg) => json!({
                "type": "network",
                "message": msg
            }),
            errors::Error::Encoding(msg) => json!({
                "type": "encoding",
                "message": msg
            }),
            _ => json!({
                "type": "other",
                "message": format!("{}", err)
            }),
        };

        results.push(json!({
            "error_info": error_info,
            "display_format": format!("{}", err),
            "debug_format": format!("{:?}", err)
        }));
    }

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_vectors/api/error_serialization.json");

    let golden = json!({
        "error_formats": results,
        "description": "Testing error type serialization and display formats"
    });

    let expected = write_or_read_golden(&p, &golden);
    assert_eq!(golden, expected, "Error serialization golden mismatch");
}

#[test]
fn error_construction_golden() {
    // Test different ways of constructing errors
    let constructed_errors = vec![
        ("rpc_method", errors::Error::rpc(404, "resource not found".to_string())),
        ("from_string", errors::Error::from("string error".to_string())),
        ("from_str", errors::Error::from("str error")),
        ("general", errors::Error::General("direct general".to_string())),
        ("network", errors::Error::Network("network issue".to_string())),
        ("encoding", errors::Error::Encoding("encoding problem".to_string())),
    ];

    let mut results = Vec::new();

    for (name, error) in constructed_errors {
        let serialized = match &error {
            errors::Error::Rpc { code, message } => json!({
                "type": "rpc",
                "code": code,
                "message": message
            }),
            errors::Error::General(msg) => json!({
                "type": "general",
                "message": msg
            }),
            errors::Error::Network(msg) => json!({
                "type": "network",
                "message": msg
            }),
            errors::Error::Encoding(msg) => json!({
                "type": "encoding",
                "message": msg
            }),
            _ => json!({
                "type": "unknown",
                "display": format!("{}", error)
            }),
        };

        results.push(json!({
            "construction_method": name,
            "error": serialized,
            "display": format!("{}", error)
        }));
    }

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_vectors/api/error_construction.json");

    let golden = json!({
        "construction_tests": results,
        "description": "Testing different error construction methods"
    });

    let expected = write_or_read_golden(&p, &golden);
    assert_eq!(golden, expected, "Error construction golden mismatch");
}

#[test]
fn signature_error_golden() {
    // Test signature-specific error types
    let sig_errors = vec![
        ("invalid_format", errors::SignatureError::InvalidFormat),
        ("verification_failed", errors::SignatureError::VerificationFailed("bad signature".to_string())),
        ("unsupported_type", errors::SignatureError::UnsupportedType("unknown".to_string())),
        ("invalid_public_key", errors::SignatureError::InvalidPublicKey),
        ("invalid_signature", errors::SignatureError::InvalidSignature),
        ("crypto_error", errors::SignatureError::Crypto("crypto failure".to_string())),
    ];

    let mut results = Vec::new();

    for (name, sig_error) in sig_errors {
        // Convert to main error type
        let main_error = errors::Error::Signature(sig_error.clone());

        let serialized = json!({
            "signature_error_type": name,
            "signature_error_display": format!("{}", sig_error),
            "main_error_display": format!("{}", main_error),
            "signature_error_debug": format!("{:?}", sig_error)
        });

        results.push(serialized);
    }

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_vectors/api/signature_errors.json");

    let golden = json!({
        "signature_error_tests": results,
        "description": "Testing signature-specific error types"
    });

    let expected = write_or_read_golden(&p, &golden);
    assert_eq!(golden, expected, "Signature error golden mismatch");
}