//! Function generation module for V2 backend
//! 
//! This module provides a safe, high-level API for generating complete functions.
//! All internal complexity is hidden behind the `FunctionBuilder` interface.
//! 
//! ## Architecture
//! 
//! - `builder.rs` - Public safe API (FunctionBuilder)
//! - `lowering.rs` - Internal function lowering logic (NOT exposed)
//! - `calling_convention.rs` - Internal calling convention (NOT exposed)
//! 
//! Only the builder API is exposed to prevent misuse of internal components.

// Internal modules - implementation details
mod lowering;
mod calling_convention;
mod builder;

// Public exports - ONLY the safe API
pub use builder::FunctionBuilder;
pub use calling_convention::{CallArg, CallingConvention};  // Re-export CallArg and CallingConvention

// Note: setup_call_with_context was removed as it was unnecessary indirection.
// Callers should directly use CallingConvention::new().setup_call_args() instead.

// Tests
#[cfg(test)]
mod tests;