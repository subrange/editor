//! Helper functions for binary operation lowering

use crate::ir::{Value, IrBinaryOp};
use crate::v2::regmgmt::RegisterPressureManager;
use crate::v2::naming::NameGenerator;
use rcc_codegen::Reg;

/// Calculate register needs for operands
pub(super) fn calculate_register_needs(
    mgr: &RegisterPressureManager,
    lhs: &Value,
    rhs: &Value,
) -> (usize, usize) {
    let left_need = calculate_value_need(mgr, lhs);
    let right_need = calculate_value_need(mgr, rhs);
    (left_need, right_need)
}

/// Calculate register need for a single value
fn calculate_value_need(_mgr: &RegisterPressureManager, value: &Value) -> usize {
    match value {
        Value::Constant(_) => 1, // Need to load into register
        Value::Temp(_id) => {
            // Conservative estimate: assume temp needs a register
            // Note: RegisterPressureManager handles checking if already in register
            // internally when get_register is called
            1
        }
        Value::Global(_) => 1, // Need to load address
        Value::FatPtr(_) => 2, // Fat pointers need 2 registers
        Value::Function(_) => 1, // Function addresses need a register
        Value::Undef => 0, // Undefined values don't need registers
    }
}

/// Check if operation is commutative
pub(super) fn is_commutative(op: IrBinaryOp) -> bool {
    matches!(op, 
        IrBinaryOp::Add | 
        IrBinaryOp::Mul | 
        IrBinaryOp::And | 
        IrBinaryOp::Or | 
        IrBinaryOp::Xor |
        IrBinaryOp::Eq |  // Equality is commutative
        IrBinaryOp::Ne    // Not-equal is commutative
    )
}

/// Check if we can reuse the first operand's register for the result
pub(super) fn can_reuse_register(_op: IrBinaryOp) -> bool {
    // Most operations can reuse the destination register
    // Some architectures may have restrictions
    true
}

/// Get register for a value
pub(super) fn get_value_register(
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
            // Need to emit LI instruction to load constant
            // This will be captured by take_instructions()
            reg
        }
        Value::Global(name) => {
            let global_name = naming.load_global_addr(name);
            let reg = mgr.get_register(global_name);
            // Global address loading would be handled here
            reg
        }
        Value::Function(name) => {
            // Function addresses are like globals
            let func_name = format!("func_{}_{}", name, naming.next_operation_id());
            let reg = mgr.get_register(func_name);
            // Function address loading would be handled here
            reg
        }
        Value::FatPtr(fp) => {
            // For binary ops, we typically just need the address part
            get_value_register(mgr, naming, &fp.addr)
        }
        Value::Undef => {
            panic!("Cannot use undefined value in binary operation");
        }
    }
}