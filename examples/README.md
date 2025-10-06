# Examples

Complete usage examples for the Accumulate Rust SDK. All examples are runnable and demonstrate real-world integration patterns.

## Available Examples

### Setup & Key Management
- `100_keygen_lite_urls.rs` - Generate Ed25519 keys and derive Lite accounts
- `110_testnet_faucet.rs` - Request testnet tokens via faucet
- `120_faucet_local_devnet.rs` - Request tokens from local DevNet

### Identity Management
- `200_create_identity_v3.rs` - Create Accumulate Digital Identifier (ADI)
- `210_buy_credits_lite.rs` - Purchase credits for Lite accounts
- `220_create_adi_v3.rs` - Advanced ADI creation with authorities

### Token Operations
- `230_send_tokens_v3.rs` - Send tokens between accounts
- `240_buy_credits_keypage.rs` - Purchase credits for ADI key pages
- `250_create_token_account.rs` - Create custom token accounts
- `280_send_tokens_lta_to_adi.rs` - Transfer from Lite to ADI accounts

### Data Management
- `260_create_data_account.rs` - Create data storage accounts
- `270_write_data.rs` - Write data to accounts

### Complete Workflows
- `999_zero_to_hero.rs` - Full workflow: keys → identity → tokens → data

## Quick Start

### 1. Environment Setup
```bash
# Optional: Set target network (defaults to DevNet)
export ACC_NET=devnet  # Options: mainnet, testnet, devnet
```

### 2. Start DevNet (for local development)
```bash
# Start local DevNet instance
cd /path/to/devnet-accumulate-instance
./start-devnet.sh
```

DevNet URLs:
- V2 API: http://localhost:26660/v2
- V3 API: http://localhost:26661/v3

### 3. Generate Keys
```bash
cargo run --example 100_keygen_lite_urls
```
Output:
```json
{
  "privateKeyHex": "1234567890abcdef...",
  "publicKeyHex": "abcdef1234567890...",
  "liteIdentity": "acc://a1b2c3d4e5f6.../",
  "liteTokenAccount": "acc://a1b2c3d4e5f6.../ACME"
}
```

### 4. Get DevNet Tokens
```bash
# Use LTA from step 3
cargo run --example 120_faucet_local_devnet
```

### 5. Complete Zero-to-Hero
```bash
cargo run --example 999_zero_to_hero
```

## Environment Variables

Examples support these optional environment variables:

| Variable | Purpose | Default |
|----------|---------|---------|
| `ACC_NET` | Network (mainnet/testnet/devnet) | devnet |
| `ACC_RPC_URL_V2` | V2 API endpoint override | http://localhost:26660/v2 |
| `ACC_RPC_URL_V3` | V3 API endpoint override | http://localhost:26661/v3 |
| `ACC_LTA_URL` | Your Lite Token Account URL | Generated |
| `ACC_FROM_URL` | Transaction source account | Generated |
| `ACC_TO_URL` | Transaction destination account | Generated |
| `ACC_AMOUNT` | Token amount to send | 1000 |

## Network Configuration

### DevNet (Recommended for Development)
- Run locally for fastest iteration
- Full protocol support
- Faucet available for free tokens
- No real value tokens

### TestNet (Integration Testing)
- Stable environment for testing
- Same protocol as mainnet
- Faucet available for free tokens
- External hosted service

### MainNet (Production)
- Real tokens with value
- Use with caution
- Same API as testnet/devnet
- No faucet available

## Key Features Demonstrated

- **Ed25519 Cryptography**: Rust native implementation with ed25519-dalek
- **Protocol Compatibility**: V2 and V3 API support in unified interface
- **Transaction Building**: Type-safe builders for all operations
- **Error Handling**: Comprehensive error types and retry logic
- **Network Integration**: JSON-RPC client with automatic endpoint discovery
- **Async/Await**: Modern async Rust patterns with tokio

## Running Examples

### Individual Examples
```bash
# Run specific example
cargo run --example 100_keygen_lite_urls

# With debug output
RUST_LOG=debug cargo run --example 999_zero_to_hero

# With environment variables
ACC_NET=testnet cargo run --example 110_testnet_faucet
```

### Building All Examples
```bash
# Build all examples
cargo build --examples

# Check examples compile
cargo check --examples
```

### Example Categories

#### **Beginner Examples**
Start with these for basic SDK usage:
- `100_keygen_lite_urls` - Key generation fundamentals
- `120_faucet_local_devnet` - Basic API interaction
- `230_send_tokens_v3` - Simple token transfer

#### **Intermediate Examples**
For identity and account management:
- `200_create_identity_v3` - ADI creation
- `210_buy_credits_lite` - Credit management
- `250_create_token_account` - Custom accounts

#### **Advanced Examples**
Complete workflows and data operations:
- `260_create_data_account` - Data storage
- `270_write_data` - Data manipulation
- `999_zero_to_hero` - End-to-end workflow

## Error Handling Patterns

Examples demonstrate comprehensive error handling:

```rust
use accumulate_client::{AccumulateClient, JsonRpcError};

match client.status().await {
    Ok(status) => println!("✅ Success: {:?}", status),
    Err(JsonRpcError::Http(e)) => eprintln!("❌ HTTP error: {}", e),
    Err(JsonRpcError::Rpc { code, message }) => {
        eprintln!("❌ RPC error {}: {}", code, message)
    },
    Err(e) => eprintln!("❌ Other error: {}", e),
}
```

## DevNet Integration

All examples are designed to work seamlessly with local DevNet:

1. **Automatic Discovery**: Examples detect DevNet endpoints automatically
2. **Faucet Integration**: Built-in faucet support for token requests
3. **Realistic Data**: Examples use realistic account URLs and amounts
4. **Error Recovery**: Graceful handling of DevNet restart scenarios

## Contributing Examples

When adding new examples:

1. Follow the naming pattern: `NNN_description.rs`
2. Include comprehensive error handling
3. Add environment variable support where appropriate
4. Document the example purpose and usage
5. Test with both DevNet and TestNet
6. Update this README with the new example