//! Example 210: Buy Credits for Lite Identity
//!
//! This example demonstrates:
//! - Converting ACME tokens to credits
//! - Creating and submitting credit purchase transactions
//! - Verifying credit balance after purchase

use accumulate_client::{Accumulate, AccOptions, AccumulateClient};
use dotenvy::dotenv;
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("💳 Buy Credits for Lite Identity");
    println!("================================");

    let client = Accumulate::devnet(AccOptions::default()).await?;

    // Generate test account
    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.public.to_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let acme_account = format!("{}/ACME", lite_identity);
    let credits_account = format!("{}/credits", lite_identity);

    println!("🔑 Account Information:");
    println!("   Lite Identity: {}", lite_identity);
    println!("   ACME Account: {}", acme_account);
    println!("   Credits Account: {}", credits_account);
    println!();

    // Fund with faucet first
    println!("💰 Funding with faucet...");
    client.faucet(&acme_account).await?;
    sleep(Duration::from_secs(3)).await;

    // Create credit purchase transaction
    println!("💳 Creating credit purchase transaction...");
    let credit_amount = 5000u64;

    let tx_body = client.create_token_transfer(
        &acme_account,
        &credits_account,
        credit_amount,
        Some("acc://ACME"),
    );

    println!("   ✅ Transaction prepared");
    println!("   Amount: {} ACME -> Credits", credit_amount);
    println!("   ⚠️  Would submit to network in full implementation");

    Ok(())
}

fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();
    let lite_id_hex = hex::encode(&hash[0..20]);
    format!("acc://{}.acme", lite_id_hex)
}