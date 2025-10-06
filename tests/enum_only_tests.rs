// Standalone enum tests that don't depend on the broken codebase
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Include the generated enums directly
include!("../src/generated/enums.rs");

fn goldens_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join("enums")
}

fn write_if_missing(path: &std::path::Path, body: &str) {
    if std::env::var("INSTA_UPDATE").unwrap_or_default() == "auto" && !path.exists() {
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(path, body).unwrap();
    }
}

fn load_manifest() -> serde_json::Value {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("generated")
        .join("enums_manifest.json");
    let content = fs::read_to_string(&manifest_path)
        .expect("Failed to read enums manifest");
    serde_json::from_str(&content).expect("Failed to parse manifest JSON")
}

#[test]
fn test_enum_json_roundtrips() {
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        let variants = e["variants"].as_array().unwrap();

        println!("Testing enum: {}", enum_name);

        for v in variants {
            let wire_tag = v.as_str().unwrap();
            println!("  Testing variant: {} -> '{}'", enum_name, wire_tag);

            // Test the roundtrip using the generated helper
            match __roundtrip_one(enum_name, wire_tag) {
                Ok(()) => {
                    println!("    ✓ Roundtrip succeeded");
                }
                Err(e) => {
                    panic!("Roundtrip failed for {}::{}: {}", enum_name, wire_tag, e);
                }
            }

            // Write golden vector if enabled
            let golden_file = goldens_dir().join(format!("{}_{}.json", enum_name, wire_tag));
            let golden_content = format!("\"{}\"", wire_tag);
            write_if_missing(&golden_file, &golden_content);

            // If golden file exists, verify it matches
            if golden_file.exists() {
                let existing_content = fs::read_to_string(&golden_file)
                    .expect("Failed to read golden file");
                assert_eq!(
                    existing_content.trim(),
                    golden_content,
                    "Golden vector mismatch for {}::{}",
                    enum_name,
                    wire_tag
                );
            }
        }
    }
}

#[test]
fn test_enum_deserialization_failures() {
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");

    for e in enums {
        let enum_name = e["name"].as_str().unwrap();
        println!("Testing invalid deserialization for: {}", enum_name);

        // Test that invalid tags fail to deserialize
        let invalid_json = "\"invalid_unknown_tag\"";

        let result = match enum_name {
            "ExecutorVersion" => {
                serde_json::from_str::<ExecutorVersion>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "PartitionType" => {
                serde_json::from_str::<PartitionType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "DataEntryType" => {
                serde_json::from_str::<DataEntryType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "ObjectType" => {
                serde_json::from_str::<ObjectType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "SignatureType" => {
                serde_json::from_str::<SignatureType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "KeyPageOperationType" => {
                serde_json::from_str::<KeyPageOperationType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "AccountAuthOperationType" => {
                serde_json::from_str::<AccountAuthOperationType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "NetworkMaintenanceOperationType" => {
                serde_json::from_str::<NetworkMaintenanceOperationType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "TransactionMax" => {
                serde_json::from_str::<TransactionMax>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "TransactionType" => {
                serde_json::from_str::<TransactionType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "AccountType" => {
                serde_json::from_str::<AccountType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "AllowedTransactionBit" => {
                serde_json::from_str::<AllowedTransactionBit>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "VoteType" => {
                serde_json::from_str::<VoteType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            "BookType" => {
                serde_json::from_str::<BookType>(invalid_json)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            _ => panic!("Unknown enum in test: {}", enum_name),
        };

        match result {
            Ok(()) => {
                panic!("Expected deserialization to fail for {}::invalid_unknown_tag", enum_name);
            }
            Err(_) => {
                println!("  ✓ Correctly rejected invalid tag");
            }
        }
    }
}

#[test]
fn test_specific_enum_values() {
    // Test some specific known values from the Go truth

    // ExecutorVersion should have v1, v2, etc.
    let v1: ExecutorVersion = serde_json::from_str("\"v1\"").expect("v1 should deserialize");
    assert_eq!(serde_json::to_string(&v1).unwrap(), "\"v1\"");

    let v2: ExecutorVersion = serde_json::from_str("\"v2\"").expect("v2 should deserialize");
    assert_eq!(serde_json::to_string(&v2).unwrap(), "\"v2\"");

    // TransactionType should have specific transaction names
    let write_data: TransactionType = serde_json::from_str("\"writeData\"").expect("writeData should deserialize");
    assert_eq!(serde_json::to_string(&write_data).unwrap(), "\"writeData\"");

    let create_identity: TransactionType = serde_json::from_str("\"createIdentity\"").expect("createIdentity should deserialize");
    assert_eq!(serde_json::to_string(&create_identity).unwrap(), "\"createIdentity\"");

    // SignatureType should have ed25519, rsa, etc.
    let ed25519: SignatureType = serde_json::from_str("\"ed25519\"").expect("ed25519 should deserialize");
    assert_eq!(serde_json::to_string(&ed25519).unwrap(), "\"ed25519\"");

    // AccountType should have identity, tokenaccount, etc.
    let identity: AccountType = serde_json::from_str("\"identity\"").expect("identity should deserialize");
    assert_eq!(serde_json::to_string(&identity).unwrap(), "\"identity\"");

    println!("✓ All specific enum value tests passed");
}

#[test]
fn test_enum_count() {
    let manifest = load_manifest();
    let enums = manifest["enums"].as_array().expect("enums array");
    let count = enums.len();

    assert_eq!(count, 14, "Expected exactly 14 enums, found {}", count);
    println!("✓ Confirmed 14 enums generated");
}