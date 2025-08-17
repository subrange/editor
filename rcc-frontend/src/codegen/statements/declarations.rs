//! Variable declaration code generation

use super::TypedStatementGenerator;
use crate::ir::{IrType, Value};
use crate::typed_ast::{TypedExpr};
use crate::types::{Type, BankTag};
use crate::codegen::{VarInfo, types::convert_type};
use crate::CompilerError;

// Helper function for convert_type with default location
fn convert_type_default(ast_type: &Type) -> Result<IrType, CompilerError> {
    convert_type(ast_type, rcc_common::SourceLocation::new_simple(0, 0))
}

pub fn generate_declaration(
    gen: &mut TypedStatementGenerator,
    name: &str,
    var_type: &Type,
    initializer: Option<&TypedExpr>,
) -> Result<(), CompilerError> {
    // Handle incomplete arrays with initializers
    // If we have arr[] = {1,2,3}, we need to infer the size from the initializer
    let actual_type = if let Type::Array { element_type, size: None } = var_type {
        if let Some(TypedExpr::ArrayInitializer { elements, .. }) = initializer {
            // Create a complete array type with the size from the initializer
            Type::Array {
                element_type: element_type.clone(),
                size: Some(elements.len() as u64),
            }
        } else {
            // No initializer for incomplete array - keep it as is
            var_type.clone()
        }
    } else {
        var_type.clone()
    };
    
    let ir_type = convert_type_default(&actual_type)?;
    
    // Allocate stack space for the variable
    // For arrays, we need to pass the array size as the count
    let var_addr = match &ir_type {
        IrType::Array { size, element_type } => {
            // For arrays, allocate space for all elements
            // Pass the element type and the count
            gen.builder.build_alloca(element_type.as_ref().clone(), Some(Value::Constant(*size as i64)))?
        }
        _ => {
            // For non-arrays, allocate a single element
            gen.builder.build_alloca(ir_type.clone(), None)?
        }
    };
    
    // Track the variable
    // For arrays, the allocated type is a pointer to the element type
    let tracked_type = match &ir_type {
        IrType::Array { element_type, .. } => {
            // Arrays decay to pointers to their element type
            IrType::FatPtr(element_type.clone())
        }
        _ => ir_type.clone()
    };
    
    let var_info = VarInfo {
        value: var_addr.clone(),
        ir_type: tracked_type,
        bank: Some(BankTag::Stack),
    };
    gen.variables.insert(name.to_string(), var_info);
    
    // Handle array variables specially
    if matches!(var_type, Type::Array { .. }) {
        gen.array_variables.insert(name.to_string());
    }
    
    // Initialize if needed
    if let Some(init_expr) = initializer {
        match init_expr {
            TypedExpr::ArrayInitializer { elements, .. } => {
                // For array initializers, store each element individually
                for (i, elem_expr) in elements.iter().enumerate() {
                    let mut expr_gen = gen.create_expression_generator();
                    let elem_val = expr_gen.generate(elem_expr)?;
                    
                    // Calculate element address using GEP
                    let index_val = Value::Constant(i as i64);
                    let elem_addr = gen.builder.build_pointer_offset(
                        var_addr.clone(),
                        index_val,
                        ir_type.clone(),
                    )?;
                    
                    // Store the element
                    gen.builder.build_store(elem_val, elem_addr)?;
                }
            }
            _ => {
                // For non-array initializers, generate and store normally
                let mut expr_gen = gen.create_expression_generator();
                let init_val = expr_gen.generate(init_expr)?;
                gen.builder.build_store(init_val, var_addr)?;
            }
        }
    }
    
    Ok(())
}