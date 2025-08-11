use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0x00,
    Add = 0x01,
    Sub = 0x02,
    And = 0x03,
    Or = 0x04,
    Xor = 0x05,
    Sll = 0x06,
    Srl = 0x07,
    Slt = 0x08,
    Sltu = 0x09,
    Addi = 0x0A,
    Andi = 0x0B,
    Ori = 0x0C,
    Xori = 0x0D,
    Li = 0x0E,
    Slli = 0x0F,
    Srli = 0x10,
    Load = 0x11,
    Store = 0x12,
    Jal = 0x13,
    Jalr = 0x14,
    Beq = 0x15,
    Bne = 0x16,
    Blt = 0x17,
    Bge = 0x18,
    Brk = 0x19,
    Mul = 0x1A,
    Div = 0x1B,
    Mod = 0x1C,
    Muli = 0x1D,
    Divi = 0x1E,
    Modi = 0x1F,
}

impl Opcode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "NOP" => Some(Opcode::Nop),
            "ADD" => Some(Opcode::Add),
            "SUB" => Some(Opcode::Sub),
            "AND" => Some(Opcode::And),
            "OR" => Some(Opcode::Or),
            "XOR" => Some(Opcode::Xor),
            "SLL" => Some(Opcode::Sll),
            "SRL" => Some(Opcode::Srl),
            "SLT" => Some(Opcode::Slt),
            "SLTU" => Some(Opcode::Sltu),
            "ADDI" => Some(Opcode::Addi),
            "ANDI" => Some(Opcode::Andi),
            "ORI" => Some(Opcode::Ori),
            "XORI" => Some(Opcode::Xori),
            "LI" => Some(Opcode::Li),
            "SLLI" => Some(Opcode::Slli),
            "SRLI" => Some(Opcode::Srli),
            "LOAD" => Some(Opcode::Load),
            "STORE" => Some(Opcode::Store),
            "JAL" => Some(Opcode::Jal),
            "JALR" => Some(Opcode::Jalr),
            "BEQ" => Some(Opcode::Beq),
            "BNE" => Some(Opcode::Bne),
            "BLT" => Some(Opcode::Blt),
            "BGE" => Some(Opcode::Bge),
            "BRK" => Some(Opcode::Brk),
            "MUL" => Some(Opcode::Mul),
            "DIV" => Some(Opcode::Div),
            "MOD" => Some(Opcode::Mod),
            "MULI" => Some(Opcode::Muli),
            "DIVI" => Some(Opcode::Divi),
            "MODI" => Some(Opcode::Modi),
            _ => None,
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Opcode::Nop),
            0x01 => Some(Opcode::Add),
            0x02 => Some(Opcode::Sub),
            0x03 => Some(Opcode::And),
            0x04 => Some(Opcode::Or),
            0x05 => Some(Opcode::Xor),
            0x06 => Some(Opcode::Sll),
            0x07 => Some(Opcode::Srl),
            0x08 => Some(Opcode::Slt),
            0x09 => Some(Opcode::Sltu),
            0x0A => Some(Opcode::Addi),
            0x0B => Some(Opcode::Andi),
            0x0C => Some(Opcode::Ori),
            0x0D => Some(Opcode::Xori),
            0x0E => Some(Opcode::Li),
            0x0F => Some(Opcode::Slli),
            0x10 => Some(Opcode::Srli),
            0x11 => Some(Opcode::Load),
            0x12 => Some(Opcode::Store),
            0x13 => Some(Opcode::Jal),
            0x14 => Some(Opcode::Jalr),
            0x15 => Some(Opcode::Beq),
            0x16 => Some(Opcode::Bne),
            0x17 => Some(Opcode::Blt),
            0x18 => Some(Opcode::Bge),
            0x19 => Some(Opcode::Brk),
            0x1A => Some(Opcode::Mul),
            0x1B => Some(Opcode::Div),
            0x1C => Some(Opcode::Mod),
            0x1D => Some(Opcode::Muli),
            0x1E => Some(Opcode::Divi),
            0x1F => Some(Opcode::Modi),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Opcode::Nop => "NOP",
            Opcode::Add => "ADD",
            Opcode::Sub => "SUB",
            Opcode::And => "AND",
            Opcode::Or => "OR",
            Opcode::Xor => "XOR",
            Opcode::Sll => "SLL",
            Opcode::Srl => "SRL",
            Opcode::Slt => "SLT",
            Opcode::Sltu => "SLTU",
            Opcode::Addi => "ADDI",
            Opcode::Andi => "ANDI",
            Opcode::Ori => "ORI",
            Opcode::Xori => "XORI",
            Opcode::Li => "LI",
            Opcode::Slli => "SLLI",
            Opcode::Srli => "SRLI",
            Opcode::Load => "LOAD",
            Opcode::Store => "STORE",
            Opcode::Jal => "JAL",
            Opcode::Jalr => "JALR",
            Opcode::Beq => "BEQ",
            Opcode::Bne => "BNE",
            Opcode::Blt => "BLT",
            Opcode::Bge => "BGE",
            Opcode::Brk => "BRK",
            Opcode::Mul => "MUL",
            Opcode::Div => "DIV",
            Opcode::Mod => "MOD",
            Opcode::Muli => "MULI",
            Opcode::Divi => "DIVI",
            Opcode::Modi => "MODI",
        }
    }

    pub fn all() -> Vec<&'static str> {
        vec![
            "NOP", "ADD", "SUB", "AND", "OR", "XOR", "SLL", "SRL", "SLT", "SLTU",
            "ADDI", "ANDI", "ORI", "XORI", "LI", "SLLI", "SRLI", "LOAD", "STORE",
            "JAL", "JALR", "BEQ", "BNE", "BLT", "BGE", "BRK", "MUL", "DIV", "MOD",
            "MULI", "DIVI", "MODI"
        ]
    }

    pub fn format(&self) -> InstructionFormat {
        match self {
            Opcode::Nop | Opcode::Add | Opcode::Sub | Opcode::And | Opcode::Or | 
            Opcode::Xor | Opcode::Sll | Opcode::Srl | Opcode::Slt | Opcode::Sltu | 
            Opcode::Jalr | Opcode::Brk | Opcode::Mul | Opcode::Div | Opcode::Mod => InstructionFormat::R,
            
            Opcode::Addi | Opcode::Andi | Opcode::Ori | Opcode::Xori | 
            Opcode::Slli | Opcode::Srli | Opcode::Load | Opcode::Store | 
            Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge | 
            Opcode::Jal | Opcode::Muli | Opcode::Divi | Opcode::Modi => InstructionFormat::I,
            
            Opcode::Li => InstructionFormat::I1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    R0 = 0,
    Pc = 1,
    Pcb = 2,
    Ra = 3,
    Rab = 4,
    R3 = 5,
    R4 = 6,
    R5 = 7,
    R6 = 8,
    R7 = 9,
    R8 = 10,
    R9 = 11,
    R10 = 12,
    R11 = 13,
    R12 = 14,
    R13 = 15,
    R14 = 16,
    R15 = 17,
}

impl Register {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Register::R0),
            1 => Some(Register::Pc),
            2 => Some(Register::Pcb),
            3 => Some(Register::Ra),
            4 => Some(Register::Rab),
            5 => Some(Register::R3),
            6 => Some(Register::R4),
            7 => Some(Register::R5),
            8 => Some(Register::R6),
            9 => Some(Register::R7),
            10 => Some(Register::R8),
            11 => Some(Register::R9),
            12 => Some(Register::R10),
            13 => Some(Register::R11),
            14 => Some(Register::R12),
            15 => Some(Register::R13),
            16 => Some(Register::R14),
            17 => Some(Register::R15),
            _ => None,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "R0" => Some(Register::R0),
            "PC" => Some(Register::Pc),
            "PCB" => Some(Register::Pcb),
            "RA" => Some(Register::Ra),
            "RAB" => Some(Register::Rab),
            "R3" => Some(Register::R3),
            "R4" => Some(Register::R4),
            "R5" => Some(Register::R5),
            "R6" => Some(Register::R6),
            "R7" => Some(Register::R7),
            "R8" => Some(Register::R8),
            "R9" => Some(Register::R9),
            "R10" => Some(Register::R10),
            "R11" => Some(Register::R11),
            "R12" => Some(Register::R12),
            "R13" => Some(Register::R13),
            "R14" => Some(Register::R14),
            "R15" => Some(Register::R15),
            _ => None,
        }
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            Register::R0 => "R0",
            Register::Pc => "PC",
            Register::Pcb => "PCB",
            Register::Ra => "RA",
            Register::Rab => "RAB",
            Register::R3 => "R3",
            Register::R4 => "R4",
            Register::R5 => "R5",
            Register::R6 => "R6",
            Register::R7 => "R7",
            Register::R8 => "R8",
            Register::R9 => "R9",
            Register::R10 => "R10",
            Register::R11 => "R11",
            Register::R12 => "R12",
            Register::R13 => "R13",
            Register::R14 => "R14",
            Register::R15 => "R15",
        }
    }

    pub fn to_macro_str(&self) -> &'static str {
        match self {
            Register::R0 => "@R0",
            Register::Pc => "@PC",
            Register::Pcb => "@PCB",
            Register::Ra => "@RA",
            Register::Rab => "@RAB",
            Register::R3 => "@R3",
            Register::R4 => "@R4",
            Register::R5 => "@R5",
            Register::R6 => "@R6",
            Register::R7 => "@R7",
            Register::R8 => "@R8",
            Register::R9 => "@R9",
            Register::R10 => "@R10",
            Register::R11 => "@R11",
            Register::R12 => "@R12",
            Register::R13 => "@R13",
            Register::R14 => "@R14",
            Register::R15 => "@R15",
        }
    }

    pub fn all() -> Vec<&'static str> {
        vec![
            "R0", "PC", "PCB", "RA", "RAB", "R3", "R4", "R5", 
            "R6", "R7", "R8", "R9", "R10", "R11", "R12", "R13", 
            "R14", "R15"
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    R,  // Register format
    I,  // Immediate format
    I1, // Special immediate format for LI
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: u8,
    pub word0: u8,
    pub word1: u16,
    pub word2: u16,
    pub word3: u16,
}

impl Instruction {
    pub fn new(opcode: Opcode, word1: u16, word2: u16, word3: u16) -> Self {
        Self {
            opcode: opcode as u8,
            word0: opcode as u8,
            word1,
            word2,
            word3,
        }
    }

    pub fn is_halt(&self) -> bool {
        self.opcode == Opcode::Nop as u8 && 
        self.word1 == 0 && 
        self.word2 == 0 && 
        self.word3 == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    pub bank: u16,
    pub offset: u16,
    pub absolute_address: u32,
}

#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub label: Option<String>,
    pub mnemonic: Option<String>,
    pub operands: Vec<String>,
    pub directive: Option<String>,
    pub directive_args: Vec<String>,
    pub line_number: usize,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Code,
    Data,
}

#[derive(Debug)]
pub struct AssemblerOptions {
    pub case_insensitive: bool,
    pub start_bank: u16,
    pub bank_size: u16,
    pub max_immediate: u32,
    pub memory_offset: u16,  // Offset to add to all memory addresses (default 2 for VM special values)
}

impl Default for AssemblerOptions {
    fn default() -> Self {
        Self {
            case_insensitive: true,
            start_bank: 0,
            bank_size: DEFAULT_BANK_SIZE,
            max_immediate: DEFAULT_MAX_IMMEDIATE,
            memory_offset: 2,  // Default to 2 to account for VM special values
        }
    }
}

pub const DEFAULT_BANK_SIZE: u16 = 16;
pub const INSTRUCTION_SIZE: u16 = 4;
pub const DEFAULT_MAX_IMMEDIATE: u32 = 65535;

#[derive(Debug)]
pub struct AssemblerState {
    pub current_bank: u16,
    pub current_offset: u16,
    pub labels: HashMap<String, Label>,
    pub data_labels: HashMap<String, u32>,
    pub pending_references: HashMap<usize, PendingReference>,
    pub instructions: Vec<Instruction>,
    pub memory_data: Vec<u8>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PendingReference {
    pub label: String,
    pub ref_type: ReferenceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
    Branch,
    Absolute,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectFile {
    pub version: u32,
    pub instructions: Vec<Instruction>,
    pub data: Vec<u8>,
    pub labels: HashMap<String, Label>,
    pub data_labels: HashMap<String, u32>,
    pub unresolved_references: HashMap<usize, UnresolvedReference>,
    pub entry_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedReference {
    pub label: String,
    pub ref_type: String, // "branch", "absolute", "data"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archive {
    pub version: u32,
    pub objects: Vec<ArchiveEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub name: String,  // Original filename or module name
    pub object: ObjectFile,
}

// Virtual instruction definitions for extensibility
pub trait VirtualInstruction {
    fn name(&self) -> &str;
    fn expand(&self, operands: &[String]) -> Result<Vec<ParsedLine>, String>;
}