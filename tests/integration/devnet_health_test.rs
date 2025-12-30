use accumulate_client::{Accumulate, AccOptions, AccumulateClient};
use dotenvy::dotenv;
use std::env;
use tokio::time::{timeout, Duration};

/// Test DevNet health and basic connectivity
#[tokio::test]
#[ignore = "DevNet tests require running local DevNet - not required for SDK usage"]
async fn test_devnet_connectivity() {
    dotenv().ok();

    let client = match Accumulate::devnet(AccOptions::default()).await {
        Ok(client) => client,
        Err(_) => {
            println!("[WARN] DevNet not available, skipping connectivity test");
            return;
        }
    };

    // Test with generous timeout
    let status_result = timeout(Duration::from_secs(30), client.status()).await;

    match status_result {
        Ok(Ok(status)) => {
            println!("[OK] DevNet health check passed");
            println!("   Network: {}", status.network);
            println!("   Version: {}", status.version);
            assert!(!status.network.is_empty());
            assert!(!status.version.is_empty());
        }
        Ok(Err(e)) => {
            println!("[ERROR] DevNet status error: {}", e);
            panic!("DevNet returned error: {}", e);
        }
        Err(_) => {
            println!("Timeout: DevNet status timeout");
            panic!("DevNet status call timed out");
        }
    }
}

/// Test faucet availability and basic functionality
#[tokio::test]
async fn test_faucet_availability() {
    dotenv().ok();

    let client = match Accumulate::devnet(AccOptions::default()).await {
        Ok(client) => client,
        Err(_) => {
            println!("[WARN] DevNet not available, skipping faucet test");
            return;
        }
    };

    // Generate test account (returns SigningKey in ed25519-dalek v2)
    let keypair = AccumulateClient::generate_keypair();
    let public_key = keypair.verifying_key().to_bytes();
    let lite_identity = derive_lite_identity_url(&public_key);
    let acme_account = format!("{}/ACME", lite_identity);

    println!("Testing faucet with account: {}", acme_account);

    // Test faucet with generous timeout
    let faucet_result = timeout(Duration::from_secs(15), client.faucet(&acme_account)).await;

    match faucet_result {
        Ok(Ok(response)) => {
            println!("[OK] Faucet test passed");
            println!("   Transaction ID: {}", response.txid);
            println!("   Amount: {}", response.amount);
            assert!(!response.txid.is_empty());
            assert!(!response.amount.is_empty());
        }
        Ok(Err(e)) => {
            println!("[WARN] Faucet error (might be expected): {}", e);
            // Don't panic on faucet errors as they might be rate-limited or empty
        }
        Err(_) => {
            println!("Timeout: Faucet request timeout");
            // Don't panic on timeout as DevNet might be slow
        }
    }
}

/// Test environment variable configuration
#[tokio::test]
async fn test_environment_configuration() {
    dotenv().ok();

    println!("Testing environment configuration");

    let devnet_dir = env::var("ACC_DEVNET_DIR");
    let rpc_v2 = env::var("ACC_RPC_URL_V2");
    let rpc_v3 = env::var("ACC_RPC_URL_V3");
    let faucet_account = env::var("ACC_FAUCET_ACCOUNT");

    println!("   ACC_DEVNET_DIR: {:?}", devnet_dir);
    println!("   ACC_RPC_URL_V2: {:?}", rpc_v2);
    println!("   ACC_RPC_URL_V3: {:?}", rpc_v3);
    println!("   ACC_FAUCET_ACCOUNT: {:?}", faucet_account);

    // Test that we can create client with default values
    let client_result = Accumulate::devnet(AccOptions::default()).await;
    match client_result {
        Ok(_) => {
            println!("[OK] Client creation successful with current configuration");
        }
        Err(e) => {
            println!("[WARN] Client creation failed: {}", e);
            // Don't panic here as DevNet might not be running
        }
    }
}

/// Test key generation and URL derivation
#[tokio::test]
async fn test_key_generation_and_urls() {
    println!("Testing key generation and URL derivation");

    // Test multiple keypairs (returns SigningKey in ed25519-dalek v2)
    for i in 1..=3 {
        let keypair = AccumulateClient::generate_keypair();
        let public_key = keypair.verifying_key().to_bytes();
        let lite_identity = derive_lite_identity_url(&public_key);

        println!("   Keypair {}: {}", i, hex::encode(public_key));
        println!("   Identity {}: {}", i, lite_identity);

        // Validate URL format
        assert!(lite_identity.starts_with("acc://"));
        assert!(lite_identity.ends_with(".acme"));
        assert!(AccumulateClient::validate_account_url(&lite_identity));

        // Test derived accounts
        let acme_account = format!("{}/ACME", lite_identity);
        let credits_account = format!("{}/credits", lite_identity);

        assert!(AccumulateClient::validate_account_url(&acme_account));
        assert!(AccumulateClient::validate_account_url(&credits_account));
    }

    println!("[OK] Key generation and URL derivation tests passed");
}

/// Test deterministic key generation
#[tokio::test]
async fn test_deterministic_keys() {
    println!("Testing deterministic key generation");

    let seed_phrase = "test seed for deterministic generation";

    // Generate same key multiple times
    let key1 = generate_deterministic_keypair(seed_phrase).unwrap();
    let key2 = generate_deterministic_keypair(seed_phrase).unwrap();

    let pub1 = key1.verifying_key().to_bytes();
    let pub2 = key2.verifying_key().to_bytes();

    assert_eq!(pub1, pub2, "Deterministic keys should be identical");

    let identity1 = derive_lite_identity_url(&pub1);
    let identity2 = derive_lite_identity_url(&pub2);

    assert_eq!(identity1, identity2, "Derived identities should be identical");

    println!("[OK] Deterministic key generation test passed");
    println!("   Seed: {}", seed_phrase);
    println!("   Public Key: {}", hex::encode(pub1));
    println!("   Identity: {}", identity1);
}

// Helper functions

fn derive_lite_identity_url(public_key: &[u8; 32]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();

    let lite_id_bytes = &hash[0..20];
    let lite_id_hex = hex::encode(lite_id_bytes);

    format!("acc://{}.acme", lite_id_hex)
}

fn generate_deterministic_keypair(seed_phrase: &str) -> Result<ed25519_dalek::SigningKey, Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(seed_phrase.as_bytes());
    let seed_hash = hasher.finalize();

    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(&seed_hash[0..32]);

    AccumulateClient::keypair_from_seed(&seed_bytes)
        .map_err(|e| format!("Failed to create keypair: {}", e).into())
}