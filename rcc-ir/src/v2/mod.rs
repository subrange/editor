//! V2 Backend Implementation with Correct ABI and Bank Handling
//! 
//! This module contains the corrected v2 implementation that fixes all
//! critical issues from the conformance report:
//! - R13 properly initialized to 1 for stack bank
//! - Correct calling convention (params in R5-R11, returns in R3-R4)
//! - Proper fat pointer handling
//! - Bank-aware GEP operations
//! - Correct memory operation bank registers

pub mod regalloc;
pub mod function;
pub mod calling_convention;
pub mod register_pressure;

#[cfg(test)]
mod tests;

pub use regalloc::RegAllocV2;
pub use function::FunctionLowering;
pub use calling_convention::CallingConvention;
pub use register_pressure::RegisterPressureManager;

/// Bank size in instructions (from ASSEMBLY_FORMAT.md)
/// Each bank holds 4096 instructions
/// Since each instruction is 4 cells, that's 16384 cells per bank
pub const BANK_SIZE_INSTRUCTIONS: u16 = 4096;
pub const BANK_SIZE_CELLS: u16 = 16384; // 4096 * 4