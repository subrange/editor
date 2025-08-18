//! Binary operation lowering for V2 backend
//! 
//! Implements all binary arithmetic and logical operations with
//! Sethi-Ullman ordering for optimal register usage.
//!
//! ## Architecture
//!
//! - `lowering.rs` - Main binary operation lowering logic
//! - `arithmetic.rs` - Arithmetic and logical operations (Add, Sub, Mul, Div, And, Or, Xor, Shift)
//! - `comparison.rs` - Comparison operations (Eq, Ne, Lt, Le, Gt, Ge)
//! - `helpers.rs` - Helper functions for register allocation and value handling

mod lowering;
mod arithmetic;
mod comparison;
mod helpers;

// Public exports
pub use lowering::{lower_binary_op, lower_binary_op_immediate};

// Internal exports for use within this module
 use helpers::{is_commutative, can_reuse_register, calculate_register_needs};
 use arithmetic::generate_arithmetic_instruction;
 use comparison::generate_comparison_instructions;