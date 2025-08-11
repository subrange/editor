//! IR to Assembly Lowering
//! 
//! This module handles the translation from simple IR to Ripple assembly
//! instructions. It includes register assignment and instruction selection.

use crate::ir_commands::{SimpleIR, SimpleProgram};
use rcc_codegen::{AsmInst, Reg};
use rcc_common::TempId;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoweringError {
    #[error("Register allocation failed for temporary {0}")]
    RegisterAllocationFailed(TempId),
    
    #[error("Undefined label: {0}")]
    UndefinedLabel(String),
    
    #[error("Invalid temporary: {0}")]
    InvalidTemporary(TempId),
    
    #[error("Codegen error: {0}")]
    CodegenError(#[from] rcc_codegen::CodegenError),
}

/// Lower simple IR to Ripple assembly
pub fn lower_to_assembly(program: SimpleProgram) -> Result<Vec<AsmInst>, LoweringError> {
    let mut lowerer = IRLowerer::new();
    lowerer.lower(program)
}

/// IR to assembly lowering implementation
struct IRLowerer {
    temp_to_reg: HashMap<TempId, Reg>,
    scratch_reg: Reg,
}

impl IRLowerer {
    fn new() -> Self {
        Self {
            temp_to_reg: HashMap::new(),
            scratch_reg: Reg::R3, // Use R3 as scratch register (R1/R2 don't exist)
        }
    }
    
    /// Lower a complete program
    fn lower(&mut self, program: SimpleProgram) -> Result<Vec<AsmInst>, LoweringError> {
        let mut asm_instructions = Vec::new();
        
        for instr in program.instructions {
            let mut lowered = self.lower_instruction(instr)?;
            asm_instructions.append(&mut lowered);
        }
        
        Ok(asm_instructions)
    }
    
    /// Lower a single IR instruction
    fn lower_instruction(&mut self, instr: SimpleIR) -> Result<Vec<AsmInst>, LoweringError> {
        match instr {
            SimpleIR::Const(temp, value) => {
                let reg = self.get_or_assign_reg(temp)?;
                Ok(vec![AsmInst::LI(reg, value)])
            }
            
            SimpleIR::Add(dest, src1, src2) => {
                let dest_reg = self.get_or_assign_reg(dest)?;
                let src1_reg = self.get_or_assign_reg(src1)?;
                let src2_reg = self.get_or_assign_reg(src2)?;
                Ok(vec![AsmInst::Add(dest_reg, src1_reg, src2_reg)])
            }
            
            SimpleIR::Sub(dest, src1, src2) => {
                let dest_reg = self.get_or_assign_reg(dest)?;
                let src1_reg = self.get_or_assign_reg(src1)?;
                let src2_reg = self.get_or_assign_reg(src2)?;
                Ok(vec![AsmInst::Sub(dest_reg, src1_reg, src2_reg)])
            }
            
            SimpleIR::Mul(dest, src1, src2) => {
                let dest_reg = self.get_or_assign_reg(dest)?;
                let src1_reg = self.get_or_assign_reg(src1)?;
                let src2_reg = self.get_or_assign_reg(src2)?;
                Ok(vec![AsmInst::Mul(dest_reg, src1_reg, src2_reg)])
            }
            
            SimpleIR::Div(dest, src1, src2) => {
                let dest_reg = self.get_or_assign_reg(dest)?;
                let src1_reg = self.get_or_assign_reg(src1)?;
                let src2_reg = self.get_or_assign_reg(src2)?;
                Ok(vec![AsmInst::Div(dest_reg, src1_reg, src2_reg)])
            }
            
            SimpleIR::Store(value, bank, addr) => {
                let value_reg = self.get_or_assign_reg(value)?;
                let bank_reg = self.get_or_assign_reg(bank)?;
                let addr_reg = self.get_or_assign_reg(addr)?;
                Ok(vec![AsmInst::Store(value_reg, bank_reg, addr_reg)])
            }
            
            SimpleIR::Load(dest, bank, addr) => {
                let dest_reg = self.get_or_assign_reg(dest)?;
                let bank_reg = self.get_or_assign_reg(bank)?;
                let addr_reg = self.get_or_assign_reg(addr)?;
                Ok(vec![AsmInst::Load(dest_reg, bank_reg, addr_reg)])
            }
            
            SimpleIR::Call(name) => {
                Ok(vec![AsmInst::Call(name)])
            }
            
            SimpleIR::Return(Some(temp)) => {
                // Move return value to R3 (standard return register) if not already there
                let temp_reg = self.get_or_assign_reg(temp)?;
                if temp_reg != Reg::R3 {
                    Ok(vec![
                        AsmInst::Move(Reg::R3, temp_reg),
                        AsmInst::Ret
                    ])
                } else {
                    Ok(vec![AsmInst::Ret])
                }
            }
            
            SimpleIR::Return(None) => {
                Ok(vec![AsmInst::Ret])
            }
            
            SimpleIR::Label(name) => {
                Ok(vec![AsmInst::Label(name)])
            }
            
            SimpleIR::Jump(label) => {
                // Use JAL with return address bank=0, addr=0 for unconditional jump
                // This is a simplification - real implementation would handle this better
                Ok(vec![AsmInst::Call(label)])
            }
            
            SimpleIR::JumpIfZero(temp, label) => {
                let temp_reg = self.get_or_assign_reg(temp)?;
                // Compare with zero and branch
                Ok(vec![
                    AsmInst::Beq(temp_reg, Reg::R0, label)
                ])
            }
            
            SimpleIR::JumpIfNotZero(temp, label) => {
                let temp_reg = self.get_or_assign_reg(temp)?;
                // Compare with zero and branch if not equal
                Ok(vec![
                    AsmInst::Bne(temp_reg, Reg::R0, label)
                ])
            }
            
            SimpleIR::Comment(text) => {
                Ok(vec![AsmInst::Comment(text)])
            }
        }
    }
    
    /// Get register for temporary, assigning if needed
    fn get_or_assign_reg(&mut self, temp: TempId) -> Result<Reg, LoweringError> {
        if let Some(&reg) = self.temp_to_reg.get(&temp) {
            return Ok(reg);
        }
        
        let reg = self.get_or_assign_reg(temp)
            .map_err(|_| LoweringError::RegisterAllocationFailed(temp))?;
        
        self.temp_to_reg.insert(temp, reg);
        Ok(reg)
    }
}

/// Helper functions for creating test IR programs
pub mod test_helpers {
    use super::*;
    use crate::ir_commands::{SimpleIRBuilder, SimpleProgram};
    
    /// Create a simple "hello world" IR program
    pub fn create_hello_world_ir() -> SimpleProgram {
        let mut builder = SimpleIRBuilder::new();
        
        builder.comment("Hello World program");
        builder.label("main");
        
        // Print 'H'
        let h_char = builder.const_val('H' as i16);
        let zero = builder.const_val(0);
        builder.store(h_char, zero, zero);
        
        // Print 'i'
        let i_char = builder.const_val('i' as i16);
        builder.store(i_char, zero, zero);
        
        // Print newline
        let newline = builder.const_val('\n' as i16);
        builder.store(newline, zero, zero);
        
        builder.return_val(None);
        
        builder.build()
    }
    
    /// Create an arithmetic test program
    pub fn create_arithmetic_ir() -> SimpleProgram {
        let mut builder = SimpleIRBuilder::new();
        
        builder.comment("Arithmetic test");
        builder.label("test");
        
        let a = builder.const_val(10);
        let b = builder.const_val(5);
        
        let sum = builder.add(a, b);      // 10 + 5 = 15
        let diff = builder.sub(a, b);     // 10 - 5 = 5
        let prod = builder.mul(sum, diff); // 15 * 5 = 75
        
        let zero = builder.const_val(0);
        builder.store(prod, zero, zero);
        
        builder.return_val(Some(prod));
        
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_helpers::*;
    use pretty_assertions::assert_eq;
    
    #[test]
    fn test_lower_constants() {
        let mut program = SimpleProgram::new();
        let temp1 = program.new_temp();
        let temp2 = program.new_temp();
        
        program.push(SimpleIR::Const(temp1, 42));
        program.push(SimpleIR::Const(temp2, -10));
        
        let asm = lower_to_assembly(program).unwrap();
        
        assert_eq!(asm.len(), 2);
        assert!(matches!(asm[0], AsmInst::LI(_, 42)));
        assert!(matches!(asm[1], AsmInst::LI(_, -10)));
    }
    
    #[test]
    fn test_lower_arithmetic() {
        let mut program = SimpleProgram::new();
        let temp1 = program.new_temp(); // t0
        let temp2 = program.new_temp(); // t1
        let temp3 = program.new_temp(); // t2
        
        program.push(SimpleIR::Const(temp1, 5));
        program.push(SimpleIR::Const(temp2, 10));
        program.push(SimpleIR::Add(temp3, temp1, temp2));
        
        let asm = lower_to_assembly(program).unwrap();
        
        assert_eq!(asm.len(), 3);
        assert!(matches!(asm[0], AsmInst::LI(_, 5)));
        assert!(matches!(asm[1], AsmInst::LI(_, 10)));
        assert!(matches!(asm[2], AsmInst::Add(_, _, _)));
    }
    
    #[test]
    fn test_lower_memory_operations() {
        let mut program = SimpleProgram::new();
        let value = program.new_temp();
        let bank = program.new_temp();
        let addr = program.new_temp();
        let dest = program.new_temp();
        
        program.push(SimpleIR::Const(value, 42));
        program.push(SimpleIR::Const(bank, 0));
        program.push(SimpleIR::Const(addr, 0));
        program.push(SimpleIR::Store(value, bank, addr));
        program.push(SimpleIR::Load(dest, bank, addr));
        
        let asm = lower_to_assembly(program).unwrap();
        
        // Should have 3 constants, 1 store, 1 load
        assert_eq!(asm.len(), 5);
        assert!(matches!(asm[3], AsmInst::Store(_, _, _)));
        assert!(matches!(asm[4], AsmInst::Load(_, _, _)));
    }
    
    #[test]
    fn test_lower_hello_world() {
        let program = create_hello_world_ir();
        let asm = lower_to_assembly(program).unwrap();
        
        // Should contain label, constants for characters, store instructions
        let has_main_label = asm.iter().any(|inst| matches!(inst, AsmInst::Label(l) if l == "main"));
        let has_h_const = asm.iter().any(|inst| matches!(inst, AsmInst::LI(_, 72))); // 'H'
        let has_stores = asm.iter().filter(|inst| matches!(inst, AsmInst::Store(_, _, _))).count();
        let has_return = asm.iter().any(|inst| matches!(inst, AsmInst::Ret));
        
        assert!(has_main_label);
        assert!(has_h_const);
        assert_eq!(has_stores, 3); // 'H', 'i', '\n'
        assert!(has_return);
    }
    
    #[test]
    fn test_lower_arithmetic_program() {
        let program = create_arithmetic_ir();
        let asm = lower_to_assembly(program).unwrap();
        
        // Should contain arithmetic instructions
        let has_add = asm.iter().any(|inst| matches!(inst, AsmInst::Add(_, _, _)));
        let has_sub = asm.iter().any(|inst| matches!(inst, AsmInst::Sub(_, _, _)));
        let has_mul = asm.iter().any(|inst| matches!(inst, AsmInst::Mul(_, _, _)));
        
        assert!(has_add);
        assert!(has_sub);
        assert!(has_mul);
    }
    
    #[test]
    fn test_register_assignment() {
        let mut program = SimpleProgram::new();
        
        // Create several temporaries
        let temps: Vec<_> = (0..6).map(|_| program.new_temp()).collect();
        
        // Use them in operations
        for (i, &temp) in temps.iter().enumerate() {
            program.push(SimpleIR::Const(temp, i as i16));
        }
        
        let asm = lower_to_assembly(program).unwrap();
        
        // Should have successfully assigned registers to all temporaries
        assert_eq!(asm.len(), 6);
        for inst in asm {
            assert!(matches!(inst, AsmInst::LI(_, _)));
        }
    }
    
    #[test]
    fn test_return_value_handling() {
        let mut program = SimpleProgram::new();
        let temp = program.new_temp();
        
        program.push(SimpleIR::Const(temp, 42));
        program.push(SimpleIR::Return(Some(temp)));
        
        let asm = lower_to_assembly(program).unwrap();
        
        // Should move value to R3 if not already there, then return
        assert!(asm.len() >= 2);
        assert!(matches!(asm.last().unwrap(), AsmInst::Ret));
    }
}