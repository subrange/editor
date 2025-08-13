//! V2 Backend Implementation with Correct ABI and Bank Handling
//! 
//! This module contains the corrected v2 implementation that fixes all
//! critical issues from the conformance report:
//! - R13 properly initialized to 1 for stack bank
//! - Correct calling convention (params in R5-R11, returns in R3-R4)
//! - Proper fat pointer handling
//! - Bank-aware GEP operations
//! - Correct memory operation bank registers
//! 
//! ## Architecture
//! 
//! The V2 backend follows a strict encapsulation pattern:
//! - Public APIs provide safe, high-level interfaces
//! - Internal modules handle complex implementation details
//! - Safety invariants are enforced automatically
//! 
//! Consumers should use:
//! - `FunctionBuilder` for building complete functions
//! - `RegisterPressureManager` for register allocation (if needed directly)
//! - `lower_load/lower_store` for instruction lowering

// Public modules - safe APIs for consumers
pub mod regmgmt;
pub mod naming;
pub mod instr;
pub mod function;  // Now a module with controlled exports
pub mod lower;  // Main lowering module
pub mod globals;  // Global variable management

#[cfg(test)]
mod tests;

// Public exports - only what consumers need
pub use regmgmt::{RegisterPressureManager, BankInfo};
pub use function::{FunctionBuilder, CallArg};
pub use instr::{lower_load, lower_store, lower_gep};
pub use lower::{lower_module_v2, lower_function_v2};

// Note: Internal components like FunctionLowering and CallingConvention
// are completely hidden inside the function module.

/// Bank size in instructions (from ASSEMBLY_FORMAT.md)
/// Each bank holds 4096 instructions
/// Since each instruction is 4 cells, that's 16384 cells per bank
pub const BANK_SIZE_INSTRUCTIONS: u16 = 4096;
pub const BANK_SIZE_CELLS: u16 = 16384; // 4096 * 4