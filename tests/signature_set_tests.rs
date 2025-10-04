use accumulate_client::*;

fn msg() -> Vec<u8> {
    b"accumulate-proof".to_vec()
}

fn create_test_signature_set(signatures: Vec<Box<Signature>>) -> SignatureSet {
    SignatureSet {
        vote: None,
        signer: "acc://test.acme/signer".to_string(),
        transaction_hash: None,
        signatures,
        authority: "acc://test.acme/authority".to_string(),
    }
}

#[test]
fn threshold_invariants() {
    let base_set = create_test_signature_set(vec![]);

    // Test threshold of 0 should fail
    let set_with_threshold = SignatureSetWithThreshold::new(base_set.clone(), 0);
    assert!(set_with_threshold.is_err());
    assert!(matches!(set_with_threshold.unwrap_err(), SigRuntimeError::InvalidSignatureSetThreshold));

    // Test threshold greater than signature count should fail
    let sigs = vec![
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer1".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer2".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
    ];
    let base_set = create_test_signature_set(sigs);
    let set_with_threshold = SignatureSetWithThreshold::new(base_set, 3); // threshold > 2 signatures
    assert!(set_with_threshold.is_err());
    assert!(matches!(set_with_threshold.unwrap_err(), SigRuntimeError::InvalidSignatureSetThreshold));
}

#[test]
fn threshold_pass_fail_counts() {
    // Build a set with 3 signatures; all will verify false with zero keys/signatures
    let s1 = Box::new(Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer1".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    }));
    let s2 = Box::new(Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer2".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    }));
    let s3 = Box::new(Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer3".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    }));

    let base_set = create_test_signature_set(vec![s1, s2, s3]);
    let set_with_threshold = SignatureSetWithThreshold::new(base_set, 1).unwrap();

    // Since all signatures return false, threshold of 1 should fail
    assert_eq!(evaluate_signature_set(&set_with_threshold, &msg()).unwrap(), false);

    // Test the count_valid_sigs function directly
    let mock_sigs = vec![
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer1".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer2".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer3".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
    ];

    // Test that counting works correctly (all should be false)
    let count = count_valid_sigs(&mock_sigs, &msg());
    assert_eq!(count, 0, "All signatures should verify as false");
}

#[test]
fn test_signature_set_construction() {
    // Test valid construction
    let sigs = vec![
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer1".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer2".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer3".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
    ];

    let base_set = create_test_signature_set(sigs);

    // Valid thresholds
    for threshold in 1..=3 {
        let result = SignatureSetWithThreshold::new(base_set.clone(), threshold);
        assert!(result.is_ok(), "Threshold {} should be valid for 3 signatures", threshold);
    }

    // Invalid threshold 0
    let result = SignatureSetWithThreshold::new(base_set.clone(), 0);
    assert!(result.is_err());

    // Invalid threshold > count
    let result = SignatureSetWithThreshold::new(base_set, 4);
    assert!(result.is_err());
}

#[test]
fn test_empty_signature_set() {
    let empty_set = create_test_signature_set(vec![]);

    // Any threshold > 0 should fail for empty set
    let result = SignatureSetWithThreshold::new(empty_set, 1);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SigRuntimeError::InvalidSignatureSetThreshold));
}

#[test]
fn test_single_signature_set() {
    let single_sig = vec![Box::new(Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    }))];
    let base_set = create_test_signature_set(single_sig);

    // Threshold of 1 should be valid
    let set_with_threshold = SignatureSetWithThreshold::new(base_set.clone(), 1);
    assert!(set_with_threshold.is_ok());

    let set = set_with_threshold.unwrap();
    // Should fail since the signature verifies as false
    assert_eq!(evaluate_signature_set(&set, &msg()).unwrap(), false);

    // Threshold of 2 should be invalid (> signature count)
    let result = SignatureSetWithThreshold::new(base_set, 2);
    assert!(result.is_err());
}

#[test]
fn test_count_valid_sigs_with_different_types() {
    // Test that count_valid_sigs works with different signature types
    let mixed_sigs = vec![
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
        Box::new(Signature::Internal(InternalSignature {
            cause: [0u8; 32],
            transaction_hash: [0u8; 32],
        })),
        Box::new(Signature::Partition(PartitionSignature {
            source_network: "acc://source.acme".to_string(),
            destination_network: "acc://dest.acme".to_string(),
            sequence_number: 12345,
            transaction_hash: None,
        })),
    ];

    // All should verify as false with current implementation
    let count = count_valid_sigs(&mixed_sigs, &msg());
    assert_eq!(count, 0, "All signature types should verify as false with test data");
}

#[test]
fn test_signature_set_serialization() {
    // Test that SignatureSet can be serialized and deserialized
    let sigs = vec![
        Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: "acc://test.acme/signer1".to_string(),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })),
        Box::new(Signature::Internal(InternalSignature {
            cause: [1u8; 32],
            transaction_hash: [2u8; 32],
        })),
    ];

    let signature_set = create_test_signature_set(sigs);

    // Serialize to JSON
    let json_value = serde_json::to_value(&signature_set).expect("Should serialize");

    // Deserialize back
    let deserialized: SignatureSet = serde_json::from_value(json_value).expect("Should deserialize");

    // Verify structure is preserved
    assert_eq!(deserialized.signatures.len(), 2);
    assert_eq!(deserialized.signer, "acc://test.acme/signer");
    assert_eq!(deserialized.authority, "acc://test.acme/authority");

    // Test that we can create a threshold wrapper from deserialized set
    let result = SignatureSetWithThreshold::new(deserialized, 2);
    assert!(result.is_ok());
}

#[test]
fn test_evaluation_with_various_thresholds() {
    // Create a set with 5 signatures (all will verify as false)
    let sigs: Vec<Box<Signature>> = (0..5)
        .map(|i| Box::new(Signature::ED25519(ED25519Signature {
            public_key: vec![0u8; 32],
            signature: vec![0u8; 64],
            signer: format!("acc://test.acme/signer{}", i),
            signer_version: 1,
            timestamp: Some(1234567890),
            vote: None,
            transaction_hash: None,
            memo: None,
            data: None,
        })))
        .collect();

    let base_set = create_test_signature_set(sigs);

    // Test different threshold values
    for threshold in 1..=5 {
        let set_with_threshold = SignatureSetWithThreshold::new(base_set.clone(), threshold).unwrap();

        // Since all signatures verify as false (0 valid), all thresholds should fail
        let result = evaluate_signature_set(&set_with_threshold, &msg()).unwrap();
        assert_eq!(result, false, "Threshold {} should fail when no signatures are valid", threshold);
    }
}