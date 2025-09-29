//! Example 220: Create ADI using V3 API
//!
//! This example demonstrates:
//! - Creating an Accumulate Digital Identity (ADI)
//! - Using V3 transaction envelope format
//! - Setting up key books and pages

use accumulate_client::{Accumulate, AccOptions, AccumulateClient};
use accumulate_client::protocol::{EnvelopeBuilder, helpers};
use dotenvy::dotenv;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    println!("🏢 Create ADI using V3 API");
    println!("==========================");

    let client = Accumulate::devnet(AccOptions::default()).await?;

    // Generate keypair for ADI
    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.public.to_bytes();
    let public_key_hex = hex::encode(public_key);

    // Create unique ADI URL
    let adi_url = format!("acc://demo-{}.acme", &public_key_hex[0..8]);

    println!("🔑 ADI Information:");
    println!("   Public Key: {}", public_key_hex);
    println!("   ADI URL: {}", adi_url);
    println!();

    // Create ADI transaction body
    println!("📝 Creating ADI transaction...");
    let adi_tx_body = json!({
        "type": "createIdentity",
        "url": adi_url,
        "keyBook": {
            "publicKeyHash": public_key_hex
        },
        "keyPage": {
            "keys": [{
                "publicKeyHash": public_key_hex
            }]
        }
    });

    // Create signed envelope
    println!("✍️  Creating signed envelope...");
    let envelope = EnvelopeBuilder::create_envelope_from_json(
        &adi_url,
        adi_tx_body,
        &keypair,
        &format!("{}/book/1", adi_url),
        1,
    )?;

    println!("   ✅ Envelope created successfully");
    println!("   Signatures: {}", envelope.signatures.len());
    println!("   Transaction hash: {}", envelope.signatures[0].transaction_hash);
    println!("   ⚠️  Would submit to V3 API in full implementation");

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