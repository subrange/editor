/// VM module - Virtual Machine implementation for the Ripple architecture

mod instruction;
mod state;
mod mmio;
mod display;
mod terminal;
mod execution;
mod storage;

pub use instruction::Instr;
pub use state::{VMState, KeyboardState};
pub use terminal::install_terminal_cleanup_hook;

use std::collections::{VecDeque, HashMap};
use ripple_asm::Register;
use crate::constants::*;
use crate::display_rgb565::RGB565Display;
use crate::vm::storage::Storage;

/// The Ripple Virtual Machine
pub struct VM {
    // Program memory - stores instructions
    pub instructions: Vec<Instr>,
    
    // Data memory - separate from instructions
    pub memory: Vec<u16>,
    
    // Registers (32 total: R0-R31)
    pub registers: [u16; 32],
    
    // VM state
    pub state: VMState,
    
    // Configuration
    pub bank_size: u16,
    
    // Debug mode flag
    pub debug_mode: bool,
    
    // Verbose mode flag
    pub verbose: bool,
    
    // Do not increment PC flag (set by jump/branch instructions)
    skip_pc_increment: bool,
    
    // Output buffer for I/O
    pub output_buffer: VecDeque<u8>,
    output_ready: bool,
    
    // Input buffer for TTY_IN
    pub input_buffer: VecDeque<u8>,
    
    // TTY input mode state
    tty_input_enabled: bool,
    
    // RNG state (simple LCG)
    rng_state: u32,
    
    // Display state
    display_mode: u16,
    display_enabled: bool,
    display_flush_done: bool,
    
    // Terminal state for TEXT40 mode
    terminal_raw_mode: bool,
    terminal_saved_screen: bool,
    
    // Keyboard state (for TEXT40 mode)
    keyboard_state: KeyboardState,
    
    // RGB565 display
    pub rgb565_display: Option<RGB565Display>,
    pub display_resolution: u16, // hi8=width, lo8=height
    
    // Debug information: maps instruction indices to function names
    pub debug_symbols: HashMap<usize, String>,
    
    // Storage subsystem
    storage: Option<Storage>,
}

impl VM {
    pub fn new(bank_size: u16) -> Self {
        Self::with_memory_size(bank_size, DEFAULT_MEMORY_SIZE)
    }
    
    pub fn with_memory_size(bank_size: u16, memory_size: usize) -> Self {
        Self::with_options(bank_size, memory_size, None)
    }
    
    pub fn with_options(bank_size: u16, memory_size: usize, disk_path: Option<std::path::PathBuf>) -> Self {
        let memory_size = memory_size.max(MIN_MEMORY_SIZE);
        
        // Try to initialize storage, but don't fail if it can't be created
        let storage = match disk_path {
            Some(path) => match Storage::with_path(path) {
                Ok(s) => Some(s),
                Err(e) => {
                    eprintln!("Warning: Could not initialize storage with custom path: {}", e);
                    None
                }
            },
            None => match Storage::new() {
                Ok(s) => Some(s),
                Err(e) => {
                    eprintln!("Warning: Could not initialize storage: {}", e);
                    None
                }
            }
        };
        
        VM {
            instructions: Vec::new(),
            memory: vec![0; memory_size],
            registers: [0; 32],
            state: VMState::Setup,
            bank_size,
            debug_mode: false,
            verbose: false,
            skip_pc_increment: false,
            output_buffer: VecDeque::new(),
            output_ready: true,
            input_buffer: VecDeque::new(),
            tty_input_enabled: false,
            rng_state: 0x12345678,  // Fixed seed for reproducibility
            display_mode: DISP_OFF,
            display_enabled: false,
            display_flush_done: true,
            terminal_raw_mode: false,
            terminal_saved_screen: false,
            keyboard_state: KeyboardState::default(),
            rgb565_display: None,
            display_resolution: 0,
            debug_symbols: HashMap::new(),
            storage,
        }
    }
    
    #[allow(dead_code)]
    pub fn new_default() -> Self {
        Self::new(DEFAULT_BANK_SIZE)
    }
    
    pub fn set_rng_seed(&mut self, seed: u32) {
        self.rng_state = seed;
    }
    
    pub fn load_binary(&mut self, binary: &[u8]) -> Result<(), String> {
        // Check magic number
        if binary.len() < 5 || &binary[0..5] != MAGIC_RLINK {
            return Err("Invalid binary format: missing RLINK magic".to_string());
        }
        
        let mut pos = 5;
        
        // Read bank size (new field in binary format)
        // Always expect it to be present in new format
        if pos + 2 > binary.len() {
            return Err("Invalid binary: missing bank size".to_string());
        }
        let binary_bank_size = u16::from_le_bytes([binary[pos], binary[pos+1]]);
        if binary_bank_size != 0 {  // 0 means not specified, use default
            self.bank_size = binary_bank_size;
        }
        pos += 2;
        
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
        
        // ☠️💀🔥 CATASTROPHIC CHECK: PROGRAM MUST FIT IN ONE BLOCK 🔥💀☠️
        if instruction_count > self.bank_size as usize {
            eprintln!("\n");
            eprintln!("💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩");
            eprintln!("🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨");
            eprintln!("╔══════════════════════════════════════════════════════════════════╗");
            eprintln!("║              💀💀💀 SUPER DUPER ERROR!!! 💀💀💀                 ║");
            eprintln!("║                  🎪 ABSOLUTELY BONKERS! 🎪                      ║");
            eprintln!("║                                                                  ║");
            eprintln!("║     🔥🔥 PROGRAM TOO THICC FOR SINGLE BLOCK! 🔥🔥              ║");
            eprintln!("║                                                                  ║");
            eprintln!("║     🍔 Program size: {} instructions (CHONKY!)                  ", instruction_count);
            eprintln!("║     🥗 Block size:   {} instructions (smol bean)                ", self.bank_size);
            eprintln!("║     💩 Overflow:     {} instructions (POOPY OVERFLOW) 💩        ", instruction_count - self.bank_size as usize);
            eprintln!("║                                                                  ║");
            eprintln!("║                    💩💩💩 OH CRAP! 💩💩💩                      ║");
            eprintln!("║              Your code has pooped the bed!                      ║");
            eprintln!("║                                                                  ║");
            eprintln!("║     🌌 THE UNIVERSE SAID: \"NOPE!\" 🌌                          ║");
            eprintln!("║     🦖 DINOSAURS ARE COMING BACK 🦖                             ║");
            eprintln!("║     🍕 PIZZA IS GETTING COLD 🍕                                 ║");
            eprintln!("║     🐧 PENGUINS ARE MIGRATING TO MARS 🐧                        ║");
            eprintln!("║     🌮 TACOS ARE RAINING FROM THE SKY 🌮                        ║");
            eprintln!("║     🦄 UNICORNS ARE REAL NOW 🦄                                 ║");
            eprintln!("║     💩 EVERYTHING IS POO NOW 💩                                 ║");
            eprintln!("║     🎭 REALITY.EXE HAS STOPPED WORKING 🎭                       ║");
            eprintln!("║                                                                  ║");
            eprintln!("║          HERE'S A REALLY BAD ASCII DRAGON:                      ║");
            eprintln!("║                   ,     \\    /      ,                           ║");
            eprintln!("║                  / \\    )\\__/(     / \\                          ║");
            eprintln!("║                 /   \\  (_\\  /_)   /   \\                         ║");
            eprintln!("║            ____/_____\\__\\@  @/___/_____\\____                    ║");
            eprintln!("║           |             |\\../|              |                   ║");
            eprintln!("║           |              \\VV/               |                   ║");
            eprintln!("║           |         OH NO ITS BROKEN        |                   ║");
            eprintln!("║           |_________________________________|                   ║");
            eprintln!("║            |    /\\ /      \\\\       \\ /\\    |                    ║");
            eprintln!("║            |  /   V        ))       V   \\  |                    ║");
            eprintln!("║            |/     `       //        '     \\|                    ║");
            eprintln!("║            `              V                '                    ║");
            eprintln!("║                                                                  ║");
            eprintln!("║     ⚡ INITIATING EMERGENCY POO PROTOCOL ⚡                      ║");
            eprintln!("║     🚁 HELICOPTER NOISES: SOI SOI SOI SOI 🚁                    ║");
            eprintln!("║     🎵 PLAYING SAD TROMBONE: WOMP WOMP 🎵                       ║");
            eprintln!("║     🤖 ROBOTS ARE CRYING OIL TEARS 🤖                           ║");
            eprintln!("║     🌈 RAINBOW MACHINE BROKE (AND POOPED) 🌈                    ║");
            eprintln!("║                                                                  ║");
            eprintln!("║     📢 ATTENTION ALL EPIC GAMERS 📢                             ║");
            eprintln!("║     Your code is in great danger and needs YOUR help           ║");
            eprintln!("║     to fit in the block! All it needs is your credit           ║");
            eprintln!("║     card number... (just kidding, refactor your code)          ║");
            eprintln!("║                                                                  ║");
            eprintln!("║     🛸 ALIENS ARE LAUGHING AT YOUR POOPY CODE 🛸               ║");
            eprintln!("║     🔮 CRYSTAL BALL SAYS: \"BIG OOF\" 🔮                        ║");
            eprintln!("║     🎰 YOU ROLLED NAT 1 ON COMPILATION 🎰                       ║");
            eprintln!("║     💩 POO COUNTER: {} MEGA-POOS 💩                             ", (instruction_count - self.bank_size as usize) / 100 + 1);
            eprintln!("║     💣 YEET SEQUENCE: 5...4...3...2...1... 💣                   ║");
            eprintln!("║                                                                  ║");
            eprintln!("╚══════════════════════════════════════════════════════════════════╝");
            eprintln!("🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨🚨");
            eprintln!("💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩💩");
            eprintln!("\n");
            eprintln!("        HERE'S A TERRIBLE ASCII EXPLOSION:");
            eprintln!("                    💩");
            eprintln!("              💩    💩    💩");
            eprintln!("         💩   \\  💀💀💀  /   💩");
            eprintln!("       💩  ___--  RIP  --___  💩");
            eprintln!("         💩   /  YOUR  \\   💩");
            eprintln!("              💩  CODE  💩");
            eprintln!("                    💩");
            eprintln!("");
            eprintln!("💥💥💥 KABOOM! KAPOW! BAZINGA! POO-SPLOSION! 💥💥💥");
            eprintln!("              GAME OVER, INSERT COIN\n");
            
            return Err(format!(
                "💩🔥💀🎪 SUPER DUPER MEGA ULTRA POO ERROR: Program ({} instructions) is too THICC for block ({} instructions). UNIVERSE.EXE HAS CRASHED AND POOPED ITSELF! SEND HELP AND TOILET PAPER! 🎪💀🔥💩",
                instruction_count, self.bank_size
            ));
        }
        
        // Read instructions
        self.instructions.clear();
        for i in 0..instruction_count {
            if pos + INSTRUCTION_SIZE > binary.len() {
                return Err(format!("Invalid binary: missing instruction {i}"));
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
        
        // Determine data section offset based on display mode
        // Note: At this point, display mode might not be set yet, so we need to check
        // if the binary is intended for RGB565 mode by looking for display setup
        let data_offset = self.get_data_section_offset();
        
        // Load data into memory starting at the calculated offset
        for (i, &byte) in binary[pos..pos + data_size].iter().enumerate() {
            if i + data_offset < self.memory.len() {
                self.memory[i + data_offset] = byte as u16;
            }
        }
        pos += data_size;
        
        // Try to read debug section if present
        if pos + 5 <= binary.len() && &binary[pos..pos + 5] == b"DEBUG" {
            pos += 5;
            
            // Read number of debug entries
            if pos + 4 <= binary.len() {
                let debug_count = u32::from_le_bytes([
                    binary[pos], binary[pos+1], binary[pos+2], binary[pos+3]
                ]) as usize;
                pos += 4;
                
                // Read each debug entry
                for _ in 0..debug_count {
                    if pos + 4 > binary.len() {
                        break; // Incomplete debug info, just skip
                    }
                    
                    // Read name length
                    let name_len = u32::from_le_bytes([
                        binary[pos], binary[pos+1], binary[pos+2], binary[pos+3]
                    ]) as usize;
                    pos += 4;
                    
                    if pos + name_len > binary.len() {
                        break; // Incomplete debug info
                    }
                    
                    // Read name
                    let name = String::from_utf8_lossy(&binary[pos..pos + name_len]).to_string();
                    pos += name_len;
                    
                    if pos + 4 > binary.len() {
                        break; // Incomplete debug info
                    }
                    
                    // Read instruction index
                    let instr_idx = u32::from_le_bytes([
                        binary[pos], binary[pos+1], binary[pos+2], binary[pos+3]
                    ]) as usize;
                    pos += 4;
                    
                    // Store in debug symbols map
                    self.debug_symbols.insert(instr_idx, name);
                }
            }
        }
        
        // Set entry point
        let entry_bank = (entry_point / (self.bank_size as u32 * 4)) as u16;
        let entry_offset = ((entry_point / 4) % (self.bank_size as u32)) as u16;
        self.registers[Register::Pcb as usize] = entry_bank;
        self.registers[Register::Pc as usize] = entry_offset;
        
        // Initialize MMIO state (memory is handled via MMIO read/write handlers)
        self.output_ready = true;
        self.display_mode = DISP_OFF;
        self.display_enabled = false;
        self.display_flush_done = true;
        
        self.state = VMState::Running;
        Ok(())
    }
    
    pub fn step(&mut self) -> Result<(), String> {
        match self.state {
            VMState::Running => {},
            VMState::Halted => return Ok(()),
            VMState::Breakpoint => {
                // In debug mode at breakpoint, allow single stepping
                // State will be reset to Running by the debugger_ui
            },
            VMState::Error(ref e) => return Err(e.clone()),
            VMState::Setup => return Err("VM not initialized".to_string()),
        }
        
        // Poll stdin for TTY input (non-blocking) - always poll when not in debug/verbose mode
        // This populates the input buffer for TTY_IN_POP/TTY_IN_STATUS
        if !self.debug_mode && !self.verbose {
            self.poll_stdin();
        }
        
        // DON'T poll keyboard here - only poll when actually reading keyboard MMIO
        
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
        
        // Print instruction in verbose mode
        if self.verbose {
            eprint!("[{instr_idx:04X}] ");
            self.print_instruction(&instr);
        }
        
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
    
    pub fn run(&mut self) -> Result<(), String> {
        while matches!(self.state, VMState::Running) {
            self.step()?;
            // Stop if we hit a breakpoint in debug mode
            if matches!(self.state, VMState::Breakpoint) {
                break;
            }
        }
        Ok(())
    }
    
    pub fn get_output(&mut self) -> Vec<u8> {
        self.output_buffer.drain(..).collect()
    }
    
    #[allow(dead_code)]
    pub fn push_input(&mut self, byte: u8) {
        self.input_buffer.push_back(byte);
    }
    
    #[allow(dead_code)]
    pub fn push_input_string(&mut self, input: &str) {
        for byte in input.bytes() {
            self.input_buffer.push_back(byte);
        }
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
        self.registers = [0; 32];
        
        // Reset to entry point (instruction 0)
        self.registers[Register::Pc as usize] = 0;
        self.registers[Register::Pcb as usize] = 0;
        
        // Reset state to running (ready to execute)
        self.state = VMState::Running;
        self.skip_pc_increment = false;
        
        // Clear I/O buffers
        self.output_buffer.clear();
        self.input_buffer.clear();
        self.output_ready = true;
        
        // Disable TTY input if it was enabled
        if self.tty_input_enabled {
            self.disable_tty_input();
        }
        
        // Clear keyboard state
        self.clear_keyboard_state();
        
        // Reset display state (and exit any active display mode)
        if self.display_mode == DISP_TTY && self.terminal_raw_mode {
            self.exit_tty_mode();
        } else if self.display_mode == DISP_TEXT40 && self.terminal_raw_mode {
            self.exit_text40_mode();
        } else if self.display_mode == DISP_RGB565 {
            if let Some(mut display) = self.rgb565_display.take() {
                display.shutdown();
            }
        }
        self.display_mode = DISP_OFF;
        self.display_enabled = false;
        self.display_flush_done = true;
        self.display_resolution = 0;
        
        // Clear all memory (reset to zeros)
        self.memory.fill(0);
        
        // Note: We keep the loaded instructions, data, debug symbols, and storage intact
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        // Ensure terminal is restored when VM is dropped
        if self.tty_input_enabled {
            self.disable_tty_input();
        }
        if self.terminal_raw_mode {
            if self.display_mode == DISP_TTY {
                self.exit_tty_mode();
            } else if self.display_mode == DISP_TEXT40 {
                self.exit_text40_mode();
            }
        }
        
        // Flush storage if present
        if let Some(ref mut storage) = self.storage {
            storage.flush();
        }
    }
}