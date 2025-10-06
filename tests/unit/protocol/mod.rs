//! Protocol unit tests
//!
//! Tests for protocol types, enums, transactions, and type validation

pub mod enum_edge_case_tests;
pub mod enum_integration_tests;
pub mod enum_only_tests;
pub mod enum_performance_tests;
pub mod enum_property_tests;
pub mod enum_roundtrip_tests;
pub mod enum_stability_tests;
pub mod envelope_shape_tests;
pub mod golden_vector_tests;
pub mod test_protocol_types;
pub mod transaction_edge_case_tests;
pub mod tx_allowlist_tests;
pub mod tx_body_serializer_tests;
pub mod tx_header_parity_tests;
pub mod url_derivation_test;