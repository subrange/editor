//! Function Lowering - Handles lowering of functions
//! 
//! This module is responsible for lowering IR functions to assembly,
//! including parameter binding, basic block generation, and epilogue handling.

use rcc_frontend::ir::{Function, Instruction, Value, IrType};
use rcc_codegen::{AsmInst, Reg};
use std::collections::HashMap;
use log::{debug, info, trace};
use crate::v2::RegisterPressureManager;
use crate::v2::naming::NameGenerator;
use crate::v2::globals::GlobalManager;
use crate::v2::function::{FunctionBuilder, CallingConvention};
use crate::v2::instr::helpers::get_bank_register;
use super::instruction::lower_instruction;

/// Compute stack offsets (relative to FP) for each alloca result
pub fn compute_alloca_offsets(function: &Function) -> HashMap<rcc_common::TempId, i16> {
    let mut offsets = HashMap::new();
    let mut cur: i16 = 0;
    for block in &function.blocks {
        for inst in &block.instructions {
            if let Instruction::Alloca { result, alloc_type, count, .. } = inst {
                let type_size = alloc_type.size_in_bytes().unwrap_or(1) as i16;
                let total_size = if let Some(Value::Constant(n)) = count {
                    type_size * (*n as i16)
                } else {
                    type_size
                };
                // Assign the lowest available offset for this alloca
                offsets.insert(*result, cur);
                cur += total_size;
            }
        }
    }
    offsets
}

/// Calculate the number of local slots needed for a function
pub fn calculate_local_slots(function: &Function) -> i16 {
    // Count allocas and other locals
    let mut slots = 0i16;
    
    for block in &function.blocks {
        for instruction in &block.instructions {
            if let Instruction::Alloca { alloc_type, count, .. } = instruction {
                let type_size = alloc_type.size_in_bytes().unwrap_or(1) as i16;
                let total_size = if let Some(Value::Constant(n)) = count {
                    type_size * (*n as i16)
                } else {
                    type_size
                };
                slots += total_size;
            }
        }
    }
    
    // Add some buffer for temporaries
    slots + 8
}

/// Lower a complete function using the V2 backend
pub fn lower_function_v2(
    function: &Function, 
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    global_manager: &GlobalManager
) -> Result<Vec<AsmInst>, String> {
    info!("V2: Lowering function '{}' with {} blocks", function.name, function.blocks.len());
    
    let mut instructions = Vec::new();
    
    // Add function label first
    instructions.push(AsmInst::Label(function.name.clone()));
    
    // Create the function builder with parameter types
    let param_types: Vec<(rcc_common::TempId, IrType)> = function.parameters
        .iter()
        .map(|(id, ty)| (*id, ty.clone()))
        .collect();
    
    let pt = param_types.clone();
    
    let mut builder = FunctionBuilder::with_params(param_types);
    
    // Calculate local slots needed
    let local_slots = calculate_local_slots(function);

    let alloca_offsets = compute_alloca_offsets(function);
    
    // Begin the function (this will emit the prologue)
    builder.begin_function(local_slots);

    // === Bind function parameters at entry ===
    // Load parameters using the calling convention and bind them to their SSA temps
    {
        let cc = CallingConvention::new();
        for (idx, (param_id, _ty)) in function.parameters.iter().enumerate() {
            // Generate load instructions for this parameter
            let (param_insts, preg, bank_reg) = cc.load_param(idx, &pt, mgr, naming);
            // Emit them at the top of the function
            builder.add_instructions(param_insts);
            // Bind the temp name to the register so later uses resolve correctly
            let pname = naming.temp_name(*param_id);
            mgr.bind_value_to_register(pname.clone(), preg);
            
            // If this is a fat pointer parameter, track the bank register
            if let Some(bank_reg) = bank_reg {
                debug!("Parameter {} is a fat pointer with bank in {:?}", idx, bank_reg);
                mgr.set_pointer_bank(pname, crate::v2::BankInfo::Register(bank_reg));
            }
        }
    }
    
    // Track basic block labels for branching
    let mut block_labels = HashMap::new();
    for block in function.blocks.iter() {
        block_labels.insert(block.id, naming.block_label(&function.name, block.id));
    }
    
    // Generate a common epilogue label
    let epilogue_label = naming.block_label(&function.name, 99999); // Use a high ID for epilogue
    let mut has_any_return = false;
    let mut return_values: Vec<(Option<(Reg, Option<Reg>)>, usize)> = Vec::new();
    
    // Lower each basic block
    for (block_idx, block) in function.blocks.iter().enumerate() {
        debug!("V2: Lowering block {}", block.id);
        
        // Add label for the block (except for entry block which is implicit)
        if block.id != 0 {
            let label_name = block_labels.get(&block.id).unwrap().clone();
            builder.add_instruction(AsmInst::Label(label_name));
        }
        
        // Lower each instruction in the block
        for instruction in &block.instructions {
            trace!("V2: Lowering instruction: {:?}", instruction);
            
            match instruction {
                Instruction::Return(value) => {
                    has_any_return = true;
                    
                    // Prepare return value if any
                    let return_regs = if let Some(val) = value {
                        match val {
                            Value::Temp(t) => {
                                let temp_name = naming.temp_name(*t);
                                
                                // Check if returning a pointer (would need bank too)
                                if function.return_type.is_pointer() {
                                    // Get the bank register for this pointer first, before moving temp_name
                                    let bank_info = mgr.get_pointer_bank(&temp_name)
                                        .unwrap_or_else(|| {
                                            // This is a compiler bug - all pointers must have bank info
                                            panic!("V2: COMPILER BUG: No bank info for pointer return value t{}. All pointers must have tracked bank information!", t);
                                        });
                                    let bank_reg = get_bank_register(&bank_info);
                                    
                                    let temp_reg = mgr.get_register(temp_name);
                                    builder.add_instructions(mgr.take_instructions());
                                    
                                    // Move to return registers if needed
                                    if temp_reg != Reg::Rv0 {
                                        builder.add_instruction(AsmInst::Move(Reg::Rv0, temp_reg));
                                    }
                                    if bank_reg != Reg::Rv1 {
                                        builder.add_instruction(AsmInst::Move(Reg::Rv1, bank_reg));
                                    }
                                    Some((Reg::Rv0, Some(Reg::Rv1)))
                                } else {
                                    let temp_reg = mgr.get_register(temp_name);
                                    builder.add_instructions(mgr.take_instructions());
                                    
                                    // Move to return register if needed
                                    if temp_reg != Reg::Rv0 {
                                        builder.add_instruction(AsmInst::Move(Reg::Rv0, temp_reg));
                                    }
                                    Some((Reg::Rv0, None))
                                }
                            }
                            Value::Constant(c) => {
                                // Load constant directly into return register
                                builder.add_instruction(AsmInst::Li(Reg::Rv0, *c as i16));
                                Some((Reg::Rv0, None))
                            }
                            _ => None
                        }
                    } else {
                        None
                    };
                    
                    return_values.push((return_regs, block_idx));
                    
                    // Jump to common epilogue
                    builder.add_instruction(AsmInst::Comment("Jump to epilogue".to_string()));
                    builder.add_instruction(AsmInst::Beq(Reg::R0, Reg::R0, epilogue_label.clone()));
                }
                
                _ => {
                    // Use the existing lower_instruction for other instructions
                    let insts = lower_instruction(mgr, naming, instruction, &function.name, &alloca_offsets, global_manager)?;
                    builder.add_instructions(insts);
                }
            }
        }
    }
    
    // Add the common epilogue
    if has_any_return {
        builder.add_instruction(AsmInst::Label(epilogue_label));
        // Use the first return value format (they should all be consistent)
        let return_regs = if !return_values.is_empty() {
            return_values[0].0
        } else {
            None
        };
        builder.end_function(return_regs);
    } else {
        // No explicit returns, add default return at the end
        builder.end_function(None);
    }
    
    // Build and return the final instruction list
    let mut builder_instructions = builder.build();
    instructions.append(&mut builder_instructions);
    Ok(instructions)
}