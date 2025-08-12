//! Unary operation lowering for V2 backend
//! 
//! Implements all unary operations including negation, bitwise NOT,
//! and type conversions (zero/sign extend, truncate, pointer casts).

use crate::ir::{Value, IrUnaryOp, IrType};
use rcc_common::TempId;
use crate::v2::regmgmt::RegisterPressureManager;
use crate::v2::naming::NameGenerator;
use rcc_codegen::{AsmInst, Reg};
use log::{debug, trace, warn};

/// Lower a unary operation to assembly instructions
/// 
/// # Parameters
/// - `mgr`: Register pressure manager for allocation and spilling
/// - `naming`: Name generator for unique temporary names
/// - `op`: The unary operation to perform
/// - `operand`: The operand value
/// - `result_type`: Type of the result
/// - `result_temp`: Temp ID for the result
/// 
/// # Returns
/// Vector of assembly instructions for the unary operation
pub fn lower_unary_op(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    op: IrUnaryOp,
    operand: &Value,
    result_type: &IrType,
    result_temp: TempId,
) -> Vec<AsmInst> {
    debug!("lower_unary_op: op={:?}, operand={:?}, result_type={:?}, result=t{}", 
           op, operand, result_type, result_temp);
    trace!("  Current register state: spill_count={}", mgr.get_spill_count());
    
    let mut insts = vec![];
    let result_name = naming.temp_name(result_temp);
    
    // Get register for operand
    let operand_reg = get_value_register(mgr, naming, operand);
    insts.extend(mgr.take_instructions());
    trace!("  Operand in {:?}", operand_reg);
    
    // Allocate result register (can often reuse operand register for some operations)
    let result_reg = if can_reuse_register(op) {
        trace!("  Reusing operand register {:?} for result", operand_reg);
        operand_reg
    } else {
        let reg = mgr.get_register(result_name.clone());
        insts.extend(mgr.take_instructions());
        trace!("  Allocated new register {:?} for result", reg);
        reg
    };
    
    // Generate the appropriate instruction(s)
    match op {
        IrUnaryOp::Not => {
            // Bitwise NOT: result = ~operand
            // In RISC architectures, NOT is often XOR with -1 (all ones)
            let all_ones_reg = mgr.get_register(format!("all_ones_{}", naming.next_operation_id()));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::LI(all_ones_reg, -1)); // Load -1 (0xFFFF)
            insts.push(AsmInst::Xor(result_reg, operand_reg, all_ones_reg));
            mgr.free_register(all_ones_reg);
            trace!("  Generated NOT using XOR with -1");
        }
        
        IrUnaryOp::Neg => {
            // Arithmetic negation: result = -operand
            // Implement as: result = 0 - operand
            let zero_reg = mgr.get_register(format!("zero_{}", naming.next_operation_id()));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::LI(zero_reg, 0));
            insts.push(AsmInst::Sub(result_reg, zero_reg, operand_reg));
            mgr.free_register(zero_reg);
            trace!("  Generated NEG using 0 - operand");
        }
        
        IrUnaryOp::ZExt => {
            // Zero extend - for 16-bit architecture, this is often a no-op
            // since we're already working with 16-bit values
            // Just move the value if needed
            if result_reg != operand_reg {
                insts.push(AsmInst::Add(result_reg, operand_reg, Reg::R0)); // MOV using ADD with 0
                trace!("  Generated ZEXT (move to result register)");
            } else {
                trace!("  ZEXT is no-op (same register)");
            }
        }
        
        IrUnaryOp::SExt => {
            // Sign extend - need to check the sign bit and extend appropriately
            // For now, treat as move since we're on 16-bit architecture
            // TODO: Implement proper sign extension if needed for smaller types
            warn!("  SExt not fully implemented - treating as move");
            if result_reg != operand_reg {
                insts.push(AsmInst::Add(result_reg, operand_reg, Reg::R0)); // MOV
                trace!("  Generated SEXT (move to result register)");
            }
        }
        
        IrUnaryOp::Trunc => {
            // Truncate - mask off higher bits if needed
            // For 16-bit to smaller types, use AND with appropriate mask
            match result_type {
                IrType::I8 => {
                    // Truncate to 8 bits
                    let mask_reg = mgr.get_register(format!("mask_i8_{}", naming.next_operation_id()));
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::LI(mask_reg, 0xFF)); // 8-bit mask
                    insts.push(AsmInst::And(result_reg, operand_reg, mask_reg));
                    mgr.free_register(mask_reg);
                    trace!("  Generated TRUNC to i8 using AND 0xFF");
                }
                IrType::I1 => {
                    // Truncate to 1 bit (boolean)
                    let mask_reg = mgr.get_register(format!("mask_i1_{}", naming.next_operation_id()));
                    insts.extend(mgr.take_instructions());
                    insts.push(AsmInst::LI(mask_reg, 1)); // 1-bit mask
                    insts.push(AsmInst::And(result_reg, operand_reg, mask_reg));
                    mgr.free_register(mask_reg);
                    trace!("  Generated TRUNC to i1 using AND 0x1");
                }
                _ => {
                    // For same-size or larger truncation, just move
                    if result_reg != operand_reg {
                        insts.push(AsmInst::Add(result_reg, operand_reg, Reg::R0)); // MOV
                    }
                    trace!("  TRUNC is move/no-op for target type {:?}", result_type);
                }
            }
        }
        
        IrUnaryOp::PtrToInt => {
            // Pointer to integer cast - just treat the pointer address as an integer
            // Fat pointers: only use the address part, ignore the bank
            if result_reg != operand_reg {
                insts.push(AsmInst::Add(result_reg, operand_reg, Reg::R0)); // MOV
            }
            trace!("  PtrToInt: treating pointer address as integer");
        }
        
        IrUnaryOp::IntToPtr => {
            // Integer to pointer cast - treat integer as pointer address
            // For fat pointers, we'd need to set a default bank (Gp register)
            // But since this is just the address component, we handle it as a move
            if result_reg != operand_reg {
                insts.push(AsmInst::Add(result_reg, operand_reg, Reg::R0)); // MOV
            }
            trace!("  IntToPtr: treating integer as pointer address");
            // Note: The bank component would be handled separately by the caller
        }
    }
    
    // Free operand register if not reused
    if result_reg != operand_reg {
        mgr.free_register(operand_reg);
        trace!("  Freed operand register {:?}", operand_reg);
    }
    
    debug!("lower_unary_op complete: generated {} instructions", insts.len());
    trace!("  Final register state: spill_count={}", mgr.get_spill_count());
    
    insts
}

/// Check if we can reuse the operand's register for the result
fn can_reuse_register(op: IrUnaryOp) -> bool {
    // Most unary operations can reuse the operand register
    // since they only have one input
    match op {
        IrUnaryOp::Not | IrUnaryOp::Neg => false, // Need both registers during operation
        _ => true, // Conversions can often reuse the register
    }
}

/// Get register for a value (similar to binary operations helper)
fn get_value_register(
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
            // Create a unique name for this constant load
            let name = format!("const_{}_{}", val, naming.next_operation_id());
            let reg = mgr.get_register(name);
            // The RegisterPressureManager will emit LI instruction
            reg
        }
        Value::Global(name) => {
            let global_name = naming.load_global_addr(name);
            let reg = mgr.get_register(global_name);
            reg
        }
        Value::Function(name) => {
            let func_name = format!("func_{}_{}", name, naming.next_operation_id());
            let reg = mgr.get_register(func_name);
            reg
        }
        Value::FatPtr(fp) => {
            // For unary ops on fat pointers, we typically just need the address part
            get_value_register(mgr, naming, &fp.addr)
        }
        Value::Undef => {
            panic!("Cannot use undefined value in unary operation");
        }
    }
}