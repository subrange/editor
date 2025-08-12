//! Instruction lowering modules for V2 backend
//! 
//! This module contains the implementation for lowering individual IR instructions
//! to Ripple assembly instructions.

pub mod load;
pub mod store;
pub mod gep;

pub use load::lower_load;
pub use store::lower_store;
pub use gep::lower_gep;

// Test modules
#[cfg(test)]
mod tests;

// TODO: Add these modules as they are implemented
// pub mod icmp;
// pub mod branch;
// pub mod alloca;
// pub mod call;
// pub mod ret;