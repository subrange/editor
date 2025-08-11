use std::collections::VecDeque;
use ripple_asm::Register;
use crate::constants::*;

#[derive(Debug, Clone, Copy)]
pub struct Instr {
    pub opcode: u8,
    pub word0: u8,
    pub word1: u16,
    pub word2: u16,
    pub word3: u16,
}

impl Instr {
    #[allow(dead_code)]
    pub fn new(opcode: u8, word0: u8, word1: u16, word2: u16, word3: u16) -> Self {
        Self { opcode, word0, word1, word2, word3 }
    }
    
    #[allow(dead_code)]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        Some(Self {
            opcode: bytes[0],
            word0: bytes[1],
            word1: u16::from_le_bytes([bytes[2], bytes[3]]),
            word2: u16::from_le_bytes([bytes[4], bytes[5]]),
            word3: u16::from_le_bytes([bytes[6], bytes[7]]),
        })
    }
    
    #[allow(dead_code)]
    pub fn is_halt(&self) -> bool {
        self.opcode == 0x00 && self.word1 == 0 && self.word2 == 0 && self.word3 == 0
    }
}

#[derive(Debug)]
pub enum VMState {
    Setup,
    Running,
    Halted,
    Breakpoint,  // Hit a BRK instruction in debug mode
    Error(String),
}

pub struct VM {
    // Program memory - stores instructions
    pub instructions: Vec<Instr>,
    
    // Data memory - separate from instructions
    pub memory: Vec<u16>,
    
    // Registers (18 total: R0, PC, PCB, RA, RAB, R3-R15)
    pub registers: [u16; 18],
    
    // VM state
    pub state: VMState,
    
    // Configuration
    pub bank_size: u16,
    
    // Debug mode flag
    pub debug_mode: bool,
    
    // Do not increment PC flag (set by jump/branch instructions)
    skip_pc_increment: bool,
    
    // Output buffer for I/O
    pub output_buffer: VecDeque<u8>,
    output_ready: bool,
}


impl VM {
    pub fn new(bank_size: u16) -> Self {
        Self::with_memory_size(bank_size, DEFAULT_MEMORY_SIZE)
    }
    
    pub fn with_memory_size(bank_size: u16, memory_size: usize) -> Self {
        let memory_size = memory_size.max(MIN_MEMORY_SIZE);
        VM {
            instructions: Vec::new(),
            memory: vec![0; memory_size],
            registers: [0; MAX_REGISTERS],
            state: VMState::Setup,
            bank_size,
            debug_mode: false,
            skip_pc_increment: false,
            output_buffer: VecDeque::new(),
            output_ready: true,
        }
    }
    
    #[allow(dead_code)]
    pub fn new_default() -> Self {
        Self::new(DEFAULT_BANK_SIZE)
    }
    
    pub fn load_binary(&mut self, binary: &[u8]) -> Result<(), String> {
        // Check magic number
        if binary.len() < 5 || &binary[0..5] != MAGIC_RLINK {
            return Err("Invalid binary format: missing RLINK magic".to_string());
        }
        
        let mut pos = 5;
        
        // Read entry point
        if pos + 4 > binary.len() {
            return Err("Invalid binary: missing entry point".to_string());
        }
        let entry_point = u32::from_le_bytes([
            binary[pos], binary[pos+1], binary[pos+2], binary[pos+3]
        ]);
        pos += 4;
        
        // Read instruction count
        if pos + 4 > binary.len() {
            return Err("Invalid binary: missing instruction count".to_string());
        }
        let instruction_count = u32::from_le_bytes([
            binary[pos], binary[pos+1], binary[pos+2], binary[pos+3]
        ]) as usize;
        pos += 4;
        
        // Read instructions
        self.instructions.clear();
        for i in 0..instruction_count {
            if pos + INSTRUCTION_SIZE > binary.len() {
                return Err(format!("Invalid binary: missing instruction {}", i));
            }
            
            let instr = Instr {
                opcode: binary[pos],
                word0: binary[pos + 1],
                word1: u16::from_le_bytes([binary[pos + 2], binary[pos + 3]]),
                word2: u16::from_le_bytes([binary[pos + 4], binary[pos + 5]]),
                word3: u16::from_le_bytes([binary[pos + 6], binary[pos + 7]]),
            };
            self.instructions.push(instr);
            pos += INSTRUCTION_SIZE;
        }
        
        // Read data section size
        if pos + 4 > binary.len() {
            return Err("Invalid binary: missing data size".to_string());
        }
        let data_size = u32::from_le_bytes([
            binary[pos], binary[pos+1], binary[pos+2], binary[pos+3]
        ]) as usize;
        pos += 4;
        
        // Read data section
        if pos + data_size > binary.len() {
            return Err("Invalid binary: incomplete data section".to_string());
        }
        
        // Load data into memory starting at address 2 (after I/O registers)
        for (i, &byte) in binary[pos..pos + data_size].iter().enumerate() {
            if i < self.memory.len() - DATA_SECTION_OFFSET {
                self.memory[i + DATA_SECTION_OFFSET] = byte as u16;
            }
        }
        
        // Set entry point
        let entry_bank = (entry_point / (self.bank_size as u32 * 4)) as u16;
        let entry_offset = ((entry_point / 4) % (self.bank_size as u32)) as u16;
        self.registers[Register::Pcb as usize] = entry_bank;
        self.registers[Register::Pc as usize] = entry_offset;
        
        // Initialize memory-mapped I/O
        self.memory[MMIO_OUT] = 0;
        self.memory[MMIO_OUT_FLAG] = OUTPUT_READY;
        
        self.state = VMState::Running;
        Ok(())
    }
    
    pub fn step(&mut self) -> Result<(), String> {
        match self.state {
            VMState::Running => {},
            VMState::Halted => return Ok(()),
            VMState::Breakpoint => {
                // In debug mode at breakpoint.rs, allow single stepping
                // State will be reset to Running by the debugger_ui
            },
            VMState::Error(ref e) => return Err(e.clone()),
            VMState::Setup => return Err("VM not initialized".to_string()),
        }
        
        // Calculate instruction address
        let pc = self.registers[Register::Pc as usize];
        let pcb = self.registers[Register::Pcb as usize];
        let instr_idx = (pcb as usize * self.bank_size as usize) + pc as usize;
        
        if instr_idx >= self.instructions.len() {
            self.state = VMState::Error(format!("PC out of bounds: bank={}, offset={}, idx={}, total_instructions={}", 
                                               pcb, pc, instr_idx, self.instructions.len()));
            return Err(format!("PC out of bounds: idx={} >= len={}", instr_idx, self.instructions.len()));
        }
        
        let instr = self.instructions[instr_idx];
        self.skip_pc_increment = false;
        
        // Execute instruction
        self.execute_instruction(instr)?;
        
        // Increment PC unless instruction set the skip flag
        if !self.skip_pc_increment {
            let mut new_pc = self.registers[Register::Pc as usize] as u32 + 1;
            let mut new_pcb = self.registers[Register::Pcb as usize] as u32;
            
            if new_pc >= self.bank_size as u32 {
                new_pc = 0;
                new_pcb += 1;
            }
            
            self.registers[Register::Pc as usize] = (new_pc & 0xFFFF) as u16;
            self.registers[Register::Pcb as usize] = (new_pcb & 0xFFFF) as u16;
        }
        
        Ok(())
    }
    
    fn execute_instruction(&mut self, instr: Instr) -> Result<(), String> {
        // R0 always reads as 0
        self.registers[Register::R0 as usize] = 0;
        
        match instr.opcode {
            0x00 => {
                // NOP or HALT (HALT is NOP with all operands 0)
                if instr.word0 == 0 && instr.word1 == 0 && instr.word2 == 0 && instr.word3 == 0 {
                    // HALT
                    self.state = VMState::Halted;
                }
                // else NOP - do nothing
            },
            
            // ALU R-type operations
            0x01 => { // ADD
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    self.registers[rd] = self.registers[rs].wrapping_add(self.registers[rt]);
                }
            },
            0x02 => { // SUB
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    self.registers[rd] = self.registers[rs].wrapping_sub(self.registers[rt]);
                }
            },
            0x03 => { // AND
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    self.registers[rd] = self.registers[rs] & self.registers[rt];
                }
            },
            0x04 => { // OR
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    self.registers[rd] = self.registers[rs] | self.registers[rt];
                }
            },
            0x05 => { // XOR
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    self.registers[rd] = self.registers[rs] ^ self.registers[rt];
                }
            },
            0x06 => { // SLL
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    let shift = (self.registers[rt] & 15) as u32;
                    self.registers[rd] = self.registers[rs].wrapping_shl(shift);
                }
            },
            0x07 => { // SRL
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    let shift = (self.registers[rt] & 15) as u32;
                    self.registers[rd] = self.registers[rs].wrapping_shr(shift);
                }
            },
            0x08 => { // SLT (signed compare)
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    let rs_val = self.registers[rs] as i16;
                    let rt_val = self.registers[rt] as i16;
                    self.registers[rd] = if rs_val < rt_val { 1 } else { 0 };
                }
            },
            0x09 => { // SLTU (unsigned compare)
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    self.registers[rd] = if self.registers[rs] < self.registers[rt] { 1 } else { 0 };
                }
            },
            
            // ALU I-type operations
            0x0A => { // ADDI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 18 && rs < 18 {
                    self.registers[rd] = self.registers[rs].wrapping_add(imm);
                }
            },
            0x0B => { // ANDI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 18 && rs < 18 {
                    self.registers[rd] = self.registers[rs] & imm;
                }
            },
            0x0C => { // ORI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 18 && rs < 18 {
                    self.registers[rd] = self.registers[rs] | imm;
                }
            },
            0x0D => { // XORI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 18 && rs < 18 {
                    self.registers[rd] = self.registers[rs] ^ imm;
                }
            },
            0x0E => { // LI
                let rd = instr.word1 as usize;
                let imm = instr.word2;
                if rd < 18 {
                    self.registers[rd] = imm;
                }
            },
            0x0F => { // SLLI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3 as u32;
                if rd < 18 && rs < 18 {
                    self.registers[rd] = self.registers[rs].wrapping_shl(imm & 15);
                }
            },
            0x10 => { // SRLI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3 as u32;
                if rd < 18 && rs < 18 {
                    self.registers[rd] = self.registers[rs].wrapping_shr(imm & 15);
                }
            },
            
            // Memory operations
            0x11 => { // LOAD - rd = memory[bank_reg][addr_reg]
                let rd = instr.word1 as usize;
                let bank_reg = instr.word2 as usize;
                let addr_reg = instr.word3 as usize;
                if rd < 18 && bank_reg < 18 && addr_reg < 18 {
                    let bank_val = self.registers[bank_reg];
                    let addr_val = self.registers[addr_reg];
                    
                    // Memory is separate from instructions
                    // BANK_SIZE refers to memory cells, not instructions
                    let mem_addr = (bank_val as usize * self.bank_size as usize) + addr_val as usize;
                    if mem_addr < self.memory.len() {
                        self.registers[rd] = self.memory[mem_addr];
                    } else {
                        return Err(format!("LOAD: memory address out of bounds: {}", mem_addr));
                    }
                }
            },
            0x12 => { // STORE - memory[bank_reg][addr_reg] = rs
                let rs = instr.word1 as usize;
                let bank_reg = instr.word2 as usize;
                let addr_reg = instr.word3 as usize;
                if rs < 18 && bank_reg < 18 && addr_reg < 18 {
                    let bank_val = self.registers[bank_reg];
                    let addr_val = self.registers[addr_reg];
                    
                    // Memory is separate from instructions
                    // BANK_SIZE refers to memory cells, not instructions
                    let mem_addr = (bank_val as usize * self.bank_size as usize) + addr_val as usize;
                    if mem_addr < self.memory.len() {
                        let value = self.registers[rs];
                        self.memory[mem_addr] = value;
                        
                        // Handle memory-mapped I/O
                        if mem_addr == MMIO_OUT {
                            // Output register
                            self.output_buffer.push_back((value & 0xFF) as u8);
                            self.output_ready = false;
                            // Simulate output delay
                            self.memory[MMIO_OUT_FLAG] = OUTPUT_BUSY;
                        }
                    } else {
                        return Err(format!("STORE: memory address out of bounds: {}", mem_addr));
                    }
                }
            },
            
            // Control flow
            0x13 => { // JAL
                // JAL format: rd, offset_high, offset_low
                // The actual jump address is in word3 (after linking)
                let rd = instr.word1 as usize;
                let addr = instr.word3;
                
                // Save return address in rd (typically RA)
                if rd < 18 {
                    self.registers[rd] = self.registers[Register::Pc as usize].wrapping_add(1);
                }
                self.registers[Register::Rab as usize] = self.registers[Register::Pcb as usize];
                
                // Jump to address
                self.registers[Register::Pc as usize] = addr;
                self.skip_pc_increment = true;
            },
            0x14 => { // JALR
                let rd = instr.word1 as usize;
                let rs = instr.word3 as usize; // Note: rs is in word3 for JALR
                if rd < 18 && rs < 18 {
                    // Save return address
                    self.registers[rd] = self.registers[Register::Pc as usize].wrapping_add(1);
                    self.registers[Register::Rab as usize] = self.registers[Register::Pcb as usize];
                    // Jump
                    self.registers[Register::Pc as usize] = self.registers[rs];
                    self.skip_pc_increment = true;
                }
            },
            0x15 => { // BEQ
                let rs = instr.word1 as usize;
                let rt = instr.word2 as usize;
                let offset = instr.word3 as i16;
                if rs < 18 && rt < 18 {
                    if self.registers[rs] == self.registers[rt] {
                        let new_pc = (self.registers[Register::Pc as usize] as i16).wrapping_add(offset);
                        self.registers[Register::Pc as usize] = new_pc as u16;
                        self.skip_pc_increment = true;
                    }
                }
            },
            0x16 => { // BNE
                let rs = instr.word1 as usize;
                let rt = instr.word2 as usize;
                let offset = instr.word3 as i16;
                if rs < 18 && rt < 18 {
                    if self.registers[rs] != self.registers[rt] {
                        let new_pc = (self.registers[Register::Pc as usize] as i16).wrapping_add(offset);
                        self.registers[Register::Pc as usize] = new_pc as u16;
                        self.skip_pc_increment = true;
                    }
                }
            },
            0x17 => { // BLT
                let rs = instr.word1 as usize;
                let rt = instr.word2 as usize;
                let offset = instr.word3 as i16;
                if rs < 18 && rt < 18 {
                    let rs_val = self.registers[rs] as i16;
                    let rt_val = self.registers[rt] as i16;
                    if rs_val < rt_val {
                        let new_pc = (self.registers[Register::Pc as usize] as i16).wrapping_add(offset);
                        self.registers[Register::Pc as usize] = new_pc as u16;
                        self.skip_pc_increment = true;
                    }
                }
            },
            0x18 => { // BGE
                let rs = instr.word1 as usize;
                let rt = instr.word2 as usize;
                let offset = instr.word3 as i16;
                if rs < 18 && rt < 18 {
                    let rs_val = self.registers[rs] as i16;
                    let rt_val = self.registers[rt] as i16;
                    if rs_val >= rt_val {
                        let new_pc = (self.registers[Register::Pc as usize] as i16).wrapping_add(offset);
                        self.registers[Register::Pc as usize] = new_pc as u16;
                        self.skip_pc_increment = true;
                    }
                }
            },
            
            0x19 => { // BRK - debugger_ui breakpoint.rs
                if self.debug_mode {
                    // In debug mode, just pause execution
                    self.state = VMState::Breakpoint;
                    // Don't print here - the debugger_ui will handle it
                } else {
                    // In normal mode, dump state and halt
                    eprintln!("\n=== BRK: VM State Dump ===");
                    eprintln!("PC: {} (bank: {})", self.registers[Register::Pc as usize], self.registers[Register::Pcb as usize]);
                    eprintln!("RA: {} (bank: {})", self.registers[Register::Ra as usize], self.registers[Register::Rab as usize]);
                    
                    eprintln!("\nRegisters:");
                    eprintln!("  R0:  0x{:04X} ({})", self.registers[Register::R0 as usize], self.registers[Register::R0 as usize]);
                    eprintln!("  R3:  0x{:04X} ({})", self.registers[Register::R3 as usize], self.registers[Register::R3 as usize]);
                    eprintln!("  R4:  0x{:04X} ({})", self.registers[Register::R4 as usize], self.registers[Register::R4 as usize]);
                    eprintln!("  R5:  0x{:04X} ({})", self.registers[Register::R5 as usize], self.registers[Register::R5 as usize]);
                    eprintln!("  R6:  0x{:04X} ({})", self.registers[Register::R6 as usize], self.registers[Register::R6 as usize]);
                    eprintln!("  R7:  0x{:04X} ({})", self.registers[Register::R7 as usize], self.registers[Register::R7 as usize]);
                    eprintln!("  R8:  0x{:04X} ({})", self.registers[Register::R8 as usize], self.registers[Register::R8 as usize]);
                    eprintln!("  R9:  0x{:04X} ({})", self.registers[Register::R9 as usize], self.registers[Register::R9 as usize]);
                    eprintln!("  R10: 0x{:04X} ({})", self.registers[Register::R10 as usize], self.registers[Register::R10 as usize]);
                    eprintln!("  R11: 0x{:04X} ({})", self.registers[Register::R11 as usize], self.registers[Register::R11 as usize]);
                    eprintln!("  R12: 0x{:04X} ({})", self.registers[Register::R12 as usize], self.registers[Register::R12 as usize]);
                    eprintln!("  R13: 0x{:04X} ({})", self.registers[Register::R13 as usize], self.registers[Register::R13 as usize]);
                    eprintln!("  R14: 0x{:04X} ({})", self.registers[Register::R14 as usize], self.registers[Register::R14 as usize]);
                    eprintln!("  R15: 0x{:04X} ({})", self.registers[Register::R15 as usize], self.registers[Register::R15 as usize]);
                    
                    eprintln!("\nMemory (first {} words):", DEBUG_MEMORY_DISPLAY_WORDS);
                    for i in (0..DEBUG_MEMORY_DISPLAY_WORDS.min(self.memory.len())).step_by(DEBUG_MEMORY_WORDS_PER_LINE) {
                        eprint!("  {:04X}: ", i);
                        for j in 0..DEBUG_MEMORY_WORDS_PER_LINE {
                            if i + j < self.memory.len() {
                                eprint!("{:04X} ", self.memory[i + j]);
                            }
                        }
                        eprintln!();
                    }
                    
                    eprintln!("\nInstruction at PC:");
                    let pc_val = self.registers[Register::Pc as usize] as usize;
                    if pc_val < self.instructions.len() {
                        let inst = &self.instructions[pc_val];
                        eprintln!("  [{:04X}] opcode: 0x{:02X}, w1: 0x{:04X}, w2: 0x{:04X}, w3: 0x{:04X}", 
                            pc_val, inst.opcode, inst.word1, inst.word2, inst.word3);
                    }
                    
                    eprintln!("=========================\n");
                    
                    // Halt execution
                    self.state = VMState::Halted;
                    return Ok(());
                }
            },
            
            // Multiplication and division
            0x1A => { // MUL
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    self.registers[rd] = self.registers[rs].wrapping_mul(self.registers[rt]);
                }
            },
            0x1B => { // DIV
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    if self.registers[rt] != 0 {
                        self.registers[rd] = self.registers[rs] / self.registers[rt];
                    } else {
                        self.registers[rd] = 0; // Division by zero results in 0
                    }
                }
            },
            0x1C => { // MOD
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 18 && rs < 18 && rt < 18 {
                    if self.registers[rt] != 0 {
                        self.registers[rd] = self.registers[rs] % self.registers[rt];
                    } else {
                        self.registers[rd] = 0; // Modulo by zero results in 0
                    }
                }
            },
            0x1D => { // MULI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 18 && rs < 18 {
                    self.registers[rd] = self.registers[rs].wrapping_mul(imm);
                }
            },
            0x1E => { // DIVI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 18 && rs < 18 {
                    if imm != 0 {
                        self.registers[rd] = self.registers[rs] / imm;
                    } else {
                        self.registers[rd] = 0;
                    }
                }
            },
            0x1F => { // MODI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 18 && rs < 18 {
                    if imm != 0 {
                        self.registers[rd] = self.registers[rs] % imm;
                    } else {
                        self.registers[rd] = 0;
                    }
                }
            },
            
            _ => {
                return Err(format!("Unknown opcode: 0x{:02X}", instr.opcode));
            }
        }
        
        // R0 always reads as 0 (enforce after every instruction)
        self.registers[Register::R0 as usize] = 0;
        
        // Simulate output ready after some cycles (instant for now)
        if !self.output_ready {
            self.output_ready = true;
            self.memory[1] = 1;
        }
        
        Ok(())
    }
    
    pub fn run(&mut self) -> Result<(), String> {
        while matches!(self.state, VMState::Running) {
            self.step()?;
            // Stop if we hit a breakpoint.rs in debug mode
            if matches!(self.state, VMState::Breakpoint) {
                break;
            }
        }
        Ok(())
    }
    
    pub fn get_output(&mut self) -> Vec<u8> {
        self.output_buffer.drain(..).collect()
    }
    
    pub fn get_current_instruction(&self) -> Option<Instr> {
        let pc = self.registers[Register::Pc as usize];
        let pcb = self.registers[Register::Pcb as usize];
        let idx = (pcb as usize * self.bank_size as usize) + pc as usize;
        
        if idx < self.instructions.len() {
            Some(self.instructions[idx])
        } else {
            None
        }
    }
    
    pub fn reset(&mut self) {
        // Clear registers but preserve bank size
        self.registers = [0; 18];
        
        // Reset to entry point (instruction 0)
        self.registers[Register::Pc as usize] = 0;
        self.registers[Register::Pcb as usize] = 0;
        
        // Reset state to running (ready to execute)
        self.state = VMState::Running;
        self.skip_pc_increment = false;
        
        // Clear output
        self.output_buffer.clear();
        self.output_ready = true;
        
        // Reset memory I/O registers
        self.memory[0] = 0;
        self.memory[1] = 1;
        
        // Note: We keep the loaded instructions and data intact
    }
}