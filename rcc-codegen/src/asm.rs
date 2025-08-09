//! Ripple Assembly Instruction Definitions
//! 
//! This module defines the instruction set and register model for the Ripple VM.

use std::fmt;

/// Ripple VM Register Set
/// 
/// The Ripple VM has 20 registers total:
/// - R0-R15: General purpose registers (R0 is typically zero/scratch)
/// - PC, PCB: Program counter and program counter bank
/// - RA, RAB: Return address and return address bank
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reg {
    // General purpose registers
    R0, R1, R2, R3, R4, R5, R6, R7,
    R8, R9, R10, R11, R12, R13, R14, R15,
    
    // Special registers
    PC,   // Program Counter
    PCB,  // Program Counter Bank
    RA,   // Return Address
    RAB,  // Return Address Bank
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reg::R0 => write!(f, "R0"),
            Reg::R1 => write!(f, "R1"),
            Reg::R2 => write!(f, "R2"),
            Reg::R3 => write!(f, "R3"),
            Reg::R4 => write!(f, "R4"),
            Reg::R5 => write!(f, "R5"),
            Reg::R6 => write!(f, "R6"),
            Reg::R7 => write!(f, "R7"),
            Reg::R8 => write!(f, "R8"),
            Reg::R9 => write!(f, "R9"),
            Reg::R10 => write!(f, "R10"),
            Reg::R11 => write!(f, "R11"),
            Reg::R12 => write!(f, "R12"),
            Reg::R13 => write!(f, "R13"),
            Reg::R14 => write!(f, "R14"),
            Reg::R15 => write!(f, "R15"),
            Reg::PC => write!(f, "PC"),
            Reg::PCB => write!(f, "PCB"),
            Reg::RA => write!(f, "RA"),
            Reg::RAB => write!(f, "RAB"),
        }
    }
}

/// Ripple Assembly Instructions
/// 
/// This enum represents all instructions that the Ripple VM can execute.
/// Instructions are categorized into arithmetic, logical, memory, control flow,
/// and pseudo-instructions for assembly generation.
#[derive(Debug, Clone, PartialEq)]
pub enum AsmInst {
    // Arithmetic Instructions
    Add(Reg, Reg, Reg),           // rd = rs + rt
    Sub(Reg, Reg, Reg),           // rd = rs - rt
    Mul(Reg, Reg, Reg),           // rd = rs * rt
    Div(Reg, Reg, Reg),           // rd = rs / rt
    Mod(Reg, Reg, Reg),           // rd = rs % rt
    
    // Arithmetic with Immediate
    AddI(Reg, Reg, i16),          // rd = rs + imm
    SubI(Reg, Reg, i16),          // rd = rs - imm
    MulI(Reg, Reg, i16),          // rd = rs * imm
    DivI(Reg, Reg, i16),          // rd = rs / imm
    ModI(Reg, Reg, i16),          // rd = rs % imm
    
    // Logical Instructions
    And(Reg, Reg, Reg),           // rd = rs & rt
    Or(Reg, Reg, Reg),            // rd = rs | rt
    Xor(Reg, Reg, Reg),           // rd = rs ^ rt
    Sll(Reg, Reg, Reg),           // rd = rs << rt
    Srl(Reg, Reg, Reg),           // rd = rs >> rt (logical)
    Slt(Reg, Reg, Reg),           // rd = (rs < rt) ? 1 : 0
    Sltu(Reg, Reg, Reg),          // rd = (rs < rt) ? 1 : 0 (unsigned)
    
    // Memory Instructions
    Load(Reg, Reg, Reg),          // rd = memory[bank:addr]
    Store(Reg, Reg, Reg),         // memory[bank:addr] = rs
    LI(Reg, i16),                 // rd = immediate
    
    // Control Flow Instructions
    Jal(i16, i16),                // Jump and link (bank_imm, addr_imm)
    Jalr(Reg, Reg, Reg),          // Jump and link register (bank_reg, addr_reg, link_reg)
    Beq(Reg, Reg, String),        // Branch if equal (rs, rt, label)
    Bne(Reg, Reg, String),        // Branch if not equal (rs, rt, label)
    Blt(Reg, Reg, String),        // Branch if less than (rs, rt, label)
    Bge(Reg, Reg, String),        // Branch if greater or equal (rs, rt, label)
    
    // Virtual/Pseudo Instructions (expand to real instructions)
    Move(Reg, Reg),               // rd = rs
    Inc(Reg),                     // rd = rd + 1
    Dec(Reg),                     // rd = rd - 1
    Push(Reg),                    // Push register to stack
    Pop(Reg),                     // Pop from stack to register
    Call(String),                 // Call function by label
    Ret,                          // Return from function
    
    // System Instructions
    Brk,                          // Breakpoint
    Halt,                         // Halt execution
    
    // Assembly Pseudo-Instructions
    Label(String),                // Label for jumps/calls
    Comment(String),              // Assembly comment
}

impl fmt::Display for AsmInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Arithmetic
            AsmInst::Add(rd, rs, rt) => write!(f, "ADD {}, {}, {}", rd, rs, rt),
            AsmInst::Sub(rd, rs, rt) => write!(f, "SUB {}, {}, {}", rd, rs, rt),
            AsmInst::Mul(rd, rs, rt) => write!(f, "MUL {}, {}, {}", rd, rs, rt),
            AsmInst::Div(rd, rs, rt) => write!(f, "DIV {}, {}, {}", rd, rs, rt),
            AsmInst::Mod(rd, rs, rt) => write!(f, "MOD {}, {}, {}", rd, rs, rt),
            
            // Arithmetic Immediate
            AsmInst::AddI(rd, rs, imm) => write!(f, "ADDI {}, {}, {}", rd, rs, imm),
            AsmInst::SubI(rd, rs, imm) => write!(f, "SUBI {}, {}, {}", rd, rs, imm),
            AsmInst::MulI(rd, rs, imm) => write!(f, "MULI {}, {}, {}", rd, rs, imm),
            AsmInst::DivI(rd, rs, imm) => write!(f, "DIVI {}, {}, {}", rd, rs, imm),
            AsmInst::ModI(rd, rs, imm) => write!(f, "MODI {}, {}, {}", rd, rs, imm),
            
            // Logical
            AsmInst::And(rd, rs, rt) => write!(f, "AND {}, {}, {}", rd, rs, rt),
            AsmInst::Or(rd, rs, rt) => write!(f, "OR {}, {}, {}", rd, rs, rt),
            AsmInst::Xor(rd, rs, rt) => write!(f, "XOR {}, {}, {}", rd, rs, rt),
            AsmInst::Sll(rd, rs, rt) => write!(f, "SLL {}, {}, {}", rd, rs, rt),
            AsmInst::Srl(rd, rs, rt) => write!(f, "SRL {}, {}, {}", rd, rs, rt),
            AsmInst::Slt(rd, rs, rt) => write!(f, "SLT {}, {}, {}", rd, rs, rt),
            AsmInst::Sltu(rd, rs, rt) => write!(f, "SLTU {}, {}, {}", rd, rs, rt),
            
            // Memory
            AsmInst::Load(rd, bank, addr) => write!(f, "LOAD {}, {}, {}", rd, bank, addr),
            AsmInst::Store(rs, bank, addr) => write!(f, "STORE {}, {}, {}", rs, bank, addr),
            AsmInst::LI(rd, imm) => write!(f, "LI {}, {}", rd, imm),
            
            // Control Flow
            AsmInst::Jal(bank, addr) => write!(f, "JAL {}, {}", bank, addr),
            AsmInst::Jalr(bank, addr, link) => write!(f, "JALR {}, {}, {}", bank, addr, link),
            AsmInst::Beq(rs, rt, label) => write!(f, "BEQ {}, {}, {}", rs, rt, label),
            AsmInst::Bne(rs, rt, label) => write!(f, "BNE {}, {}, {}", rs, rt, label),
            AsmInst::Blt(rs, rt, label) => write!(f, "BLT {}, {}, {}", rs, rt, label),
            AsmInst::Bge(rs, rt, label) => write!(f, "BGE {}, {}, {}", rs, rt, label),
            
            // Virtual/Pseudo
            AsmInst::Move(rd, rs) => write!(f, "MOVE {}, {}", rd, rs),
            AsmInst::Inc(rd) => write!(f, "INC {}", rd),
            AsmInst::Dec(rd) => write!(f, "DEC {}", rd),
            AsmInst::Push(rs) => write!(f, "PUSH {}", rs),
            AsmInst::Pop(rd) => write!(f, "POP {}", rd),
            AsmInst::Call(label) => write!(f, "CALL {}", label),
            AsmInst::Ret => write!(f, "RET"),
            
            // System
            AsmInst::Brk => write!(f, "BRK"),
            AsmInst::Halt => write!(f, "HALT"),
            
            // Pseudo
            AsmInst::Label(label) => write!(f, "{}:", label),
            AsmInst::Comment(text) => write!(f, "; {}", text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_display() {
        assert_eq!(format!("{}", Reg::R0), "R0");
        assert_eq!(format!("{}", Reg::R15), "R15");
        assert_eq!(format!("{}", Reg::PC), "PC");
        assert_eq!(format!("{}", Reg::RA), "RA");
    }

    #[test]
    fn test_instruction_display() {
        assert_eq!(format!("{}", AsmInst::LI(Reg::R3, 42)), "LI R3, 42");
        assert_eq!(format!("{}", AsmInst::Add(Reg::R1, Reg::R2, Reg::R3)), "ADD R1, R2, R3");
        assert_eq!(format!("{}", AsmInst::Store(Reg::R3, Reg::R0, Reg::R1)), "STORE R3, R0, R1");
        assert_eq!(format!("{}", AsmInst::Label("main".to_string())), "main:");
        assert_eq!(format!("{}", AsmInst::Comment("Hello world".to_string())), "; Hello world");
    }

    #[test]
    fn test_hello_world_instructions() {
        let instructions = vec![
            AsmInst::Label("main".to_string()),
            AsmInst::LI(Reg::R3, 'H' as i16),
            AsmInst::Store(Reg::R3, Reg::R0, Reg::R0),
            AsmInst::LI(Reg::R3, 'i' as i16),
            AsmInst::Store(Reg::R3, Reg::R0, Reg::R0),
            AsmInst::LI(Reg::R3, '\n' as i16),
            AsmInst::Store(Reg::R3, Reg::R0, Reg::R0),
            AsmInst::Halt,
        ];

        for inst in instructions {
            println!("{}", inst);
        }
    }
}