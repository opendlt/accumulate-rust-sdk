//! Accumulate Rust SDK Test Suite
//!
//! Comprehensive test organization for the Accumulate Protocol Rust SDK

// Unit tests organized by module
pub mod unit;

// Conformance tests for cross-SDK compatibility
pub mod conformance;

// Integration tests for end-to-end scenarios
pub mod integration;

// Golden test vectors and fixtures
pub mod golden;

// Fuzzing and property-based tests
pub mod fuzz;

// Coverage validation tests
pub mod coverage;

// Specialized test scenarios
pub mod specialized;

// Quarantined/experimental tests
pub mod quarantine;

// Repository-specific tests
pub mod repo;