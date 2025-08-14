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
    let ir_type = convert_type_default(var_type)?;
    
    // Allocate stack space for the variable
    let var_addr = gen.builder.build_alloca(ir_type.clone(), None)?;
    
    // Track the variable
    let var_info = VarInfo {
        value: var_addr.clone(),
        ir_type: ir_type.clone(),
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