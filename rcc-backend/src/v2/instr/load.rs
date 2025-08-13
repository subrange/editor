//! Load instruction lowering for V2 backend
//! 
//! Handles loading values from memory with proper bank management.
//! Supports both scalar loads and fat pointer loads (2-component).

use rcc_frontend::ir::{Value, IrType as Type};
use rcc_common::TempId;
use crate::v2::regmgmt::{RegisterPressureManager, BankInfo};
use crate::v2::naming::NameGenerator;
use rcc_codegen::{AsmInst, Reg};
use log::{debug, trace, warn};
use rcc_frontend::BankTag;

/// Lower a Load instruction to assembly
/// 
/// # Parameters
/// - `mgr`: Register pressure manager for allocation and spilling
/// - `naming`: Name generator for unique temporary names
/// - `ptr_value`: Pointer value to load from (must contain bank info)
/// - `result_type`: Type of the value being loaded
/// - `result_temp`: Temp ID for the result
/// 
/// # Returns
/// Vector of assembly instructions for the load operation
pub fn lower_load(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    ptr_value: &Value,
    result_type: &Type,
    result_temp: TempId,
) -> Vec<AsmInst> {
    debug!("lower_load: ptr={:?}, type={:?}, result=t{}", ptr_value, result_type, result_temp);
    
    let mut insts = vec![];
    let result_name = naming.temp_name(result_temp);
    
    // Step 1: Get the pointer address register
    let (addr_reg, ptr_name) = match ptr_value {
        Value::Temp(t) => {
            let name = naming.temp_name(*t);
            trace!("  Loading from temp pointer: {}", name);
            let reg = mgr.get_register(name.clone());
            (reg, name)
        }
        Value::FatPtr(fp) => {
            // Fat pointer has explicit address and bank
            trace!("  Loading from fat pointer with bank {:?}", fp.bank);
            let addr_reg = match fp.addr.as_ref() {
                Value::Temp(t) => {
                    let name = naming.temp_name(*t);
                    mgr.get_register(name)
                }
                Value::Constant(c) => {
                    // Load constant address into register
                    let temp_reg_name = naming.load_const_addr(result_temp);
                    let temp_reg = mgr.get_register(temp_reg_name);
                    insts.push(AsmInst::Li(temp_reg, *c as i16));
                    trace!("  Loaded constant address {} into {:?}", c, temp_reg);
                    temp_reg
                }
                _ => {
                    warn!("  Unexpected address type in fat pointer: {:?}", fp.addr);
                    panic!("Invalid fat pointer address type in LOAD: {:?}", fp.addr);
                }
            };

            // Determine how to source the bank for this fat pointer
            // - Global/Stack: known at compile-time, record a unique key
            // - Mixed: bank is runtime-determined and should already be tracked
            //          against the *underlying temp* that holds the address
            let ptr_name = match fp.bank {
                BankTag::Global => {
                    let key = naming.load_src_ptr_bank(result_temp);
                    mgr.set_pointer_bank(key.clone(), BankInfo::Global);
                    key
                }
                BankTag::Stack => {
                    let key = naming.load_src_ptr_bank(result_temp);
                    mgr.set_pointer_bank(key.clone(), BankInfo::Stack);
                    key
                }
                BankTag::Mixed => {
                    // For Mixed, the bank lives in a register associated with the
                    // underlying temp for the address. Reuse that temp's name so the
                    // manager can provide the correct BankInfo::Register(..).
                    match fp.addr.as_ref() {
                        Value::Temp(t) => {
                            let name = naming.temp_name(*t);
                            if mgr.get_pointer_bank(&name).is_none() {
                                // This should have been set up by the parameter loader.
                                // Default to Stack to avoid crashing, but warn loudly.
                                warn!(
                                    "lower_load: FatPtr(Mixed) for '{}' but no bank recorded; defaulting to Stack",
                                    name
                                );
                                mgr.set_pointer_bank(name.clone(), BankInfo::Stack);
                            }
                            name
                        }
                        other => {
                            panic!(
                                "BE: LOAD: FatPtr(Mixed) must carry a Temp address so bank can be tracked, got: {:?}",
                                other
                            );
                        }
                    }
                }
                other => {
                    // Future-proofing: if new BankTag variants appear, don't crash the backend.
                    warn!(
                        "BE: LOAD: Unsupported bank type {:?} for fat pointer; defaulting to Stack",
                        other
                    );
                    let key = naming.load_src_ptr_bank(result_temp);
                    mgr.set_pointer_bank(key.clone(), BankInfo::Stack);
                    key
                }
            };

            (addr_reg, ptr_name)
        }
        Value::Global(name) => {
            // This should never happen - globals should be resolved to FatPtr in lower.rs
            panic!("Unexpected Value::Global('{}') as load source - should have been resolved to FatPtr in lower.rs", name);
        }
        _ => {
            warn!("  Invalid pointer value for load: {:?}", ptr_value);
            panic!("Invalid pointer value for load")
        }
    };
    
    // Step 2: Get the bank register based on pointer's bank info
    let bank_info = mgr.get_pointer_bank(&ptr_name)
        .unwrap_or_else(|| {
            warn!("  No bank info for pointer {}, defaulting to Stack", ptr_name);
            BankInfo::Stack
        });
    
    debug!("  Pointer {} has bank info: {:?}", ptr_name, bank_info);
    
    let bank_reg = match bank_info {
        BankInfo::Global => {
            trace!("  Using GP for global bank");
            Reg::Gp  // Global pointer register for globals
        }
        BankInfo::Stack => {
            trace!("  Using SB for stack bank");
            Reg::Sb  // SB - Stack Bank register
        }
        BankInfo::Register(r) => {
            trace!("  Using {:?} for dynamic bank", r);
            r
        }
    };
    
    // Step 3: Allocate destination register and generate LOAD instruction
    let dest_reg = mgr.get_register(result_name.clone());
    debug!("  Allocated {:?} for result {}", dest_reg, result_name);
    
    // Take any instructions generated by register allocation
    insts.extend(mgr.take_instructions());
    
    // Generate the actual LOAD instruction
    let load_inst = AsmInst::Load(dest_reg, bank_reg, addr_reg);
    trace!("  Generated LOAD: {:?}", load_inst);
    insts.push(load_inst);
    
    // Step 4: If loading a fat pointer, also load the bank component
    if result_type.is_pointer() {
        debug!("  Result is a pointer, loading bank component");
        
        // Calculate address for bank component (addr + 1)
        let bank_addr_name = naming.load_bank_addr(result_temp);
        let bank_addr_reg = mgr.get_register(bank_addr_name);
        insts.extend(mgr.take_instructions());
        
        insts.push(AsmInst::AddI(bank_addr_reg, addr_reg, 1));
        trace!("  Bank component at address {:?} + 1", addr_reg);
        
        // Load the bank value
        let bank_value_name = naming.load_bank_value(result_temp);
        let bank_dest_reg = mgr.get_register(bank_value_name);
        insts.extend(mgr.take_instructions());
        
        let bank_load = AsmInst::Load(bank_dest_reg, bank_reg, bank_addr_reg);
        trace!("  Generated bank LOAD: {:?}", bank_load);
        insts.push(bank_load);
        
        // Store bank info for the loaded pointer
        if result_type.is_pointer() {
            mgr.set_pointer_bank(result_name.clone(), BankInfo::Register(bank_dest_reg));
        }
        debug!("  Fat pointer loaded: addr in {:?}, bank in {:?}", dest_reg, bank_dest_reg);
        
        // Free the temporary bank address register
        mgr.free_register(bank_addr_reg);
    }
    
    debug!("lower_load complete: generated {} instructions", insts.len());
    insts
}