use std::collections::{VecDeque, HashMap};
use ripple_asm::Register;
use crate::constants::*;
use crate::display_rgb565::RGB565Display;
use crossterm::{
    terminal::{self, ClearType},
    cursor,
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor},
    event::{self, Event, KeyCode},
    ExecutableCommand,
};

#[derive(Debug, Default, Clone, Copy)]
struct KeyboardState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    z: bool,
    x: bool,
    last_read_counter: u32,  // Counter to track when keys were last read
}

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
    pub debug_symbols: std::collections::HashMap<usize, String>,
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
            registers: [0; 32],
            state: VMState::Setup,
            bank_size,
            debug_mode: false,
            verbose: false,
            skip_pc_increment: false,
            output_buffer: VecDeque::new(),
            output_ready: true,
            input_buffer: VecDeque::new(),
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
        
        // Load data into memory starting after VRAM (word 1032)
        for (i, &byte) in binary[pos..pos + data_size].iter().enumerate() {
            if i + DATA_SECTION_OFFSET < self.memory.len() {
                self.memory[i + DATA_SECTION_OFFSET] = byte as u16;
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
                // In debug mode at breakpoint.rs, allow single stepping
                // State will be reset to Running by the debugger_ui
            },
            VMState::Error(ref e) => return Err(e.clone()),
            VMState::Setup => return Err("VM not initialized".to_string()),
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
    
    // Handle MMIO reads for special addresses in bank 0
    fn handle_mmio_read(&mut self, addr: usize) -> Option<u16> {
        // Only handle bank 0 MMIO addresses
        if addr >= TEXT40_BASE_WORD {
            return None; // Regular memory access for VRAM and beyond
        }
        
        match addr {
            HDR_TTY_OUT => Some(0), // Write-only, return 0
            HDR_TTY_STATUS => {
                let value = if self.output_ready { TTY_READY } else { 0 };
                self.memory[HDR_TTY_STATUS] = value;
                Some(value)
            },
            HDR_TTY_IN_POP => {
                // Pop a byte from input buffer
                let value = if let Some(byte) = self.input_buffer.pop_front() {
                    byte as u16
                } else {
                    0 // Return 0 if buffer is empty
                };
                // Store the popped value in memory
                self.memory[HDR_TTY_IN_POP] = value;
                Some(value)
            },
            HDR_TTY_IN_STATUS => {
                let value = if !self.input_buffer.is_empty() { TTY_HAS_BYTE } else { 0 };
                self.memory[HDR_TTY_IN_STATUS] = value;
                Some(value)
            },
            HDR_RNG => {
                // Simple LCG: next = (a * prev + c) mod m
                self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                let value = (self.rng_state >> 16) as u16;
                // Store the generated value in memory
                self.memory[HDR_RNG] = value;
                Some(value) // Return the value
            },
            HDR_RNG_SEED => {
                // Return the low 16 bits of current seed
                let value = (self.rng_state & 0xFFFF) as u16;
                self.memory[HDR_RNG_SEED] = value;
                Some(value)
            },
            HDR_DISP_MODE => {
                self.memory[HDR_DISP_MODE] = self.display_mode;
                Some(self.display_mode)
            },
            HDR_DISP_STATUS => {
                let mut status = 0;
                if self.output_ready { status |= DISP_READY; }
                if self.display_flush_done { status |= DISP_FLUSH_DONE; }
                self.memory[HDR_DISP_STATUS] = status;
                Some(status)
            },
            HDR_DISP_CTL => {
                let value = if self.display_enabled { DISP_ENABLE } else { 0 };
                self.memory[HDR_DISP_CTL] = value;
                Some(value)
            },
            HDR_DISP_FLUSH => Some(0), // Write-only, return 0
            HDR_KEY_UP | HDR_KEY_DOWN | HDR_KEY_LEFT | HDR_KEY_RIGHT | HDR_KEY_Z | HDR_KEY_X => {
                // Poll keyboard only when reading keyboard addresses
                if self.display_mode == DISP_TEXT40 && self.terminal_raw_mode {
                    self.poll_keyboard();
                    
                    // Auto-clear keys after they've been read a few times
                    // This prevents keys from getting "stuck" but allows for repeated reads
                    self.keyboard_state.last_read_counter += 1;
                    if self.keyboard_state.last_read_counter > 10 {
                        // Clear the keyboard state after ~10 reads without new input
                        self.keyboard_state = KeyboardState::default();
                    }
                }
                
                // Return the appropriate key state
                let value = match addr {
                    HDR_KEY_UP => if self.keyboard_state.up { 1 } else { 0 },
                    HDR_KEY_DOWN => if self.keyboard_state.down { 1 } else { 0 },
                    HDR_KEY_LEFT => if self.keyboard_state.left { 1 } else { 0 },
                    HDR_KEY_RIGHT => if self.keyboard_state.right { 1 } else { 0 },
                    HDR_KEY_Z => if self.keyboard_state.z { 1 } else { 0 },
                    HDR_KEY_X => if self.keyboard_state.x { 1 } else { 0 },
                    _ => 0,
                };
                self.memory[addr] = value;
                Some(value)
            },
            HDR_DISP_RESOLUTION => {
                self.memory[HDR_DISP_RESOLUTION] = self.display_resolution;
                Some(self.display_resolution)
            },
            17..=31 => Some(0), // Reserved addresses return 0
            _ => None, // Not an MMIO address
        }
    }
    
    // Handle MMIO writes for special addresses in bank 0
    fn handle_mmio_write(&mut self, addr: usize, value: u16) -> bool {
        // Only handle bank 0 MMIO addresses
        if addr >= TEXT40_BASE_WORD {
            return false; // Regular memory write for VRAM and beyond
        }
        
        match addr {
            HDR_TTY_OUT => {
                // Output low byte to stdout
                let byte = (value & 0xFF) as u8;
                
                // Print immediately to stdout for real-time effect
                use std::io::{self, Write};
                let _ = io::stdout().write_all(&[byte]);
                let _ = io::stdout().flush();
                
                // Also store in buffer for compatibility
                self.output_buffer.push_back(byte);
                self.output_ready = false;
                // Simulate output delay (will be set ready in next cycle)
                true
            },
            HDR_TTY_STATUS => true, // Read-only, ignore write
            HDR_TTY_IN_POP => true, // Read-only, ignore write
            HDR_TTY_IN_STATUS => true, // Read-only, ignore write
            HDR_RNG => true, // Read-only, ignore write
            HDR_RNG_SEED => {
                // Set the low 16 bits of the RNG seed
                // Keep high 16 bits unchanged to maintain some state
                self.rng_state = (self.rng_state & 0xFFFF0000) | (value as u32);
                // Store the value in memory for read-back
                self.memory[HDR_RNG_SEED] = value;
                true
            },
            HDR_DISP_MODE => {
                let new_mode = value & 0x3; // Support 4 modes (0-3)
                if new_mode != self.display_mode {
                    // Mode change - handle terminal setup/teardown
                    self.handle_display_mode_change(new_mode);
                }
                self.display_mode = new_mode;
                true
            },
            HDR_DISP_STATUS => true, // Read-only, ignore write
            HDR_DISP_CTL => {
                // Handle control bits
                if value & DISP_ENABLE != 0 {
                    self.display_enabled = true;
                }
                if value & DISP_CLEAR != 0 {
                    // Clear VRAM (edge-triggered)
                    for i in TEXT40_BASE_WORD..=TEXT40_LAST_WORD {
                        if i < self.memory.len() {
                            self.memory[i] = 0;
                        }
                    }
                    // Auto-clear the CLEAR bit (edge-triggered)
                }
                true
            },
            HDR_DISP_FLUSH => {
                if value != 0 {
                    self.display_flush_done = false;
                    // Trigger display rendering based on mode
                    if self.display_mode == DISP_TEXT40 {
                        self.flush_text40_display();
                    } else if self.display_mode == DISP_RGB565 {
                        self.flush_rgb565_display();
                    }
                    self.display_flush_done = true;
                }
                true
            },
            HDR_KEY_UP | HDR_KEY_DOWN | HDR_KEY_LEFT | HDR_KEY_RIGHT | HDR_KEY_Z | HDR_KEY_X => {
                true // Read-only keyboard flags, ignore writes
            },
            HDR_DISP_RESOLUTION => {
                // Set display resolution for RGB565 mode
                self.display_resolution = value;
                self.memory[HDR_DISP_RESOLUTION] = value;
                true
            },
            17..=31 => true, // Reserved addresses, ignore writes
            _ => false, // Not an MMIO address
        }
    }
    
    // Handle display mode changes
    fn handle_display_mode_change(&mut self, new_mode: u16) {
        // Exit current mode
        match self.display_mode {
            DISP_TEXT40 if self.terminal_raw_mode => {
                self.exit_text40_mode();
            },
            DISP_RGB565 => {
                // Shutdown RGB565 display
                if let Some(mut display) = self.rgb565_display.take() {
                    display.shutdown();
                }
            },
            _ => {}
        }
        
        // Enter new mode
        match new_mode {
            DISP_TEXT40 if !self.terminal_raw_mode => {
                self.enter_text40_mode();
            },
            DISP_RGB565 => {
                // Initialize RGB565 display
                let width = ((self.display_resolution >> 8) & 0xFF) as u8;
                let height = (self.display_resolution & 0xFF) as u8;
                
                eprintln!("Setting RGB565 mode with resolution {}x{}", width, height);
                
                if width > 0 && height > 0 {
                    // Use existing display if available, or create new one
                    if self.rgb565_display.is_none() {
                        self.rgb565_display = Some(RGB565Display::new());
                    }
                    
                    if let Some(ref mut display) = self.rgb565_display {
                        if let Err(e) = display.init(width, height, self.bank_size as usize) {
                            eprintln!("Failed to initialize RGB565 display: {}", e);
                            // Reset to OFF mode on failure
                            self.display_mode = DISP_OFF;
                            return;
                        }
                        eprintln!("RGB565 display initialized successfully: {}x{}", width, height);
                    }
                } else {
                    eprintln!("Invalid display resolution: {}x{}", width, height);
                    self.display_mode = DISP_OFF;
                }
            },
            _ => {}
        }
    }
    
    // Poll keyboard events and update keyboard state (non-blocking)
    fn poll_keyboard(&mut self) {
        use std::time::Duration;
        
        // Only poll keyboard in TEXT40 mode with terminal raw mode
        if self.display_mode != DISP_TEXT40 || !self.terminal_raw_mode {
            return;
        }
        
        // Poll for a single keyboard event with zero timeout (non-blocking)
        // Only process one event at a time to maintain state between reads
        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                // Clear all keys first, then set the current key
                // This gives a "last key pressed" behavior
                self.keyboard_state.up = false;
                self.keyboard_state.down = false;
                self.keyboard_state.left = false;
                self.keyboard_state.right = false;
                self.keyboard_state.z = false;
                self.keyboard_state.x = false;
                self.keyboard_state.last_read_counter = 0;  // Reset counter on new input
                
                // Set the current key as pressed
                match key_event.code {
                    KeyCode::Up => {
                        self.keyboard_state.up = true;
                    },
                    KeyCode::Down => {
                        self.keyboard_state.down = true;
                    },
                    KeyCode::Left => {
                        self.keyboard_state.left = true;
                    },
                    KeyCode::Right => {
                        self.keyboard_state.right = true;
                    },
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        self.keyboard_state.z = true;
                    },
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        self.keyboard_state.x = true;
                    },
                    _ => {
                        // For other keys, don't change state
                    }
                }
            }
        }
        // If no event, keep the current state (don't clear)
    }
    
    // Clear keyboard state flags
    fn clear_keyboard_state(&mut self) {
        self.keyboard_state = KeyboardState::default();
    }
    
    // Enter TEXT40 terminal mode
    fn enter_text40_mode(&mut self) {
        use std::io::{self, Write};
        
        // Only enter raw mode if not in verbose/debug mode
        if self.verbose || self.debug_mode {
            return; // Keep normal mode for debugging
        }
        
        // Save current screen and enter alternate screen
        let _ = io::stderr().execute(terminal::EnterAlternateScreen);
        let _ = io::stderr().execute(cursor::Hide);
        let _ = terminal::enable_raw_mode();
        
        self.terminal_raw_mode = true;
        self.terminal_saved_screen = true;
        
        // Clear keyboard state when entering TEXT40 mode
        self.clear_keyboard_state();
        
        // Clear the screen
        let _ = io::stderr().execute(terminal::Clear(ClearType::All));
        let _ = io::stderr().execute(cursor::MoveTo(0, 0));
        let _ = io::stderr().flush();
    }
    
    // Exit TEXT40 terminal mode
    fn exit_text40_mode(&mut self) {
        use std::io::{self, Write};
        
        if !self.terminal_raw_mode {
            return;
        }
        
        // Restore terminal
        let _ = terminal::disable_raw_mode();
        let _ = io::stderr().execute(cursor::Show);
        let _ = io::stderr().execute(terminal::LeaveAlternateScreen);
        let _ = io::stderr().flush();
        
        self.terminal_raw_mode = false;
        self.terminal_saved_screen = false;
    }
    
    // Flush RGB565 display (swap buffers)
    fn flush_rgb565_display(&mut self) {
        if let Some(ref mut display) = self.rgb565_display {
            display.flush();
            if self.verbose {
                eprintln!("RGB565: Flushed display");
            }
        } else {
            eprintln!("Warning: flush_rgb565_display called but no display initialized");
        }
    }
    
    // Render TEXT40 display to terminal
    fn flush_text40_display(&mut self) {
        if self.display_mode != DISP_TEXT40 || !self.display_enabled {
            return; // Display not in TEXT40 mode or not enabled
        }
        
        use std::io::{self, Write};
        
        if self.terminal_raw_mode {
            // Render to actual terminal in TEXT40 mode
            let mut stderr = io::stderr();
            
            // Move to top-left
            let _ = stderr.execute(cursor::MoveTo(0, 0));
            
            // Render the 40x25 display
            for row in 0..25 {
                for col in 0..40 {
                    let addr = TEXT40_BASE_WORD + row * 40 + col;
                    if addr < self.memory.len() {
                        let cell = self.memory[addr];
                        let ch = (cell & 0xFF) as u8;
                        let attr = ((cell >> 8) & 0xFF) as u8;
                        
                        // Extract foreground and background colors from attribute byte
                        // Lower 4 bits: foreground color (0-15)
                        // Upper 4 bits: background color (0-15)
                        let fg_color = (attr & 0x0F) as usize;
                        let bg_color = ((attr >> 4) & 0x0F) as usize;
                        
                        // Convert theme color index to terminal Color
                        let theme_to_terminal = |color_idx: usize| -> Color {
                            if color_idx < THEME_COLORS.len() {
                                let (r, g, b) = THEME_COLORS[color_idx];
                                Color::Rgb { r, g, b }
                            } else {
                                // Default to light gray for invalid indices
                                let (r, g, b) = THEME_COLORS[6];
                                Color::Rgb { r, g, b }
                            }
                        };
                        
                        // Set colors
                        let _ = stderr.execute(SetForegroundColor(theme_to_terminal(fg_color)));
                        let _ = stderr.execute(SetBackgroundColor(theme_to_terminal(bg_color)));
                        
                        // Print character
                        if (32..127).contains(&ch) {
                            let _ = stderr.write(&[ch]);
                        } else if ch == 0 {
                            let _ = stderr.write(b" ");
                        } else {
                            let _ = stderr.write(b".");
                        }
                        
                        // Reset colors after each character to prevent color bleeding
                        let _ = stderr.execute(ResetColor);
                    } else {
                        let _ = stderr.write(b" ");
                    }
                }
                // Move to next line
                if row < 24 {
                    let _ = stderr.write(b"\r\n");
                }
            }
            
            let _ = stderr.flush();
        } else if self.verbose {
            // Fallback to debug output
            eprintln!("\n=== TEXT40 Display (40x25) ===");
            for row in 0..25 {
                eprint!("  ");
                for col in 0..40 {
                    let addr = TEXT40_BASE_WORD + row * 40 + col;
                    if addr < self.memory.len() {
                        let cell = self.memory[addr];
                        let ch = (cell & 0xFF) as u8;
                        // Print ASCII character or '.' for non-printable
                        if (32..127).contains(&ch) {
                            eprint!("{}", ch as char);
                        } else if ch == 0 {
                            eprint!(" ");
                        } else {
                            eprint!(".");
                        }
                    }
                }
                eprintln!();
            }
            eprintln!("==============================\n");
        }
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
                if rd < 32 && rs < 32 && rt < 32 {
                    self.registers[rd] = self.registers[rs].wrapping_add(self.registers[rt]);
                }
            },
            0x02 => { // SUB
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    self.registers[rd] = self.registers[rs].wrapping_sub(self.registers[rt]);
                }
            },
            0x03 => { // AND
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    self.registers[rd] = self.registers[rs] & self.registers[rt];
                }
            },
            0x04 => { // OR
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    self.registers[rd] = self.registers[rs] | self.registers[rt];
                }
            },
            0x05 => { // XOR
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    self.registers[rd] = self.registers[rs] ^ self.registers[rt];
                }
            },
            0x06 => { // SLL
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    let shift = (self.registers[rt] & 15) as u32;
                    self.registers[rd] = self.registers[rs].wrapping_shl(shift);
                }
            },
            0x07 => { // SRL
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    let shift = (self.registers[rt] & 15) as u32;
                    self.registers[rd] = self.registers[rs].wrapping_shr(shift);
                }
            },
            0x08 => { // SLT (signed compare)
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    let rs_val = self.registers[rs] as i16;
                    let rt_val = self.registers[rt] as i16;
                    self.registers[rd] = if rs_val < rt_val { 1 } else { 0 };
                }
            },
            0x09 => { // SLTU (unsigned compare)
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
                    self.registers[rd] = if self.registers[rs] < self.registers[rt] { 1 } else { 0 };
                }
            },
            
            // ALU I-type operations
            0x0A => { // ADDI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 32 && rs < 32 {
                    self.registers[rd] = self.registers[rs].wrapping_add(imm);
                }
            },
            0x0B => { // ANDI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 32 && rs < 32 {
                    self.registers[rd] = self.registers[rs] & imm;
                }
            },
            0x0C => { // ORI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 32 && rs < 32 {
                    self.registers[rd] = self.registers[rs] | imm;
                }
            },
            0x0D => { // XORI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 32 && rs < 32 {
                    self.registers[rd] = self.registers[rs] ^ imm;
                }
            },
            0x0E => { // LI
                let rd = instr.word1 as usize;
                let imm = instr.word2;
                if rd < 32 {
                    self.registers[rd] = imm;
                }
            },
            0x0F => { // SLLI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3 as u32;
                if rd < 32 && rs < 32 {
                    self.registers[rd] = self.registers[rs].wrapping_shl(imm & 15);
                }
            },
            0x10 => { // SRLI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3 as u32;
                if rd < 32 && rs < 32 {
                    self.registers[rd] = self.registers[rs].wrapping_shr(imm & 15);
                }
            },
            
            // Memory operations
            0x11 => { // LOAD - rd = memory[bank_reg][addr_reg]
                let rd = instr.word1 as usize;
                let bank_reg = instr.word2 as usize;
                let addr_reg = instr.word3 as usize;
                if rd < 32 && bank_reg < 32 && addr_reg < 32 {
                    let bank_val = self.registers[bank_reg];
                    let addr_val = self.registers[addr_reg];
                    
                    // Check if this is a bank 0 MMIO access
                    if bank_val == 0 && addr_val < TEXT40_LAST_WORD as u16 + 1 {
                        // Try MMIO read first
                        if let Some(value) = self.handle_mmio_read(addr_val as usize) {
                            self.registers[rd] = value;
                        } else {
                            // Regular memory read for VRAM and other bank 0 addresses
                            self.registers[rd] = self.memory[addr_val as usize];
                        }
                    } else if bank_val == 0 && self.display_mode == DISP_RGB565 {
                        // Check if this is RGB565 framebuffer access
                        if let Some(ref display) = self.rgb565_display {
                            if let Some(value) = display.read_memory(addr_val as usize) {
                                self.registers[rd] = value;
                            } else {
                                // Regular memory access
                                let mem_addr = addr_val as usize;
                                if mem_addr < self.memory.len() {
                                    self.registers[rd] = self.memory[mem_addr];
                                } else {
                                    return Err(format!("LOAD: memory address out of bounds: {mem_addr}"));
                                }
                            }
                        } else {
                            self.registers[rd] = 0;
                        }
                    } else {
                        // Regular memory access for non-bank-0
                        let mem_addr = (bank_val as usize * self.bank_size as usize) + addr_val as usize;
                        if mem_addr < self.memory.len() {
                            self.registers[rd] = self.memory[mem_addr];
                        } else {
                            return Err(format!("LOAD: memory address out of bounds: {mem_addr}"));
                        }
                    }
                }
            },
            0x12 => { // STORE - memory[bank_reg][addr_reg] = rs
                let rs = instr.word1 as usize;
                let bank_reg = instr.word2 as usize;
                let addr_reg = instr.word3 as usize;
                if rs < 32 && bank_reg < 32 && addr_reg < 32 {
                    let bank_val = self.registers[bank_reg];
                    let addr_val = self.registers[addr_reg];
                    let value = self.registers[rs];
                    
                    // Check if this is a bank 0 MMIO access
                    if bank_val == 0 && addr_val < TEXT40_LAST_WORD as u16 + 1 {
                        // Try MMIO write first
                        if !self.handle_mmio_write(addr_val as usize, value) {
                            // Regular memory write for VRAM and other bank 0 addresses
                            self.memory[addr_val as usize] = value;
                        }
                    } else if bank_val == 0 && self.display_mode == DISP_RGB565 {
                        // Check if this is RGB565 framebuffer access
                        if let Some(ref mut display) = self.rgb565_display {
                            display.write_memory(addr_val as usize, value);
                        } else {
                            // Regular memory write
                            let mem_addr = addr_val as usize;
                            if mem_addr < self.memory.len() {
                                self.memory[mem_addr] = value;
                            } else {
                                return Err(format!("STORE: memory address out of bounds: {mem_addr}"));
                            }
                        }
                    } else {
                        // Regular memory access for non-bank-0
                        let mem_addr = (bank_val as usize * self.bank_size as usize) + addr_val as usize;
                        if mem_addr < self.memory.len() {
                            self.memory[mem_addr] = value;
                        } else {
                            return Err(format!("STORE: memory address out of bounds: {mem_addr}"));
                        }
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
                if rd < 32 {
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
                if rd < 32 && rs < 32 {
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
                if rs < 32 && rt < 32
                    && self.registers[rs] == self.registers[rt] {
                        let new_pc = (self.registers[Register::Pc as usize] as i16).wrapping_add(offset);
                        self.registers[Register::Pc as usize] = new_pc as u16;
                        self.skip_pc_increment = true;
                    }
            },
            0x16 => { // BNE
                let rs = instr.word1 as usize;
                let rt = instr.word2 as usize;
                let offset = instr.word3 as i16;
                if rs < 32 && rt < 32
                    && self.registers[rs] != self.registers[rt] {
                        let new_pc = (self.registers[Register::Pc as usize] as i16).wrapping_add(offset);
                        self.registers[Register::Pc as usize] = new_pc as u16;
                        self.skip_pc_increment = true;
                    }
            },
            0x17 => { // BLT
                let rs = instr.word1 as usize;
                let rt = instr.word2 as usize;
                let offset = instr.word3 as i16;
                if rs < 32 && rt < 32 {
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
                if rs < 32 && rt < 32 {
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
                    eprintln!("  R0 (ZR):  0x{:04X} ({})", self.registers[Register::R0 as usize], self.registers[Register::R0 as usize]);
                    eprintln!("  R1 (PC):  0x{:04X} ({})", self.registers[Register::Pc as usize], self.registers[Register::Pc as usize]);
                    eprintln!("  R2 (PCB): 0x{:04X} ({})", self.registers[Register::Pcb as usize], self.registers[Register::Pcb as usize]);
                    eprintln!("  R3 (RA):  0x{:04X} ({})", self.registers[Register::Ra as usize], self.registers[Register::Ra as usize]);
                    eprintln!("  R4 (RAB): 0x{:04X} ({})", self.registers[Register::Rab as usize], self.registers[Register::Rab as usize]);
                    eprintln!("  R5 (RV0): 0x{:04X} ({})", self.registers[Register::Rv0 as usize], self.registers[Register::Rv0 as usize]);
                    eprintln!("  R6 (RV1): 0x{:04X} ({})", self.registers[Register::Rv1 as usize], self.registers[Register::Rv1 as usize]);
                    eprintln!("  R7 (A0):  0x{:04X} ({})", self.registers[Register::A0 as usize], self.registers[Register::A0 as usize]);
                    eprintln!("  R8 (A1):  0x{:04X} ({})", self.registers[Register::A1 as usize], self.registers[Register::A1 as usize]);
                    eprintln!("  R9 (A2):  0x{:04X} ({})", self.registers[Register::A2 as usize], self.registers[Register::A2 as usize]);
                    eprintln!("  R10 (A3): 0x{:04X} ({})", self.registers[Register::A3 as usize], self.registers[Register::A3 as usize]);
                    eprintln!("  R11 (X0): 0x{:04X} ({})", self.registers[Register::X0 as usize], self.registers[Register::X0 as usize]);
                    eprintln!("  R12 (X1): 0x{:04X} ({})", self.registers[Register::X1 as usize], self.registers[Register::X1 as usize]);
                    eprintln!("  R13 (X2): 0x{:04X} ({})", self.registers[Register::X2 as usize], self.registers[Register::X2 as usize]);
                    eprintln!("  R14 (X3): 0x{:04X} ({})", self.registers[Register::X3 as usize], self.registers[Register::X3 as usize]);
                    eprintln!("  R15 (T0): 0x{:04X} ({})", self.registers[Register::T0 as usize], self.registers[Register::T0 as usize]);
                    eprintln!("  R16 (T1): 0x{:04X} ({})", self.registers[Register::T1 as usize], self.registers[Register::T1 as usize]);
                    eprintln!("  R17 (T2): 0x{:04X} ({})", self.registers[Register::T2 as usize], self.registers[Register::T2 as usize]);
                    eprintln!("  R18 (T3): 0x{:04X} ({})", self.registers[Register::T3 as usize], self.registers[Register::T3 as usize]);
                    eprintln!("  R19 (T4): 0x{:04X} ({})", self.registers[Register::T4 as usize], self.registers[Register::T4 as usize]);
                    eprintln!("  R20 (T5): 0x{:04X} ({})", self.registers[Register::T5 as usize], self.registers[Register::T5 as usize]);
                    eprintln!("  R21 (T6): 0x{:04X} ({})", self.registers[Register::T6 as usize], self.registers[Register::T6 as usize]);
                    eprintln!("  R22 (T7): 0x{:04X} ({})", self.registers[Register::T7 as usize], self.registers[Register::T7 as usize]);
                    eprintln!("  R23 (S0): 0x{:04X} ({})", self.registers[Register::S0 as usize], self.registers[Register::S0 as usize]);
                    eprintln!("  R24 (S1): 0x{:04X} ({})", self.registers[Register::S1 as usize], self.registers[Register::S1 as usize]);
                    eprintln!("  R25 (S2): 0x{:04X} ({})", self.registers[Register::S2 as usize], self.registers[Register::S2 as usize]);
                    eprintln!("  R26 (S3): 0x{:04X} ({})", self.registers[Register::S3 as usize], self.registers[Register::S3 as usize]);
                    eprintln!("  R27 (SC): 0x{:04X} ({})", self.registers[Register::Sc as usize], self.registers[Register::Sc as usize]);
                    eprintln!("  R28 (SB): 0x{:04X} ({})", self.registers[Register::Sb as usize], self.registers[Register::Sb as usize]);
                    eprintln!("  R29 (SP): 0x{:04X} ({})", self.registers[Register::Sp as usize], self.registers[Register::Sp as usize]);
                    eprintln!("  R30 (FP): 0x{:04X} ({})", self.registers[Register::Fp as usize], self.registers[Register::Fp as usize]);
                    eprintln!("  R31 (GP): 0x{:04X} ({})", self.registers[Register::Gp as usize], self.registers[Register::Gp as usize]);
                    
                    eprintln!("\nMemory (first {DEBUG_MEMORY_DISPLAY_WORDS} words):");
                    for i in (0..DEBUG_MEMORY_DISPLAY_WORDS.min(self.memory.len())).step_by(DEBUG_MEMORY_WORDS_PER_LINE) {
                        eprint!("  {i:04X}: ");
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
                if rd < 32 && rs < 32 && rt < 32 {
                    self.registers[rd] = self.registers[rs].wrapping_mul(self.registers[rt]);
                }
            },
            0x1B => { // DIV
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let rt = instr.word3 as usize;
                if rd < 32 && rs < 32 && rt < 32 {
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
                if rd < 32 && rs < 32 && rt < 32 {
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
                if rd < 32 && rs < 32 {
                    self.registers[rd] = self.registers[rs].wrapping_mul(imm);
                }
            },
            0x1E => { // DIVI
                let rd = instr.word1 as usize;
                let rs = instr.word2 as usize;
                let imm = instr.word3;
                if rd < 32 && rs < 32 {
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
                if rd < 32 && rs < 32 {
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
        
        // Clear keyboard state
        self.clear_keyboard_state();
        
        // Reset display state (and exit any active display mode)
        if self.display_mode == DISP_TEXT40 && self.terminal_raw_mode {
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
        
        // Note: We keep the loaded instructions, data, and debug symbols intact
    }
    
    fn print_instruction(&self, instr: &Instr) {
        match instr.opcode {
            0x00 => {
                if instr.word0 == 0 && instr.word1 == 0 && instr.word2 == 0 && instr.word3 == 0 {
                    eprintln!("HALT");
                } else {
                    eprintln!("NOP");
                }
            },
            0x01 => eprintln!("ADD R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x02 => eprintln!("SUB R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x03 => eprintln!("AND R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x04 => eprintln!("OR R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x05 => eprintln!("XOR R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x06 => eprintln!("SLL R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x07 => eprintln!("SRL R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x08 => eprintln!("SLT R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x09 => eprintln!("SLTU R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x0A => eprintln!("ADDI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x0B => eprintln!("ANDI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x0C => eprintln!("ORI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x0D => eprintln!("XORI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x0E => eprintln!("LI R{}, {}", instr.word1, instr.word2),
            0x0F => eprintln!("SLLI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x10 => eprintln!("SRLI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x11 => {
                eprintln!("LOAD R{}, R{}, R{} ; R{}=mem[R{}*{}+R{}]={}", 
                    instr.word1, instr.word2, instr.word3,
                    instr.word1, instr.word2, self.bank_size, instr.word3,
                    if instr.word1 < 32 && instr.word2 < 32 && instr.word3 < 32 {
                        let addr = (self.registers[instr.word2 as usize] as usize * self.bank_size as usize) 
                                 + self.registers[instr.word3 as usize] as usize;
                        if addr < self.memory.len() {
                            self.memory[addr]
                        } else {
                            0
                        }
                    } else {
                        0
                    });
            },
            0x12 => {
                eprintln!("STORE R{}, R{}, R{} ; mem[R{}*{}+R{}]=R{}={}", 
                    instr.word1, instr.word2, instr.word3,
                    instr.word2, self.bank_size, instr.word3, instr.word1,
                    if instr.word1 < 32 {
                        self.registers[instr.word1 as usize]
                    } else {
                        0
                    });
            },
            0x13 => eprintln!("JAL R{}, {}", instr.word1, instr.word3),
            0x14 => eprintln!("JALR R{}, R{}", instr.word1, instr.word3),
            0x15 => eprintln!("BEQ R{}, R{}, {} ; if R{}==R{} goto PC+{}", 
                instr.word1, instr.word2, instr.word3 as i16,
                instr.word1, instr.word2, instr.word3 as i16),
            0x16 => eprintln!("BNE R{}, R{}, {} ; if R{}!=R{} goto PC+{}", 
                instr.word1, instr.word2, instr.word3 as i16,
                instr.word1, instr.word2, instr.word3 as i16),
            0x17 => eprintln!("BLT R{}, R{}, {}", instr.word1, instr.word2, instr.word3 as i16),
            0x18 => eprintln!("BGE R{}, R{}, {}", instr.word1, instr.word2, instr.word3 as i16),
            0x19 => eprintln!("BRK"),
            0x1A => eprintln!("MUL R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x1B => eprintln!("DIV R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x1C => eprintln!("MOD R{}, R{}, R{}", instr.word1, instr.word2, instr.word3),
            0x1D => eprintln!("MULI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x1E => eprintln!("DIVI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            0x1F => eprintln!("MODI R{}, R{}, {}", instr.word1, instr.word2, instr.word3),
            _ => eprintln!("UNKNOWN 0x{:02X}", instr.opcode),
        }
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        // Ensure terminal is restored when VM is dropped
        if self.terminal_raw_mode {
            self.exit_text40_mode();
        }
    }
}

/// Install a panic hook to ensure terminal cleanup
pub fn install_terminal_cleanup_hook() {
    use std::panic;
    
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Try to restore terminal before panicking
        let _ = terminal::disable_raw_mode();
        let _ = std::io::stderr().execute(cursor::Show);
        let _ = std::io::stderr().execute(terminal::LeaveAlternateScreen);
        let _ = std::io::stderr().execute(ResetColor);
        
        // Call the original panic hook
        original_hook(panic_info);
    }));
}