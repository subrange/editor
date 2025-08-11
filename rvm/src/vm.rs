use std::collections::VecDeque;

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
}

#[derive(Debug)]
pub enum VMState {
    Setup,
    Running,
    Halted,
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
    
    // Do not increment PC flag (set by jump/branch instructions)
    skip_pc_increment: bool,
    
    // Output buffer for I/O
    pub output_buffer: VecDeque<u8>,
    output_ready: bool,
}

// Register indices
const R0: usize = 0;   // Always reads 0
const PC: usize = 1;   // Program counter (offset within bank)
const PCB: usize = 2;  // Program counter bank
#[allow(dead_code)]
const RA: usize = 3;   // Return address (low)
const RAB: usize = 4;  // Return address bank (high)

impl VM {
    pub fn new(bank_size: u16) -> Self {
        VM {
            instructions: Vec::new(),
            memory: vec![0; 65536], // 64K memory space
            registers: [0; 18],
            state: VMState::Setup,
            bank_size,
            skip_pc_increment: false,
            output_buffer: VecDeque::new(),
            output_ready: true,
        }
    }
    
    #[allow(dead_code)]
    pub fn new_default() -> Self {
        Self::new(4096) // Default bank size
    }
    
    pub fn load_binary(&mut self, binary: &[u8]) -> Result<(), String> {
        // Check magic number
        if binary.len() < 5 || &binary[0..5] != b"RLINK" {
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
            if pos + 8 > binary.len() {
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
            pos += 8;
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
            if i < self.memory.len() - 2 {
                self.memory[i + 2] = byte as u16;
            }
        }
        
        // Set entry point
        let entry_bank = (entry_point / (self.bank_size as u32 * 4)) as u16;
        let entry_offset = ((entry_point / 4) % (self.bank_size as u32)) as u16;
        self.registers[PCB] = entry_bank;
        self.registers[PC] = entry_offset;
        
        // Initialize memory-mapped I/O
        self.memory[0] = 0; // OUT register
        self.memory[1] = 1; // OUT_FLAG (ready)
        
        self.state = VMState::Running;
        Ok(())
    }
    
    pub fn step(&mut self) -> Result<(), String> {
        match self.state {
            VMState::Running => {},
            VMState::Halted => return Ok(()),
            VMState::Error(ref e) => return Err(e.clone()),
            VMState::Setup => return Err("VM not initialized".to_string()),
        }
        
        // Calculate instruction address
        let pc = self.registers[PC];
        let pcb = self.registers[PCB];
        let instr_idx = (pcb as usize * self.bank_size as usize) + pc as usize;
        
        if instr_idx >= self.instructions.len() {
            self.state = VMState::Error(format!("PC out of bounds: bank={}, offset={}", pcb, pc));
            return Err(format!("PC out of bounds"));
        }
        
        let instr = self.instructions[instr_idx];
        self.skip_pc_increment = false;
        
        // Execute instruction
        self.execute_instruction(instr)?;
        
        // Increment PC unless instruction set the skip flag
        if !self.skip_pc_increment {
            let mut new_pc = self.registers[PC] as u32 + 1;
            let mut new_pcb = self.registers[PCB] as u32;
            
            if new_pc >= self.bank_size as u32 {
                new_pc = 0;
                new_pcb += 1;
            }
            
            self.registers[PC] = (new_pc & 0xFFFF) as u16;
            self.registers[PCB] = (new_pcb & 0xFFFF) as u16;
        }
        
        Ok(())
    }
    
    fn execute_instruction(&mut self, instr: Instr) -> Result<(), String> {
        // R0 always reads as 0
        self.registers[R0] = 0;
        
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
            0x11 => { // LOAD
                let rd = instr.word1 as usize;
                let bank = instr.word2;
                let addr = instr.word3;
                if rd < 18 {
                    let mem_addr = (bank as usize * self.bank_size as usize * 4) + addr as usize;
                    if mem_addr < self.memory.len() {
                        self.registers[rd] = self.memory[mem_addr];
                    } else {
                        return Err(format!("LOAD: memory address out of bounds: {}", mem_addr));
                    }
                }
            },
            0x12 => { // STORE
                let rd = instr.word1 as usize;
                let bank = instr.word2;
                let addr = instr.word3;
                if rd < 18 {
                    let mem_addr = (bank as usize * self.bank_size as usize * 4) + addr as usize;
                    if mem_addr < self.memory.len() {
                        let value = self.registers[rd];
                        self.memory[mem_addr] = value;
                        
                        // Handle memory-mapped I/O
                        if mem_addr == 0 {
                            // Output register
                            self.output_buffer.push_back((value & 0xFF) as u8);
                            self.output_ready = false;
                            // Simulate output delay
                            self.memory[1] = 0;
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
                    self.registers[rd] = self.registers[PC].wrapping_add(1);
                }
                self.registers[RAB] = self.registers[PCB];
                
                // Jump to address
                self.registers[PC] = addr;
                self.skip_pc_increment = true;
            },
            0x14 => { // JALR
                let rd = instr.word1 as usize;
                let rs = instr.word3 as usize; // Note: rs is in word3 for JALR
                if rd < 18 && rs < 18 {
                    // Save return address
                    self.registers[rd] = self.registers[PC].wrapping_add(1);
                    self.registers[RAB] = self.registers[PCB];
                    // Jump
                    self.registers[PC] = self.registers[rs];
                    self.skip_pc_increment = true;
                }
            },
            0x15 => { // BEQ
                let rs = instr.word1 as usize;
                let rt = instr.word2 as usize;
                let offset = instr.word3 as i16;
                if rs < 18 && rt < 18 {
                    if self.registers[rs] == self.registers[rt] {
                        let new_pc = (self.registers[PC] as i16).wrapping_add(offset);
                        self.registers[PC] = new_pc as u16;
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
                        let new_pc = (self.registers[PC] as i16).wrapping_add(offset);
                        self.registers[PC] = new_pc as u16;
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
                        let new_pc = (self.registers[PC] as i16).wrapping_add(offset);
                        self.registers[PC] = new_pc as u16;
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
                        let new_pc = (self.registers[PC] as i16).wrapping_add(offset);
                        self.registers[PC] = new_pc as u16;
                        self.skip_pc_increment = true;
                    }
                }
            },
            
            0x19 => { // BRK - debugger breakpoint
                // For now, just continue
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
        self.registers[R0] = 0;
        
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
        }
        Ok(())
    }
    
    pub fn get_output(&mut self) -> Vec<u8> {
        self.output_buffer.drain(..).collect()
    }
    
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.registers = [0; 18];
        self.state = VMState::Setup;
        self.skip_pc_increment = false;
        self.output_buffer.clear();
        self.output_ready = true;
        self.memory[0] = 0;
        self.memory[1] = 1;
    }
}