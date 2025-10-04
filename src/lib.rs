//! Accumulate Rust SDK (V2/V3 unified) with DevNet-first flows
//!
//! This crate provides a unified client for interacting with Accumulate blockchain
//! networks, supporting both V2 and V3 protocol versions.

pub use crate::client::AccumulateClient;
pub use crate::codec::{
    TransactionCodec, TransactionEnvelope, TransactionSignature,
    TransactionBodyBuilder, TokenRecipient, KeySpec, BinaryReader, BinaryWriter,
    AccumulateHash, UrlHash, canonical_json, sha256_bytes, to_canonical_string
};
pub use crate::canonjson::{dumps_canonical, canonicalize};
// pub use crate::crypto::{Ed25519Signer, verify, verify_prehashed, verify_signature, verify_signature_prehashed}; // Broken - commented out
pub use crate::generated::enums::*;
pub use crate::generated::signatures::*;
pub use crate::generated::header::*;
pub use crate::generated::transactions::*;
pub use crate::generated::api_methods::*;
pub use crate::runtime::signing::*;
pub use crate::runtime::rpc::*;
#[cfg(test)]
pub use crate::runtime::signing_test_shims;

pub mod canonjson;
pub mod client;
pub mod codec;
pub mod crypto;
pub mod errors;
pub mod generated;
pub mod json_rpc_client;
pub mod runtime;
pub mod types;
pub mod types_matrix;

use anyhow::Result;
use std::time::Duration;
use url::Url;

/// Configuration options for the Accumulate client
#[derive(Debug, Clone)]
pub struct AccOptions {
    /// Request timeout duration
    pub timeout: Duration,
    /// Default headers to include with requests
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for AccOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            headers: std::collections::HashMap::new(),
        }
    }
}

impl AccumulateClient {
    /// Create a new client from explicit V2 and V3 endpoints
    pub async fn from_endpoints(v2: Url, v3: Url, opts: AccOptions) -> Result<Self> {
        Self::new_with_options(v2, v3, opts)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Create a new client from environment variables
    ///
    /// Reads the following environment variables:
    /// - `ACCUMULATE_V2_URL`: V2 endpoint URL
    /// - `ACCUMULATE_V3_URL`: V3 endpoint URL
    /// - `ACCUMULATE_TIMEOUT_MS`: Request timeout in milliseconds (optional, defaults to 30000)
    pub async fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present, ignore errors

        let v2_url = std::env::var("ACCUMULATE_V2_URL")
            .map_err(|_| anyhow::anyhow!("ACCUMULATE_V2_URL environment variable not set"))?;
        let v3_url = std::env::var("ACCUMULATE_V3_URL")
            .map_err(|_| anyhow::anyhow!("ACCUMULATE_V3_URL environment variable not set"))?;

        let v2 = Url::parse(&v2_url)?;
        let v3 = Url::parse(&v3_url)?;

        let timeout_ms = std::env::var("ACCUMULATE_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30_000);

        let opts = AccOptions {
            timeout: Duration::from_millis(timeout_ms),
            ..Default::default()
        };

        Self::from_endpoints(v2, v3, opts).await
    }
}
