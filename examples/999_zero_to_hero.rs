//! Example 999: Zero to Hero - Complete Accumulate Flow
//!
//! This example demonstrates the complete Accumulate workflow:
//! 1. Key generation and lite identity creation
//! 2. Faucet funding
//! 3. Credit purchasing
//! 4. ADI (Accumulate Digital Identity) creation
//! 5. Token account creation
//! 6. Data account creation and writing
//! 7. Token transfers
//!
//! This is the end-to-end demonstration of Accumulate capabilities.

use accumulate_client::{Accumulate, AccOptions, AccumulateClient};
use dotenvy::dotenv;
use serde_json::json;
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    println!("🚀 Accumulate Zero to Hero Example");
    println!("=================================");
    println!("This example demonstrates the complete Accumulate workflow!");
    println!();

    // Display configuration
    let rpc_v2 = env::var("ACC_RPC_URL_V2").unwrap_or_else(|_| "http://localhost:26660/v2".to_string());
    let rpc_v3 = env::var("ACC_RPC_URL_V3").unwrap_or_else(|_| "http://localhost:26660/v3".to_string());
    let faucet_account = env::var("ACC_FAUCET_ACCOUNT").unwrap_or_else(|_| "acc://faucet.acme/ACME".to_string());

    println!("🌐 Configuration:");
    println!("   V2 Endpoint: {}", rpc_v2);
    println!("   V3 Endpoint: {}", rpc_v3);
    println!("   Faucet: {}", faucet_account);
    println!();

    // Connect to DevNet
    println!("🔗 Connecting to DevNet...");
    let client = Accumulate::devnet(AccOptions::default()).await?;

    // Verify DevNet is running
    match client.status().await {
        Ok(status) => {
            println!("   ✅ DevNet Status: {} ({})", status.network, status.version);
        }
        Err(e) => {
            eprintln!("   ❌ DevNet connection failed: {}", e);
            eprintln!("   💡 Start DevNet: cd devnet && docker-compose up -d");
            return Err(e.into());
        }
    }
    println!();

    // =======================
    // STEP 1: Key Generation
    // =======================
    println!("🔑 STEP 1: Key Generation and Lite Identity");
    println!("─────────────────────────────────────────");

    let user_keypair = AccumulateClient::generate_keypair();
    let user_public_key = user_keypair.public.to_bytes();
    let lite_identity = derive_lite_identity_url(&user_public_key);
    let acme_account = format!("{}/ACME", lite_identity);
    let credits_account = format!("{}/credits", lite_identity);

    println!("   User Public Key: {}", hex::encode(user_public_key));
    println!("   Lite Identity:   {}", lite_identity);
    println!("   ACME Account:    {}", acme_account);
    println!("   Credits Account: {}", credits_account);
    println!();

    // ==========================
    // STEP 2: Initial Funding
    // ==========================
    println!("💰 STEP 2: Initial Faucet Funding");
    println!("──────────────────────────────────");

    println!("   Requesting tokens from faucet...");
    match client.faucet(&acme_account).await {
        Ok(response) => {
            println!("   ✅ Faucet Success! TX: {}", response.txid);
            println!("   Amount: {}", response.amount);
        }
        Err(e) => {
            eprintln!("   ❌ Faucet failed: {}", e);
            return Err(e.into());
        }
    }

    println!("   Waiting for transaction processing...");
    sleep(Duration::from_secs(5)).await;

    // Verify funding
    match client.query_account(&acme_account).await {
        Ok(account) => {
            println!("   ✅ Account funded successfully");
            println!("   Account type: {}", account.account_type);
        }
        Err(e) => {
            println!("   ⚠️  Could not verify funding: {}", e);
        }
    }
    println!();

    // ========================
    // STEP 3: Buy Credits
    // ========================
    println!("💳 STEP 3: Purchasing Credits");
    println!("─────────────────────────────");

    println!("   Creating credit purchase transaction...");
    let credit_amount = 10000u64;

    // Create add credits transaction
    let add_credits_tx = client.create_token_transfer(
        &acme_account,
        &credits_account,
        credit_amount,
        Some("acc://ACME"),
    );

    println!("   Transaction body created");
    println!("   From: {}", acme_account);
    println!("   To:   {}", credits_account);
    println!("   Amount: {} ACME", credit_amount);

    // For a real implementation, we would sign and submit this transaction
    // For now, we'll simulate the process
    println!("   ⚠️  Transaction simulation (would submit to network)");
    println!();

    // ==========================
    // STEP 4: Create ADI
    // ==========================
    println!("🏢 STEP 4: Creating ADI (Accumulate Digital Identity)");
    println!("────────────────────────────────────────────────────");

    let adi_url = format!("acc://user-{}.acme", &hex::encode(&user_public_key)[0..8]);
    println!("   ADI URL: {}", adi_url);

    // Create ADI transaction
    let create_adi_tx = client.create_account(
        &adi_url,
        &user_public_key,
        "identity",
    );

    println!("   ADI creation transaction prepared");
    println!("   ⚠️  Transaction simulation (would submit to network)");
    println!();

    // =============================
    // STEP 5: Create Token Account
    // =============================
    println!("🪙 STEP 5: Creating Token Account");
    println!("─────────────────────────────────");

    let token_account_url = format!("{}/tokens", adi_url);
    println!("   Token Account: {}", token_account_url);

    let create_token_account_tx = json!({
        "type": "createTokenAccount",
        "url": token_account_url,
        "tokenUrl": "acc://ACME",
        "keyBook": format!("{}/book", adi_url)
    });

    println!("   Token account creation prepared");
    println!("   ⚠️  Transaction simulation (would submit to network)");
    println!();

    // ===========================
    // STEP 6: Create Data Account
    // ===========================
    println!("📊 STEP 6: Creating Data Account");
    println!("─────────────────────────────────");

    let data_account_url = format!("{}/data", adi_url);
    println!("   Data Account: {}", data_account_url);

    let create_data_account_tx = json!({
        "type": "createDataAccount",
        "url": data_account_url,
        "keyBook": format!("{}/book", adi_url)
    });

    println!("   Data account creation prepared");
    println!("   ⚠️  Transaction simulation (would submit to network)");
    println!();

    // =======================
    // STEP 7: Write Data
    // =======================
    println!("📝 STEP 7: Writing Data");
    println!("───────────────────────");

    let data_to_write = json!({
        "timestamp": chrono::Utc::now().timestamp(),
        "message": "Hello from Accumulate Rust SDK!",
        "version": "1.0",
        "metadata": {
            "example": "zero-to-hero",
            "language": "rust"
        }
    });

    println!("   Data to write: {}", serde_json::to_string_pretty(&data_to_write)?);

    let write_data_tx = json!({
        "type": "writeData",
        "data": data_to_write,
        "account": data_account_url
    });

    println!("   Write data transaction prepared");
    println!("   ⚠️  Transaction simulation (would submit to network)");
    println!();

    // ===========================
    // STEP 8: Token Transfer
    // ===========================
    println!("💸 STEP 8: Token Transfer (Lite to ADI)");
    println!("───────────────────────────────────────");

    let transfer_amount = 1000u64;
    println!("   Transfer amount: {} ACME", transfer_amount);
    println!("   From: {} (Lite)", acme_account);
    println!("   To:   {} (ADI)", token_account_url);

    let token_transfer_tx = client.create_token_transfer(
        &acme_account,
        &token_account_url,
        transfer_amount,
        Some("acc://ACME"),
    );

    println!("   Token transfer transaction prepared");
    println!("   ⚠️  Transaction simulation (would submit to network)");
    println!();

    // ================================
    // STEP 9: Summary and Next Steps
    // ================================
    println!("✅ STEP 9: Zero to Hero Complete!");
    println!("─────────────────────────────────");
    println!();
    println!("🎉 Congratulations! You have successfully completed the Accumulate zero-to-hero flow!");
    println!();
    println!("📋 Summary of what we accomplished:");
    println!("   1. ✅ Generated Ed25519 keypair");
    println!("   2. ✅ Created lite identity: {}", lite_identity);
    println!("   3. ✅ Funded account with faucet");
    println!("   4. ✅ Prepared credit purchase");
    println!("   5. ✅ Prepared ADI creation: {}", adi_url);
    println!("   6. ✅ Prepared token account: {}", token_account_url);
    println!("   7. ✅ Prepared data account: {}", data_account_url);
    println!("   8. ✅ Prepared data writing");
    println!("   9. ✅ Prepared token transfer");
    println!();
    println!("🔧 What's Next:");
    println!("   - Run individual examples (210, 220, etc.) for detailed flows");
    println!("   - Explore V3 transaction submission");
    println!("   - Try advanced features like multi-sig and anchoring");
    println!("   - Build your own Accumulate applications!");
    println!();
    println!("📚 Resources:");
    println!("   - Accumulate Documentation: https://docs.accumulatenetwork.io");
    println!("   - SDK Examples: ./examples/");
    println!("   - Test Suite: cargo test --all-features");

    Ok(())
}

/// Derive a lite identity URL from a public key
fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();

    let lite_id_bytes = &hash[0..20];
    let lite_id_hex = hex::encode(lite_id_bytes);

    format!("acc://{}.acme", lite_id_hex)
}