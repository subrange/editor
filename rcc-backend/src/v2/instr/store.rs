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
            
            (reg, bank_reg.is_some(), bank_reg)
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
                    // For global addresses, we need to load the address
                    trace!("  Storing fat pointer global address: {}", name);
                    let addr_reg_name = naming.store_global_addr(name);
                    let addr_reg = mgr.get_register(addr_reg_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Comment(format!("Load address of global {}", name)));
                    let label = naming.store_global_label(name);
                    insts.push(AsmInst::Label(label));
                    insts.push(AsmInst::Li(addr_reg, 0)); // Placeholder for linker
                    addr_reg
                }
                _ => panic!("Invalid fat pointer address type: {:?} in STORE", fp.addr),
            };
            
            // Get bank value into a register
            let bank_reg_name = naming.store_fatptr_bank();
            let bank_reg = mgr.get_register(bank_reg_name);
            insts.extend(mgr.take_instructions());
            let bank_val = match fp.bank {
                BankTag::Global => 0,
                BankTag::Stack => 1,
                _ => panic!("Unsupported bank type for fat pointer: {:?}", fp.bank),
            };
            insts.push(AsmInst::Li(bank_reg, bank_val));
            
            (addr_reg, true, Some(bank_reg))
        }
        Value::Global(name) => {
            // For storing a global address
            trace!("  Storing global address: {}", name);
            let addr_reg_name = naming.store_global_addr(name);
            let addr_reg = mgr.get_register(addr_reg_name);
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Comment(format!("Load address of global {}", name)));
            let label = naming.store_global_label(name);
            insts.push(AsmInst::Label(label));
            insts.push(AsmInst::Li(addr_reg, 0)); // Placeholder for linker
            (addr_reg, false, None)
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
                _ => panic!("Invalid fat pointer address type: {:?} in STORE", fp.addr),
            };
            
            // Set bank info for the pointer
            let bank_info = match fp.bank {
                BankTag::Global => BankInfo::Global,
                BankTag::Stack => BankInfo::Stack,
                _ => panic!("BE: Unsupported bank type for fat pointer: {:?}", fp.bank),
            };
            let dest_ptr_key = naming.pointer_bank_key("dest_ptr");
            mgr.set_pointer_bank(dest_ptr_key.clone(), bank_info);
            
            (addr_reg, dest_ptr_key)
        }
        Value::Global(name) => {
            // Global variables are in bank 0
            trace!("  Storing to global: {}", name);
            mgr.set_pointer_bank(name.clone(), BankInfo::Global);
            
            let addr_reg_name = naming.store_global_addr(name);
            let addr_reg = mgr.get_register(addr_reg_name);
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Comment(format!("Load address of global {}", name)));
            let label = naming.store_global_label(name);
            insts.push(AsmInst::Label(label));
            insts.push(AsmInst::Li(addr_reg, 0)); // Placeholder for linker
            
            (addr_reg, name.clone())
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
        if let Some(bank_reg) = ptr_bank_reg {
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
            
            // Free temporary registers
            mgr.free_register(bank_addr_reg);
            mgr.free_register(bank_reg);
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