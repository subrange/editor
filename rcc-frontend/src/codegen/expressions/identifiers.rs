//! Identifier and variable reference code generation

use super::TypedExpressionGenerator;
use crate::ir::{Value, FatPointer};
use crate::types::BankTag;
use crate::CompilerError;

pub fn generate_identifier(
    gen: &mut TypedExpressionGenerator,
    name: &str,
) -> Result<Value, CompilerError> {
    if let Some(var_info) = gen.variables.get(name) {
        if gen.array_variables.contains(name) {
            // For arrays, return the address (as fat pointer if needed)
            Ok(var_info.as_fat_ptr())
        } else if gen.parameter_variables.contains(name) {
            // For parameters, load the value
            let result = gen
                .builder
                .build_load(var_info.value.clone(), var_info.ir_type.clone())?;
            
            // If it's a pointer type, we need to wrap the loaded value as a FatPtr
            // For pointer parameters, we use Mixed bank to indicate runtime-determined bank
            if var_info.ir_type.is_pointer() {
                Ok(Value::FatPtr(FatPointer {
                    addr: Box::new(Value::Temp(result)),
                    bank: BankTag::Mixed,  // Runtime-determined bank
                }))
            } else {
                Ok(Value::Temp(result))
            }
        } else {
            // For regular variables, load the value
            let result = gen
                .builder
                .build_load(var_info.value.clone(), var_info.ir_type.clone())?;
            
            // If it's a pointer type, wrap it in FatPtr to preserve bank information
            if var_info.ir_type.is_pointer() {
                // For local pointer variables that have been loaded, use Mixed bank
                // The backend will track the actual bank from the load instruction
                Ok(Value::FatPtr(FatPointer {
                    addr: Box::new(Value::Temp(result)),
                    bank: BankTag::Mixed,  // Runtime-determined bank
                }))
            } else {
                Ok(Value::Temp(result))
            }
        }
    } else {
        // Check if it's a function
        if gen.module.get_function(name).is_some() {
            Ok(Value::Function(name.to_string()))
        } else {
            // Otherwise, assume it's a global variable
            Ok(Value::Global(name.to_string()))
        }
    }
}