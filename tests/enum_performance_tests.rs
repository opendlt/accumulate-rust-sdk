// Performance and memory characteristic tests for enum implementation
use serde_json;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

// Include the generated enums directly
include!("../src/generated/enums.rs");

#[test]
fn test_enum_serialization_performance() {
    // Test serialization performance for different enum types

    let iterations = 100_000;

    // Test TransactionType serialization
    let tx_type = TransactionType::WriteData;
    let start = Instant::now();

    for _ in 0..iterations {
        let _json = serde_json::to_string(&tx_type).unwrap();
    }

    let tx_serialize_duration = start.elapsed();
    println!("TransactionType serialization: {} iterations in {:?} ({:.2} ns/op)",
             iterations, tx_serialize_duration, tx_serialize_duration.as_nanos() as f64 / iterations as f64);

    // Should be very fast - less than 10 microseconds per operation on modern hardware
    assert!(tx_serialize_duration < Duration::from_millis(100),
           "TransactionType serialization too slow: {:?}", tx_serialize_duration);

    // Test AccountType serialization
    let account_type = AccountType::Identity;
    let start = Instant::now();

    for _ in 0..iterations {
        let _json = serde_json::to_string(&account_type).unwrap();
    }

    let account_serialize_duration = start.elapsed();
    println!("AccountType serialization: {} iterations in {:?} ({:.2} ns/op)",
             iterations, account_serialize_duration, account_serialize_duration.as_nanos() as f64 / iterations as f64);

    assert!(account_serialize_duration < Duration::from_millis(100),
           "AccountType serialization too slow: {:?}", account_serialize_duration);
}

#[test]
fn test_enum_deserialization_performance() {
    // Test deserialization performance

    let iterations = 100_000;

    // Test TransactionType deserialization
    let json_str = "\"writeData\"";
    let start = Instant::now();

    for _ in 0..iterations {
        let _tx: TransactionType = serde_json::from_str(json_str).unwrap();
    }

    let tx_deserialize_duration = start.elapsed();
    println!("TransactionType deserialization: {} iterations in {:?} ({:.2} ns/op)",
             iterations, tx_deserialize_duration, tx_deserialize_duration.as_nanos() as f64 / iterations as f64);

    assert!(tx_deserialize_duration < Duration::from_millis(200),
           "TransactionType deserialization too slow: {:?}", tx_deserialize_duration);

    // Test AccountType deserialization
    let account_json = "\"identity\"";
    let start = Instant::now();

    for _ in 0..iterations {
        let _account: AccountType = serde_json::from_str(account_json).unwrap();
    }

    let account_deserialize_duration = start.elapsed();
    println!("AccountType deserialization: {} iterations in {:?} ({:.2} ns/op)",
             iterations, account_deserialize_duration, account_deserialize_duration.as_nanos() as f64 / iterations as f64);

    assert!(account_deserialize_duration < Duration::from_millis(200),
           "AccountType deserialization too slow: {:?}", account_deserialize_duration);
}

#[test]
fn test_enum_comparison_performance() {
    // Test enum comparison performance

    let iterations = 10_000_000; // More iterations for simple operations

    let tx1 = TransactionType::WriteData;
    let tx2 = TransactionType::WriteData;
    let tx3 = TransactionType::CreateIdentity;

    // Test equality comparison
    let start = Instant::now();
    let mut equal_count = 0;

    for i in 0..iterations {
        if i % 2 == 0 {
            if tx1 == tx2 { equal_count += 1; }
        } else {
            if tx1 == tx3 { equal_count += 1; }
        }
    }

    let comparison_duration = start.elapsed();
    println!("Enum equality comparison: {} iterations in {:?} ({:.2} ns/op)",
             iterations, comparison_duration, comparison_duration.as_nanos() as f64 / iterations as f64);

    // Should be extremely fast - less than 1ns per operation
    assert!(comparison_duration < Duration::from_millis(100),
           "Enum comparison too slow: {:?}", comparison_duration);
    assert_eq!(equal_count, iterations / 2); // Verify logic worked
}

#[test]
fn test_enum_hashing_performance() {
    // Test enum hashing performance for HashMap usage

    let iterations = 1_000_000;

    let mut map: HashMap<TransactionType, u32> = HashMap::new();
    let tx_types = vec![
        TransactionType::WriteData,
        TransactionType::CreateIdentity,
        TransactionType::SendTokens,
        TransactionType::CreateTokenAccount,
        TransactionType::AddCredits,
    ];

    // Test HashMap insertion performance
    let start = Instant::now();

    for i in 0..iterations {
        let tx_type = &tx_types[i % tx_types.len()];
        map.insert(tx_type.clone(), i as u32);
    }

    let insert_duration = start.elapsed();
    println!("HashMap insertion: {} iterations in {:?} ({:.2} ns/op)",
             iterations, insert_duration, insert_duration.as_nanos() as f64 / iterations as f64);

    // Test HashMap lookup performance
    let start = Instant::now();
    let mut lookup_count = 0;

    for i in 0..iterations {
        let tx_type = &tx_types[i % tx_types.len()];
        if map.get(tx_type).is_some() {
            lookup_count += 1;
        }
    }

    let lookup_duration = start.elapsed();
    println!("HashMap lookup: {} iterations in {:?} ({:.2} ns/op)",
             iterations, lookup_duration, lookup_duration.as_nanos() as f64 / iterations as f64);

    assert!(insert_duration < Duration::from_millis(500),
           "HashMap insertion too slow: {:?}", insert_duration);
    assert!(lookup_duration < Duration::from_millis(100),
           "HashMap lookup too slow: {:?}", lookup_duration);
    assert_eq!(lookup_count, iterations);
}

#[test]
fn test_enum_collection_performance() {
    // Test enum performance in various collection types

    let iterations = 100_000;

    // Test HashSet operations
    let mut set: HashSet<TransactionType> = HashSet::new();
    let start = Instant::now();

    for i in 0..iterations {
        match i % 5 {
            0 => { set.insert(TransactionType::WriteData); },
            1 => { set.insert(TransactionType::CreateIdentity); },
            2 => { set.insert(TransactionType::SendTokens); },
            3 => { set.insert(TransactionType::CreateTokenAccount); },
            4 => { set.insert(TransactionType::AddCredits); },
            _ => unreachable!(),
        }
    }

    let set_insert_duration = start.elapsed();
    println!("HashSet insertion: {} iterations in {:?} ({:.2} ns/op)",
             iterations, set_insert_duration, set_insert_duration.as_nanos() as f64 / iterations as f64);

    // Test Vec operations
    let mut vec: Vec<TransactionType> = Vec::new();
    let start = Instant::now();

    for i in 0..iterations {
        match i % 5 {
            0 => vec.push(TransactionType::WriteData),
            1 => vec.push(TransactionType::CreateIdentity),
            2 => vec.push(TransactionType::SendTokens),
            3 => vec.push(TransactionType::CreateTokenAccount),
            4 => vec.push(TransactionType::AddCredits),
            _ => unreachable!(),
        }
    }

    let vec_push_duration = start.elapsed();
    println!("Vec push: {} iterations in {:?} ({:.2} ns/op)",
             iterations, vec_push_duration, vec_push_duration.as_nanos() as f64 / iterations as f64);

    assert!(set_insert_duration < Duration::from_millis(200),
           "HashSet insertion too slow: {:?}", set_insert_duration);
    assert!(vec_push_duration < Duration::from_millis(50),
           "Vec push too slow: {:?}", vec_push_duration);

    assert_eq!(set.len(), 5); // Should have 5 unique values
    assert_eq!(vec.len(), iterations); // Should have all pushed values
}

#[test]
fn test_enum_match_performance() {
    // Test pattern matching performance

    let iterations = 10_000_000;

    let tx_types = vec![
        TransactionType::WriteData,
        TransactionType::CreateIdentity,
        TransactionType::SendTokens,
        TransactionType::CreateTokenAccount,
        TransactionType::AddCredits,
        TransactionType::BurnTokens,
        TransactionType::LockAccount,
        TransactionType::UpdateKeyPage,
    ];

    let start = Instant::now();
    let mut category_counts = [0u32; 4]; // user, synthetic, system, other

    for i in 0..iterations {
        let tx_type = &tx_types[i % tx_types.len()];

        // Categorize transaction types (realistic use case)
        match tx_type {
            TransactionType::WriteData |
            TransactionType::CreateIdentity |
            TransactionType::SendTokens |
            TransactionType::CreateTokenAccount |
            TransactionType::AddCredits => {
                category_counts[0] += 1; // user
            },
            TransactionType::SyntheticCreateIdentity |
            TransactionType::SyntheticWriteData |
            TransactionType::SyntheticDepositTokens => {
                category_counts[1] += 1; // synthetic
            },
            TransactionType::SystemGenesis |
            TransactionType::DirectoryAnchor => {
                category_counts[2] += 1; // system
            },
            _ => {
                category_counts[3] += 1; // other
            },
        }
    }

    let match_duration = start.elapsed();
    println!("Pattern matching: {} iterations in {:?} ({:.2} ns/op)",
             iterations, match_duration, match_duration.as_nanos() as f64 / iterations as f64);

    // Should be extremely fast
    assert!(match_duration < Duration::from_millis(200),
           "Pattern matching too slow: {:?}", match_duration);

    // Verify the logic worked
    let total_categorized: u32 = category_counts.iter().sum();
    assert_eq!(total_categorized, iterations as u32);
}

#[test]
fn test_enum_clone_performance() {
    // Test cloning performance (should be trivial for Copy types)

    let iterations = 10_000_000;

    let original = TransactionType::WriteData;
    let start = Instant::now();

    for _ in 0..iterations {
        let _cloned = original.clone();
    }

    let clone_duration = start.elapsed();
    println!("Enum cloning: {} iterations in {:?} ({:.2} ns/op)",
             iterations, clone_duration, clone_duration.as_nanos() as f64 / iterations as f64);

    // Should be essentially free (Copy semantics)
    assert!(clone_duration < Duration::from_millis(50),
           "Enum cloning too slow: {:?}", clone_duration);
}

#[test]
fn test_enum_memory_layout() {
    // Test memory characteristics of enums
    use std::mem;

    // All our enums should be very small
    assert_eq!(mem::size_of::<ExecutorVersion>(), 1, "ExecutorVersion should be 1 byte");
    assert_eq!(mem::size_of::<PartitionType>(), 1, "PartitionType should be 1 byte");
    assert_eq!(mem::size_of::<DataEntryType>(), 1, "DataEntryType should be 1 byte");
    assert_eq!(mem::size_of::<ObjectType>(), 1, "ObjectType should be 1 byte");
    assert_eq!(mem::size_of::<SignatureType>(), 1, "SignatureType should be 1 byte");
    assert_eq!(mem::size_of::<KeyPageOperationType>(), 1, "KeyPageOperationType should be 1 byte");
    assert_eq!(mem::size_of::<AccountAuthOperationType>(), 1, "AccountAuthOperationType should be 1 byte");
    assert_eq!(mem::size_of::<NetworkMaintenanceOperationType>(), 1, "NetworkMaintenanceOperationType should be 1 byte");
    assert_eq!(mem::size_of::<TransactionMax>(), 1, "TransactionMax should be 1 byte");
    assert_eq!(mem::size_of::<TransactionType>(), 1, "TransactionType should be 1 byte");
    assert_eq!(mem::size_of::<AccountType>(), 1, "AccountType should be 1 byte");
    assert_eq!(mem::size_of::<AllowedTransactionBit>(), 1, "AllowedTransactionBit should be 1 byte");
    assert_eq!(mem::size_of::<VoteType>(), 1, "VoteType should be 1 byte");
    assert_eq!(mem::size_of::<BookType>(), 1, "BookType should be 1 byte");

    // Test alignment
    assert_eq!(mem::align_of::<TransactionType>(), 1, "TransactionType should have 1-byte alignment");
    assert_eq!(mem::align_of::<AccountType>(), 1, "AccountType should have 1-byte alignment");

    // Test Option<Enum> optimization (should use niche optimization)
    assert_eq!(mem::size_of::<Option<TransactionType>>(), 1, "Option<TransactionType> should be optimized");
    assert_eq!(mem::size_of::<Option<AccountType>>(), 1, "Option<AccountType> should be optimized");

    // Test that arrays of enums are tightly packed
    assert_eq!(mem::size_of::<[TransactionType; 10]>(), 10, "Array of 10 TransactionTypes should be 10 bytes");
    assert_eq!(mem::size_of::<[AccountType; 5]>(), 5, "Array of 5 AccountTypes should be 5 bytes");
}

#[test]
fn test_enum_bulk_operations() {
    // Test performance with large collections of enums

    let size = 1_000_000;

    // Create a large Vec of random enum values
    let mut transactions = Vec::with_capacity(size);
    let tx_variants = [
        TransactionType::WriteData,
        TransactionType::CreateIdentity,
        TransactionType::SendTokens,
        TransactionType::CreateTokenAccount,
        TransactionType::AddCredits,
        TransactionType::BurnTokens,
        TransactionType::UpdateKeyPage,
        TransactionType::LockAccount,
    ];

    let start = Instant::now();
    for i in 0..size {
        transactions.push(tx_variants[i % tx_variants.len()].clone());
    }
    let creation_duration = start.elapsed();

    println!("Created {} enum vector in {:?}", size, creation_duration);

    // Test bulk filtering
    let start = Instant::now();
    let user_transactions: Vec<_> = transactions.iter()
        .filter(|&tx| matches!(tx,
            TransactionType::WriteData |
            TransactionType::CreateIdentity |
            TransactionType::SendTokens |
            TransactionType::CreateTokenAccount |
            TransactionType::AddCredits
        ))
        .collect();
    let filter_duration = start.elapsed();

    println!("Filtered {} items to {} user transactions in {:?}",
             size, user_transactions.len(), filter_duration);

    // Test bulk counting
    let start = Instant::now();
    let mut counts = HashMap::new();
    for tx in &transactions {
        *counts.entry(tx.clone()).or_insert(0) += 1;
    }
    let count_duration = start.elapsed();

    println!("Counted {} items into {} categories in {:?}",
             size, counts.len(), count_duration);

    // All operations should be reasonably fast
    assert!(creation_duration < Duration::from_millis(100),
           "Vector creation too slow: {:?}", creation_duration);
    assert!(filter_duration < Duration::from_millis(200),
           "Filtering too slow: {:?}", filter_duration);
    assert!(count_duration < Duration::from_millis(500),
           "Counting too slow: {:?}", count_duration);

    // Verify the results make sense
    assert!(!user_transactions.is_empty());
    assert_eq!(counts.len(), tx_variants.len());

    let total_counted: usize = counts.values().sum();
    assert_eq!(total_counted, size);
}

#[test]
fn test_enum_json_bulk_performance() {
    // Test JSON serialization/deserialization performance with bulk data

    let size = 10_000;

    // Create bulk enum data
    let transactions: Vec<TransactionType> = (0..size)
        .map(|i| match i % 8 {
            0 => TransactionType::WriteData,
            1 => TransactionType::CreateIdentity,
            2 => TransactionType::SendTokens,
            3 => TransactionType::CreateTokenAccount,
            4 => TransactionType::AddCredits,
            5 => TransactionType::BurnTokens,
            6 => TransactionType::UpdateKeyPage,
            7 => TransactionType::LockAccount,
            _ => unreachable!(),
        })
        .collect();

    // Test bulk serialization
    let start = Instant::now();
    let json = serde_json::to_string(&transactions).unwrap();
    let serialize_duration = start.elapsed();

    println!("Serialized {} enums to JSON ({} bytes) in {:?}",
             size, json.len(), serialize_duration);

    // Test bulk deserialization
    let start = Instant::now();
    let deserialized: Vec<TransactionType> = serde_json::from_str(&json).unwrap();
    let deserialize_duration = start.elapsed();

    println!("Deserialized {} enums from JSON in {:?}",
             size, deserialize_duration);

    // Verify correctness
    assert_eq!(transactions.len(), deserialized.len());
    assert_eq!(transactions, deserialized);

    // Performance should be reasonable for this size
    assert!(serialize_duration < Duration::from_millis(500),
           "Bulk serialization too slow: {:?}", serialize_duration);
    assert!(deserialize_duration < Duration::from_millis(500),
           "Bulk deserialization too slow: {:?}", deserialize_duration);
}