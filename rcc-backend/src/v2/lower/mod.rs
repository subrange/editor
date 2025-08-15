//! V2 Lowering Module - Integrates All Lowering Components
//! 
//! This module provides the main entry point for lowering IR to assembly using
//! the V2 backend. It organizes the lowering logic into separate modules for
//! better maintainability.

mod module;
mod function;
mod instruction;

// Re-export the main public interface
pub use module::lower_module_v2;

// Re-export function lowering as public since it's used in public API
pub use function::lower_function_v2;

// Re-export function-level utilities for internal use

// Re-export instruction lowering for internal use
