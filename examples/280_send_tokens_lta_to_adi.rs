//! Example 280: Send Tokens from Lite Token Account to ADI
//!
//! This example demonstrates:
//! - Token transfers between different account types
//! - Lite Token Account (LTA) to ADI transfers
//! - Transaction signing and submission patterns

use accumulate_client::{Accumulate, AccOptions, AccumulateClient};
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("💸 Send Tokens: Lite Token Account to ADI");
    println!("==========================================");

    let client = Accumulate::devnet(AccOptions::default()).await?;

    // Generate source account (Lite)
    let source_keypair = AccumulateClient::generate_keypair();
    let source_public_key = source_keypair.public.to_bytes();
    let source_lite_identity = derive_lite_identity_url(&source_public_key);
    let source_acme_account = format!("{}/ACME", source_lite_identity);

    // Generate destination ADI
    let dest_keypair = AccumulateClient::generate_keypair();
    let dest_public_key = dest_keypair.public.to_bytes();
    let dest_public_key_hex = hex::encode(dest_public_key);
    let dest_adi_url = format!("acc://recipient-{}.acme", &dest_public_key_hex[0..8]);
    let dest_token_account = format!("{}/tokens", dest_adi_url);

    println!("📋 Transfer Information:");
    println!("   Source (LTA): {}", source_acme_account);
    println!("   Destination ADI: {}", dest_adi_url);
    println!("   Destination Tokens: {}", dest_token_account);
    println!();

    // Fund source account
    println!("💰 Funding source account...");
    match client.faucet(&source_acme_account).await {
        Ok(response) => {
            println!("   ✅ Faucet successful: {}", response.txid);
        }
        Err(e) => {
            println!("   ❌ Faucet failed: {}", e);
            return Err(e.into());
        }
    }
    println!();

    // Create token transfer transaction
    println!("💸 Creating token transfer...");
    let transfer_amount = 1500u64;

    let transfer_tx = client.create_token_transfer(
        &source_acme_account,
        &dest_token_account,
        transfer_amount,
        Some("acc://ACME"),
    );

    println!("   ✅ Transfer transaction prepared");
    println!("   From: {}", source_acme_account);
    println!("   To: {}", dest_token_account);
    println!("   Amount: {} ACME", transfer_amount);
    println!("   Token URL: acc://ACME");
    println!();

    println!("🔍 Transaction Details:");
    println!("{}", serde_json::to_string_pretty(&transfer_tx)?);
    println!();

    println!("✅ Token transfer prepared successfully!");
    println!("⚠️  In a full implementation, this would be signed and submitted to the network");

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