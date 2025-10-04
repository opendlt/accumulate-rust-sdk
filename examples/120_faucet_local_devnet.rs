//! # Faucet Local DevNet Example
//!
//! This example demonstrates:
//! - Loading DevNet configuration from .env.local
//! - Connecting to local DevNet
//! - Requesting tokens from the faucet
//! - Checking account balances
//! - Querying transaction status
//! - Working with lite token accounts
//!
//! ## Prerequisites
//! 1. Run: `cargo run --bin devnet_discovery` to generate .env.local
//! 2. Ensure DevNet is running: `cd devnet && docker compose up -d`
//!
//! ## Usage
//! ```bash
//! cargo run --example 120_faucet_local_devnet
//! ```

use accumulate_client::{AccOptions, AccumulateClient};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💰 Accumulate Faucet Local DevNet Example");
    println!("==========================================");

    // Step 1: Load environment configuration
    println!("\n📋 Loading DevNet configuration...");
    dotenvy::dotenv().ok();

    let v2_url = std::env::var("ACC_RPC_URL_V2")
        .unwrap_or_else(|_| "http://localhost:26660/v2".to_string());
    let v3_url = std::env::var("ACC_RPC_URL_V3")
        .unwrap_or_else(|_| "http://localhost:26660/v3".to_string());
    let faucet_account = std::env::var("ACC_FAUCET_ACCOUNT")
        .unwrap_or_else(|_| "acc://faucet.acme/ACME".to_string());

    println!("  V2 API: {}", v2_url);
    println!("  V3 API: {}", v3_url);
    println!("  Faucet: {}", faucet_account);

    // Step 2: Connect to DevNet
    println!("\n🔗 Connecting to DevNet...");
    let client = create_client(&v2_url, &v3_url).await?;

    // Test connectivity
    match test_devnet_status(&client).await {
        Ok(_) => println!("  ✅ DevNet is accessible"),
        Err(e) => {
            println!("  ❌ DevNet connectivity issue: {}", e);
            println!("  💡 Make sure DevNet is running: cd devnet && docker compose up -d");
            return Err(e.into());
        }
    }

    // Step 3: Generate a test account
    println!("\n🔑 Generating test account...");
    let test_account = generate_test_account().await?;

    // Step 4: Request tokens from faucet
    println!("\n💧 Requesting tokens from faucet...");
    request_faucet_tokens(&client, &test_account, &faucet_account).await?;

    // Step 5: Verify funding and test queries
    println!("\n🔍 Verifying funding and testing queries...");
    verify_funding(&client, &test_account).await?;

    println!("\n✅ Faucet funding example completed successfully!");
    println!("💡 Next: Run `cargo run --example 210_buy_credits_lite`");

    Ok(())
}

async fn create_client(v2_url: &str, v3_url: &str) -> Result<AccumulateClient, Box<dyn std::error::Error>> {
    use url::Url;

    let v2_parsed = Url::parse(v2_url)?;
    let v3_parsed = Url::parse(v3_url)?;

    let options = AccOptions {
        timeout: Duration::from_secs(15),
        headers: std::collections::HashMap::new(),
    };

    AccumulateClient::from_endpoints(v2_parsed, v3_parsed, options).await
        .map_err(|e| e.into())
}

async fn test_devnet_status(client: &AccumulateClient) -> Result<(), Box<dyn std::error::Error>> {
    match client.status().await {
        Ok(status) => {
            println!("  DevNet Status: {} ({})", status.network, status.version);
            Ok(())
        }
        Err(e) => {
            Err(format!("Failed to get DevNet status: {}", e).into())
        }
    }
}

#[derive(Debug)]
struct TestAccount {
    public_key: [u8; 32],
    lite_identity: String,
    acme_account: String,
}

async fn generate_test_account() -> Result<TestAccount, Box<dyn std::error::Error>> {
    use accumulate_client::crypto::ed25519::Ed25519Signer;

    let signer = Ed25519Signer::generate();
    let public_key = signer.public_key_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let acme_account = format!("{}/ACME", lite_identity);

    println!("  Public key:    {}", hex::encode(&public_key));
    println!("  Lite identity: {}", lite_identity);
    println!("  ACME account:  {}", acme_account);

    Ok(TestAccount {
        public_key,
        lite_identity,
        acme_account,
    })
}

async fn request_faucet_tokens(
    client: &AccumulateClient,
    test_account: &TestAccount,
    faucet_account: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Target account: {}", test_account.acme_account);
    println!("  Faucet source:  {}", faucet_account);

    // Check initial balance
    print!("  Checking initial balance... ");
    match client.query_account(&test_account.acme_account).await {
        Ok(_) => println!("Account already exists"),
        Err(_) => println!("Account does not exist (expected)"),
    }

    // Request tokens from faucet
    print!("  Requesting faucet tokens... ");
    match client.faucet(&test_account.acme_account).await {
        Ok(response) => {
            println!("✅ Success!");
            println!("    Transaction ID: {}", response.txid);
            if !response.amount.is_empty() {
                println!("    Amount: {}", response.amount);
            }

            // Wait for transaction processing
            println!("  ⏳ Waiting 3 seconds for processing...");
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            println!("    💡 Common issues:");
            println!("       - DevNet not fully started");
            println!("       - Faucet account empty or disabled");
            println!("       - Network connectivity issues");
            return Err(e.into());
        }
    }

    Ok(())
}

async fn verify_funding(
    client: &AccumulateClient,
    test_account: &TestAccount,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if account was created and funded
    print!("  Checking funded account... ");
    match client.query_account(&test_account.acme_account).await {
        Ok(account) => {
            println!("✅ Account exists!");
            println!("    URL: {}", account.url);
            println!("    Type: {}", account.account_type);

            if let Some(credits) = account.credits {
                println!("    Credits: {}", credits);
            }

            if let Some(nonce) = account.nonce {
                println!("    Nonce: {}", nonce);
            }

            // Show raw account data
            println!("    Data: {}", serde_json::to_string_pretty(&account.data)?);
        }
        Err(e) => {
            println!("⚠️ Account not found: {}", e);
            println!("    This might indicate:");
            println!("    - Transaction still processing");
            println!("    - Faucet request failed");
            println!("    - DevNet synchronization issues");
        }
    }

    // Test querying the lite identity root
    print!("  Checking lite identity... ");
    match client.query_account(&test_account.lite_identity).await {
        Ok(account) => {
            println!("✅ Identity exists (type: {})", account.account_type);
        }
        Err(_) => {
            println!("❌ Identity not found");
        }
    }

    Ok(())
}

// Helper function for lite identity derivation (same as example 100)
fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use accumulate_client::crypto::ed25519::sha256;

    let hash = sha256(public_key);
    let lite_id_bytes = &hash[0..20];
    let lite_id_hex = hex::encode(lite_id_bytes);

    format!("acc://{}.acme", lite_id_hex)
}
