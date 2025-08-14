//! Instruction Lowering - Handles lowering of individual instructions
//! 
//! This module is responsible for lowering individual IR instructions to assembly,
//! delegating to specialized modules for each instruction type.

use rcc_frontend::ir::{Instruction, Value, IrType, FatPointer};
use rcc_frontend::BankTag;
use rcc_codegen::{AsmInst, Reg};
use std::collections::HashMap;
use log::{debug, warn};

use crate::v2::RegisterPressureManager;
use crate::v2::naming::NameGenerator;
use crate::v2::globals::GlobalManager;
use crate::v2::function::{FunctionBuilder, CallArg, CallTarget};

// Import all the existing lowering functions
use crate::v2::instr::{
    lower_load, lower_store, lower_gep,
    lower_binary_op, lower_unary_op,
    helpers::{get_bank_register_with_mgr, resolve_global_to_fatptr, canonicalize_value, get_value_register}
};

/// Lower a single instruction using the existing infrastructure
pub fn lower_instruction(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    instruction: &Instruction,
    function_name: &str,
    alloca_offsets: &HashMap<rcc_common::TempId, i16>,
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
                // Resolve global to fat pointer
                let global_ptr = resolve_global_to_fatptr(name, global_manager)?;
                lower_load(mgr, naming, &global_ptr, result_type, *result)
            } else if let Value::FatPtr(_) = ptr {
                // Canonicalize to resolve any global references
                let canonical_ptr = canonicalize_value(ptr, global_manager)?;
                lower_load(mgr, naming, &canonical_ptr, result_type, *result)
            } else {
                lower_load(mgr, naming, ptr, result_type, *result)
            };
            insts.extend(load_insts);
        }
        
        Instruction::Store { value, ptr } => {
            debug!("V2: Store: {:?} -> {:?}", value, ptr);

            // Canonicalize both value and pointer to resolve any global references
            let canon_value = canonicalize_value(value, global_manager)?;
            let canon_ptr = canonicalize_value(ptr, global_manager)?;

            // Delegate to store lowering with canonicalized operands
            let store_insts = lower_store(mgr, naming, &canon_value, &canon_ptr);
            insts.extend(store_insts);
        }
        
        Instruction::GetElementPtr { result, ptr, indices, result_type } => {
            debug!("V2: GEP: t{} = gep {:?} + {:?}", result, ptr, indices);
            
            // Calculate element size from result type
            // IMPORTANT: The VM uses 16-bit cells, so we need to convert bytes to cells
            let element_size = if let Some(elem_type) = result_type.element_type() {
                // size_in_bytes returns the size in bytes
                // We need to convert to cells (16-bit words)
                let size_words = elem_type.size_in_words().unwrap_or(1) as i16;
                let size_cells = size_words;
                // Minimum size is 1 cell
                if size_cells == 0 { 1 } else { size_cells }
            } else {
                1
            };
            
            // Canonicalize pointer to resolve any global references
            let canonical_ptr = canonicalize_value(ptr, global_manager)?;
            let gep_insts = lower_gep(mgr, naming, &canonical_ptr, indices, element_size, *result);
            insts.extend(gep_insts);
        }
        
        Instruction::Alloca { result, alloc_type, .. } => {
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
            mgr.set_pointer_bank(result_name.clone(), crate::v2::BankInfo::Stack);
            
            // Register this alloca so it can be recomputed if needed later
            mgr.register_alloca(result_name.clone(), offset);
            
            mgr.bind_value_to_register(result_name, result_reg);

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
                            
                            let bank_reg = get_bank_register_with_mgr(&bank_info, mgr);
                            
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
                            Value::Global(name) => {
                                // Resolve global and extract address
                                let global_ptr = resolve_global_to_fatptr(name, global_manager)?;
                                if let Value::FatPtr(fp) = global_ptr {
                                    if let Value::Constant(addr) = *fp.addr {
                                        let temp_name = naming.const_value(addr);
                                        let reg = mgr.get_register(temp_name);
                                        insts.push(AsmInst::Li(reg, addr as i16));
                                        reg
                                    } else {
                                        panic!("Resolved global should have constant address");
                                    }
                                } else {
                                    panic!("resolve_global_to_fatptr should return FatPtr");
                                }
                            }
                            _ => panic!("Unsupported fat pointer address type, {:?}", fp.addr),
                        };
                        insts.extend(mgr.take_instructions());
                        
                        // Use the helper function to resolve bank tag to bank info
                        // This handles all bank types including Mixed properly
                        use crate::v2::instr::helpers::resolve_bank_tag_to_info;
                        let bank_info = resolve_bank_tag_to_info(&fp.bank, fp, mgr, naming);
                        let bank_reg = get_bank_register_with_mgr(&bank_info, mgr);
                        
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
            
            // Prepare the result name if there's a return value
            let result_name = result.map(|id| naming.temp_name(id));
            
            // Use FunctionBuilder's standalone call method
            // This handles everything: argument setup, call, return value binding, and stack cleanup
            let call_insts = FunctionBuilder::make_standalone_call(
                mgr,
                naming,
                CallTarget::Label(func_name.clone()),
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
        
        Instruction::Cast { result, value, target_type: _ } => {
            debug!("V2: Cast: t{} = cast {:?}", result, value);
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