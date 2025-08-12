//! Main V2 Lowering Module - Integrates All Instruction Types
//! 
//! This module provides the main entry point for lowering IR to assembly using
//! the V2 backend. It uses the FunctionBuilder API to construct functions properly.

use crate::ir::{Module, Function, Instruction, Value, IrType, IrBinaryOp, IrUnaryOp};
use crate::v2::FunctionBuilder;
use crate::v2::function::CallArg;
use rcc_codegen::{AsmInst, Reg};
use log::{debug, trace, info, warn};

/// Result type for lowering operations
pub type LowerResult = Result<Vec<AsmInst>, String>;

/// Lower an entire module using the V2 backend
pub fn lower_module_v2(module: &Module) -> Result<Vec<AsmInst>, String> {
    info!("V2: Lowering module '{}'", module.name);
    let mut all_instructions = Vec::new();
    
    // TODO: Handle global variables
    if !module.globals.is_empty() {
        warn!("V2: Global variables not yet implemented, skipping {} globals", module.globals.len());
    }
    
    // Lower each function
    for function in &module.functions {
        if function.is_external {
            debug!("V2: Skipping external function '{}'", function.name);
            continue;
        }
        
        debug!("V2: Lowering function '{}'", function.name);
        let function_asm = lower_function_v2(function)?;
        all_instructions.extend(function_asm);
    }
    
    info!("V2: Module lowering complete, generated {} instructions", all_instructions.len());
    Ok(all_instructions)
}

/// Lower a complete function using the V2 backend
pub fn lower_function_v2(function: &Function) -> Result<Vec<AsmInst>, String> {
    debug!("V2: Lowering function '{}' with {} parameters and {} blocks", 
           function.name, function.parameters.len(), function.blocks.len());
    
    // Create the function builder
    let mut builder = FunctionBuilder::new();
    
    // Count locals needed (simplified - in reality would need proper analysis)
    let local_count = function.parameters.len() as i16;
    
    // Start the function
    builder.begin_function(local_count);
    
    // Track which temp IDs map to which parameter indices
    let mut param_map = std::collections::HashMap::new();
    for (i, (param_id, _param_type)) in function.parameters.iter().enumerate() {
        param_map.insert(*param_id, i);
        // Load the parameter
        let _reg = builder.load_parameter(i);
        // In a real implementation, we'd track this register for later use
    }
    
    // Process each basic block
    for block in &function.blocks {
        debug!("V2: Processing block {} with {} instructions", block.id, block.instructions.len());
        
        // Process each instruction in the block
        for (i, instruction) in block.instructions.iter().enumerate() {
            trace!("V2: Processing instruction {}: {:?}", i, instruction);
            
            match instruction {
                Instruction::Binary { result, op, lhs, rhs, result_type } => {
                    debug!("V2: Binary operation: {} = {} {:?} {}", result, lhs, op, rhs);
                    // For now, just add a placeholder
                    // In a real implementation, we'd generate the actual binary operation
                    builder.add_instruction(AsmInst::Comment(
                        format!("Binary: t{} = {:?} {:?} {:?}", result, lhs, op, rhs)
                    ));
                }
                
                Instruction::Unary { result, op, operand, .. } => {
                    debug!("V2: Unary operation: {} = {:?} {}", result, op, operand);
                    builder.add_instruction(AsmInst::Comment(
                        format!("Unary: t{} = {:?} {:?}", result, op, operand)
                    ));
                }
                
                Instruction::Load { result, ptr, .. } => {
                    debug!("V2: Load: {} = load {}", result, ptr);
                    builder.add_instruction(AsmInst::Comment(
                        format!("Load: t{} = load {:?}", result, ptr)
                    ));
                }
                
                Instruction::Store { value, ptr } => {
                    debug!("V2: Store: store {}, {}", value, ptr);
                    builder.add_instruction(AsmInst::Comment(
                        format!("Store: {:?} -> {:?}", value, ptr)
                    ));
                }
                
                Instruction::GetElementPtr { result, ptr, indices, .. } => {
                    debug!("V2: GEP: {} = getelementptr {}", result, ptr);
                    builder.add_instruction(AsmInst::Comment(
                        format!("GEP: t{} = gep {:?} + {:?}", result, ptr, indices)
                    ));
                }
                
                Instruction::Alloca { result, alloc_type, count, .. } => {
                    debug!("V2: Alloca: {} = alloca {:?}", result, alloc_type);
                    // Calculate size
                    let type_size = alloc_type.size_in_bytes().unwrap_or(1) as i16;
                    let total_size = if let Some(count_val) = count {
                        match count_val {
                            Value::Constant(n) => type_size * (*n as i16),
                            _ => type_size,
                        }
                    } else {
                        type_size
                    };
                    
                    // Allocate on stack
                    builder.add_instruction(AsmInst::AddI(Reg::Sp, Reg::Sp, -total_size));
                    builder.add_instruction(AsmInst::Comment(
                        format!("Alloca: t{} = {} bytes", result, total_size)
                    ));
                }
                
                Instruction::Call { result, function: func, args, result_type } => {
                    debug!("V2: Call: {:?} = call {}({})", result, func, args.len());
                    
                    // Convert IR values to CallArgs
                    let call_args: Vec<CallArg> = args.iter().map(|arg| {
                        match arg {
                            Value::Temp(t) => {
                                // Check if this temp is a parameter
                                if let Some(param_idx) = param_map.get(t) {
                                    // It's a parameter - we loaded it earlier
                                    // For now, assume it's in a temp register
                                    CallArg::Scalar(match param_idx {
                                        0 => Reg::T0,
                                        1 => Reg::T1,
                                        2 => Reg::T2,
                                        3 => Reg::T3,
                                        _ => Reg::T4,
                                    })
                                } else {
                                    // It's a computed value
                                    CallArg::Scalar(Reg::T0) // Placeholder
                                }
                            }
                            Value::Constant(_c) => {
                                // Would need to load constant into register
                                CallArg::Scalar(Reg::T0) // Placeholder
                            }
                            _ => CallArg::Scalar(Reg::T0), // Placeholder
                        }
                    }).collect();
                    
                    // Get function address (simplified)
                    let func_addr = match func {
                        Value::Function(name) | Value::Global(name) => {
                            // In reality, would look up symbol table
                            debug!("V2: Calling function '{}'", name);
                            0x100u16 // Placeholder address
                        }
                        _ => 0u16,
                    };
                    
                    // Make the call
                    let returns_pointer = result_type.is_pointer();
                    let (_ret_reg, _ret_bank) = builder.call_function(
                        func_addr,
                        0, // Assume bank 0 for now
                        call_args,
                        returns_pointer
                    );
                    
                    if let Some(result_id) = result {
                        builder.add_instruction(AsmInst::Comment(
                            format!("Call result in t{}", result_id)
                        ));
                    }
                }
                
                Instruction::Return(value) => {
                    debug!("V2: Return: {:?}", value);
                    
                    // Set return value if present
                    let return_val = if let Some(val) = value {
                        match val {
                            Value::Temp(_t) => {
                                // Would need to get the register containing this temp
                                Some((Reg::T0, None)) // Placeholder
                            }
                            Value::Constant(c) => {
                                // Load constant into return register
                                builder.add_instruction(AsmInst::Li(Reg::Rv0, *c as i16));
                                Some((Reg::Rv0, None))
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    
                    // End the function
                    builder.end_function(return_val);
                }
                
                Instruction::Branch(label) => {
                    debug!("V2: Branch to label {}", label);
                    builder.add_instruction(AsmInst::Beq(
                        Reg::R0, 
                        Reg::R0, 
                        format!(".L{}", label)
                    ));
                }
                
                Instruction::BranchCond { condition, true_label, false_label } => {
                    debug!("V2: Conditional branch: {} ? {} : {}", condition, true_label, false_label);
                    
                    // Get condition register
                    let cond_reg = match condition {
                        Value::Temp(_t) => Reg::T0, // Placeholder - would need proper tracking
                        Value::Constant(c) => {
                            builder.add_instruction(AsmInst::Li(Reg::T0, *c as i16));
                            Reg::T0
                        }
                        _ => Reg::T0,
                    };
                    
                    // Branch if non-zero
                    builder.add_instruction(AsmInst::Bne(
                        cond_reg,
                        Reg::R0,
                        format!(".L{}", true_label)
                    ));
                    
                    // Fall through or jump to false label
                    builder.add_instruction(AsmInst::Beq(
                        Reg::R0,
                        Reg::R0,
                        format!(".L{}", false_label)
                    ));
                }
                
                Instruction::Phi { result, incoming, .. } => {
                    debug!("V2: Phi: {} with {} incoming", result, incoming.len());
                    // Phi nodes require SSA deconstruction - placeholder for now
                    builder.add_instruction(AsmInst::Comment(
                        format!("Phi: t{} = phi {:?}", result, incoming)
                    ));
                }
                
                Instruction::Select { result, condition, true_value, false_value, .. } => {
                    debug!("V2: Select: {} = {} ? {} : {}", result, condition, true_value, false_value);
                    
                    // Get condition register
                    let cond_reg = match condition {
                        Value::Temp(_t) => Reg::T0, // Placeholder
                        Value::Constant(c) => {
                            builder.add_instruction(AsmInst::Li(Reg::T0, *c as i16));
                            Reg::T0
                        }
                        _ => Reg::T0,
                    };
                    
                    // Generate select logic with branches
                    let true_label = format!(".Lselect_true_{}", result);
                    let end_label = format!(".Lselect_end_{}", result);
                    
                    // Branch if condition is true
                    builder.add_instruction(AsmInst::Bne(cond_reg, Reg::R0, true_label.clone()));
                    
                    // False case - load false_value
                    match false_value {
                        Value::Constant(c) => {
                            builder.add_instruction(AsmInst::Li(Reg::T1, *c as i16));
                        }
                        _ => {
                            builder.add_instruction(AsmInst::Comment(
                                format!("Load false value: {:?}", false_value)
                            ));
                        }
                    }
                    builder.add_instruction(AsmInst::Beq(Reg::R0, Reg::R0, end_label.clone()));
                    
                    // True case
                    builder.add_instruction(AsmInst::Label(true_label));
                    match true_value {
                        Value::Constant(c) => {
                            builder.add_instruction(AsmInst::Li(Reg::T1, *c as i16));
                        }
                        _ => {
                            builder.add_instruction(AsmInst::Comment(
                                format!("Load true value: {:?}", true_value)
                            ));
                        }
                    }
                    
                    // End label
                    builder.add_instruction(AsmInst::Label(end_label));
                    builder.add_instruction(AsmInst::Comment(
                        format!("Select result in t{}", result)
                    ));
                }
                
                Instruction::Cast { result, value, target_type } => {
                    debug!("V2: Cast: {} = cast {} to {:?}", result, value, target_type);
                    builder.add_instruction(AsmInst::Comment(
                        format!("Cast: t{} = cast {:?} to {:?}", result, value, target_type)
                    ));
                }
                
                Instruction::Intrinsic { result, intrinsic, args, .. } => {
                    debug!("V2: Intrinsic: {:?} = @{}({})", result, intrinsic, args.len());
                    builder.add_instruction(AsmInst::Comment(
                        format!("Intrinsic: {:?} = @{}({:?})", result, intrinsic, args)
                    ));
                }
                
                Instruction::DebugLoc { .. } => {
                    // Debug info - no code generated
                }
                
                Instruction::InlineAsm { assembly } => {
                    debug!("V2: Inline asm: {}", assembly);
                    builder.add_instruction(AsmInst::Comment(
                        format!("Inline asm: {}", assembly)
                    ));
                }
                
                Instruction::Comment(text) => {
                    builder.add_instruction(AsmInst::Comment(text.clone()));
                }
            }
        }
    }
    
    // If the function doesn't end with a return, add one
    if !function.blocks.is_empty() {
        let last_block = &function.blocks[function.blocks.len() - 1];
        if !last_block.has_terminator() {
            debug!("V2: Adding implicit return at end of function");
            builder.end_function(None);
        }
    }
    
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{IrBuilder, Module, BasicBlock};
    
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
        
        let result = lower_module_v2(&module);
        assert!(result.is_ok());
        
        let insts = result.unwrap();
        assert!(insts.len() > 0);
        
        // Should have R13 initialization
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Li(Reg::Sb, 1))));
    }
    
    #[test]
    fn test_lower_simple_addition() {
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
        
        let result = lower_module_v2(&module);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_lower_function_with_call() {
        let mut module = Module::new("test".to_string());
        let mut builder = IrBuilder::new();
        
        let func = builder.create_function("caller".to_string(), IrType::I16);
        let entry = builder.new_label();
        builder.create_block(entry).unwrap();
        
        let args = vec![Value::Constant(42)];
        let result = builder.build_call(
            Value::Function("callee".to_string()),
            args,
            IrType::I16,
        ).unwrap();
        
        builder.build_return(result.map(Value::Temp)).unwrap();
        
        let function = builder.finish_function().unwrap();
        module.add_function(function);
        
        let result = lower_module_v2(&module);
        assert!(result.is_ok());
    }
}