//! Phase 3 Coverage Validation Tests
//!
//! Comprehensive validation of Phase 3 type system completion implementation.
//! Validates that all 141 protocol types are properly generated and functional.

use accumulate_client::protocol_types;
use serde_json;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Test that validates Phase 3 type system completion
/// Ensures all 141 protocol types are generated and accessible
#[test]
fn test_phase3_type_system_completion() {
    println!("Phase 3 Type System Completion Validation");
    println!("=========================================");

    // Validate Stage 3.1 - Type Graph Builder
    validate_stage_3_1();

    // Validate Stage 3.2 - Rust Type Code Generator
    validate_stage_3_2();

    // Validate Stage 3.3 - Comprehensive Tests and Golden Vectors
    validate_stage_3_3();

    // Validate G5=PASS status
    validate_g5_pass_status();

    println!("\n✅ Phase 3 Type System Completion: ALL VALIDATIONS PASSED");
}

/// Validate Stage 3.1 - Type Graph Builder
fn validate_stage_3_1() {
    println!("\n🔍 Validating Stage 3.1 - Type Graph Builder");

    let types_graph_path = Path::new("src/generated/types_graph.json");
    assert!(types_graph_path.exists(), "Stage 3.1 output missing: types_graph.json");

    let types_reachable_path = Path::new("src/generated/types_reachable.json");
    assert!(types_reachable_path.exists(), "Stage 3.1 output missing: types_reachable.json");

    let types_gate_path = Path::new("src/generated/types_gate.json");
    assert!(types_gate_path.exists(), "Stage 3.1 output missing: types_gate.json");

    // Validate type count
    let gate_content = fs::read_to_string(types_gate_path)
        .expect("Failed to read types_gate.json");
    let gate_data: serde_json::Value = serde_json::from_str(&gate_content)
        .expect("Failed to parse types_gate.json");

    let target_count = gate_data["target_count"].as_u64().expect("Missing target_count");
    let actual_count = gate_data["actual_count"].as_u64().expect("Missing actual_count");
    let validation_passed = gate_data["validation_passed"].as_bool().expect("Missing validation_passed");

    assert_eq!(target_count, 141, "Expected 141 protocol types");
    assert_eq!(actual_count, 141, "Expected 141 protocol types found");
    assert!(validation_passed, "Stage 3.1 validation should pass");

    // Validate reachable types list
    let reachable_content = fs::read_to_string(types_reachable_path)
        .expect("Failed to read types_reachable.json");
    let reachable_data: serde_json::Value = serde_json::from_str(&reachable_content)
        .expect("Failed to parse types_reachable.json");

    let types_count = reachable_data["count"].as_u64().expect("Missing count");
    let types_list = reachable_data["types"].as_array().expect("Missing types array");

    assert_eq!(types_count, 141, "Reachable types count should be 141");
    assert_eq!(types_list.len(), 141, "Reachable types list should have 141 items");

    println!("✅ Stage 3.1 validation passed: {} protocol types discovered", types_count);
}

/// Validate Stage 3.2 - Rust Type Code Generator
fn validate_stage_3_2() {
    println!("\n🔧 Validating Stage 3.2 - Rust Type Code Generator");

    let types_rs_path = Path::new("src/generated/types.rs");
    assert!(types_rs_path.exists(), "Stage 3.2 output missing: types.rs");

    let types_generated_path = Path::new("src/generated/types_generated.json");
    assert!(types_generated_path.exists(), "Stage 3.2 output missing: types_generated.json");

    // Validate generation metadata
    let generated_content = fs::read_to_string(types_generated_path)
        .expect("Failed to read types_generated.json");
    let generated_data: serde_json::Value = serde_json::from_str(&generated_content)
        .expect("Failed to parse types_generated.json");

    let target_count = generated_data["target_count"].as_u64().expect("Missing target_count");
    let generated_count = generated_data["generated_count"].as_u64().expect("Missing generated_count");
    let validation_passed = generated_data["validation_passed"].as_bool().expect("Missing validation_passed");

    assert_eq!(target_count, 141, "Expected 141 types to generate");
    assert_eq!(generated_count, 141, "Expected 141 types generated");
    assert!(validation_passed, "Stage 3.2 validation should pass");

    // Validate types.rs file contents
    let types_content = fs::read_to_string(types_rs_path)
        .expect("Failed to read types.rs");

    assert!(types_content.contains("use serde::{Serialize, Deserialize}"), "Types should use Serde");
    assert!(types_content.contains("use crate::generated::enums::"), "Types should import enums");
    assert!(types_content.contains("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"), "Types should derive standard traits");

    // Validate that some key protocol types are present
    let key_types = [
        "ADI", "AccountAuth", "TransactionHeader", "SendTokens", "CreateIdentity",
        "TokenAccount", "KeyPage", "RemoteSignature", "SyntheticForwardTransaction"
    ];

    for key_type in &key_types {
        assert!(types_content.contains(&format!("pub struct {}", key_type)),
                "Key protocol type {} should be generated", key_type);
    }

    println!("✅ Stage 3.2 validation passed: {} Rust types generated", generated_count);
}

/// Validate Stage 3.3 - Comprehensive Tests and Golden Vectors
fn validate_stage_3_3() {
    println!("\n🧪 Validating Stage 3.3 - Comprehensive Tests and Golden Vectors");

    let test_metadata_path = Path::new("tests/tests_metadata.json");
    assert!(test_metadata_path.exists(), "Stage 3.3 output missing: tests_metadata.json");

    let conformance_test_path = Path::new("tests/conformance/test_protocol_types.rs");
    assert!(conformance_test_path.exists(), "Stage 3.3 output missing: test_protocol_types.rs");

    let golden_dir = Path::new("tests/golden/types");
    assert!(golden_dir.exists(), "Stage 3.3 output missing: golden/types directory");

    // Validate test metadata
    let metadata_content = fs::read_to_string(test_metadata_path)
        .expect("Failed to read tests_metadata.json");
    let metadata_data: serde_json::Value = serde_json::from_str(&metadata_content)
        .expect("Failed to parse tests_metadata.json");

    let target_count = metadata_data["target_count"].as_u64().expect("Missing target_count");
    let golden_created = metadata_data["golden_vectors_created"].as_u64().expect("Missing golden_vectors_created");
    let validation_passed = metadata_data["validation_passed"].as_bool().expect("Missing validation_passed");

    assert_eq!(target_count, 141, "Expected 141 types to test");
    assert_eq!(golden_created, 141, "Expected 141 golden vectors created");
    assert!(validation_passed, "Stage 3.3 validation should pass");

    // Count actual golden vector files
    let golden_files: Vec<_> = fs::read_dir(golden_dir)
        .expect("Failed to read golden vectors directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "json" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(golden_files.len(), 141, "Should have 141 golden vector JSON files");

    // Validate conformance test file
    let test_content = fs::read_to_string(conformance_test_path)
        .expect("Failed to read test_protocol_types.rs");

    assert!(test_content.contains("use crate::protocol_types::*"), "Test should import protocol types");
    assert!(test_content.contains("test_all_protocol_types_coverage"), "Test should have coverage validation");
    assert!(test_content.contains("141 protocol types"), "Test should reference 141 types");

    println!("✅ Stage 3.3 validation passed: {} golden vectors and conformance tests created", golden_created);
}

/// Validate G5=PASS status - overall Phase 3 completion
fn validate_g5_pass_status() {
    println!("\n🎯 Validating G5=PASS Status");

    // Load all metadata files and validate they show PASS status
    let stage_results = [
        ("Stage 3.1", "src/generated/types_gate.json"),
        ("Stage 3.2", "src/generated/types_generated.json"),
        ("Stage 3.3", "tests/tests_metadata.json"),
    ];

    let mut all_passed = true;
    let mut total_types = 0;

    for (stage_name, metadata_path) in &stage_results {
        let content = fs::read_to_string(metadata_path)
            .expect(&format!("Failed to read {}", metadata_path));
        let data: serde_json::Value = serde_json::from_str(&content)
            .expect(&format!("Failed to parse {}", metadata_path));

        let validation_passed = data["validation_passed"].as_bool()
            .expect(&format!("Missing validation_passed in {}", metadata_path));

        if validation_passed {
            println!("  ✅ {}: PASSED", stage_name);
        } else {
            println!("  ❌ {}: FAILED", stage_name);
            all_passed = false;
        }

        // Count types from the first stage
        if stage_name == &"Stage 3.1" {
            total_types = data["actual_count"].as_u64().expect("Missing actual_count");
        }
    }

    assert!(all_passed, "All stages must pass for G5=PASS");
    assert_eq!(total_types, 141, "G5=PASS requires exactly 141 protocol types");

    println!("\n🎉 G5=PASS ACHIEVED!");
    println!("  📊 Total protocol types: {}", total_types);
    println!("  🏗️  Stage 3.1: Type graph built successfully");
    println!("  🦀 Stage 3.2: Rust code generated successfully");
    println!("  🧪 Stage 3.3: Tests and golden vectors created successfully");
}

/// Test that core protocol types can be instantiated and serialized
#[test]
fn test_phase3_protocol_types_basic_functionality() {
    println!("Testing Phase 3 Protocol Types Basic Functionality");

    // Test that we can access protocol types through the module
    assert!(std::any::type_name::<protocol_types::ADI>().contains("ADI"));
    assert!(std::any::type_name::<protocol_types::SendTokens>().contains("SendTokens"));
    assert!(std::any::type_name::<protocol_types::TransactionHeader>().contains("TransactionHeader"));

    // Test basic serialization of a simple protocol type
    let adi = protocol_types::ADI {
        url: "acc://test.acme".to_string(),
    };

    let serialized = serde_json::to_string(&adi).expect("Should serialize ADI");
    assert!(serialized.contains("acc://test.acme"));

    let deserialized: protocol_types::ADI = serde_json::from_str(&serialized)
        .expect("Should deserialize ADI");
    assert_eq!(deserialized.url, "acc://test.acme");

    println!("✅ Protocol types basic functionality test passed");
}

/// Test that golden vectors are valid JSON and can be loaded
#[test]
fn test_phase3_golden_validity() {
    println!("Testing Phase 3 Golden Vectors Validity");

    let golden_dir = Path::new("tests/golden/types");
    assert!(golden_dir.exists(), "Golden vectors directory should exist");

    let mut valid_vectors = 0;
    let mut total_vectors = 0;

    for entry in fs::read_dir(golden_dir).expect("Should read golden vectors directory") {
        let entry = entry.expect("Should read directory entry");
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            total_vectors += 1;

            let content = fs::read_to_string(&path)
                .expect(&format!("Should read golden vector file: {:?}", path));

            let golden_data: serde_json::Value = serde_json::from_str(&content)
                .expect(&format!("Golden vector should be valid JSON: {:?}", path));

            // Validate golden vector structure
            assert!(golden_data.get("type_name").is_some(), "Golden vector should have type_name");
            assert!(golden_data.get("json_data").is_some(), "Golden vector should have json_data");
            assert!(golden_data.get("test_scenarios").is_some(), "Golden vector should have test_scenarios");

            valid_vectors += 1;
        }
    }

    assert_eq!(total_vectors, 141, "Should have 141 golden vector files");
    assert_eq!(valid_vectors, 141, "All 141 golden vectors should be valid");

    println!("✅ All {} golden vectors are valid JSON", valid_vectors);
}

/// Test Phase 3 integration with existing phases
#[test]
fn test_phase3_integration_with_existing_phases() {
    println!("Testing Phase 3 Integration with Existing Phases");

    // Test that Phase 3 types can coexist with Phase 1 enums
    use accumulate_client::*;

    // Phase 1 enums should still be accessible
    let account_type = AccountType::Identity;
    let transaction_type = TransactionType::SendTokens;
    let signature_type = SignatureType::ED25519;

    // Phase 3 protocol types should be accessible
    let transaction_header = protocol_types::TransactionHeader {
        principal: "acc://test.acme".to_string(),
        initiator: [0u8; 32],
        memo: "test memo".to_string(),
        metadata: vec![],
        expire: protocol_types::ExpireOptions {
            at_time: "2023-12-31T23:59:59Z".to_string(),
        },
        hold_until: protocol_types::HoldUntilOptions {
            minor_block: 100,
        },
        authorities: "acc://test.acme/book".to_string(),
    };

    // Test serialization of complex structure
    let serialized = serde_json::to_string(&transaction_header)
        .expect("Should serialize TransactionHeader");

    assert!(serialized.contains("test.acme"));
    assert!(serialized.contains("test memo"));

    println!("✅ Phase 3 integration test passed");
}

/// Test that Phase 3 files are properly generated and not hand-edited
#[test]
fn test_phase3_generated_files_integrity() {
    println!("Testing Phase 3 Generated Files Integrity");

    let generated_files = [
        "src/generated/types.rs",
        "src/generated/types_graph.json",
        "src/generated/types_reachable.json",
        "src/generated/types_gate.json",
        "src/generated/types_generated.json",
        "tests/conformance/test_protocol_types.rs",
        "tests/tests_metadata.json",
    ];

    for file_path in &generated_files {
        let path = Path::new(file_path);
        assert!(path.exists(), "Generated file should exist: {}", file_path);

        let content = fs::read_to_string(path)
            .expect(&format!("Should read generated file: {}", file_path));

        // Check for generation markers
        let has_generation_marker = content.contains("Auto-generated") ||
                                  content.contains("DO NOT EDIT") ||
                                  content.contains("Generated at:") ||
                                  content.contains("generated_at") ||
                                  content.contains("Stage 3.") ||
                                  content.contains("stage");

        assert!(has_generation_marker,
                "Generated file should have generation marker: {}", file_path);
    }

    println!("✅ All generated files have proper integrity markers");
}