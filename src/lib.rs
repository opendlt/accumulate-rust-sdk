//! Accumulate Rust SDK (V2/V3 unified) with DevNet-first flows
//!
//! A Rust client library for the Accumulate blockchain JSON-RPC API with support for both V2 and V3 protocols.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use accumulate_client::{Accumulate, AccOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to DevNet
//!     let client = Accumulate::devnet(AccOptions::default()).await?;
//!
//!     // Or connect to a custom network
//!     let client = Accumulate::custom("http://localhost:26660", AccOptions::default()).await?;
//!
//!     let status = client.status().await?;
//!     println!("Node status: {:?}", status);
//!
//!     Ok(())
//! }
//! ```

use std::time::Duration;
use url::Url;

pub mod client;
pub mod json_rpc_client;
pub mod types;
pub mod codec;
pub mod protocol;

pub use client::AccumulateClient;
pub use json_rpc_client::{JsonRpcClient, JsonRpcError};
pub use types::*;

/// Configuration options for Accumulate client
#[derive(Debug, Clone)]
pub struct AccOptions {
    /// Request timeout duration
    pub timeout: Duration,
    /// Additional HTTP headers
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

/// Main entry point for Accumulate SDK
pub struct Accumulate;

impl Accumulate {
    /// Connect to DevNet (localhost:26660 for V2, localhost:26661 for V3)
    pub async fn devnet(options: AccOptions) -> Result<AccumulateClient, Box<dyn std::error::Error>> {
        let v2_url = "http://localhost:26660/v2";
        let v3_url = "http://localhost:26661/v3";
        Self::custom_with_versions(v2_url, v3_url, options).await
    }

    /// Connect to testnet
    pub async fn testnet(options: AccOptions) -> Result<AccumulateClient, Box<dyn std::error::Error>> {
        let v2_url = "https://testnet.accumulatenetwork.io/v2";
        let v3_url = "https://testnet.accumulatenetwork.io/v3";
        Self::custom_with_versions(v2_url, v3_url, options).await
    }

    /// Connect to mainnet
    pub async fn mainnet(options: AccOptions) -> Result<AccumulateClient, Box<dyn std::error::Error>> {
        let v2_url = "https://mainnet.accumulatenetwork.io/v2";
        let v3_url = "https://mainnet.accumulatenetwork.io/v3";
        Self::custom_with_versions(v2_url, v3_url, options).await
    }

    /// Connect to a custom network with base URL (will append /v2 and /v3)
    pub async fn custom(base_url: &str, options: AccOptions) -> Result<AccumulateClient, Box<dyn std::error::Error>> {
        let base = base_url.trim_end_matches('/');
        let v2_url = format!("{}/v2", base);
        let v3_url = format!("{}/v3", base);
        Self::custom_with_versions(&v2_url, &v3_url, options).await
    }

    /// Connect to a custom network with explicit V2 and V3 URLs
    pub async fn custom_with_versions(
        v2_url: &str,
        v3_url: &str,
        options: AccOptions,
    ) -> Result<AccumulateClient, Box<dyn std::error::Error>> {
        let v2_parsed = Url::parse(v2_url)?;
        let v3_parsed = Url::parse(v3_url)?;

        AccumulateClient::new_with_options(v2_parsed, v3_parsed, options).await
    }
}