use accumulate_client::types_matrix::{
    SampleGenerator, RoundtripTestable, TYPE_NAMES, get_type_name, verify_type_coverage
};
use accumulate_client::codec::{
    TransactionEnvelope, TransactionHeader, TransactionSignature, TransactionKeyPage,
    TokenRecipient, KeySpec, TransactionCodec, BinaryWriter, BinaryReader
};
use accumulate_client::types::{
    StatusResponse, NodeInfo, TransactionResponse, TransactionResult, Event, Attribute,
    SignedTransaction, Signature, Account, FaucetResponse, V3SubmitRequest, V3SubmitResponse,
    SubmitResult, V3Signature
};
use serde_json::Value;
use std::collections::HashMap;

/// Comprehensive roundtrip testing for all protocol types
#[test]
fn test_all_types_roundtrip() {
    println!("Testing roundtrip encoding/decoding for {} protocol types...", TYPE_NAMES.len());

    let mut tested_types = Vec::new();
    let mut failed_types = Vec::new();
    let mut total_samples = 0;

    // Test TransactionEnvelope
    test_type_roundtrip::<TransactionEnvelope>("TransactionEnvelope", &mut tested_types, &mut failed_types, &mut total_samples);

    // Test TransactionHeader
    test_type_roundtrip::<TransactionHeader>("TransactionHeader", &mut tested_types, &mut failed_types, &mut total_samples);

    // Test TransactionSignature
    test_type_roundtrip::<TransactionSignature>("TransactionSignature", &mut tested_types, &mut failed_types, &mut total_samples);

    // Test TransactionKeyPage
    test_type_roundtrip::<TransactionKeyPage>("TransactionKeyPage", &mut tested_types, &mut failed_types, &mut total_samples);

    // Test TokenRecipient
    test_type_roundtrip::<TokenRecipient>("TokenRecipient", &mut tested_types, &mut failed_types, &mut total_samples);

    // Test KeySpec
    test_type_roundtrip::<KeySpec>("KeySpec", &mut tested_types, &mut failed_types, &mut total_samples);

    // Add manual tests for types that don't have SampleGenerator implementations yet
    test_manual_types(&mut tested_types, &mut failed_types, &mut total_samples);

    // Summary
    println!("\nROUNDTRIP TEST SUMMARY");
    println!("========================");
    println!("Total samples tested: {}", total_samples);
    println!("Types tested: {}", tested_types.len());
    println!("Types in TYPE_NAMES: {}", TYPE_NAMES.len());

    if !failed_types.is_empty() {
        println!("\n[FAIL] FAILED TYPES:");
        for failed_type in &failed_types {
            println!("  - {}", failed_type);
        }
        panic!("Roundtrip tests failed for {} types", failed_types.len());
    }

    println!("\n[OK] All types passed roundtrip tests!");

    // Verify that we tested types from our TYPE_NAMES list
    verify_type_coverage_implementation(&tested_types);
}

fn test_type_roundtrip<T>(
    type_name: &str,
    tested_types: &mut Vec<String>,
    failed_types: &mut Vec<String>,
    total_samples: &mut usize,
) where
    T: SampleGenerator + RoundtripTestable + std::fmt::Debug,
{
    println!("Testing {} roundtrips...", type_name);

    let samples = T::generate_samples();
    *total_samples += samples.len();

    let mut sample_failures = Vec::new();

    for (i, sample) in samples.iter().enumerate() {
        match sample.test_json_roundtrip() {
            Ok(()) => {
                println!("  ✓ Sample {} passed JSON roundtrip", i);
            }
            Err(e) => {
                println!("  [ERROR] Sample {} failed JSON roundtrip: {}", i, e);
                sample_failures.push(format!("Sample {}: {}", i, e));
            }
        }

        // Test binary roundtrip if supported
        match sample.test_binary_roundtrip() {
            Ok(()) => {
                // Only print success for types that actually implement binary roundtrip
            }
            Err(e) if !e.is_empty() => {
                println!("  [ERROR] Sample {} failed binary roundtrip: {}", i, e);
                sample_failures.push(format!("Sample {} binary: {}", i, e));
            }
            _ => {} // Ignore empty errors (default implementation)
        }
    }

    if sample_failures.is_empty() {
        println!("  [OK] {} passed all roundtrip tests ({} samples)", type_name, samples.len());
        tested_types.push(type_name.to_string());
    } else {
        println!("  [ERROR] {} failed roundtrip tests:", type_name);
        for failure in &sample_failures {
            println!("    {}", failure);
        }
        failed_types.push(type_name.to_string());
    }
}

fn test_manual_types(
    tested_types: &mut Vec<String>,
    failed_types: &mut Vec<String>,
    total_samples: &mut usize,
) {
    // Test types that don't have SampleGenerator implementations yet

    // StatusResponse
    let status_response = StatusResponse {
        network: "accumulate-testnet".to_string(),
        version: "1.0.0".to_string(),
        commit: "abc123".to_string(),
        node_info: NodeInfo {
            id: "node-123".to_string(),
            listen_addr: "tcp://0.0.0.0:26656".to_string(),
            network: "accumulate-testnet".to_string(),
            version: "1.0.0".to_string(),
            channels: "40202122233038606100".to_string(),
            moniker: "test-node".to_string(),
            other: HashMap::new(),
        },
    };
    test_single_sample("StatusResponse", &status_response, tested_types, failed_types, total_samples);

    // TransactionResponse
    let tx_response = TransactionResponse {
        txid: "abcdef123456".to_string(),
        hash: "fedcba654321".to_string(),
        height: 12345,
        index: 0,
        tx: serde_json::json!({"type": "test"}),
        tx_result: TransactionResult {
            code: 0,
            data: Some("success".to_string()),
            log: "transaction executed successfully".to_string(),
            info: "".to_string(),
            gas_wanted: "1000".to_string(),
            gas_used: "750".to_string(),
            events: vec![Event {
                event_type: "transfer".to_string(),
                attributes: vec![Attribute {
                    key: "amount".to_string(),
                    value: "1000".to_string(),
                    index: true,
                }],
            }],
            codespace: "".to_string(),
        },
    };
    test_single_sample("TransactionResponse", &tx_response, tested_types, failed_types, total_samples);

    // Account
    let account = Account {
        url: "acc://alice.acme/tokens".to_string(),
        account_type: "token-account".to_string(),
        data: serde_json::json!({"balance": "1000"}),
        credits: Some(500),
        nonce: Some(42),
    };
    test_single_sample("Account", &account, tested_types, failed_types, total_samples);

    // FaucetResponse
    let faucet_response = FaucetResponse {
        txid: "faucet-tx-123".to_string(),
        link: "https://explorer.accumulate.io/tx/faucet-tx-123".to_string(),
        account: "acc://alice.acme/tokens".to_string(),
        amount: "10000".to_string(),
    };
    test_single_sample("FaucetResponse", &faucet_response, tested_types, failed_types, total_samples);

    // V3Signature
    let v3_signature = V3Signature {
        public_key: vec![0x42; 32],
        signature: vec![0x24; 64],
        timestamp: 1234567890,
        vote: Some("approve".to_string()),
    };
    test_single_sample("V3Signature", &v3_signature, tested_types, failed_types, total_samples);
}

fn test_single_sample<T>(
    type_name: &str,
    sample: &T,
    tested_types: &mut Vec<String>,
    failed_types: &mut Vec<String>,
    total_samples: &mut usize,
) where
    T: RoundtripTestable + std::fmt::Debug,
{
    println!("Testing {} roundtrip...", type_name);
    *total_samples += 1;

    match sample.test_json_roundtrip() {
        Ok(()) => {
            println!("  [OK] {} passed JSON roundtrip test", type_name);
            tested_types.push(type_name.to_string());
        }
        Err(e) => {
            println!("  [ERROR] {} failed JSON roundtrip test: {}", type_name, e);
            failed_types.push(type_name.to_string());
        }
    }
}

fn verify_type_coverage_implementation(tested_types: &[String]) {
    let missing_types: Vec<_> = TYPE_NAMES
        .iter()
        .filter(|type_name| !tested_types.contains(&type_name.to_string()))
        .collect();

    if !missing_types.is_empty() {
        println!("\n[WARN] WARNING: Some types from TYPE_NAMES are not tested:");
        for missing_type in &missing_types {
            println!("  - {}", missing_type);
        }
        println!("Consider implementing SampleGenerator for these types or adding manual tests.");
    }

    // Also check for tested types not in TYPE_NAMES
    let extra_types: Vec<_> = tested_types
        .iter()
        .filter(|tested_type| !TYPE_NAMES.contains(&tested_type.as_str()))
        .collect();

    if !extra_types.is_empty() {
        println!("\nTip: Some tested types are not in TYPE_NAMES:");
        for extra_type in &extra_types {
            println!("  - {}", extra_type);
        }
        println!("Consider adding these to TYPE_NAMES if they are protocol types.");
    }
}

#[test]
fn test_transaction_codec_binary_roundtrip() {
    println!("Testing TransactionCodec binary roundtrip...");

    let envelope = TransactionEnvelope::generate_sample();

    // Test binary encoding roundtrip
    match TransactionCodec::encode_envelope(&envelope) {
        Ok(encoded) => {
            println!("  ✓ Envelope encoded to {} bytes", encoded.len());

            match TransactionCodec::decode_envelope(&encoded) {
                Ok(decoded) => {
                    println!("  ✓ Envelope decoded successfully");

                    // Re-encode to verify consistency
                    match TransactionCodec::encode_envelope(&decoded) {
                        Ok(re_encoded) => {
                            if encoded == re_encoded {
                                println!("  [OK] Binary roundtrip successful - bytes match exactly");
                            } else {
                                panic!(
                                    "Binary roundtrip failed - re-encoded bytes differ\nOriginal: {} bytes\nRe-encoded: {} bytes",
                                    encoded.len(),
                                    re_encoded.len()
                                );
                            }
                        }
                        Err(e) => {
                            panic!("Failed to re-encode envelope: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    panic!("Failed to decode envelope: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to encode envelope: {:?}", e);
        }
    }
}

#[test]
fn test_transaction_codec_header_roundtrip() {
    println!("Testing TransactionHeader binary roundtrip...");

    let headers = TransactionHeader::generate_samples();

    for (i, header) in headers.iter().enumerate() {
        println!("  Testing header sample {}...", i);

        // Test binary encoding roundtrip
        match TransactionCodec::encode_header(header) {
            Ok(encoded) => {
                match TransactionCodec::decode_header(&encoded) {
                    Ok(decoded) => {
                        // Re-encode to verify consistency
                        match TransactionCodec::encode_header(&decoded) {
                            Ok(re_encoded) => {
                                if encoded == re_encoded {
                                    println!("    ✓ Header {} binary roundtrip successful", i);
                                } else {
                                    panic!(
                                        "Header {} binary roundtrip failed - re-encoded bytes differ",
                                        i
                                    );
                                }
                            }
                            Err(e) => {
                                panic!("Failed to re-encode header {}: {:?}", i, e);
                            }
                        }
                    }
                    Err(e) => {
                        panic!("Failed to decode header {}: {:?}", i, e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to encode header {}: {:?}", i, e);
            }
        }
    }

    println!("  [OK] All header samples passed binary roundtrip");
}

#[test]
fn test_transaction_codec_signature_roundtrip() {
    println!("Testing TransactionSignature binary roundtrip...");

    let signatures = TransactionSignature::generate_samples();

    for (i, signature) in signatures.iter().enumerate() {
        println!("  Testing signature sample {}...", i);

        // Test binary encoding roundtrip
        match TransactionCodec::encode_signature(signature) {
            Ok(encoded) => {
                match TransactionCodec::decode_signature(&encoded) {
                    Ok(decoded) => {
                        // Re-encode to verify consistency
                        match TransactionCodec::encode_signature(&decoded) {
                            Ok(re_encoded) => {
                                if encoded == re_encoded {
                                    println!("    ✓ Signature {} binary roundtrip successful", i);
                                } else {
                                    panic!(
                                        "Signature {} binary roundtrip failed - re-encoded bytes differ",
                                        i
                                    );
                                }
                            }
                            Err(e) => {
                                panic!("Failed to re-encode signature {}: {:?}", i, e);
                            }
                        }
                    }
                    Err(e) => {
                        panic!("Failed to decode signature {}: {:?}", i, e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to encode signature {}: {:?}", i, e);
            }
        }
    }

    println!("  [OK] All signature samples passed binary roundtrip");
}

#[test]
fn test_type_matrix_completeness() {
    println!("Verifying TYPE_NAMES completeness...");

    // Verify that TYPE_NAMES contains expected core types
    let required_types = [
        "TransactionEnvelope",
        "TransactionHeader",
        "TransactionSignature",
        "TokenRecipient",
        "KeySpec",
    ];

    let missing_required: Vec<_> = required_types
        .iter()
        .filter(|req_type| !TYPE_NAMES.contains(req_type))
        .collect();

    if !missing_required.is_empty() {
        panic!(
            "TYPE_NAMES is missing required types: {:?}",
            missing_required
        );
    }

    println!("  [OK] All required types are present in TYPE_NAMES");
    println!("  TYPE_NAMES contains {} types total", TYPE_NAMES.len());

    // Verify type coverage implementation
    match verify_type_coverage() {
        Ok(()) => {
            println!("  [OK] Type coverage verification passed");
        }
        Err(missing) => {
            println!("  [WARN] Type coverage issues found:");
            for missing_type in missing {
                println!("    - {}", missing_type);
            }
        }
    }
}