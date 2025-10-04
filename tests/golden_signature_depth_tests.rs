use accumulate_client::*;
use serde_json::json;
use std::{fs, path::PathBuf};

fn mk_delegated_chain(depth: usize) -> generated::signatures::Signature {
    // Create a leaf ED25519 signature
    let leaf = generated::signatures::Signature::ED25519(
        generated::signatures::ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://leaf.acme/book/1".to_string(),
            signer_version: 1,
            timestamp: None,
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        }
    );

    // Wrap it in delegation layers
    (0..depth).fold(leaf, |cur, i| {
        generated::signatures::Signature::Delegated(
            generated::signatures::DelegatedSignature {
                signature: Box::new(cur),
                delegator: format!("acc://auth-{i}.acme/book/1")
            }
        )
    })
}

#[test]
fn delegated_depth_golden() {
    let test_cases = vec![
        (0, true),  // No delegation - should pass
        (1, true),  // 1 level delegation - should pass
        (3, true),  // 3 levels delegation - should pass
        (5, true),  // 5 levels delegation - should pass (limit)
        (6, false), // 6 levels delegation - should fail
        (10, false), // 10 levels delegation - should fail
    ];

    let mut results = Vec::new();

    for (depth, expected_pass) in test_cases {
        let chain = mk_delegated_chain(depth);
        let actual_depth = runtime::signing::delegated_depth(&chain);
        let enforcement_result = runtime::signing::enforce_delegated_depth(&chain);
        let passes = enforcement_result.is_ok();

        // Verify the depth calculation matches expected
        assert_eq!(actual_depth, depth, "Depth calculation mismatch for depth {}", depth);

        // Verify the enforcement result matches expected (using >= for the user's requirement)
        if expected_pass {
            assert!(passes, "Expected depth {} to pass, but it failed", depth);
        } else {
            assert!(!passes, "Expected depth {} to fail, but it passed", depth);
        }

        results.push(json!({
            "depth": depth,
            "calculated_depth": actual_depth,
            "expected_pass": expected_pass,
            "actual_pass": passes,
            "enforcement_error": if !passes {
                format!("{}", enforcement_result.unwrap_err())
            } else {
                "none".to_string()
            }
        }));
    }

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_vectors/signatures/delegated_depth.json");

    let actual = json!({
        "test_results": results,
        "max_allowed_depth": 5,
        "rule": "depth <= 5 passes, depth > 5 fails"
    });

    if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !p.exists() {
        fs::create_dir_all(p.parent().unwrap()).ok();
        fs::write(&p, actual.to_string()).unwrap();
        return; // In write mode, just generate and exit
    }

    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();

    assert_eq!(actual, expected, "delegated depth golden mismatch");
}

#[test]
fn delegated_smart_constructor_golden() {
    // Test the smart constructor that enforces depth
    let leaf = generated::signatures::Signature::ED25519(
        generated::signatures::ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://leaf.acme/book/1".to_string(),
            signer_version: 1,
            timestamp: None,
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        }
    );

    // Test valid delegation (should succeed)
    let valid_result = generated::signatures::DelegatedSignature::new_enforced(
        Box::new(leaf.clone()),
        "acc://delegator.acme/book/1".to_string()
    );
    assert!(valid_result.is_ok(), "Valid delegation should succeed");

    // Create a chain at the limit (5 levels)
    let deep_chain = mk_delegated_chain(5);

    // Try to add one more level (should fail)
    let invalid_result = generated::signatures::DelegatedSignature::new_enforced(
        Box::new(deep_chain),
        "acc://invalid.acme/book/1".to_string()
    );
    assert!(invalid_result.is_err(), "Over-limit delegation should fail");

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_vectors/signatures/delegated_smart_constructor.json");

    let actual = json!({
        "valid_delegation_passes": valid_result.is_ok(),
        "over_limit_delegation_fails": invalid_result.is_err(),
        "over_limit_error": format!("{}", invalid_result.unwrap_err())
    });

    if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !p.exists() {
        fs::create_dir_all(p.parent().unwrap()).ok();
        fs::write(&p, actual.to_string()).unwrap();
        return; // In write mode, just generate and exit
    }

    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();

    assert_eq!(actual, expected, "delegated smart constructor golden mismatch");
}

#[test]
fn signature_depth_edge_cases_golden() {
    // Test edge cases and different signature types as leaves
    let test_cases = vec![
        ("ed25519_leaf", generated::signatures::Signature::ED25519(
            generated::signatures::ED25519Signature {
                public_key: vec![0u8; 32],
                signature: vec![0u8; 64],
                signer: "acc://ed.acme/book/1".to_string(),
                signer_version: 1,
                timestamp: None,
                vote: None,
                transaction_hash: None,
                memo: None,
                data: None,
            }
        )),
        ("legacy_ed25519_leaf", generated::signatures::Signature::LegacyED25519(
            generated::signatures::LegacyED25519Signature {
                timestamp: 1234567890,
                public_key: vec![0u8; 32],
                signature: vec![0u8; 64],
                signer: "acc://legacy.acme/book/1".to_string(),
                signer_version: 1,
                vote: None,
                transaction_hash: None,
            }
        )),
        ("rcd1_leaf", generated::signatures::Signature::RCD1(
            generated::signatures::RCD1Signature {
                public_key: vec![0u8; 32],
                signature: vec![0u8; 64],
                signer: "acc://rcd1.acme/book/1".to_string(),
                signer_version: 1,
                timestamp: None,
                vote: None,
                transaction_hash: None,
                memo: None,
                data: None,
            }
        )),
    ];

    let mut results = Vec::new();

    for (name, leaf_sig) in test_cases {
        // Test leaf (depth 0)
        let depth_0 = runtime::signing::delegated_depth(&leaf_sig);
        let enforcement_0 = runtime::signing::enforce_delegated_depth(&leaf_sig);

        // Test single delegation (depth 1)
        let delegated_1 = generated::signatures::Signature::Delegated(
            generated::signatures::DelegatedSignature {
                signature: Box::new(leaf_sig.clone()),
                delegator: format!("acc://delegator-{}.acme/book/1", name)
            }
        );
        let depth_1 = runtime::signing::delegated_depth(&delegated_1);
        let enforcement_1 = runtime::signing::enforce_delegated_depth(&delegated_1);

        results.push(json!({
            "signature_type": name,
            "leaf_depth": depth_0,
            "leaf_passes": enforcement_0.is_ok(),
            "delegated_depth": depth_1,
            "delegated_passes": enforcement_1.is_ok()
        }));
    }

    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_vectors/signatures/depth_edge_cases.json");

    let actual = json!({
        "test_results": results,
        "description": "Testing delegation depth with different leaf signature types"
    });

    if std::env::var("INSTA_UPDATE").ok().as_deref() == Some("auto") || !p.exists() {
        fs::create_dir_all(p.parent().unwrap()).ok();
        fs::write(&p, actual.to_string()).unwrap();
        return; // In write mode, just generate and exit
    }

    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();

    assert_eq!(actual, expected, "signature depth edge cases golden mismatch");
}