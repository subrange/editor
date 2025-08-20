//! Function call code generation

use super::{TypedExpressionGenerator, convert_type_default};
use crate::ir::{Value, FatPointer};
use crate::typed_ast::TypedExpr;
use crate::types::{Type, BankTag};
use crate::CompilerError;

pub fn generate_function_call(
    gen: &mut TypedExpressionGenerator,
    function: &TypedExpr,
    arguments: &[TypedExpr],
    return_type: &Type,
) -> Result<Value, CompilerError> {
    // For function calls, we need the function name directly, not its loaded value
    let func_val = match function {
        TypedExpr::Variable { name, .. } => {
            // Check if it's a known variable (function pointer) or a direct function name
            if gen.variables.contains_key(name) {
                // It's a function pointer variable, load it
                gen.generate(function)?
            } else {
                // It's a direct function name
                Value::Global(name.to_string())
            }
        }
        _ => {
            // For other expressions (like function pointers), generate normally
            gen.generate(function)?
        }
    };
    
    let mut arg_vals = Vec::new();
    for arg in arguments {
        arg_vals.push(gen.generate(arg)?);
    }
    
    // Get the proper return type
    let ir_return_type = convert_type_default(return_type)?;
    let result = gen.builder.build_call(func_val, arg_vals, ir_return_type)?;
    
    // Handle the return value based on type
    match result {
        Some(temp_id) => {
            // If the return type is a pointer, wrap it in a FatPointer with Mixed bank
            // Mixed is used for pointers that can come from different sources
            if matches!(return_type, Type::Pointer { .. }) {
                Ok(Value::FatPtr(FatPointer {
                    addr: Box::new(Value::Temp(temp_id)),
                    bank: BankTag::Mixed,
                }))
            } else if matches!(return_type, Type::Struct { .. }) {
                // For struct returns, the function returns a pointer to the struct
                // Wrap it as a FatPtr with Stack bank
                Ok(Value::FatPtr(FatPointer {
                    addr: Box::new(Value::Temp(temp_id)),
                    bank: BankTag::Stack,
                }))
            } else {
                Ok(Value::Temp(temp_id))
            }
        }
        None => Ok(Value::Constant(0)), // void return
    }
}