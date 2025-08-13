//! Tests for the v2 backend implementation

mod register_pressure_tests;
mod integration_tests;
mod encapsulation_test;

// Note: Internal module tests have been moved to their respective modules:
// - function/tests/ contains function and calling convention tests
// - regmgmt/tests/ contains register allocator tests
// - instr/tests/ contains instruction lowering tests