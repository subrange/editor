//! Common helper functions for instruction lowering
//! 
//! This module contains shared utilities used across different instruction
//! lowering implementations.

use rcc_frontend::ir::{Value, FatPointer};
use rcc_frontend::BankTag;
use crate::v2::regmgmt::{RegisterPressureManager, BankInfo};
use crate::v2::naming::NameGenerator;
use crate::v2::globals::GlobalManager;
use rcc_codegen::{Reg, AsmInst};
use rcc_common::TempId;
use log::warn;

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
            // This should never happen - globals should be resolved in lower.rs
            panic!("Unexpected Value::Global('{}') - should have been resolved to FatPtr in lower.rs", name);
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
        Value::ConstantArray(_) => {
            panic!("Cannot load constant array into register - arrays should be initialized separately");
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
        Value::ConstantArray(_) => 0, // Arrays don't go in registers
        Value::Temp(_) => 1,         // Temps need a register
        Value::Global(_) => panic!("Value::Global should be resolved in lower.rs"),
        Value::FatPtr(_) => 2,       // Fat pointers need 2 registers (addr + bank)
        Value::Function(_) => 1,     // Function addresses need a register
        Value::Undef => 0,           // Undefined values don't need registers
    }
}

/// Convert BankInfo to the actual bank register
/// 
/// DEPRECATED: This function will panic on NamedValue bank info.
/// Use get_bank_register_with_mgr instead which properly handles reloading.
/// 
/// This helper converts abstract bank information to concrete register:
/// - Global -> GP register
/// - Stack -> SB register  
/// - Register(r) -> the dynamic register r
/// 
/// WARNING: For Register(r), this assumes r is still valid. If the register
/// might have been spilled, the value needs to be reloaded first!
/// For NamedValue, this will panic - use get_bank_register_with_mgr instead.
#[deprecated(note = "Use get_bank_register_with_mgr instead to handle NamedValue properly")]
pub fn get_bank_register(bank_info: &BankInfo) -> Reg {
    bank_info.to_register()
}

/// Get bank register with proper reloading support
/// 
/// This version handles NamedValue bank info by using the register manager
/// to get/reload the bank value as needed.
pub fn get_bank_register_with_mgr(
    bank_info: &BankInfo,
    mgr: &mut RegisterPressureManager
) -> Reg {
    match bank_info {
        BankInfo::Global => Reg::Gp,
        BankInfo::Stack => Reg::Sb,
        BankInfo::Register(reg) => *reg,
        BankInfo::NamedValue(name) => {
            // Get the register for this named value, which will reload if spilled
            mgr.get_register(name.clone())
        }
    }
}

/// Resolve a global variable name to a FatPointer with its address
/// 
/// Converts Value::Global to Value::FatPtr with the resolved address
/// from the global manager.
pub fn resolve_global_to_fatptr(
    name: &str, 
    global_manager: &GlobalManager
) -> Result<Value, String> {
    if let Some(info) = global_manager.get_global_info(name) {
        Ok(Value::FatPtr(FatPointer {
            addr: Box::new(Value::Constant(info.address as i64)),
            bank: BankTag::Global,
        }))
    } else {
        Err(format!("Unknown global variable: {}", name))
    }
}

/// Resolve any BankTag to concrete BankInfo
/// 
/// Converts BankTag enum values to BankInfo, handling Mixed tags specially
/// by resolving them through the register manager.
pub fn resolve_bank_tag_to_info(
    bank_tag: &BankTag,
    fp: &FatPointer,
    mgr: &RegisterPressureManager,
    naming: &NameGenerator
) -> BankInfo {
    match bank_tag {
        BankTag::Global => BankInfo::Global,
        BankTag::Stack => BankInfo::Stack,
        BankTag::Mixed => resolve_mixed_bank(fp, mgr, naming),
        other => panic!("Helpers: Unsupported bank type for fat pointer: {:?}", other),
    }
}

/// Resolve a Mixed bank tag to concrete BankInfo
/// 
/// For Mixed bank tags, the actual bank is determined at runtime and
/// should be tracked in the register manager for the underlying temp.
pub fn resolve_mixed_bank(
    fp: &FatPointer,
    mgr: &RegisterPressureManager,
    naming: &NameGenerator
) -> BankInfo {
    match fp.addr.as_ref() {
        Value::Temp(t) => {
            let temp_name = naming.temp_name(*t);
            mgr.get_pointer_bank(&temp_name)
                .unwrap_or_else(|| {
                    panic!("HELPERS: COMPILER BUG: No bank info for Mixed pointer '{}'. All pointers must have tracked bank information!", temp_name);
                })
        }
        Value::Constant(_) => {
            panic!("FatPtr with BankTag::Mixed cannot have a constant address")
        }
        other => {
            warn!("Unexpected address type for Mixed fat ptr: {:?}", other);
            BankInfo::Stack
        }
    }
}

/// Load a constant into a register
/// 
/// Allocates a register and generates an LI instruction to load the constant.
/// Returns the register and the instruction to emit.
pub fn load_constant_to_register(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    value: i64,
    result_temp: TempId
) -> (Reg, Vec<AsmInst>) {
    let temp_name = naming.const_for_temp(result_temp);
    let reg = mgr.get_register(temp_name);
    let mut insts = mgr.take_instructions();
    insts.push(AsmInst::Li(reg, value as i16));
    (reg, insts)
}

/// Materialize a bank tag to a register
/// 
/// Takes a BankTag and produces a register containing the bank value.
/// Returns (register, instructions, owned) where owned indicates if 
/// the register should be freed after use.
pub fn materialize_bank_to_register(
    bank_tag: &BankTag,
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    context: &str
) -> (Reg, Vec<AsmInst>, bool) {
    let mut insts = vec![];
    
    match bank_tag {
        BankTag::Global => {
            // Copy GP into a temp register
            let name = naming.temp_with_context(context, "bank_global");
            let r = mgr.get_register(name);
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Add(r, Reg::Gp, Reg::R0));
            (r, insts, true)
        }
        BankTag::Stack => {
            // Copy SB into a temp register
            let name = naming.temp_with_context(context, "bank_stack");
            let r = mgr.get_register(name);
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Add(r, Reg::Sb, Reg::R0));
            (r, insts, true)
        }
        BankTag::Mixed => {
            panic!("Cannot materialize Mixed bank tag without additional context - use resolve_mixed_bank first")
        }
        _ => {
            panic!("HELPERS: Unexpected bank tag type: {:?}", bank_tag);
        }
    }
}

/// Get the address register from a pointer value
/// 
/// Handles Value::Temp, Value::FatPtr, and constants within FatPtr.
/// Returns (address_register, pointer_name_for_bank_lookup, instructions).
pub fn get_pointer_address_and_name(
    ptr_value: &Value,
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    result_temp: TempId
) -> (Reg, String, Vec<AsmInst>) {
    let mut insts = vec![];
    
    match ptr_value {
        Value::Temp(t) => {
            let name = naming.temp_name(*t);
            let reg = mgr.get_register(name.clone());
            insts.extend(mgr.take_instructions());
            (reg, name, insts)
        }
        Value::FatPtr(fp) => {
            let addr_reg = match fp.addr.as_ref() {
                Value::Temp(t) => {
                    let name = naming.temp_name(*t);
                    let reg = mgr.get_register(name);
                    insts.extend(mgr.take_instructions());
                    reg
                }
                Value::Constant(c) => {
                    let temp_name = naming.const_for_temp(result_temp);
                    let reg = mgr.get_register(temp_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Li(reg, *c as i16));
                    reg
                }
                Value::Global(_) => {
                    panic!("Value::Global in FatPtr address should have been resolved in lower.rs")
                }
                _ => panic!("Invalid fat pointer address type: {:?}", fp.addr)
            };
            
            // Generate a unique key for this pointer's bank info
            let ptr_name = naming.pointer_bank_key(&format!("ptr_{}", result_temp));
            (addr_reg, ptr_name, insts)
        }
        Value::Global(_) => {
            panic!("Value::Global should have been resolved to FatPtr in lower.rs")
        }
        _ => panic!("Invalid pointer value type: {:?}", ptr_value)
    }
}

/// Resolve any Value::Global references within a Value to FatPtr
/// 
/// This is used to canonicalize values before processing in instruction lowering.
pub fn canonicalize_value(
    value: &Value,
    global_manager: &GlobalManager
) -> Result<Value, String> {
    match value {
        Value::Global(name) => resolve_global_to_fatptr(name, global_manager),
        Value::FatPtr(fp) => {
            // Check if the address contains a Global that needs resolution
            match fp.addr.as_ref() {
                Value::Global(name) => {
                    if let Some(info) = global_manager.get_global_info(name) {
                        Ok(Value::FatPtr(FatPointer {
                            addr: Box::new(Value::Constant(info.address as i64)),
                            bank: fp.bank.clone(),
                        }))
                    } else {
                        Err(format!("Unknown global variable in FatPtr: {}", name))
                    }
                }
                _ => Ok(value.clone())
            }
        }
        _ => Ok(value.clone())
    }
}