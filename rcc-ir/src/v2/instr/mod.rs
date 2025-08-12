//! Instruction lowering modules for V2 backend
//! 
//! This module contains the implementation for lowering individual IR instructions
//! to Ripple assembly instructions.

pub mod load;
pub mod store;
pub mod gep;
pub mod binary;
pub mod unary;

pub use load::lower_load;
pub use store::lower_store;
pub use gep::lower_gep;
pub use binary::{lower_binary_op, lower_binary_op_immediate};
pub use unary::lower_unary_op;

// Test modules
#[cfg(test)]
mod tests;

// TODO: Add these modules as they are implemented
// pub mod icmp;
// pub mod branch;
// pub mod alloca;
// pub mod call;
// pub mod ret;