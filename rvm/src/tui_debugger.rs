use std::collections::{HashSet, HashMap};
use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Clear, Tabs, Row, Table, Cell},
    Frame, Terminal,
};
use ripple_asm::Register;
use crate::vm::{VM, VMState, Instr};

// Fixed memory columns for navigation (actual display adjusts dynamically)
pub(crate) const MEMORY_NAV_COLS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FocusedPane {
    Disassembly,
    Registers,
    Memory,
    Stack,
    Watches,
    Breakpoints,
    Output,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DebuggerMode {
    Normal,
    Command,
    MemoryEdit,
    GotoAddress,
    AddWatch,
    SetBreakpoint,
}

pub struct MemoryWatch {
    pub(crate) name: String,
    pub(crate) address: usize,
    pub(crate) size: usize,
    pub(crate) format: WatchFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum WatchFormat {
    Hex,
    Decimal,
    Char,
    Binary,
}

pub struct TuiDebugger {
    // UI State
    pub(crate) focused_pane: FocusedPane,
    pub(crate) mode: DebuggerMode,
    
    // Scrolling positions
    pub(crate) disasm_scroll: usize,
    pub(crate) memory_scroll: usize,
    pub(crate) memory_base_addr: usize,
    pub(crate) stack_scroll: usize,
    pub(crate) output_scroll: usize,
    pub(crate) watches_scroll: usize,
    
    // Debugging state
    pub(crate) breakpoints: HashSet<usize>,
    pub(crate) memory_watches: Vec<MemoryWatch>,
    pub(crate) selected_watch: usize,
    
    // Command input
    pub(crate) command_buffer: String,
    pub(crate) command_history: Vec<String>,
    pub(crate) command_history_idx: usize,
    
    // Display preferences
    pub(crate) show_help: bool,
    pub(crate) show_ascii: bool,
    
    // Performance
    last_step_time: Instant,
    step_frequency: Duration,
    
    // Execution history
    pub(crate) execution_history: Vec<usize>,
    max_history: usize,
    
    // Register highlights (changed registers)
    pub(crate) register_changes: HashMap<usize, u16>,
}

impl TuiDebugger {
    pub fn new() -> Self {
        Self {
            focused_pane: FocusedPane::Disassembly,
            mode: DebuggerMode::Normal,
            
            disasm_scroll: 0,
            memory_scroll: 0,
            memory_base_addr: 0,
            stack_scroll: 0,
            output_scroll: 0,
            watches_scroll: 0,
            
            breakpoints: HashSet::new(),
            memory_watches: Vec::new(),
            selected_watch: 0,
            
            command_buffer: String::new(),
            command_history: Vec::new(),
            command_history_idx: 0,
            
            show_help: false,
            show_ascii: true,
            
            last_step_time: Instant::now(),
            step_frequency: Duration::from_millis(100),
            
            execution_history: Vec::with_capacity(1000),
            max_history: 1000,
            
            register_changes: HashMap::new(),
        }
    }
    
    pub fn run(&mut self, vm: &mut VM) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Clear screen
        terminal.clear()?;
        
        // Main loop
        let result = self.run_app(&mut terminal, vm);
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        result
    }
    
    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>, vm: &mut VM) -> io::Result<()> {
        'main: loop {
            // Draw UI
            terminal.draw(|f| self.draw_ui(f, vm))?;
            
            // Handle input
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    match self.mode {
                        DebuggerMode::Normal => {
                            if !self.handle_normal_mode(key.code, key.modifiers, vm) {
                                break 'main; // Exit requested
                            }
                        }
                        DebuggerMode::Command => {
                            if !self.handle_command_mode(key.code, vm) {
                                break 'main; // Exit the debugger_ui
                            }
                        }
                        DebuggerMode::GotoAddress => {
                            self.handle_goto_mode(key.code);
                        }
                        DebuggerMode::AddWatch => {
                            self.handle_add_watch_mode(key.code);
                        }
                        DebuggerMode::SetBreakpoint => {
                            self.handle_breakpoint_mode(key.code, vm);
                        }
                        DebuggerMode::MemoryEdit => {
                            self.handle_memory_edit_mode(key.code, vm);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn draw_ui(&mut self, frame: &mut Frame, vm: &VM) {
        let size = frame.size();
        
        // Reserve bottom line for status/input
        let main_area = Rect::new(0, 0, size.width, size.height - 1);
        let status_area = Rect::new(0, size.height - 1, size.width, 1);
        
        // Main layout: 2 columns - left for disassembly/output, right for registers/memory/stack/watches
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45), // Left: Disassembly + Output
                Constraint::Percentage(55), // Right: Registers + Memory + Stack/Watches
            ])
            .split(main_area);
        
        // Left column: Disassembly + Output
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(15),     // Disassembly
                Constraint::Length(10),  // Output
            ])
            .split(main_chunks[0]);
        
        // Right column: Split horizontally for (Registers+Memory) and (Stack+Watches)
        let right_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65), // Left: Registers + Memory
                Constraint::Percentage(35), // Right: Stack + Watches
            ])
            .split(main_chunks[1]);
        
        // Left part of right side: Registers on top, Memory below
        let registers_memory_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),  // Registers
                Constraint::Min(15),     // Memory
            ])
            .split(right_chunks[0]);
        
        // Right part: Stack + Watches + Breakpoints (vertically stacked, full height)
        let stack_watches_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33), // Stack
                Constraint::Percentage(33), // Watches
                Constraint::Percentage(34), // Breakpoints
            ])
            .split(right_chunks[1]);
        
        // Draw each component
        self.draw_disassembly(frame, left_chunks[0], vm);
        self.draw_output(frame, left_chunks[1], vm);
        self.draw_registers(frame, registers_memory_chunks[0], vm);
        self.draw_memory(frame, registers_memory_chunks[1], vm);
        self.draw_stack(frame, stack_watches_chunks[0], vm);
        self.draw_watches(frame, stack_watches_chunks[1], vm);
        self.draw_breakpoints(frame, stack_watches_chunks[2], vm);
        
        // Draw status/input line at bottom
        match self.mode {
            DebuggerMode::Command => self.draw_input_line(frame, status_area, "Command"),
            DebuggerMode::GotoAddress => self.draw_input_line(frame, status_area, "Go to address (hex)"),
            DebuggerMode::AddWatch => self.draw_input_line(frame, status_area, "Add watch (name:addr[:format])"),
            DebuggerMode::SetBreakpoint => self.draw_input_line(frame, status_area, "Set breakpoint.rs (instr# or 0xAddr)"),
            DebuggerMode::MemoryEdit => {
                // Try to show current value at address
                let mut prompt = String::from("Edit memory (addr:value)");
                if self.command_buffer.contains(':') {
                    if let Some(colon_pos) = self.command_buffer.find(':') {
                        let addr_str = &self.command_buffer[..colon_pos];
                        if let Ok(addr) = usize::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                            if addr < vm.memory.len() {
                                prompt = format!("Edit memory @ 0x{:04X} (current: 0x{:04X})", addr, vm.memory[addr]);
                            }
                        }
                    }
                }
                self.draw_input_line(frame, status_area, &prompt)
            }
            DebuggerMode::Normal => self.draw_status_line(frame, status_area, vm),
        }
        
        // Draw help overlay if enabled
        if self.show_help {
            self.draw_help(frame, size);
        }
    }
    
    pub(crate) fn step_vm(&mut self, vm: &mut VM) {
        // Check for breakpoint.rs BEFORE executing
        let pc = vm.registers[Register::Pc as usize] as usize;
        let pcb = vm.registers[Register::Pcb as usize] as usize;
        let addr = pcb * vm.bank_size as usize + pc;
        
        // Only stop at breakpoint.rs if we're in Running state (not already at a breakpoint.rs)
        if self.breakpoints.contains(&addr) && matches!(vm.state, VMState::Running) {
            vm.state = VMState::Breakpoint;
            return;
        }
        
        self.step_vm_no_break_check(vm);
    }
    
    pub(crate) fn step_vm_no_break_check(&mut self, vm: &mut VM) {
        // Save current registers for change detection
        let old_registers = vm.registers.clone();
        
        // Get current PC for history
        let pc = vm.registers[Register::Pc as usize] as usize;
        let pcb = vm.registers[Register::Pcb as usize] as usize;
        let addr = pcb * vm.bank_size as usize + pc;
        
        // Record execution history
        self.execution_history.push(addr);
        if self.execution_history.len() > self.max_history {
            self.execution_history.remove(0);
        }
        
        // Step the VM
        let _ = vm.step();
        
        // Track register changes
        for i in 0..18 {
            if old_registers[i] != vm.registers[i] {
                self.register_changes.insert(i, old_registers[i]);
            }
        }
        
        // Auto-scroll disassembly to keep PC visible
        let new_pc = vm.registers[Register::Pc as usize] as usize;
        let new_pcb = vm.registers[Register::Pcb as usize] as usize;
        let new_addr = new_pcb * vm.bank_size as usize + new_pc;
        
        if new_addr < self.disasm_scroll || new_addr >= self.disasm_scroll + 20 {
            self.disasm_scroll = new_addr.saturating_sub(5);
        }
    }
    
    pub(crate) fn run_until_break(&mut self, vm: &mut VM) {
        while matches!(vm.state, VMState::Running) {
            // Step will check for breakpoint.rs before executing
            self.step_vm(vm);
            
            // If we hit a breakpoint.rs or other state change, stop
            if !matches!(vm.state, VMState::Running) {
                break;
            }
        }
    }
   
    

}