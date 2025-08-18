//! Comparison operation generation

use rcc_frontend::ir::IrBinaryOp;
use crate::regmgmt::RegisterPressureManager;
use crate::naming::NameGenerator;
use rcc_codegen::{AsmInst, Reg};
use rcc_common::TempId;

/// Generate comparison instructions
/// 
/// Returns the instructions needed to perform the comparison operation.
/// Comparison operations often require multiple instructions to implement.
pub(super) fn generate_comparison_instructions(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    op: IrBinaryOp,
    lhs_reg: Reg,
    rhs_reg: Reg,
    result_reg: Reg,
    result_temp: TempId,
) -> Vec<AsmInst> {
    let mut insts = vec![];
    
    match op {
        IrBinaryOp::Eq => {
            // a == b: !(a ^ b) = (a ^ b) == 0
            let xor_reg = mgr.get_register(naming.xor_temp(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Xor(xor_reg, lhs_reg, rhs_reg));
            // result = (xor_result == 0) = !(xor_result != 0) = !slt(0, xor_result) & !slt(xor_result, 0)
            // Simpler: result = (xor_result == 0) ? 1 : 0
            // We'll use SLTU with 1 to check if value is 0
            let one_reg = mgr.get_register(naming.const_one(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Li(one_reg, 1));
            insts.push(AsmInst::Sltu(result_reg, xor_reg, one_reg)); // result = (xor < 1) = (xor == 0)
            mgr.free_register(xor_reg);
            mgr.free_register(one_reg);
        }
        IrBinaryOp::Ne => {
            // a != b: !!(a ^ b) = (a ^ b) != 0
            let xor_reg = mgr.get_register(naming.xor_temp(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Xor(xor_reg, lhs_reg, rhs_reg));
            // result = (xor_result != 0) ? 1 : 0
            // We can use SLTU(0, xor_result) or SLT(0, xor_result)
            let zero_reg = mgr.get_register(naming.const_zero(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Li(zero_reg, 0));
            insts.push(AsmInst::Sltu(result_reg, zero_reg, xor_reg)); // result = (0 < xor) = (xor != 0)
            mgr.free_register(xor_reg);
            mgr.free_register(zero_reg);
        }
        IrBinaryOp::Slt => {
            // Signed less than - direct VM support
            insts.push(AsmInst::Slt(result_reg, lhs_reg, rhs_reg));
        }
        IrBinaryOp::Sle => {
            // a <= b: !(b < a) = !slt(b, a)
            let temp_reg = mgr.get_register(naming.sle_temp(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Slt(temp_reg, rhs_reg, lhs_reg)); // temp = (b < a)
            // result = !temp = 1 - temp (since temp is 0 or 1)
            let one_reg = mgr.get_register(naming.const_one(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Li(one_reg, 1));
            insts.push(AsmInst::Sub(result_reg, one_reg, temp_reg));
            mgr.free_register(temp_reg);
            mgr.free_register(one_reg);
        }
        IrBinaryOp::Sgt => {
            // a > b: b < a = slt(b, a)
            insts.push(AsmInst::Slt(result_reg, rhs_reg, lhs_reg));
        }
        IrBinaryOp::Sge => {
            // a >= b: !(a < b) = !slt(a, b)
            let temp_reg = mgr.get_register(naming.sge_temp(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Slt(temp_reg, lhs_reg, rhs_reg)); // temp = (a < b)
            // result = !temp = 1 - temp
            let one_reg = mgr.get_register(naming.const_one(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Li(one_reg, 1));
            insts.push(AsmInst::Sub(result_reg, one_reg, temp_reg));
            mgr.free_register(temp_reg);
            mgr.free_register(one_reg);
        }
        IrBinaryOp::Ult => {
            // Unsigned less than - direct VM support
            insts.push(AsmInst::Sltu(result_reg, lhs_reg, rhs_reg));
        }
        IrBinaryOp::Ule => {
            // a <= b (unsigned): !(b < a) = !sltu(b, a)
            let temp_reg = mgr.get_register(naming.ule_temp(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Sltu(temp_reg, rhs_reg, lhs_reg)); // temp = (b < a)
            // result = !temp = 1 - temp
            let one_reg = mgr.get_register(naming.const_one(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Li(one_reg, 1));
            insts.push(AsmInst::Sub(result_reg, one_reg, temp_reg));
            mgr.free_register(temp_reg);
            mgr.free_register(one_reg);
        }
        IrBinaryOp::Ugt => {
            // a > b (unsigned): b < a = sltu(b, a)
            insts.push(AsmInst::Sltu(result_reg, rhs_reg, lhs_reg));
        }
        IrBinaryOp::Uge => {
            // a >= b (unsigned): !(a < b) = !sltu(a, b)
            let temp_reg = mgr.get_register(naming.uge_temp(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Sltu(temp_reg, lhs_reg, rhs_reg)); // temp = (a < b)
            // result = !temp = 1 - temp
            let one_reg = mgr.get_register(naming.const_one(result_temp));
            insts.extend(mgr.take_instructions());
            insts.push(AsmInst::Li(one_reg, 1));
            insts.push(AsmInst::Sub(result_reg, one_reg, temp_reg));
            mgr.free_register(temp_reg);
            mgr.free_register(one_reg);
        }
        _ => panic!("Not a comparison operation: {op:?}"),
    }
    
    insts
}

/// Check if an operation is a comparison operation
pub(super) fn is_comparison(op: IrBinaryOp) -> bool {
    matches!(op,
        IrBinaryOp::Eq | IrBinaryOp::Ne |
        IrBinaryOp::Slt | IrBinaryOp::Sle | IrBinaryOp::Sgt | IrBinaryOp::Sge |
        IrBinaryOp::Ult | IrBinaryOp::Ule | IrBinaryOp::Ugt | IrBinaryOp::Uge
    )
}