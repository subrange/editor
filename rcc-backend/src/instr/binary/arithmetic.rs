//! Arithmetic and logical operation generation

use rcc_frontend::ir::IrBinaryOp;
use rcc_codegen::{AsmInst, Reg};
use log::warn;

/// Generate arithmetic instruction for the given operation
/// 
/// Returns None if the operation is not an arithmetic operation
/// (i.e., it's a comparison operation that needs special handling)
pub(super) fn generate_arithmetic_instruction(
    op: IrBinaryOp,
    result_reg: Reg,
    lhs_reg: Reg,
    rhs_reg: Reg,
    should_swap: bool,
) -> Option<AsmInst> {
    match op {
        IrBinaryOp::Add => {
            // Commutative - use the registers as evaluated
            Some(if should_swap {
                AsmInst::Add(result_reg, rhs_reg, lhs_reg)
            } else {
                AsmInst::Add(result_reg, lhs_reg, rhs_reg)
            })
        }
        IrBinaryOp::Sub => {
            // Non-commutative - always use original order
            Some(AsmInst::Sub(result_reg, lhs_reg, rhs_reg))
        }
        IrBinaryOp::Mul => {
            // Commutative - use the registers as evaluated
            Some(if should_swap {
                AsmInst::Mul(result_reg, rhs_reg, lhs_reg)
            } else {
                AsmInst::Mul(result_reg, lhs_reg, rhs_reg)
            })
        }
        IrBinaryOp::SDiv => {
            // VM DIV is unsigned, need to handle signs for signed division
            // For now, emit warning and use unsigned division
            // TODO: Implement proper signed division using sign checks and negation
            warn!("  SDiv not fully implemented - using unsigned division");
            warn!("  This will produce incorrect results for negative numbers!");
            Some(AsmInst::Div(result_reg, lhs_reg, rhs_reg))
        }
        IrBinaryOp::UDiv => {
            // VM DIV is unsigned - direct mapping
            Some(AsmInst::Div(result_reg, lhs_reg, rhs_reg))
        }
        IrBinaryOp::SRem => {
            // VM MOD is unsigned, need to handle signs for signed remainder
            // For now, emit warning and use unsigned modulo
            // TODO: Implement proper signed remainder using sign checks and negation
            warn!("  SRem not fully implemented - using unsigned modulo");
            warn!("  This will produce incorrect results for negative numbers!");
            Some(AsmInst::Mod(result_reg, lhs_reg, rhs_reg))
        }
        IrBinaryOp::URem => {
            // VM MOD is unsigned - direct mapping
            Some(AsmInst::Mod(result_reg, lhs_reg, rhs_reg))
        }
        IrBinaryOp::And => {
            // Commutative
            Some(if should_swap {
                AsmInst::And(result_reg, rhs_reg, lhs_reg)
            } else {
                AsmInst::And(result_reg, lhs_reg, rhs_reg)
            })
        }
        IrBinaryOp::Or => {
            // Commutative
            Some(if should_swap {
                AsmInst::Or(result_reg, rhs_reg, lhs_reg)
            } else {
                AsmInst::Or(result_reg, lhs_reg, rhs_reg)
            })
        }
        IrBinaryOp::Xor => {
            // Commutative
            Some(if should_swap {
                AsmInst::Xor(result_reg, rhs_reg, lhs_reg)
            } else {
                AsmInst::Xor(result_reg, lhs_reg, rhs_reg)
            })
        }
        IrBinaryOp::Shl => {
            // Non-commutative
            Some(AsmInst::Sll(result_reg, lhs_reg, rhs_reg))
        }
        IrBinaryOp::LShr | IrBinaryOp::AShr => {
            // VM only has SRL (logical shift right)
            // TODO: AShr (arithmetic shift right) needs sign extension
            if matches!(op, IrBinaryOp::AShr) {
                warn!("  AShr treated as logical shift in VM");
            }
            Some(AsmInst::Srl(result_reg, lhs_reg, rhs_reg))
        }
        // Comparison operations handled elsewhere
        _ => None,
    }
}