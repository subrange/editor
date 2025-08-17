//! Global variable code generation

use std::collections::{HashMap, HashSet};
use crate::ir::{Value, IrType, IrBuilder, Module, GlobalVariable, Linkage};
use crate::typed_ast::TypedExpr;
use crate::types::{Type, BankTag};
use crate::CompilerError;
use super::super::{VarInfo, expressions::TypedExpressionGenerator, types::complete_type_from_initializer};
use super::utils::convert_type_default;

/// Generate IR for a global variable
pub fn generate_global_variable(
    builder: &mut IrBuilder,
    module: &mut Module,
    variables: &mut HashMap<String, VarInfo>,
    array_variables: &mut HashSet<String>,
    parameter_variables: &HashSet<String>,
    string_literals: &mut HashMap<String, String>,
    next_string_id: &mut u32,
    name: &str,
    var_type: &Type,
    initializer: Option<&TypedExpr>,
) -> Result<(), CompilerError> {
    // Handle initializer first to determine array size if needed
    let init_value = if let Some(init_expr) = initializer {
        // We need to generate the initializer value
        // For global variables, only constant initializers are allowed
        let mut expr_gen = TypedExpressionGenerator {
            builder,
            module,
            variables,
            array_variables,
            parameter_variables,
            string_literals,
            next_string_id,
        };
        
        match expr_gen.generate(init_expr) {
            Ok(value) => match value {
                Value::Constant(_) | Value::ConstantArray(_) => Some(value),
                _ => None, // Non-constant initializers not supported for globals
            }
            Err(_) => None,
        }
    } else {
        None
    };
    
    // Use the general helper to complete incomplete types
    let completed_type = complete_type_from_initializer(var_type, initializer);
    let ir_type = convert_type_default(&completed_type)?;
    
    let global = GlobalVariable {
        name: name.to_string(),
        var_type: ir_type.clone(),
        is_constant: false,
        initializer: init_value,
        linkage: Linkage::External,
        symbol_id: None,
    };
    
    module.add_global(global);
    
    // Add to variables map for later reference
    variables.insert(name.to_string(), VarInfo {
        value: Value::Global(name.to_string()),
        ir_type,
        bank: Some(BankTag::Global),
    });
    
    // Track global arrays
    if matches!(var_type, Type::Array { .. }) {
        array_variables.insert(name.to_string());
    }
    
    Ok(())
}