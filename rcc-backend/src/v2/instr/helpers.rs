//! Common helper functions for instruction lowering
//! 
//! This module contains shared utilities used across different instruction
//! lowering implementations.

use rcc_frontend::ir::Value;
use crate::v2::regmgmt::RegisterPressureManager;
use crate::v2::naming::NameGenerator;
use rcc_codegen::{Reg, AsmInst};

/// Get register for a value
/// 
/// This function allocates or retrieves a register for the given value.
/// It handles all value types including constants, temporaries, globals,
/// function addresses, and fat pointers.
/// 
/// # Parameters
/// - `mgr`: Register pressure manager for allocation and spilling
/// - `naming`: Name generator for unique temporary names
/// - `value`: The value to get a register for
/// 
/// # Returns
/// The register containing or that will contain the value
/// 
/// # Panics
/// Panics if the value is Undef
pub fn get_value_register(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    value: &Value,
) -> Reg {
    match value {
        Value::Temp(id) => {
            let name = naming.temp_name(*id);
            mgr.get_register(name)
        }
        Value::Constant(val) => {
            // Use the RegisterPressureManager's get_value_register which handles constants properly
            // This will emit the LI instruction internally
            mgr.get_value_register(value)
        }
        Value::Global(name) => {
            let global_name = naming.load_global_addr(name);
            mgr.get_register(global_name)
        }
        Value::Function(name) => {
            // Function addresses are like globals
            let func_name = naming.func_addr(name);
            let reg = mgr.get_register(func_name);
            reg
        }
        Value::FatPtr(fp) => {
            // For most operations, we just need the address part
            // The bank component is handled separately when needed
            get_value_register(mgr, naming, &fp.addr)
        }
        Value::Undef => {
            panic!("Cannot use undefined value in instruction");
        }
    }
}

/// Calculate register need for a single value
/// 
/// This function estimates how many registers are needed for a value
/// without actually allocating them.
/// 
/// # Parameters
/// - `value`: The value to estimate register needs for
/// 
/// # Returns
/// Number of registers needed (0, 1, or 2)
pub fn calculate_value_need(value: &Value) -> usize {
    match value {
        Value::Constant(_) => 1,    // Need to load into register
        Value::Temp(_) => 1,         // Temps need a register
        Value::Global(_) => 1,       // Need to load address
        Value::FatPtr(_) => 2,       // Fat pointers need 2 registers (addr + bank)
        Value::Function(_) => 1,     // Function addresses need a register
        Value::Undef => 0,           // Undefined values don't need registers
    }
}