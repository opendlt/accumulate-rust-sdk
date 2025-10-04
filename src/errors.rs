// Error types for the Accumulate Rust SDK

use thiserror::Error;

/// Main error type for the Accumulate SDK
#[derive(Error, Debug)]
pub enum Error {
    #[error("Signature error: {0}")]
    Signature(#[from] SignatureError),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("General error: {0}")]
    General(String),
}

/// Signature-specific errors
#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("Invalid signature format")]
    InvalidFormat,

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Unsupported signature type: {0}")]
    UnsupportedType(String),

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid signature bytes")]
    InvalidSignature,

    #[error("Cryptographic error: {0}")]
    Crypto(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::General(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::General(s.to_string())
    }
}