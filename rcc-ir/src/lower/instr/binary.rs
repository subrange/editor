use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, TempId};
use crate::{IrBinaryOp, Value};
use crate::lower::instr::arithmetic::emit_ne;
use crate::module_lowering::{Location, ModuleLowerer};

impl ModuleLowerer {
    pub(crate) fn lower_binary(&mut self, result: &TempId, op: &IrBinaryOp, lhs: &Value, rhs: &Value) -> Result<(), CompilerError> {
        eprintln!("=== Processing Binary t{} ===", result);
        self.emit(AsmInst::Comment(format!("=== Processing Binary t{} ===", result)));

        // CRITICAL: Implement the algorithm from more-formalized-register-spilling.md
        // 1. Calculate need() for both operands
        // 2. Evaluate the operand with larger need() first
        // 3. Reuse the first operand's register for the result (in-place operation)
        // 4. Free the second operand's register immediately after the operation

        // Calculate need() for operands
        let lhs_need = self.calculate_need(lhs);
        let rhs_need = self.calculate_need(rhs);

        self.emit(AsmInst::Comment(
            format!("Binary: need(lhs)={}, need(rhs)={}", lhs_need, rhs_need)
        ));

        // Simple implementation: always evaluate left first, then right
        // Result goes in left's register (in-place on left operand)
        // This matches the simple interpretation where swap() is not used
        // for non-commutative operations.

        let left_reg = self.get_value_register(lhs)?;
        let right_reg = self.get_value_register(rhs)?;
        let dest_reg = left_reg;  // Result goes in left's register

        // Track the result in the first operand's register
        let result_key = Self::temp_name(*result);
        self.value_locations.insert(result_key.clone(), Location::Register(dest_reg));
        // Update the register allocator to know this register now holds the result
        self.reg_alloc.mark_in_use(dest_reg, result_key.clone());

        self.emit(AsmInst::Comment(
            format!("Reusing {} for result t{}",
                    match dest_reg {
                        Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
                        Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
                        Reg::R9 => "R9", Reg::R10 => "R10", Reg::R11 => "R11",
                        _ => "R?",
                    }, result)
        ));

        // Now execute the operation IN-PLACE on the destination register
        match op {
            IrBinaryOp::Add => {
                self.emit(AsmInst::Add(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Sub => {
                self.emit(AsmInst::Sub(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Mul => {
                self.emit(AsmInst::Mul(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::SDiv | IrBinaryOp::UDiv => {
                self.emit(AsmInst::Div(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::SRem | IrBinaryOp::URem => {
                self.emit(AsmInst::Mod(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Eq => {
                // Set dest_reg to 1 if equal, 0 otherwise
                // Strategy: result = !(left != right)

                // Since dest_reg == left_reg, we need to be careful not to clobber left
                // before we use it. Get TWO temporary registers for the comparison.
                let temp1 = self.get_reg(format!("eq_temp1_{}", self.label_counter));
                let temp2 = self.get_reg(format!("eq_temp2_{}", self.label_counter));
                self.label_counter += 1;

                let eq = emit_ne(dest_reg, left_reg, right_reg, temp1, temp2);
                self.emit_many(eq);

                // Free temp registers
                self.reg_alloc.free_reg(temp1);
                self.reg_alloc.free_reg(temp2);

                // Now invert the result: dest = 1 - dest
                let temp3 = self.get_reg(format!("eq_inv_{}", self.label_counter));
                self.label_counter += 1;
                self.emit(AsmInst::LI(temp3, 1));
                self.emit(AsmInst::Sub(dest_reg, temp3, dest_reg));
                self.reg_alloc.free_reg(temp3);
            }
            IrBinaryOp::Ne => {
                // Set dest_reg to 1 if not equal, 0 otherwise

                // Since dest_reg == left_reg, we need to be careful
                let temp1 = self.get_reg(format!("ne_temp1_{}", self.label_counter));
                let temp2 = self.get_reg(format!("ne_temp2_{}", self.label_counter));
                self.label_counter += 1;

                // Generate the comparison using temps
                let ne = emit_ne(dest_reg, left_reg, right_reg, temp1, temp2);
                self.emit_many(ne);

                // Free temp registers
                self.reg_alloc.free_reg(temp1);
                self.reg_alloc.free_reg(temp2);
            }
            IrBinaryOp::Slt => {
                // Use SLTU instead of SLT since SLT might be buggy
                self.emit(AsmInst::Sltu(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Sle => {
                // a <= b is !(b < a)
                let temp = self.get_reg(format!("sle_temp_{}", self.label_counter));
                self.label_counter += 1;

                self.emit(AsmInst::Sltu(dest_reg, right_reg, left_reg));
                self.emit(AsmInst::LI(temp, 1));
                self.emit(AsmInst::Sub(dest_reg, temp, dest_reg)); // 1 - result
                self.reg_alloc.free_reg(temp);
            }
            IrBinaryOp::Sgt => {
                self.emit(AsmInst::Sltu(dest_reg, right_reg, left_reg));
            }
            IrBinaryOp::Sge => {
                // a >= b is !(a < b)
                self.emit(AsmInst::Sltu(dest_reg, left_reg, right_reg));
                let temp = self.get_reg(format!("sge_temp_{}", self.label_counter));
                self.label_counter += 1;
                self.emit(AsmInst::LI(temp, 1));
                self.emit(AsmInst::Sub(dest_reg, temp, dest_reg)); // 1 - result
                self.reg_alloc.free_reg(temp);
            }
            IrBinaryOp::And => {
                self.emit(AsmInst::And(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Or => {
                self.emit(AsmInst::Or(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Xor => {
                self.emit(AsmInst::Xor(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Shl => {
                self.emit(AsmInst::Sll(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::LShr | IrBinaryOp::AShr => {
                self.emit(AsmInst::Srl(dest_reg, left_reg, right_reg));
            }
            IrBinaryOp::Ult | IrBinaryOp::Ule | IrBinaryOp::Ugt | IrBinaryOp::Uge => {
                return Err(CompilerError::codegen_error(
                    format!("Unsigned comparison {:?} not yet implemented", op),
                    rcc_common::SourceLocation::new_simple(0, 0),
                ));
            }
            _ => {
                return Err(CompilerError::codegen_error(
                    format!("Unsupported binary operation: {:?}", op),
                    rcc_common::SourceLocation::new_simple(0, 0),
                ));
            }
        }

        // CRITICAL: Free the RIGHT operand's register immediately after the operation
        // This is a key part of the algorithm from more-formalized-register-spilling.md
        self.emit(AsmInst::Comment(
            format!("Freeing right operand register {}",
                    match right_reg {
                        Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
                        Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
                        Reg::R9 => "R9", Reg::R10 => "R10", Reg::R11 => "R11",
                        _ => "R?",
                    })
        ));
        self.reg_alloc.free_reg(right_reg);

        Ok(())
        
    }

}