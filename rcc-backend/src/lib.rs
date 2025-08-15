//! Ripple C99 Compiler - Backend
//! 
//! This crate provides the backend for the Ripple C99 compiler,
//! responsible for lowering IR to assembly code.

pub mod v2;

// Re-export IR types from frontend for convenience
pub use rcc_frontend::ir::{
    Module, Function, BasicBlock, Instruction, Value, IrType,
    IrBinaryOp, IrUnaryOp, GlobalVariable, Linkage, IrBuilder
};
pub use rcc_common::LabelId;

// Re-export the main lowering function
pub use v2::lower_module_v2;

/// Options for lowering
pub struct LoweringOptions {
    pub bank_size: u16,
    pub use_v2: bool,
    pub trace_spills: bool,
}

impl Default for LoweringOptions {
    fn default() -> Self {
        Self {
            bank_size: 4096,
            use_v2: true,
            trace_spills: false,
        }
    }
}

/// Lower a module to assembly with options
pub fn lower_module_to_assembly_with_options(
    module: &Module,
    options: LoweringOptions,
) -> Result<Vec<rcc_codegen::AsmInst>, String> {
    // Always use v2 backend now that v1 is removed
    lower_module_v2(module, options.bank_size, options.trace_spills)
}