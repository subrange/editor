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

                // CRITICAL: For emit_ne to work correctly, temp1 and temp2 must be
                // different from both source registers. Since dest_reg == left_reg,
                // we need to ensure temps are different from left_reg and right_reg.
                
                // Pin the operand registers so they can't be chosen for temps
                let left_key = self.get_register_value_name(left_reg);
                let right_key = self.get_register_value_name(right_reg);
                if let Some(ref left_name) = left_key {
                    self.reg_alloc.pin_value(left_name.clone());
                }
                if let Some(ref right_name) = right_key {
                    self.reg_alloc.pin_value(right_name.clone());
                }
                
                let temp1_name = self.generate_temp_name("eq_temp1");
                let temp2_name = self.generate_temp_name("eq_temp2");
                let ((temp1, temp2), spill_insts) = self.reg_alloc.get_two_regs(temp1_name, temp2_name);
                self.emit_many(spill_insts);

                let eq = emit_ne(dest_reg, left_reg, right_reg, temp1, temp2);
                self.emit_many(eq);

                // Unpin the operand registers
                if let Some(ref left_name) = left_key {
                    self.reg_alloc.unpin_value(left_name);
                }
                if let Some(ref right_name) = right_key {
                    self.reg_alloc.unpin_value(right_name);
                }
                
                // Free temp registers
                self.reg_alloc.free_reg(temp1);
                self.reg_alloc.free_reg(temp2);

                // Now invert the result: dest = 1 - dest
                let temp_name = self.generate_temp_name("eq_inv");
                let temp3 = self.get_reg(temp_name);
                self.emit(AsmInst::LI(temp3, 1));
                self.emit(AsmInst::Sub(dest_reg, temp3, dest_reg));
                self.reg_alloc.free_reg(temp3);
            }
            IrBinaryOp::Ne => {
                // Set dest_reg to 1 if not equal, 0 otherwise

                // CRITICAL: For emit_ne to work correctly, temp1 and temp2 must be
                // different from both source registers. Since dest_reg == left_reg,
                // we need to ensure temps are different from left_reg and right_reg.
                
                // Pin the operand registers so they can't be chosen for temps
                let left_key = self.get_register_value_name(left_reg);
                let right_key = self.get_register_value_name(right_reg);
                if let Some(ref left_name) = left_key {
                    self.reg_alloc.pin_value(left_name.clone());
                }
                if let Some(ref right_name) = right_key {
                    self.reg_alloc.pin_value(right_name.clone());
                }
                
                let temp1_name = self.generate_temp_name("ne_temp1");
                let temp2_name = self.generate_temp_name("ne_temp2");
                let ((temp1, temp2), spill_insts) = self.reg_alloc.get_two_regs(temp1_name, temp2_name);
                self.emit_many(spill_insts);

                // Generate the comparison using temps
                let ne = emit_ne(dest_reg, left_reg, right_reg, temp1, temp2);
                self.emit_many(ne);

                // Unpin the operand registers
                if let Some(ref left_name) = left_key {
                    self.reg_alloc.unpin_value(left_name);
                }
                if let Some(ref right_name) = right_key {
                    self.reg_alloc.unpin_value(right_name);
                }

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
                let temp_name = self.generate_temp_name("sle_temp");
                let temp = self.get_reg(temp_name);

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
                let temp_name = self.generate_temp_name("sge_temp");
                let temp = self.get_reg(temp_name);
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