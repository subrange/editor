//! Tests for V2 instruction lowering
//! 
//! This module contains unit tests for individual instruction types
//! and integration tests for instruction combinations.

#[cfg(test)]
mod load_tests;

#[cfg(test)]
mod store_tests;

// Future test modules as instructions are implemented:
// mod gep_tests;
// mod icmp_tests;
// mod branch_tests;
// mod call_tests;
// mod return_tests;
// mod alloca_tests;

// Integration tests for instruction combinations
#[cfg(test)]
mod integration_tests;