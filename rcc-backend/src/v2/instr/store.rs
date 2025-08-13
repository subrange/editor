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

            // Check if this temp is a pointer and get its bank
            let bank_reg = if let Some(bank_info) = mgr.get_pointer_bank(&name) {
                debug!("  Value {} is a pointer with bank {:?}", name, bank_info);
                match bank_info {
                    BankInfo::Register(r) => Some(r),
                    _ => None,
                }
            } else {
                None
            };

            (reg, bank_reg.is_some(), bank_reg.map(|r| (r, false)))
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
                    // Store constant bank 0
                    let name = naming.store_fatptr_bank();
                    let r = mgr.get_register(name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Li(r, 0));
                    (r, true)
                }
                BankTag::Stack => {
                    // Store constant bank 1
                    let name = naming.store_fatptr_bank();
                    let r = mgr.get_register(name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Li(r, 1));
                    (r, true)
                }
                BankTag::Mixed => {
                    // Mixed means the bank is determined at runtime.
                    // We must source it from the tracked bank of the addr temp (set by calling convention),
                    // or synthesize it from GP/SB if the manager tracked it that way.
                    match fp.addr.as_ref() {
                        Value::Temp(t) => {
                            let temp_name = naming.temp_name(*t);
                            match mgr.get_pointer_bank(&temp_name) {
                                Some(BankInfo::Register(r)) => {
                                    // Use the dynamic bank register directly; we don't own it
                                    (r, false)
                                }
                                Some(BankInfo::Global) => {
                                    // Copy GP into a temp so we can store it
                                    let name = naming.store_fatptr_bank();
                                    let rtemp = mgr.get_register(name);
                                    insts.extend(mgr.take_instructions());
                                    insts.push(AsmInst::Add(rtemp, Reg::Gp, Reg::R0));
                                    (rtemp, true)
                                }
                                Some(BankInfo::Stack) => {
                                    // Copy SB into a temp so we can store it
                                    let name = naming.store_fatptr_bank();
                                    let rtemp = mgr.get_register(name);
                                    insts.extend(mgr.take_instructions());
                                    insts.push(AsmInst::Add(rtemp, Reg::Sb, Reg::R0));
                                    (rtemp, true)
                                }
                                None => {
                                    panic!("STORE: FatPtr with BankTag::Mixed but no tracked bank for {}", temp_name);
                                }
                            }
                        }
                        Value::Constant(_) => {
                            // A Mixed bank cannot accompany a constant address — we lack a runtime bank source
                            panic!("STORE: FatPtr with BankTag::Mixed cannot have a constant address");
                        }
                        other => {
                            warn!("STORE: Unexpected address type for Mixed fat ptr: {:?}", other);
                            // Fall back to SB copy to avoid crashing; allocate a temp and copy SB
                            let name = naming.store_fatptr_bank();
                            let rtemp = mgr.get_register(name);
                            insts.extend(mgr.take_instructions());
                            insts.push(AsmInst::Add(rtemp, Reg::Sb, Reg::R0));
                            (rtemp, true)
                        }
                    }
                }
                other => panic!("Unsupported bank type for fat pointer: {:?}", other),
            };
            (addr_reg, true, Some((bank_reg, bank_owned)))
        }
        _ => {
            warn!("  Invalid value for store: {:?}", value);
            panic!("Invalid value for store")
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
            let bank_info = match fp.bank {
                BankTag::Global => BankInfo::Global,
                BankTag::Stack  => BankInfo::Stack,
                BankTag::Mixed => {
                    // For Mixed, the bank is determined at runtime. We expect the
                    // address component to be a temp whose bank has been tracked
                    // by the calling convention (load_param). Reuse that info.
                    match fp.addr.as_ref() {
                        Value::Temp(t) => {
                            let temp_name = naming.temp_name(*t);
                            if let Some(info) = mgr.get_pointer_bank(&temp_name) {
                                info
                            } else {
                                warn!("STORE: No bank info for {}, defaulting to Stack for Mixed fat ptr", temp_name);
                                BankInfo::Stack
                            }
                        }
                        Value::Constant(_) => {
                            // A Mixed bank with a constant address cannot carry a runtime bank.
                            panic!("STORE: FatPtr with BankTag::Mixed cannot have a constant address");
                        }
                        other => {
                            warn!("STORE: Unexpected address type for Mixed fat ptr: {:?}", other);
                            BankInfo::Stack
                        }
                    }
                }
                other => panic!("Unsupported bank type for fat pointer: {:?}", other),
            };

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
            warn!("  No bank info for pointer {}, defaulting to Stack", dest_ptr_name);
            BankInfo::Stack
        });
    
    debug!("  Destination pointer {} has bank info: {:?}", dest_ptr_name, dest_bank_info);
    
    let dest_bank_reg = match dest_bank_info {
        BankInfo::Global => {
            trace!("  Using GP for global bank");
            Reg::Gp  // Global pointer register for globals
        }
        BankInfo::Stack => {
            trace!("  Using R13 (SB) for stack bank");
            Reg::Sb  // SB - Stack Bank register
        }
        BankInfo::Register(r) => {
            trace!("  Using {:?} for dynamic bank", r);
            r
        }
    };
    
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
        _ => {}
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