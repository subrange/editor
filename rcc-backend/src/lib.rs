//! Ripple C99 Compiler - Backend
//! 
//! This crate provides the backend for the Ripple C99 compiler,
//! responsible for lowering IR to assembly code.

pub mod module_lowering;
pub mod lower;
pub mod simple_regalloc;
pub mod v2;
mod module_lowering_tests;
mod simple_regalloc_tests;

// Re-export IR types from frontend for convenience
pub use rcc_frontend::ir::{
    Module, Function, BasicBlock, Instruction, Value, IrType,
    IrBinaryOp, IrUnaryOp, GlobalVariable, Linkage, IrBuilder
};
pub use rcc_common::LabelId;
pub use module_lowering::{lower_module_to_assembly, lower_module_to_assembly_with_options};