//! Ripple C99 Compiler - Intermediate Representation
//! 
//! This crate defines the intermediate representation (IR) used between
//! the frontend and backend of the compiler. For M1, this is a minimal
//! IR focused on supporting basic code generation testing.
//! 
//! For M2+, we provide a full IR that can represent complete C99 programs.

pub mod simple;
pub mod lowering;
pub mod ir;
pub mod module_lowering;
pub mod lower;
pub mod simple_regalloc;
mod module_lowering_tests;
mod simple_regalloc_tests;

pub use simple::{SimpleIR, SimpleProgram};
pub use lowering::{lower_to_assembly, LoweringError};
pub use ir::{
    Module, Function, BasicBlock, Instruction, Value, IrType,
    IrBinaryOp, IrUnaryOp, GlobalVariable, Linkage, IrBuilder
};
pub use rcc_common::LabelId;
pub use module_lowering::lower_module_to_assembly;