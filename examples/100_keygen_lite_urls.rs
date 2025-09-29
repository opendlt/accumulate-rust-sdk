//! Example 100: Key Generation and Lite Identity URLs
//!
//! This example demonstrates:
//! - Generating Ed25519 keypairs
//! - Deriving lite identity URLs from keys
//! - Creating account URLs from lite identities
//! - Displaying key information in various formats

use accumulate_client::{Accumulate, AccOptions, AccumulateClient};
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    println!("🔑 Accumulate Key Generation and Lite URLs");
    println!("==========================================");

    // Display environment configuration
    let devnet_dir = env::var("ACC_DEVNET_DIR").unwrap_or_else(|_| "Not set".to_string());
    let rpc_v2 = env::var("ACC_RPC_URL_V2").unwrap_or_else(|_| "http://localhost:26660/v2".to_string());
    let rpc_v3 = env::var("ACC_RPC_URL_V3").unwrap_or_else(|_| "http://localhost:26660/v3".to_string());

    println!("📍 DevNet Directory: {}", devnet_dir);
    println!("🌐 V2 Endpoint: {}", rpc_v2);
    println!("🌐 V3 Endpoint: {}", rpc_v3);
    println!();

    // Step 1: Generate a new keypair
    println!("🔑 Step 1: Generating Ed25519 Keypair");
    let keypair = AccumulateClient::generate_keypair();

    let public_key_bytes = keypair.public.to_bytes();
    let private_key_bytes = keypair.to_bytes();

    println!("   Public Key:  {}", hex::encode(public_key_bytes));
    println!("   Private Key: {}", hex::encode(private_key_bytes));
    println!();

    // Step 2: Derive lite identity URL from public key
    println!("🏷️  Step 2: Deriving Lite Identity URL");
    let lite_identity_url = derive_lite_identity_url(&public_key_bytes);
    println!("   Lite Identity: {}", lite_identity_url);
    println!();

    // Step 3: Create account URLs from lite identity
    println!("🏦 Step 3: Creating Account URLs");
    let accounts = create_account_urls(&lite_identity_url);

    for (account_type, url) in &accounts {
        println!("   {}: {}", account_type, url);
    }
    println!();

    // Step 4: Generate deterministic keypair from seed
    println!("🌱 Step 4: Deterministic Key Generation");
    let seed_phrase = "test seed phrase for deterministic key generation";
    let deterministic_keypair = generate_deterministic_keypair(seed_phrase)?;

    let det_public_key = deterministic_keypair.public.to_bytes();
    let det_lite_identity = derive_lite_identity_url(&det_public_key);

    println!("   Seed Phrase: {}", seed_phrase);
    println!("   Public Key:  {}", hex::encode(det_public_key));
    println!("   Lite Identity: {}", det_lite_identity);
    println!();

    // Step 5: Validate account URLs
    println!("✅ Step 5: Validating Account URLs");
    for (account_type, url) in &accounts {
        let is_valid = AccumulateClient::validate_account_url(url);
        println!("   {} - Valid: {}", account_type, if is_valid { "✅" } else { "❌" });
    }
    println!();

    // Step 6: Generate multiple keypairs for comparison
    println!("🔄 Step 6: Multiple Key Generation");
    for i in 1..=3 {
        let kp = AccumulateClient::generate_keypair();
        let pub_key = kp.public.to_bytes();
        let lite_id = derive_lite_identity_url(&pub_key);

        println!("   Keypair {}: {}", i, hex::encode(pub_key));
        println!("   Identity {}: {}", i, lite_id);
    }
    println!();

    println!("✅ Key generation and URL derivation completed successfully!");
    println!("💡 Next: Run example 120 to test faucet funding");

    Ok(())
}

/// Derive a lite identity URL from a public key
fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};

    // Hash the public key
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();

    // Use first 20 bytes for lite identity
    let lite_id_bytes = &hash[0..20];
    let lite_id_hex = hex::encode(lite_id_bytes);

    format!("acc://{}.acme", lite_id_hex)
}

/// Create various account URLs from a lite identity
fn create_account_urls(lite_identity: &str) -> Vec<(String, String)> {
    vec![
        ("ACME Tokens".to_string(), format!("{}/ACME", lite_identity)),
        ("Credits".to_string(), format!("{}/credits", lite_identity)),
        ("Book".to_string(), format!("{}/book", lite_identity)),
        ("Book Page 1".to_string(), format!("{}/book/1", lite_identity)),
        ("Data Account".to_string(), format!("{}/data", lite_identity)),
        ("Staking".to_string(), format!("{}/staking", lite_identity)),
    ]
}

/// Generate a deterministic keypair from a seed phrase
fn generate_deterministic_keypair(seed_phrase: &str) -> Result<ed25519_dalek::Keypair, Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};

    // Hash the seed phrase to get 32 bytes
    let mut hasher = Sha256::new();
    hasher.update(seed_phrase.as_bytes());
    let seed_hash = hasher.finalize();

    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(&seed_hash[0..32]);

    // Create keypair from seed
    AccumulateClient::keypair_from_seed(&seed_bytes)
        .map_err(|e| format!("Failed to create keypair: {}", e).into())
}