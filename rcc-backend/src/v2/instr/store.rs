//! Store instruction lowering for V2 backend
//! 
//! Handles storing values to memory with proper bank management.
//! Supports both scalar stores and fat pointer stores (2-component).

use rcc_frontend::ir::{Value};
use crate::v2::regmgmt::{RegisterPressureManager, BankInfo};
use crate::v2::naming::NameGenerator;
use rcc_codegen::{AsmInst, Reg};
use log::{debug, trace, warn};
use rcc_frontend::BankTag;
use super::helpers::{resolve_mixed_bank, resolve_bank_tag_to_info, get_bank_register_with_mgr, materialize_bank_to_register};

/// Lower a Store instruction to assembly
/// 
/// # Parameters
/// - `mgr`: Register pressure manager for allocation and spilling
/// - `naming`: Name generator for unique temporary names
/// - `value`: Value to store
/// - `ptr_value`: Pointer value to store to (must contain bank info)
/// 
/// # Returns
/// Vector of assembly instructions for the store operation
pub fn lower_store(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    value: &Value,
    ptr_value: &Value,
) -> Vec<AsmInst> {
    debug!("lower_store: value={:?}, ptr={:?}", value, ptr_value);
    
    let mut insts = vec![];
    
    // Step 1: Get the value to store into a register
    // The third tuple item is Option<(Reg, bool)>: the bank register and whether we own it (must free)
    let (src_reg, is_pointer, ptr_bank_reg) = match value {
        Value::Temp(t) => {
            let name = naming.temp_name(*t);
            trace!("  Storing temp value: {}", name);
            let reg = mgr.get_register(name.clone());

            let bank_src: Option<(Reg, bool)> = mgr.get_pointer_bank(&name)
                .map(|info| (get_bank_register_with_mgr(&info, mgr), false));

            (reg, bank_src.is_some(), bank_src)

            // (reg, bank_reg.is_some(), bank_reg.map(|r| (r, false)))
        }
        Value::Constant(c) => {
            // Load constant into a register
            trace!("  Storing constant value: {}", c);
            let temp_reg_name = naming.store_const_value();
            let temp_reg = mgr.get_register(temp_reg_name);
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Li(temp_reg, *c as i16));
            (temp_reg, false, None)
        }
        Value::Global(name) => {
            // This should never be reached - globals should be resolved in lower.rs
            panic!("Unexpected Value::Global('{}') as store value - should have been resolved in lower.rs", name);
        }
        Value::FatPtr(fp) => {
            // Store fat pointer - need to handle both components
            trace!("  Storing fat pointer with bank {:?}", fp.bank);

            // Get address component
            let addr_reg = match fp.addr.as_ref() {
                Value::Temp(t) => {
                    let name = naming.temp_name(*t);
                    mgr.get_register(name)
                }
                Value::Constant(c) => {
                    let temp_reg_name = naming.store_fatptr_addr();
                    let temp_reg = mgr.get_register(temp_reg_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Li(temp_reg, *c as i16));
                    temp_reg
                }
                Value::Global(name) => {
                    // This should never happen - globals should be resolved to constants in lower.rs
                    panic!("Unexpected Value::Global('{}') in FatPtr address - should have been resolved to Constant in lower.rs", name);
                }
                _ => panic!("Invalid fat pointer address type: {:?} in STORE", fp.addr),
            };

            // Decide how to materialize the bank component for the value we are storing
            // Return (bank_reg, owned) — owned = true if we allocated a temp that must be freed here
            let (bank_reg, bank_owned) = match fp.bank {
                BankTag::Global => {
                    let name = naming.store_fatptr_bank();
                    let r = mgr.get_register(name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Add(r, Reg::Gp, Reg::R0));
                    (r, true)
                }
                BankTag::Stack => {
                    let name = naming.store_fatptr_bank();
                    let r = mgr.get_register(name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Add(r, Reg::Sb, Reg::R0));
                    (r, true)
                }
                BankTag::Mixed => {
                    // Mixed means the bank is determined at runtime
                    let bank_info = resolve_mixed_bank(fp, mgr, naming);
                    match bank_info {
                        BankInfo::Register(r) => {
                            // Use the dynamic bank register directly; we don't own it
                            (r, false)
                        }
                        BankInfo::NamedValue(name) => {
                            // Get the register for this named value (may reload if spilled)
                            let r = mgr.get_register(name);
                            insts.extend(mgr.take_instructions());
                            (r, false)
                        }
                        BankInfo::Global | BankInfo::Stack => {
                            // Need to materialize the bank register value
                            let bank_tag = if matches!(bank_info, BankInfo::Global) {
                                BankTag::Global
                            } else {
                                BankTag::Stack
                            };
                            let (r, mut materialized_insts, owned) = 
                                materialize_bank_to_register(&bank_tag, mgr, naming, "store_fatptr");
                            insts.append(&mut materialized_insts);
                            (r, owned)
                        }
                    }
                }
                other => panic!("Store: Unsupported bank type for fat pointer: {:?}", other),
            };
            (addr_reg, true, Some((bank_reg, bank_owned)))
        }
        _ => {
            warn!("  Invalid value for store: {:?}", value);
            panic!("Invalid value for store: {:?}", value);
        }
    };
    
    insts.extend(mgr.take_instructions());
    
    // Step 2: Get the destination pointer address and bank
    let (dest_addr_reg, dest_ptr_name) = match ptr_value {
        Value::Temp(t) => {
            let name = naming.temp_name(*t);
            trace!("  Storing to temp pointer: {}", name);
            let reg = mgr.get_register(name.clone());
            (reg, name)
        }
        Value::FatPtr(fp) => {
            // Fat pointer has explicit address and bank
            trace!("  Storing to fat pointer with bank {:?}", fp.bank);
            let addr_reg = match fp.addr.as_ref() {
                Value::Temp(t) => {
                    let name = naming.temp_name(*t);
                    mgr.get_register(name)
                }
                Value::Constant(c) => {
                    let temp_reg_name = naming.store_dest_addr();
                    let temp_reg = mgr.get_register(temp_reg_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Li(temp_reg, *c as i16));
                    trace!("  Loaded destination address {} into {:?}", c, temp_reg);
                    temp_reg
                }
                Value::Global(name) => {
                    // This should never happen - globals should be resolved to constants in lower.rs
                    panic!("Unexpected Value::Global('{}') in FatPtr destination address - should have been resolved to Constant in lower.rs", name);
                }
                _ => panic!("Invalid fat pointer address type: {:?} in STORE", fp.addr),
            };
            
            // Set bank info for the pointer
            let bank_info = resolve_bank_tag_to_info(&fp.bank, fp, mgr, naming);

            let dest_ptr_key = naming.pointer_bank_key("dest_ptr");
            mgr.set_pointer_bank(dest_ptr_key.clone(), bank_info);
            
            (addr_reg, dest_ptr_key)
        }
        Value::Global(name) => {
            // This should never happen - globals should be resolved to FatPtr in lower.rs
            panic!("Unexpected Value::Global('{}') as store destination - should have been resolved to FatPtr in lower.rs", name);
        }
        _ => {
            warn!("  Invalid pointer value for store: {:?}", ptr_value);
            panic!("Invalid pointer value for store")
        }
    };
    
    insts.extend(mgr.take_instructions());
    
    // Step 3: Get the bank register for the destination
    let dest_bank_info = mgr.get_pointer_bank(&dest_ptr_name)
        .unwrap_or_else(|| {
            panic!("STORE: COMPILER BUG: No bank info for pointer '{}'. All pointers must have tracked bank information!", dest_ptr_name);
        });
    
    debug!("  Destination pointer {} has bank info: {:?}", dest_ptr_name, dest_bank_info);
    
    let dest_bank_reg = get_bank_register_with_mgr(&dest_bank_info, mgr);
    insts.extend(mgr.take_instructions());
    
    // Step 4: Generate STORE instruction(s)
    
    // Store the main value
    let store_inst = AsmInst::Store(src_reg, dest_bank_reg, dest_addr_reg);
    trace!("  Generated STORE: {:?}", store_inst);
    insts.push(store_inst);
    
    // If storing a fat pointer, also store the bank component
    if is_pointer {
        if let Some((bank_reg, bank_owned)) = ptr_bank_reg {
            debug!("  Storing fat pointer bank component");
            // Calculate address for bank component (addr + 1)
            let bank_addr_name = naming.store_bank_addr();
            let bank_addr_reg = mgr.get_register(bank_addr_name);
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::AddI(bank_addr_reg, dest_addr_reg, 1));
            trace!("  Bank component at address {:?} + 1", dest_addr_reg);
            // Store the bank value
            let bank_store = AsmInst::Store(bank_reg, dest_bank_reg, bank_addr_reg);
            trace!("  Generated bank STORE: {:?}", bank_store);
            insts.push(bank_store);
            // Free temporary registers we allocated in this function
            mgr.free_register(bank_addr_reg);
            if bank_owned {
                mgr.free_register(bank_reg);
            }
        }
    }
    
    // Free temporary registers if we allocated them for constants
    match value {
        Value::Constant(_) => mgr.free_register(src_reg),
        Value::FatPtr(_) => mgr.free_register(src_reg), // addr_reg
        _ => {
            // For temps, we don't free src_reg because it's still holding the value
            // and might be needed again. The register allocator will handle it.
        }
    }
    
    // Free the destination address register if it was a constant or FatPtr
    // For temps, we keep it because it's the actual pointer value
    match ptr_value {
        Value::FatPtr(fp) => {
            // If the address was a constant, we allocated a temp register
            if matches!(fp.addr.as_ref(), Value::Constant(_)) {
                mgr.free_register(dest_addr_reg);
            }
            // For temp addresses, don't free - it's still the pointer value
        }
        _ => {
            // For Value::Temp pointers, don't free - they're still valid pointers
        }
    }
    
    debug!("lower_store complete: generated {} instructions", insts.len());
    insts
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcc_frontend::ir::{FatPointer};
    use rcc_frontend::types::BankTag;
    
    #[test]
    #[should_panic(expected = "Unexpected Value::Global('test_global')")]
    fn test_panic_on_unresolved_global_as_store_value() {
        let mut mgr = RegisterPressureManager::new(10);
        let mut naming = NameGenerator::new(0);
        
        // Try to store a Global value directly - should panic
        let value = Value::Global("test_global".to_string());
        let ptr = Value::FatPtr(FatPointer {
            addr: Box::new(Value::Constant(0)),
            bank: BankTag::Global,
        });
        
        // This should panic
        lower_store(&mut mgr, &mut naming, &value, &ptr);
    }
    
    #[test]
    #[should_panic(expected = "Unexpected Value::Global('test_global') as store destination")]
    fn test_panic_on_unresolved_global_as_store_destination() {
        let mut mgr = RegisterPressureManager::new(10);
        let mut naming = NameGenerator::new(0);
        
        // Try to store to a Global directly - should panic
        let value = Value::Constant(42);
        let ptr = Value::Global("test_global".to_string());
        
        // This should panic
        lower_store(&mut mgr, &mut naming, &value, &ptr);
    }
    
    #[test]
    #[should_panic(expected = "Unexpected Value::Global('test_global') in FatPtr address")]
    fn test_panic_on_unresolved_global_in_fatptr_value() {
        let mut mgr = RegisterPressureManager::new(10);
        let mut naming = NameGenerator::new(0);
        
        // Try to store a FatPtr with Global in its address - should panic
        let value = Value::FatPtr(FatPointer {
            addr: Box::new(Value::Global("test_global".to_string())),
            bank: BankTag::Global,
        });
        let ptr = Value::FatPtr(FatPointer {
            addr: Box::new(Value::Constant(0)),
            bank: BankTag::Global,
        });
        
        // This should panic
        lower_store(&mut mgr, &mut naming, &value, &ptr);
    }
    
    #[test]
    #[should_panic(expected = "Unexpected Value::Global('test_global') in FatPtr destination")]
    fn test_panic_on_unresolved_global_in_fatptr_destination() {
        let mut mgr = RegisterPressureManager::new(10);
        let mut naming = NameGenerator::new(0);
        
        // Try to store to a FatPtr with Global in its address - should panic
        let value = Value::Constant(42);
        let ptr = Value::FatPtr(FatPointer {
            addr: Box::new(Value::Global("test_global".to_string())),
            bank: BankTag::Global,
        });
        
        // This should panic
        lower_store(&mut mgr, &mut naming, &value, &ptr);
    }
}