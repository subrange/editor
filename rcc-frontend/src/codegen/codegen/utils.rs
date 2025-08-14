//! Code generation utilities

use crate::ir::IrType;
use crate::types::Type;
use crate::CompilerError;
use super::super::types::convert_type;

/// Helper function for convert_type with default location
pub fn convert_type_default(ast_type: &Type) -> Result<IrType, CompilerError> {
    convert_type(ast_type, rcc_common::SourceLocation::new_simple(0, 0))
}