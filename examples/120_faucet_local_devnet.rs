//! Example 120: Faucet Funding on Local DevNet
//!
//! This example demonstrates:
//! - Connecting to local DevNet
//! - Requesting tokens from the faucet
//! - Checking account balances
//! - Querying transaction status

use accumulate_client::{Accumulate, AccOptions, AccumulateClient};
use dotenvy::dotenv;
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    println!("💰 Accumulate Faucet Funding Example");
    println!("====================================");

    // Get configuration from environment
    let rpc_v2 = env::var("ACC_RPC_URL_V2").unwrap_or_else(|_| "http://localhost:26660/v2".to_string());
    let rpc_v3 = env::var("ACC_RPC_URL_V3").unwrap_or_else(|_| "http://localhost:26660/v3".to_string());
    let faucet_account = env::var("ACC_FAUCET_ACCOUNT").unwrap_or_else(|_| "acc://faucet.acme/ACME".to_string());

    println!("🌐 V2 Endpoint: {}", rpc_v2);
    println!("🌐 V3 Endpoint: {}", rpc_v3);
    println!("💧 Faucet Account: {}", faucet_account);
    println!();

    // Step 1: Connect to DevNet
    println!("🔗 Step 1: Connecting to DevNet");
    let client = Accumulate::devnet(AccOptions::default()).await?;
    println!("   ✅ Connected to DevNet");

    // Test connectivity
    match client.status().await {
        Ok(status) => {
            println!("   ✅ DevNet Status: {} ({})", status.network, status.version);
        }
        Err(e) => {
            eprintln!("   ❌ Failed to get DevNet status: {}", e);
            eprintln!("   💡 Make sure DevNet is running: cd devnet && docker-compose up -d");
            return Err(e.into());
        }
    }
    println!();

    // Step 2: Generate a test account
    println!("🔑 Step 2: Generating Test Account");
    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.public.to_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let acme_account = format!("{}/ACME", lite_identity);

    println!("   Public Key: {}", hex::encode(public_key));
    println!("   Lite Identity: {}", lite_identity);
    println!("   ACME Account: {}", acme_account);
    println!();

    // Step 3: Check initial balance (should be empty/non-existent)
    println!("💰 Step 3: Checking Initial Balance");
    match client.query_account(&acme_account).await {
        Ok(account) => {
            println!("   Account exists: {}", account.url);
            if let Some(credits) = account.credits {
                println!("   Credits: {}", credits);
            }
        }
        Err(_) => {
            println!("   Account does not exist yet (expected)");
        }
    }
    println!();

    // Step 4: Request tokens from faucet
    println!("💧 Step 4: Requesting Tokens from Faucet");
    println!("   Requesting funding for: {}", acme_account);

    match client.faucet(&acme_account).await {
        Ok(faucet_response) => {
            println!("   ✅ Faucet request successful!");
            println!("   Transaction ID: {}", faucet_response.txid);
            println!("   Amount: {}", faucet_response.amount);
            if !faucet_response.link.is_empty() {
                println!("   Link: {}", faucet_response.link);
            }
        }
        Err(e) => {
            eprintln!("   ❌ Faucet request failed: {}", e);
            eprintln!("   💡 Common issues:");
            eprintln!("      - DevNet not running");
            eprintln!("      - Faucet disabled or empty");
            eprintln!("      - Network connectivity issues");
            return Err(e.into());
        }
    }
    println!();

    // Step 5: Wait for transaction processing
    println!("⏳ Step 5: Waiting for Transaction Processing");
    println!("   Waiting 5 seconds for transaction to be processed...");
    sleep(Duration::from_secs(5)).await;

    // Step 6: Check balance after funding
    println!("💰 Step 6: Checking Balance After Funding");
    match client.query_account(&acme_account).await {
        Ok(account) => {
            println!("   ✅ Account now exists: {}", account.url);
            println!("   Account type: {}", account.account_type);
            if let Some(credits) = account.credits {
                println!("   Credits: {}", credits);
            }
            if let Some(nonce) = account.nonce {
                println!("   Nonce: {}", nonce);
            }
            println!("   Raw data: {}", serde_json::to_string_pretty(&account.data)?);
        }
        Err(e) => {
            println!("   ⚠️  Could not query account: {}", e);
            println!("   This might indicate the transaction is still processing");
        }
    }
    println!();

    // Step 7: Test multiple account types
    println!("🔄 Step 7: Testing Multiple Account Types");
    let account_types = vec![
        ("Lite Identity", lite_identity.clone()),
        ("ACME Tokens", acme_account.clone()),
        ("Credits", format!("{}/credits", lite_identity)),
        ("Book", format!("{}/book", lite_identity)),
    ];

    for (account_type, account_url) in account_types {
        println!("   Checking {}: {}", account_type, account_url);
        match client.query_account(&account_url).await {
            Ok(account) => {
                println!("     ✅ Exists (type: {})", account.account_type);
            }
            Err(_) => {
                println!("     ❌ Does not exist");
            }
        }
    }
    println!();

    // Step 8: Fund multiple accounts for testing
    println!("🎯 Step 8: Funding Additional Test Accounts");
    for i in 1..=2 {
        let test_keypair = AccumulateClient::generate_keypair();
        let test_public = test_keypair.public.to_bytes();
        let test_identity = derive_lite_identity_url(&test_public);
        let test_acme_account = format!("{}/ACME", test_identity);

        println!("   Test Account {}: {}", i, test_acme_account);

        match client.faucet(&test_acme_account).await {
            Ok(response) => {
                println!("     ✅ Funded! TX: {}", response.txid);
            }
            Err(e) => {
                println!("     ❌ Failed: {}", e);
            }
        }
    }
    println!();

    println!("✅ Faucet funding completed successfully!");
    println!("💡 Next steps:");
    println!("   - Run example 210 to buy credits");
    println!("   - Run example 999 for the complete zero-to-hero flow");

    Ok(())
}

/// Derive a lite identity URL from a public key (same as in example 100)
fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();

    let lite_id_bytes = &hash[0..20];
    let lite_id_hex = hex::encode(lite_id_bytes);

    format!("acc://{}.acme", lite_id_hex)
}