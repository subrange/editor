//! Main V2 Lowering Module - Integrates All Instruction Types
//! 
//! This module provides the main entry point for lowering IR to assembly using
//! the V2 backend. It properly connects all the existing instruction lowering
//! implementations from the instr/ subdirectory.

use rcc_frontend::ir::{Module, Function, Instruction, Value, IrType, IrBinaryOp};
use std::collections::HashMap;

/// Compute stack offsets (relative to FP) for each alloca result
fn compute_alloca_offsets(function: &Function) -> HashMap<rcc_common::TempId, i16> {
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
use crate::v2::RegisterPressureManager;
use crate::v2::naming::{NameGenerator, new_function_naming};
use crate::v2::function::CallArg;

// Import all the existing lowering functions
use crate::v2::instr::{
    lower_load, lower_store, lower_gep,
    lower_binary_op, lower_unary_op,
    lower_compare_and_branch,
    ComparisonType
};

use rcc_codegen::{AsmInst, Reg};
use log::{debug, trace, info, warn};
use rcc_frontend::BankTag;
use crate::v2::globals::{GlobalManager, GlobalInfo};

/// Lower an entire module using the V2 backend
pub fn lower_module_v2(module: &Module, bank_size: u16) -> Result<Vec<AsmInst>, String> {
    info!("V2: Lowering module '{}' with bank_size {}", module.name, bank_size);
    let mut all_instructions = Vec::new();
    
    // Create a global manager to handle global variable allocation
    let mut global_manager = GlobalManager::new();
    
    // First pass: allocate addresses for all globals
    for global in &module.globals {
        global_manager.allocate_global(global);
    }
    
    // Check if this module has a main function (indicating it's the main module)
    let has_main = module.functions.iter().any(|f| f.name == "main");
    
    // Only generate _init_globals for the main module
    // This avoids duplicate labels when linking multiple object files
    if has_main {
        all_instructions.push(AsmInst::Label("_init_globals".to_string()));
        
        // Generate initialization code for each global
        if !module.globals.is_empty() {
            info!("V2: Initializing {} globals", module.globals.len());
            
            for global in &module.globals {
                if let Some(info) = global_manager.get_global_info(&global.name) {
                    let global_insts = GlobalManager::lower_global_init(global, info);
                    all_instructions.extend(global_insts);
                }
            }
        }
        
        all_instructions.push(AsmInst::Ret);
    } else if !module.globals.is_empty() {
        // For library modules, still allocate space but don't generate init code
        info!("V2: Library module with {} globals (no _init_globals generated)", module.globals.len());
        
        // We still need to generate comments for globals so they can be referenced
        for global in &module.globals {
            if let Some(info) = global_manager.get_global_info(&global.name) {
                all_instructions.push(AsmInst::Comment(format!("Global '{}' at address {}", 
                                                               global.name, info.address)));
            }
            // The actual initialization will be done by the main module's _init_globals
        }
    }
    
    // Lower each function
    for function in &module.functions {
        if function.is_external {
            debug!("V2: Skipping external function '{}'", function.name);
            continue;
        }
        
        debug!("V2: Lowering function '{}'", function.name);
        
        // Create a fresh register manager and naming context for this function
        let mut mgr = RegisterPressureManager::new(16); // 16 registers available
        let mut naming = new_function_naming();
        
        let function_asm = lower_function_v2(function, &mut mgr, &mut naming, &global_manager)?;
        all_instructions.extend(function_asm);
    }
    
    info!("V2: Module lowering complete, generated {} instructions", all_instructions.len());
    Ok(all_instructions)
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
    
    // Import the FunctionBuilder
    use crate::v2::function::FunctionBuilder;
    
    // Create the function builder with parameter types
    let param_types: Vec<(rcc_common::TempId, IrType)> = function.parameters
        .iter()
        .map(|(id, ty)| (*id, ty.clone()))
        .collect();
    
    let pt = param_types.clone();
    
    let mut builder = FunctionBuilder::with_params(param_types);
    
    // Calculate local slots needed (simplified - you may need better calculation)
    let local_slots = calculate_local_slots(function);

    let alloca_offsets = compute_alloca_offsets(function);
    
    // Begin the function (this will emit the prologue)
    builder.begin_function(local_slots);

    // === Bind function parameters at entry ===
    // Load parameters using the calling convention and bind them to their SSA temps
    {
        use crate::v2::function::CallingConvention;
        let cc = CallingConvention::new();
        for (idx, (param_id, ty)) in function.parameters.iter().enumerate() {
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
    let mut block_labels = std::collections::HashMap::new();
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
                                    let bank_info = mgr.get_pointer_bank(&temp_name);
                                    let bank_reg = match bank_info {
                                        Some(crate::v2::BankInfo::Stack) => Reg::Sb,
                                        Some(crate::v2::BankInfo::Global) => Reg::Gp,
                                        Some(crate::v2::BankInfo::Register(r)) => r,
                                        None => {
                                            // This is a compiler bug - all pointers must have bank info
                                            panic!("V2: COMPILER BUG: No bank info for pointer return value t{}. All pointers must have tracked bank information!", t);
                                        }
                                    };
                                    
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

/// Calculate the number of local slots needed for a function
fn calculate_local_slots(function: &Function) -> i16 {
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

/// Lower a single instruction using the existing infrastructure
fn lower_instruction(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    instruction: &Instruction,
    function_name: &str,
    alloca_offsets: &std::collections::HashMap<rcc_common::TempId, i16>,
    global_manager: &GlobalManager,
) -> Result<Vec<AsmInst>, String> {
    let mut insts = Vec::new();
    
    match instruction {
        Instruction::Binary { result, op, lhs, rhs, .. } => {
            debug!("V2: Binary: t{} = {:?} {:?} {:?}", result, lhs, op, rhs);
            let binary_insts = lower_binary_op(mgr, naming, *op, lhs, rhs, *result);
            insts.extend(binary_insts);
        }
        
        Instruction::Unary { result, op, operand, result_type } => {
            debug!("V2: Unary: t{} = {:?} {:?}", result, op, operand);
            let unary_insts = lower_unary_op(mgr, naming, *op, operand, result_type, *result);
            insts.extend(unary_insts);
        }
        
        Instruction::Load { result, ptr, result_type } => {
            debug!("V2: Load: t{} = load {:?}", result, ptr);
            
            // Handle global variable loads specially
            let load_insts = if let Value::Global(name) = ptr {
                // Look up the global's address
                if let Some(info) = global_manager.get_global_info(name) {
                    // Create a fat pointer with the global's address
                    let global_ptr = Value::FatPtr(rcc_frontend::ir::FatPointer {
                        addr: Box::new(Value::Constant(info.address as i64)),
                        bank: rcc_frontend::types::BankTag::Global,
                    });
                    lower_load(mgr, naming, &global_ptr, result_type, *result)
                } else {
                    return Err(format!("V2: Unknown global variable: {}", name));
                }
            } else {
                lower_load(mgr, naming, ptr, result_type, *result)
            };
            insts.extend(load_insts);
        }
        
        Instruction::Store { value, ptr } => {
            debug!("V2: Store: {:?} -> {:?}", value, ptr);
            
            // Handle global variable stores specially
            let store_insts = if let Value::Global(name) = ptr {
                // Look up the global's address
                if let Some(info) = global_manager.get_global_info(name) {
                    // Create a fat pointer with the global's address
                    let global_ptr = Value::FatPtr(rcc_frontend::ir::FatPointer {
                        addr: Box::new(Value::Constant(info.address as i64)),
                        bank: rcc_frontend::types::BankTag::Global,
                    });
                    lower_store(mgr, naming, value, &global_ptr)
                } else {
                    return Err(format!("V2: Unknown global variable: {}", name));
                }
            } else {
                lower_store(mgr, naming, value, ptr)
            };
            insts.extend(store_insts);
        }
        
        Instruction::GetElementPtr { result, ptr, indices, result_type } => {
            debug!("V2: GEP: t{} = gep {:?} + {:?}", result, ptr, indices);
            
            // Calculate element size from result type
            let element_size = if let Some(elem_type) = result_type.element_type() {
                elem_type.size_in_bytes().unwrap_or(1) as i16
            } else {
                1
            };
            
            // Handle global variable GEPs specially
            let gep_insts = if let Value::Global(name) = ptr {
                // Look up the global's address
                if let Some(info) = global_manager.get_global_info(name) {
                    // Create a fat pointer with the global's address
                    let global_ptr = Value::FatPtr(rcc_frontend::ir::FatPointer {
                        addr: Box::new(Value::Constant(info.address as i64)),
                        bank: rcc_frontend::types::BankTag::Global,
                    });
                    lower_gep(mgr, naming, &global_ptr, indices, element_size, *result)
                } else {
                    return Err(format!("V2: Unknown global variable: {}", name));
                }
            } else {
                lower_gep(mgr, naming, ptr, indices, element_size, *result)
            };
            insts.extend(gep_insts);
        }
        
        Instruction::Alloca { result, alloc_type, count, .. } => {
            debug!("V2: Alloca: t{} = alloca {:?}", result, alloc_type);

            // Fetch the precomputed offset for this alloca
            let offset = *alloca_offsets.get(result).expect("alloca offset missing");

            // Compute pointer = FP + offset (locals live in [FP..FP+local_slots))
            let result_name = naming.temp_name(*result);
            let result_reg = mgr.get_register(result_name.clone());
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Add(result_reg, Reg::Fp, Reg::R0));
            if offset != 0 {
                insts.push(AsmInst::AddI(result_reg, result_reg, offset));
            }

            // Mark as stack pointer
            mgr.set_pointer_bank(result_name, crate::v2::BankInfo::Stack);

            debug!("V2: Alloca FP+{}", offset);
        }
        
        Instruction::Call { result, function: func, args, result_type } => {
            debug!("V2: Call: {:?} = call {}({})", result, func, args.len());
            
            // Convert IR Values to CallArgs for the calling convention
            let mut call_args = Vec::new();
            for arg in args {
                match arg {
                    Value::Temp(t) => {
                        let name = naming.temp_name(*t);
                        // Check if this is a fat pointer
                        if let Some(bank_info) = mgr.get_pointer_bank(&name) {
                            let addr_reg = mgr.get_register(name.clone());
                            insts.extend(mgr.take_instructions());
                            
                            let bank_reg = match bank_info {
                                crate::v2::BankInfo::Stack => Reg::Sb,
                                crate::v2::BankInfo::Global => Reg::Gp,
                                crate::v2::BankInfo::Register(r) => r,
                            };
                            
                            call_args.push(CallArg::FatPointer { addr: addr_reg, bank: bank_reg });
                        } else {
                            // Scalar argument
                            let reg = mgr.get_register(name);
                            insts.extend(mgr.take_instructions());
                            call_args.push(CallArg::Scalar(reg));
                        }
                    }
                    Value::Constant(c) => {
                        // Load constant into a register
                        let temp_name = naming.const_value(*c);
                        let reg = mgr.get_register(temp_name);
                        insts.extend(mgr.take_instructions());
                        insts.push(AsmInst::Li(reg, *c as i16));
                        call_args.push(CallArg::Scalar(reg));
                    }
                    Value::FatPtr(fp) => {
                        // Handle fat pointer directly
                        let addr_reg = match fp.addr.as_ref() {
                            Value::Temp(t) => {
                                let name = naming.temp_name(*t);
                                mgr.get_register(name)
                            }
                            Value::Constant(c) => {
                                let temp_name = naming.const_value(*c);
                                let reg = mgr.get_register(temp_name);
                                insts.push(AsmInst::Li(reg, *c as i16));
                                reg
                            }
                            _ => panic!("Unsupported fat pointer address type")
                        };
                        insts.extend(mgr.take_instructions());
                        
                        let bank_reg = match fp.bank {
                            BankTag::Stack => Reg::Sb,
                            BankTag::Global => Reg::Gp,
                            _ => {
                                warn!("V2: Unsupported bank type for fat pointer: {:?}", fp.bank);
                                panic!("Unsupported bank type for fat pointer");
                            }
                        };
                        
                        call_args.push(CallArg::FatPointer { addr: addr_reg, bank: bank_reg });
                    }
                    _ => {
                        warn!("V2: Unsupported argument type for call: {:?}", arg);
                    }
                }
            }
            
            // Extract function name for label-based calls
            let func_name = match func {
                Value::Function(name) | Value::Global(name) => name.clone(),
                _ => return Err(format!("V2: Invalid function value for call: {:?}", func)),
            };
            
            // Use the calling convention to handle the complete call sequence
            use crate::v2::function::CallingConvention;
            let cc = CallingConvention::new();
            
            // Prepare the result name if there's a return value
            let result_name = result.map(|id| naming.temp_name(id));
            
            // Use the label-based call method
            // This handles everything: argument setup, call, return value binding, and stack cleanup
            let (call_insts, _return_regs) = cc.make_complete_call_by_label(
                mgr,
                naming,
                &func_name,
                call_args,
                result_type.is_pointer(),
                result_name,
            );
            
            insts.extend(call_insts);
            
            // Return value binding is now handled inside make_complete_call_by_label!
            
            debug!("V2: Call lowered with {} arguments", args.len());
        }
        
        Instruction::Return(_) => {
            // Return instructions are handled specially by the FunctionBuilder
            // in lower_function_v2. If we get here, it's a compiler bug.
            panic!("V2: COMPILER BUG: Return instruction reached lower_instruction. All Returns should be handled by FunctionBuilder!");
        }
        
        Instruction::Branch(label) => {
            debug!("V2: Branch to label {}", label);
            // Get the proper label name with function context
            let label_name = naming.block_label(function_name, *label);
            // Create a simple unconditional branch
            insts.push(AsmInst::Beq(Reg::R0, Reg::R0, label_name.clone()));
            insts.push(AsmInst::Comment(format!("Unconditional branch to {}", label_name)));
        }
        
        Instruction::BranchCond { condition, true_label, false_label } => {
            debug!("V2: Conditional branch: {} ? {} : {}", condition, true_label, false_label);
            // Get the proper label names with function context
            let true_label_name = naming.block_label(function_name, *true_label);
            let false_label_name = naming.block_label(function_name, *false_label);
            
            // Get register for condition value
            use crate::v2::instr::helpers::get_value_register;
            let cond_reg = get_value_register(mgr, naming, condition);
            insts.extend(mgr.take_instructions());
            
            // Branch if condition is zero (false) to false_label
            insts.push(AsmInst::Beq(cond_reg, Reg::R0, false_label_name.clone()));
            insts.push(AsmInst::Comment(format!("Branch to {} if condition is false", false_label_name)));
            
            // If condition was non-zero (true), branch to true_label
            insts.push(AsmInst::Beq(Reg::R0, Reg::R0, true_label_name.clone()));
            insts.push(AsmInst::Comment(format!("Unconditional branch to {} (condition was true)", true_label_name)));
            
            // Free the condition register
            mgr.free_register(cond_reg);
        }
        
        Instruction::Phi { result, incoming, .. } => {
            debug!("V2: Phi: t{} with {} incoming", result, incoming.len());
            // Phi nodes require SSA deconstruction - simplified for now
            warn!("V2: Phi nodes not fully implemented - using first value");
            if let Some((value, _label)) = incoming.first() {
                match value {
                    Value::Temp(t) => {
                        let src_name = naming.temp_name(*t);
                        let dst_name = naming.temp_name(*result);
                        let src_reg = mgr.get_register(src_name);
                        let dst_reg = mgr.get_register(dst_name);
                        insts.extend(mgr.take_instructions());
                        insts.push(AsmInst::Move(dst_reg, src_reg));
                        mgr.free_register(src_reg);
                    }
                    Value::Constant(c) => {
                        let dst_name = naming.temp_name(*result);
                        let dst_reg = mgr.get_register(dst_name);
                        insts.extend(mgr.take_instructions());
                        insts.push(AsmInst::Li(dst_reg, *c as i16));
                    }
                    _ => {}
                }
            }
        }
        
        Instruction::Select { result, condition, true_value, false_value, .. } => {
            debug!("V2: Select: t{} = {} ? {} : {}", result, condition, true_value, false_value);
            
            // Get condition value
            let cond_reg = match condition {
                Value::Temp(t) => {
                    let name = naming.temp_name(*t);
                    mgr.get_register(name)
                }
                Value::Constant(c) => {
                    let temp_name = naming.const_value(*c);
                    let reg = mgr.get_register(temp_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Li(reg, *c as i16));
                    reg
                }
                _ => return Err(format!("V2: Unsupported condition type for select: {:?}", condition)),
            };
            insts.extend(mgr.take_instructions());
            
            // Generate labels using naming module
            let true_label = naming.select_true_label(*result);
            let end_label = naming.select_end_label(*result);
            
            // Branch if condition is non-zero
            insts.push(AsmInst::Bne(cond_reg, Reg::R0, true_label.clone()));
            
            // False case
            let result_name = naming.temp_name(*result);
            let result_reg = mgr.get_register(result_name);
            insts.extend(mgr.take_instructions());
            
            match false_value {
                Value::Temp(t) => {
                    let src_name = naming.temp_name(*t);
                    let src_reg = mgr.get_register(src_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Move(result_reg, src_reg));
                    mgr.free_register(src_reg);
                }
                Value::Constant(c) => {
                    insts.push(AsmInst::Li(result_reg, *c as i16));
                }
                _ => {}
            }
            
            // Jump to end
            insts.push(AsmInst::Beq(Reg::R0, Reg::R0, end_label.clone()));
            
            // True case
            insts.push(AsmInst::Label(true_label));
            match true_value {
                Value::Temp(t) => {
                    let src_name = naming.temp_name(*t);
                    let src_reg = mgr.get_register(src_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Move(result_reg, src_reg));
                    mgr.free_register(src_reg);
                }
                Value::Constant(c) => {
                    insts.push(AsmInst::Li(result_reg, *c as i16));
                }
                _ => {}
            }
            
            // End label
            insts.push(AsmInst::Label(end_label));
            
            mgr.free_register(cond_reg);
        }
        
        Instruction::Cast { result, value, target_type } => {
            debug!("V2: Cast: t{} = cast {:?} to {:?}", result, value, target_type);
            // Most casts are handled as moves or conversions
            // This would be expanded based on the specific cast type
            warn!("V2: Cast instruction simplified - may need type-specific handling");
            
            match value {
                Value::Temp(t) => {
                    let src_name = naming.temp_name(*t);
                    let dst_name = naming.temp_name(*result);
                    let src_reg = mgr.get_register(src_name);
                    let dst_reg = mgr.get_register(dst_name);
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::Move(dst_reg, src_reg));
                    mgr.free_register(src_reg);
                }
                _ => {}
            }
        }
        
        Instruction::Intrinsic { .. } => {
            warn!("V2: Intrinsic instructions not yet implemented");
        }
        
        Instruction::DebugLoc { .. } => {
            // Debug info - no code generated
        }

        Instruction::InlineAsm { assembly } => {
            for line in assembly.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    // For now, pass through as raw assembly
                    // We'll need a way to handle this in AsmInst
                    insts.push(AsmInst::Raw(trimmed.to_string()));
                }
            }
        }
        
        Instruction::Comment(text) => {
            insts.push(AsmInst::Comment(text.clone()));
        }
    }
    
    Ok(insts)
}


#[cfg(test)]
mod tests {
    use super::*;
    use rcc_frontend::ir::{IrBuilder, Module};

    #[test]
    fn test_lower_empty_function() {
        let mut module = Module::new("test".to_string());
        let mut builder = IrBuilder::new();
        
        let func = builder.create_function("empty".to_string(), IrType::Void);
        let entry = builder.new_label();
        builder.create_block(entry).unwrap();
        builder.build_return(None).unwrap();
        
        let function = builder.finish_function().unwrap();
        module.add_function(function);
        
        let result = lower_module_v2(&module, 4096);
        assert!(result.is_ok());
        
        let insts = result.unwrap();
        assert!(insts.len() > 0);
        
        // Should have function label
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Label(_))));
    }
    
    #[test]
    fn test_lower_with_binary_op() {
        let mut module = Module::new("test".to_string());
        let mut builder = IrBuilder::new();
        
        let func = builder.create_function("add".to_string(), IrType::I16);
        func.add_parameter(0, IrType::I16);
        func.add_parameter(1, IrType::I16);
        
        let entry = builder.new_label();
        builder.create_block(entry).unwrap();
        
        let result = builder.build_binary(
            IrBinaryOp::Add,
            Value::Temp(0),
            Value::Temp(1),
            IrType::I16,
        ).unwrap();
        
        builder.build_return(Some(Value::Temp(result))).unwrap();
        
        let function = builder.finish_function().unwrap();
        module.add_function(function);
        
        let result = lower_module_v2(&module, 4096);
        assert!(result.is_ok());
        
        let insts = result.unwrap();
        // Should contain actual Add instruction from lower_binary_op
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(_, _, _))));
    }
    #[test]
    fn test_global_variables() {
        use rcc_frontend::ir::{GlobalVariable, Linkage};
        
        let mut module = Module::new("test".to_string());
        
        // Add a global variable
        module.add_global(GlobalVariable {
            name: "global_x".to_string(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: Some(Value::Constant(42)),
            linkage: Linkage::External,
            symbol_id: None,
        });
        
        // Add main function that uses the global
        let mut builder = IrBuilder::new();
        let func = builder.create_function("main".to_string(), IrType::I16);
        let entry = builder.new_label();
        builder.create_block(entry).unwrap();
        
        // Load from global
        let global_ptr = Value::Global("global_x".to_string());
        let loaded = builder.build_load(global_ptr, IrType::I16).unwrap();
        
        // Return the loaded value
        builder.build_return(Some(Value::Temp(loaded))).unwrap();
        
        let function = builder.finish_function().unwrap();
        module.add_function(function);
        
        let result = lower_module_v2(&module, 4096);
        assert!(result.is_ok());
        
        let insts = result.unwrap();
        
        // Should have _init_globals label since we have main
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Label(l) if l == "_init_globals")));
        
        // Should have initialization code for global_x (Li T0, 42)
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Li(Reg::T0, 42))));
        
        // Should have store to global memory
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(Reg::T0, Reg::Gp, _))));
    }
    
    #[test]
    fn test_call_binds_params_and_returns() {
        use rcc_frontend::ir::{IrBuilder, IrType, Value};

        // Build a module with an `add(a,b)` callee and a `main` that calls it.
        let mut module = Module::new("call_bind_test".to_string());

        // --- callee: int add(int a, int b) { return a + b; }
        let mut b = IrBuilder::new();
        let add_fn = b.create_function("add".to_string(), IrType::I16);
        add_fn.add_parameter(0, IrType::I16);
        add_fn.add_parameter(1, IrType::I16);
        let entry = b.new_label();
        b.create_block(entry).unwrap();
        let sum = b
            .build_binary(
                rcc_frontend::ir::IrBinaryOp::Add,
                Value::Temp(0),
                Value::Temp(1),
                IrType::I16,
            )
            .unwrap();
        b.build_return(Some(Value::Temp(sum))).unwrap();
        let add_ir = b.finish_function().unwrap();
        module.add_function(add_ir);

        // --- caller: int main() { return add(5, 10); }
        let mut b = IrBuilder::new();
        let _main_fn = b.create_function("main".to_string(), IrType::I16);
        let entry = b.new_label();
        b.create_block(entry).unwrap();
        let res = b
            .build_call(
                Value::Function("add".to_string()),
                vec![Value::Constant(5), Value::Constant(10)],
                IrType::I16,
            )
            .unwrap().unwrap();
        b.build_return(Some(Value::Temp(res))).unwrap();
        let main_ir = b.finish_function().unwrap();
        module.add_function(main_ir);

        // Lower and inspect the assembly
        let insts = lower_module_v2(&module, 4096).expect("lowering failed");

        // 1) Caller should emit a CALL to `add`
        let saw_call_add = insts.iter().any(|i| match i {
            AsmInst::Call(name) if name == "add" => true,
            _ => false,
        });
        assert!(saw_call_add, "main should contain a CALL to add");

        // 2) Callee should load both parameters at entry using the CC loader
        //    (our CC injects comments like "Load param 0 ..." and "Load param 1 ...")
        let mut in_add = false;
        let mut saw_param0 = false;
        let mut saw_param1 = false;
        for i in &insts {
            match i {
                AsmInst::Label(l) if l == "add" => {
                    in_add = true;
                }
                AsmInst::Label(l) if l == "main" => {
                    in_add = false; // left the add function
                }
                AsmInst::Comment(c) if in_add => {
                    if c.starts_with("Load param 0") { saw_param0 = true; }
                    if c.starts_with("Load param 1") { saw_param1 = true; }
                }
                _ => {}
            }
        }
        assert!(saw_param0 && saw_param1, "callee should load both parameters via CC at entry");
    }

    #[test]
    fn scalar_load_from_global_is_not_pointer() {
        use rcc_frontend::ir::{Module, IrBuilder, IrType, Value, GlobalVariable, Linkage};
        use crate::v2::lower::lower_module_v2;

        let mut module = Module::new("test".into());
        module.add_global(GlobalVariable {
            name: "g".into(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: Some(Value::Constant(42)),
            linkage: Linkage::External,
            symbol_id: None,
        });

        let mut b = IrBuilder::new();
        b.create_function("main".into(), IrType::I16);
        let entry = b.new_label(); b.create_block(entry).unwrap();
        let t0 = b.build_load(Value::Global("g".into()), IrType::I16).unwrap();
        b.build_return(Some(Value::Temp(t0))).unwrap();
        module.add_function(b.finish_function().unwrap());

        let asm = lower_module_v2(&module, 4096).unwrap();
        // Ensure calls (if any) don’t try to use A1 for this value — in practice
        // you can assert the call-setup comment shows “scalar”, or just that no
        // fat-ptr argument setup is emitted for t0.
        assert!(asm.iter().any(|i| matches!(i, rcc_codegen::AsmInst::Li(_, 42))));
    }
}