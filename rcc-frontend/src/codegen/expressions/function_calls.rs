//! Function call code generation

use super::TypedExpressionGenerator;
use crate::ir::{IrType, Value};
use crate::typed_ast::TypedExpr;
use crate::CompilerError;

pub fn generate_function_call(
    gen: &mut TypedExpressionGenerator,
    function: &TypedExpr,
    arguments: &[TypedExpr],
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
    
    // TODO: Get proper return type
    let result = gen.builder.build_call(func_val, arg_vals, IrType::I16)?;
    Ok(result.map(Value::Temp).unwrap_or(Value::Constant(0)))
}