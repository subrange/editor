//! Assignment operation code generation

use super::TypedExpressionGenerator;
use super::unary_ops::generate_lvalue_address;
use crate::ir::{Value, IrType};
use crate::typed_ast::TypedExpr;
use crate::types::Type;
use crate::CompilerError;
use crate::codegen::CodegenError;

/// Copy struct contents from source pointer to destination pointer
/// 
/// This performs a word-by-word copy of struct data, similar to memcpy.
/// Used for struct assignments and initializations.
pub fn copy_struct(
    gen: &mut TypedExpressionGenerator,
    src_ptr: Value,
    dst_ptr: Value,
    struct_type: &Type,
) -> Result<(), CompilerError> {
    // Calculate struct size
    let struct_size = struct_type.size_in_words()
        .ok_or_else(|| CodegenError::InternalError {
            message: "Cannot determine struct size".to_string(),
            location: rcc_common::SourceLocation::new_simple(0, 0),
        })?;
    
    // Copy each word from source to destination
    for offset in 0..struct_size {
        let offset_val = Value::Constant(offset as i64);
        
        // Calculate source address (source pointer + offset)
        let src_addr = gen.builder.build_pointer_offset(
            src_ptr.clone(),
            offset_val.clone(),
            IrType::I16, // Copy word by word
        )?;
        
        // Calculate destination address (destination pointer + offset)
        let dst_addr = gen.builder.build_pointer_offset(
            dst_ptr.clone(),
            offset_val,
            IrType::I16,
        )?;
        
        // Load from source
        let word = gen.builder.build_load(src_addr, IrType::I16)?;
        
        // Store to destination
        gen.builder.build_store(Value::Temp(word), dst_addr)?;
    }
    
    Ok(())
}

pub fn generate_assignment(
    gen: &mut TypedExpressionGenerator,
    lhs: &TypedExpr,
    rhs: &TypedExpr,
) -> Result<Value, CompilerError> {
    let lhs_addr = generate_lvalue_address(gen, lhs)?;
    let lhs_type = lhs.get_type();
    let rhs_type = rhs.get_type();
    
    // Check if we're assigning structs
    if matches!(lhs_type, Type::Struct { .. }) && matches!(rhs_type, Type::Struct { .. }) {
        // For struct assignment, we need to copy all fields
        // Generate the RHS - for structs, this will return a pointer
        let rhs_val = gen.generate(rhs)?;
        
        // Copy struct contents
        copy_struct(gen, rhs_val.clone(), lhs_addr, &lhs_type)?;
        
        // Return the RHS pointer as the result of the assignment expression
        Ok(rhs_val)
    } else {
        // For non-struct types, generate and store normally
        let rhs_val = gen.generate(rhs)?;
        gen.builder.build_store(rhs_val.clone(), lhs_addr)?;
        Ok(rhs_val)
    }
}
