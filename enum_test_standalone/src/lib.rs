// Standalone enum test library
use serde_json;
use std::fs;
use std::path::PathBuf;

// Include the generated enums directly
include!("../../src/generated/enums.rs");

pub fn test_enum_roundtrips() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("src")
        .join("generated")
        .join("enums_manifest.json");

    let content = fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&content)?;
    let enums = manifest["enums"].as_array().ok_or("No enums array")?;

    for e in enums {
        let enum_name = e["name"].as_str().ok_or("No enum name")?;
        let variants = e["variants"].as_array().ok_or("No variants array")?;

        println!("Testing enum: {}", enum_name);

        for v in variants {
            let wire_tag = v.as_str().ok_or("No variant string")?;

            // Test the roundtrip using the generated helper
            __roundtrip_one(enum_name, wire_tag)?;
            println!("  ✓ {} -> '{}'", enum_name, wire_tag);
        }
    }

    Ok(())
}

pub fn test_specific_values() -> Result<(), Box<dyn std::error::Error>> {
    // Test some specific known values from the Go truth

    // ExecutorVersion should have v1, v2, etc.
    let v1: ExecutorVersion = serde_json::from_str("\"v1\"")?;
    assert_eq!(serde_json::to_string(&v1)?, "\"v1\"");

    let v2: ExecutorVersion = serde_json::from_str("\"v2\"")?;
    assert_eq!(serde_json::to_string(&v2)?, "\"v2\"");

    // TransactionType should have specific transaction names
    let write_data: TransactionType = serde_json::from_str("\"writeData\"")?;
    assert_eq!(serde_json::to_string(&write_data)?, "\"writeData\"");

    let create_identity: TransactionType = serde_json::from_str("\"createIdentity\"")?;
    assert_eq!(serde_json::to_string(&create_identity)?, "\"createIdentity\"");

    // SignatureType should have ed25519, rsa, etc.
    let ed25519: SignatureType = serde_json::from_str("\"ed25519\"")?;
    assert_eq!(serde_json::to_string(&ed25519)?, "\"ed25519\"");

    // AccountType should have identity, tokenaccount, etc.
    let identity: AccountType = serde_json::from_str("\"identity\"")?;
    assert_eq!(serde_json::to_string(&identity)?, "\"identity\"");

    println!("✓ All specific enum value tests passed");
    Ok(())
}

pub fn test_enum_count() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("src")
        .join("generated")
        .join("enums_manifest.json");

    let content = fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&content)?;
    let enums = manifest["enums"].as_array().ok_or("No enums array")?;
    let count = enums.len();

    if count != 14 {
        return Err(format!("Expected exactly 14 enums, found {}", count).into());
    }

    println!("✓ Confirmed 14 enums generated");
    Ok(())
}