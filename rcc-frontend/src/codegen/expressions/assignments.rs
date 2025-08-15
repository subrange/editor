//! Assignment operation code generation

use super::TypedExpressionGenerator;
use super::unary_ops::generate_lvalue_address;
use crate::ir::{Value, IrType};
use crate::typed_ast::TypedExpr;
use crate::types::Type;
use crate::CompilerError;

pub fn generate_assignment(
    gen: &mut TypedExpressionGenerator,
    lhs: &TypedExpr,
    rhs: &TypedExpr,
) -> Result<Value, CompilerError> {
    let lhs_addr = generate_lvalue_address(gen, lhs)?;
    
    // Check if we're assigning a struct
    match rhs.get_type() {
        Type::Struct { fields, .. } => {
            // For struct assignment, we need to copy field by field
            // First get the address of the rhs if it's not already one
            let rhs_addr = match rhs {
                TypedExpr::Variable { .. } => generate_lvalue_address(gen, rhs)?,
                _ => gen.generate(rhs)?
            };
            
            // Calculate total size in words
            let mut total_size = 0;
            for field in fields {
                total_size += field.field_type.size_in_words().unwrap_or(1);
            }
            
            // Use memcpy to copy the struct
            // For now, we'll do a simple field-by-field copy
            for i in 0..total_size {
                // Load from source
                let src_offset = gen.builder.build_pointer_offset(
                    rhs_addr.clone(),
                    Value::Constant(i as i64),
                    IrType::I16
                )?;
                let value_id = gen.builder.build_load(src_offset, IrType::I16)?;
                let value = Value::Temp(value_id);
                
                // Store to destination
                let dst_offset = gen.builder.build_pointer_offset(
                    lhs_addr.clone(),
                    Value::Constant(i as i64),
                    IrType::I16
                )?;
                gen.builder.build_store(value, dst_offset)?;
            }
            
            Ok(lhs_addr)
        }
        _ => {
            // For non-struct types, use the original logic
            let rhs_val = gen.generate(rhs)?;
            gen.builder.build_store(rhs_val.clone(), lhs_addr)?;
            Ok(rhs_val)
        }
    }
}