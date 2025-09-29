# Accumulate Rust SDK (V2/V3 Unified)

A DevNet-first Rust client library for the Accumulate blockchain, providing unified access to both V2 and V3 APIs.

## Features

- **Unified API**: Single client supporting both V2 and V3 protocols
- **DevNet-first**: Optimized for local development with DevNet instances
- **Zero-to-hero**: Complete examples from basic connectivity to transaction submission
- **Fully tested**: Unit tests and integration tests with mocking support
- **Generated core**: Core client methods generated from OpenAPI specifications
- **Type-safe**: Comprehensive Rust types with serde support
- **Async/await**: Modern async Rust with tokio

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
accumulate-client = "0.1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Basic Usage

```rust
use accumulate_client::{Accumulate, AccOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to DevNet
    let client = Accumulate::devnet(AccOptions::default()).await?;

    // Get network status
    let status = client.status().await?;
    println!("Network: {}", status.network);

    Ok(())
}
```

### Network Configuration

```rust
// DevNet (default: localhost:26660/26661)
let client = Accumulate::devnet(options).await?;

// TestNet
let client = Accumulate::testnet(options).await?;

// MainNet
let client = Accumulate::mainnet(options).await?;

// Custom network
let client = Accumulate::custom("http://my-node:8080", options).await?;

// Custom V2/V3 URLs
let client = Accumulate::custom_with_versions(
    "http://node1:26660/v2",
    "http://node2:26661/v3",
    options
).await?;
```

### Transaction Creation and Signing

```rust
use accumulate_client::AccumulateClient;

// Generate keypair
let keypair = AccumulateClient::generate_keypair();

// Create transaction body
let tx_body = client.create_token_transfer(
    "acc://alice",
    "acc://bob",
    100, // amount
    None // use default ACME token
);

// Create signed envelope for V3 API
let envelope = client.create_envelope(&tx_body, &keypair)?;

// Submit to network
let result = client.submit(&envelope).await?;
println!("Transaction hash: {}", result.hash);
```

## Examples

Run the included examples:

```bash
# Basic connectivity test
cargo run --example basic_usage

# Faucet demo (DevNet only)
cargo run --example faucet_demo -- acc://my-test-account

# Transaction creation and signing
cargo run --example transaction_demo -- acc://sender acc://receiver 100
```

Set environment variables:

```bash
# Choose network
export ACCUMULATE_NETWORK=devnet  # devnet, testnet, mainnet, or custom URL

# Run example
cargo run --example basic_usage
```

## API Reference

### Client Creation

- `Accumulate::devnet(options)` - Connect to local DevNet
- `Accumulate::testnet(options)` - Connect to official TestNet
- `Accumulate::mainnet(options)` - Connect to MainNet
- `Accumulate::custom(url, options)` - Connect to custom network

### V2 API Methods

- `client.status()` - Get node status
- `client.query_tx(hash)` - Query transaction by hash
- `client.query_account(url)` - Query account by URL
- `client.faucet(account)` - Request test tokens (DevNet/TestNet)
- `client.submit_v2(tx)` - Submit V2 transaction

### V3 API Methods

- `client.submit(envelope)` - Submit single transaction
- `client.submit_multi(envelopes)` - Submit multiple transactions
- `client.query(url)` - Query using V3 API
- `client.query_block(height)` - Query block by height

### Transaction Helpers

- `client.create_envelope(tx, keypair)` - Create signed transaction envelope
- `client.create_token_transfer(from, to, amount, token)` - Create token transfer
- `client.create_account(url, pubkey, type)` - Create account creation transaction
- `AccumulateClient::generate_keypair()` - Generate new keypair
- `AccumulateClient::keypair_from_seed(seed)` - Create keypair from seed

### Utilities

- `client.get_urls()` - Get V2/V3 API URLs
- `client.validate_account_url(url)` - Validate account URL format
- `canonical_json(value)` - Create deterministic JSON for hashing

## Configuration

### AccOptions

```rust
use std::time::Duration;

let mut headers = std::collections::HashMap::new();
headers.insert("Authorization".to_string(), "Bearer token".to_string());

let options = AccOptions {
    timeout: Duration::from_secs(60),
    headers,
};
```

### Features

- `default = ["rustls-tls"]` - Use rustls for TLS
- `rustls-tls` - Enable rustls TLS backend

## Testing

```bash
# Unit tests
cargo test

# Integration tests (requires running DevNet)
cargo test --features integration

# Test with tracing output
RUST_LOG=debug cargo test
```

## Development

### Running DevNet

Start the local DevNet instance:

```bash
cd /path/to/devnet-accumulate-instance
./start-devnet.sh
```

DevNet URLs:
- V2 API: http://localhost:26660/v2
- V3 API: http://localhost:26661/v3

### Regenerating Code

The core client, types, and JSON-RPC client are generated from templates:

```bash
# Regenerate from OpenAPI spec
cd /path/to/accumulate
./tools/cmd/gen-sdk/gen-sdk.exe ./pkg/api/v3/openapi.yml \
  --lang rust \
  --template-dir "/path/to/opendlt-rust-v2v3-sdk/tooling/templates" \
  --out "/path/to/opendlt-rust-v2v3-sdk/unified/src" \
  --unified --api-version both

# Format generated code
cargo fmt
```

**Important**: The generated files are read-only. To modify behavior, update the templates in `tooling/templates/` and regenerate.

### Templates

Templates are located in `tooling/templates/`:

- `client.rs.tmpl` - Main client implementation
- `json_rpc_client.rs.tmpl` - JSON-RPC client with V2/V3 support
- `types.rs.tmpl` - Serde-compatible type definitions

## Architecture

```
unified/
├── Cargo.toml              # Dependencies and metadata
├── src/
│   ├── lib.rs             # Public facade (handwritten)
│   ├── client.rs          # Generated client implementation
│   ├── json_rpc_client.rs # Generated JSON-RPC client
│   └── types.rs           # Generated type definitions
├── examples/              # Zero-to-hero examples
├── tests/                 # Unit and integration tests
└── tooling/templates/     # Code generation templates
```

## Error Handling

All API methods return `Result<T, JsonRpcError>`:

```rust
use accumulate_client::JsonRpcError;

match client.status().await {
    Ok(status) => println!("Success: {:?}", status),
    Err(JsonRpcError::Http(e)) => eprintln!("HTTP error: {}", e),
    Err(JsonRpcError::Rpc { code, message }) => eprintln!("RPC error {}: {}", code, message),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create your feature branch
3. Make changes to templates (not generated files)
4. Run `cargo fmt` and `cargo clippy`
5. Add tests for new functionality
6. Submit a pull request

Generated code should not be edited directly - modify templates and regenerate instead.