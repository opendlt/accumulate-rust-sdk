use accumulate_client::*;
use serde_json::json;

fn mk_delegated_chain(depth: usize) -> Signature {
    // Build Delegated( Delegated( ... Ed25519(...) ) )
    let leaf = Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    });

    let mut cur = leaf;
    for i in 0..depth {
        cur = Signature::Delegated(DelegatedSignature {
            signature: Box::new(cur),
            delegator: format!("acc://authority-{}", i),
        });
    }
    cur
}

#[test]
fn test_delegated_depth_calculation() {
    // Test depth calculation for various chain lengths
    assert_eq!(delegated_depth(&mk_delegated_chain(0)), 0);
    assert_eq!(delegated_depth(&mk_delegated_chain(1)), 1);
    assert_eq!(delegated_depth(&mk_delegated_chain(3)), 3);
    assert_eq!(delegated_depth(&mk_delegated_chain(5)), 5);
    assert_eq!(delegated_depth(&mk_delegated_chain(6)), 6);
}

#[test]
fn delegated_depth_le_5_passes() {
    for d in 0..=5 {
        let s = mk_delegated_chain(d);
        assert!(enforce_delegated_depth(&s).is_ok(), "depth {} should pass", d);
    }
}

#[test]
fn delegated_depth_gt_5_fails() {
    let s = mk_delegated_chain(6);
    let err = enforce_delegated_depth(&s).unwrap_err();
    assert!(matches!(err, SigRuntimeError::DelegationDepthExceeded));

    // Test with even deeper chains
    let s = mk_delegated_chain(10);
    let err = enforce_delegated_depth(&s).unwrap_err();
    assert!(matches!(err, SigRuntimeError::DelegationDepthExceeded));
}

#[test]
fn test_delegated_signature_smart_constructor() {
    // Test successful construction within limits
    let inner = Box::new(Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    }));

    let result = DelegatedSignature::new_enforced(inner, "acc://delegator.acme".to_string());
    assert!(result.is_ok(), "Single delegation should succeed");

    // Test constructing a chain that would exceed depth limit
    let deep_chain = Box::new(mk_delegated_chain(5)); // Already at limit
    let result = DelegatedSignature::new_enforced(deep_chain, "acc://delegator.acme".to_string());
    assert!(result.is_err(), "Delegation that would exceed depth should fail");
    assert!(matches!(result.unwrap_err(), SigRuntimeError::DelegationDepthExceeded));
}

#[test]
fn test_mixed_signature_types_with_delegation() {
    // Test that delegation depth counting works with various signature types as leaves
    let internal_sig = Signature::Internal(InternalSignature {
        cause: [1u8; 32],
        transaction_hash: [2u8; 32],
    });

    let delegated = Signature::Delegated(DelegatedSignature {
        signature: Box::new(internal_sig),
        delegator: "acc://delegator.acme".to_string(),
    });

    assert_eq!(delegated_depth(&delegated), 1);
    assert!(enforce_delegated_depth(&delegated).is_ok());

    // Test with nested delegations around different signature types
    let partition_sig = Signature::Partition(PartitionSignature {
        source_network: "acc://source.acme".to_string(),
        destination_network: "acc://dest.acme".to_string(),
        sequence_number: 12345,
        transaction_hash: None,
    });

    let mut nested = partition_sig;
    for i in 0..3 {
        nested = Signature::Delegated(DelegatedSignature {
            signature: Box::new(nested),
            delegator: format!("acc://level-{}.acme", i),
        });
    }

    assert_eq!(delegated_depth(&nested), 3);
    assert!(enforce_delegated_depth(&nested).is_ok());
}

#[test]
fn test_serialization_roundtrip_with_depth_limits() {
    // Test that delegated chains can be serialized and deserialized while maintaining depth limits
    let chain = mk_delegated_chain(3);

    // Serialize to JSON
    let json_value = serde_json::to_value(&chain).expect("Should serialize");

    // Deserialize back
    let deserialized: Signature = serde_json::from_value(json_value).expect("Should deserialize");

    // Verify depth is preserved
    assert_eq!(delegated_depth(&deserialized), 3);
    assert!(enforce_delegated_depth(&deserialized).is_ok());
}

#[test]
fn test_edge_case_empty_delegation_chain() {
    // Test with non-delegated signatures (depth should be 0)
    let ed25519_sig = Signature::ED25519(ED25519Signature {
        public_key: vec![0u8; 32],
        signature: vec![0u8; 64],
        signer: "acc://test.acme/signer".to_string(),
        signer_version: 1,
        timestamp: Some(1234567890),
        vote: None,
        transaction_hash: None,
        memo: None,
        data: None,
    });

    assert_eq!(delegated_depth(&ed25519_sig), 0);
    assert!(enforce_delegated_depth(&ed25519_sig).is_ok());

    // Test with other signature types
    let internal_sig = Signature::Internal(InternalSignature {
        cause: [0u8; 32],
        transaction_hash: [0u8; 32],
    });

    assert_eq!(delegated_depth(&internal_sig), 0);
    assert!(enforce_delegated_depth(&internal_sig).is_ok());
}