//! Type conversion utilities

use crate::ast::Type;
use rcc_ir::IrType;
use rcc_common::SourceLocation;
use crate::CompilerError;
use super::errors::CodegenError;

/// Convert AST type to IR type
pub fn convert_type(ast_type: &Type, location: SourceLocation) -> Result<IrType, CompilerError> {
    match ast_type {
        Type::Void => Ok(IrType::Void),
        Type::Bool => Ok(IrType::I1),
        Type::Char | Type::SignedChar | Type::UnsignedChar => Ok(IrType::I8),
        Type::Short | Type::UnsignedShort => Ok(IrType::I16),
        Type::Int | Type::UnsignedInt => Ok(IrType::I16), // 16-bit int on Ripple
        Type::Long | Type::UnsignedLong => Ok(IrType::I32),
        Type::Pointer(target) => {
            let target_type = convert_type(target, location)?;
            Ok(IrType::Ptr(Box::new(target_type)))
        }
        Type::Array { element_type, size } => {
            let elem_type = convert_type(element_type, location)?;
            if let Some(size) = size {
                Ok(IrType::Array { size: *size, element_type: Box::new(elem_type) })
            } else {
                // Incomplete array type - treat as pointer for now
                Ok(IrType::Ptr(Box::new(elem_type)))
            }
        }
        Type::Struct { fields, .. } => {
            // For structs, allocate as array of words
            // Calculate total size in words
            let mut total_words = 0u64;
            for field in fields {
                if let Some(size) = field.field_type.size_in_words() {
                    total_words += size;
                } else {
                    return Err(CodegenError::InternalError {
                        message: format!("Cannot compute size of struct field: {}", field.name),
                        location,
                    }.into());
                }
            }
            // Return as array of I16 (words)
            Ok(IrType::Array { size: total_words, element_type: Box::new(IrType::I16) })
        }
        Type::Union { fields, .. } => {
            // For unions, allocate the size of the largest field
            let mut max_words = 0u64;
            for field in fields {
                if let Some(size) = field.field_type.size_in_words() {
                    max_words = max_words.max(size);
                } else {
                    return Err(CodegenError::InternalError {
                        message: format!("Cannot compute size of union field: {}", field.name),
                        location,
                    }.into());
                }
            }
            // Return as array of I16 (words)
            Ok(IrType::Array { size: max_words, element_type: Box::new(IrType::I16) })
        }
        _ => Err(CodegenError::InvalidType {
            ast_type: ast_type.clone(),
            location,
        }.into()),
    }
}

/// Get the size of an AST type in bytes
pub fn get_ast_type_size(ast_type: &Type) -> u64 {
    match ast_type {
        Type::Void => 0,
        Type::Bool => 1,
        Type::Char | Type::SignedChar | Type::UnsignedChar => 1,
        Type::Short | Type::UnsignedShort => 2,
        Type::Int | Type::UnsignedInt => 2, // 16-bit int on Ripple
        Type::Long | Type::UnsignedLong => 4,
        Type::Pointer(_) => 2, // 16-bit pointers
        Type::Array { element_type, size } => {
            let elem_size = get_ast_type_size(element_type);
            if let Some(size) = size {
                elem_size * size
            } else {
                // Incomplete array type
                0
            }
        }
        Type::Function { .. } => 0, // Functions don't have size
        Type::Struct { .. } | Type::Union { .. } => {
            // Use the size_in_bytes method from Type
            ast_type.size_in_bytes().unwrap_or(0)
        }
        _ => 0, // TODO: Handle other types
    }
}