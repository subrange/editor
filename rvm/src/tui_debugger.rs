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
const MEMORY_NAV_COLS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedPane {
    Disassembly,
    Registers,
    Memory,
    Stack,
    Watches,
    Output,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DebuggerMode {
    Normal,
    Command,
    MemoryEdit,
    GotoAddress,
    AddWatch,
    SetBreakpoint,
}

pub struct MemoryWatch {
    name: String,
    address: usize,
    size: usize,
    format: WatchFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WatchFormat {
    Hex,
    Decimal,
    Char,
    Binary,
}

pub struct TuiDebugger {
    // UI State
    focused_pane: FocusedPane,
    mode: DebuggerMode,
    
    // Scrolling positions
    disasm_scroll: usize,
    memory_scroll: usize,
    memory_base_addr: usize,
    stack_scroll: usize,
    output_scroll: usize,
    
    // Debugging state
    breakpoints: HashSet<usize>,
    memory_watches: Vec<MemoryWatch>,
    selected_watch: usize,
    
    // Command input
    command_buffer: String,
    command_history: Vec<String>,
    command_history_idx: usize,
    
    // Display preferences
    show_help: bool,
    show_ascii: bool,
    
    // Performance
    last_step_time: Instant,
    step_frequency: Duration,
    
    // Execution history
    execution_history: Vec<usize>,
    max_history: usize,
    
    // Register highlights (changed registers)
    register_changes: HashMap<usize, u16>,
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
                                break 'main; // Exit the debugger
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
    
    fn draw_ui(&self, frame: &mut Frame, vm: &VM) {
        let size = frame.size();
        
        // Reserve bottom line for status/input
        let main_area = Rect::new(0, 0, size.width, size.height - 1);
        let status_area = Rect::new(0, size.height - 1, size.width, 1);
        
        // Main layout: 2 columns - left smaller, right bigger for memory
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45), // Left: Disassembly + Registers
                Constraint::Percentage(55), // Right: Memory + others
            ])
            .split(main_area);
        
        // Left column: Disassembly + Registers
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(15),     // Disassembly
                Constraint::Length(10),  // Registers (same as bottom section)
            ])
            .split(main_chunks[0]);
        
        // Right column: Memory + Stack/Watches/Output
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(15),     // Memory (takes most space)
                Constraint::Length(10),  // Bottom section (same as registers)
            ])
            .split(main_chunks[1]);
        
        // Bottom right: Stack + Watches + Output
        let bottom_right_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33), // Stack
                Constraint::Percentage(33), // Watches  
                Constraint::Percentage(34), // Output
            ])
            .split(right_chunks[1]);
        
        // Draw each component
        self.draw_disassembly(frame, left_chunks[0], vm);
        self.draw_registers(frame, left_chunks[1], vm);
        self.draw_memory(frame, right_chunks[0], vm);
        self.draw_stack(frame, bottom_right_chunks[0], vm);
        self.draw_watches(frame, bottom_right_chunks[1], vm);
        self.draw_output(frame, bottom_right_chunks[2], vm);
        
        // Draw status/input line at bottom
        match self.mode {
            DebuggerMode::Command => self.draw_input_line(frame, status_area, "Command"),
            DebuggerMode::GotoAddress => self.draw_input_line(frame, status_area, "Go to address (hex)"),
            DebuggerMode::AddWatch => self.draw_input_line(frame, status_area, "Add watch (name:addr[:format])"),
            DebuggerMode::SetBreakpoint => self.draw_input_line(frame, status_area, "Set breakpoint (instr# or 0xAddr)"),
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
    
    fn draw_disassembly(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let current_pc = vm.registers[Register::Pc as usize] as usize;
        let current_pcb = vm.registers[Register::Pcb as usize] as usize;
        let current_idx = current_pcb * vm.bank_size as usize + current_pc;
        
        let mut items = Vec::new();
        let visible_lines = area.height.saturating_sub(2) as usize;
        
        // Calculate visible range
        let start_idx = self.disasm_scroll;
        let end_idx = (start_idx + visible_lines).min(vm.instructions.len());
        
        for idx in start_idx..end_idx {
            let instr = &vm.instructions[idx];
            let is_current = idx == current_idx;
            let has_breakpoint = self.breakpoints.contains(&idx);
            let in_history = self.execution_history.contains(&idx);
            
            // Format the instruction
            let addr = format!("{:04X}", idx);
            let mnemonic = self.format_instruction(instr);
            
            // Build the line with appropriate styling
            let mut spans = vec![];
            
            // Breakpoint indicator
            if has_breakpoint {
                spans.push(Span::styled("● ", Style::default().fg(Color::Red)));
            } else if is_current {
                spans.push(Span::styled("→ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            } else if in_history {
                spans.push(Span::styled("· ", Style::default().fg(Color::Gray)));
            } else {
                spans.push(Span::raw("  "));
            }
            
            // Address
            spans.push(Span::styled(
                addr,
                if is_current {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                }
            ));
            
            spans.push(Span::raw("  "));
            
            // Instruction
            let instr_style = self.get_instruction_style(instr);
            spans.push(Span::styled(mnemonic, instr_style));
            
            items.push(ListItem::new(Line::from(spans)));
        }
        
        let title = format!(" Disassembly [{}] ", if self.focused_pane == FocusedPane::Disassembly { "ACTIVE" } else { "F1" });
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused_pane == FocusedPane::Disassembly {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });
        
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
    
    fn draw_registers(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut text = Vec::new();
        
        // Special registers
        text.push(Line::from(vec![
            Span::raw("PC: "),
            Span::styled(
                format!("{:04X}:{:04X}", vm.registers[Register::Pcb as usize], vm.registers[Register::Pc as usize]),
                Style::default().fg(Color::Green)
            ),
            Span::raw("  RA: "),
            Span::styled(
                format!("{:04X}:{:04X}", vm.registers[Register::Rab as usize], vm.registers[Register::Ra as usize]),
                Style::default().fg(Color::Magenta)
            ),
        ]));
        
        text.push(Line::from(""));
        
        // General purpose registers in grid
        for row in 0..3 {
            let mut spans = Vec::new();
            for col in 0..5 {
                let reg_idx = 5 + row * 5 + col;
                if reg_idx <= 17 {
                    let value = vm.registers[reg_idx];
                    let name = format!("R{:2}", reg_idx - 2);
                    
                    // Check if changed
                    let style = if self.register_changes.get(&reg_idx) == Some(&value) {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else if value != 0 {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    
                    spans.push(Span::raw(format!("{}=", name)));
                    spans.push(Span::styled(format!("{:04X}", value), style));
                    if col < 4 && (5 + row * 5 + col + 1) <= 17 {
                        spans.push(Span::raw("  "));
                    }
                }
            }
            text.push(Line::from(spans));
        }
        
        text.push(Line::from(""));
        
        // VM State
        let state_color = match vm.state {
            VMState::Running => Color::Green,
            VMState::Halted => Color::Red,
            VMState::Breakpoint => Color::Yellow,
            VMState::Error(_) => Color::Red,
            VMState::Setup => Color::Gray,
        };
        
        text.push(Line::from(vec![
            Span::raw("State: "),
            Span::styled(format!("{:?}", vm.state), Style::default().fg(state_color)),
        ]));
        
        let title = format!(" Registers [{}] ", if self.focused_pane == FocusedPane::Registers { "ACTIVE" } else { "F2" });
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused_pane == FocusedPane::Registers {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });
        
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
    
    fn draw_memory(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut text = Vec::new();
        
        // Calculate how many columns we can fit
        // Format: "XXXX: " (6) + "XXXX " per column (5) + " | " (3) + 1 char per column for ASCII
        let available_width = area.width as usize;
        let addr_width = 6; // "XXXX: "
        let hex_per_col = 5; // "XXXX "
        let separator = if self.show_ascii { 3 } else { 0 }; // " | "
        let ascii_per_col = if self.show_ascii { 1 } else { 0 };
        
        // Calculate maximum columns that fit
        let mut bytes_per_row = 8; // Start with 8 columns as default
        if available_width > addr_width {
            let remaining = available_width - addr_width - separator;
            let per_column = hex_per_col + ascii_per_col;
            if per_column > 0 {
                bytes_per_row = (remaining / per_column).min(16).max(4); // Between 4 and 16 columns
            }
        }
        
        let visible_rows = area.height.saturating_sub(3) as usize;
        
        let start_addr = self.memory_base_addr + self.memory_scroll * bytes_per_row;
        
        for row in 0..visible_rows {
            let addr = start_addr + row * bytes_per_row;
            if addr >= vm.memory.len() {
                break;
            }
            
            let mut spans = vec![
                Span::styled(format!("{:04X}: ", addr), Style::default().fg(Color::DarkGray)),
            ];
            
            // Hex values
            for col in 0..bytes_per_row {
                let idx = addr + col;
                if idx < vm.memory.len() {
                    let value = vm.memory[idx];
                    let style = if idx < 2 {
                        // Special I/O registers
                        Style::default().fg(Color::Magenta)
                    } else if value != 0 {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    spans.push(Span::styled(format!("{:04X} ", value), style));
                } else {
                    spans.push(Span::raw("     "));
                }
            }
            
            // ASCII representation if enabled
            if self.show_ascii {
                spans.push(Span::raw(" | "));
                
                for col in 0..bytes_per_row {
                    let idx = addr + col;
                    if idx < vm.memory.len() {
                        let value = (vm.memory[idx] & 0xFF) as u8;
                        let ch = if value >= 0x20 && value < 0x7F {
                            value as char
                        } else {
                            '.'
                        };
                        let style = if idx < 2 {
                            Style::default().fg(Color::Magenta)
                        } else if value != 0 {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        spans.push(Span::styled(ch.to_string(), style));
                    } else {
                        spans.push(Span::raw(" "));
                    }
                }
            }
            
            text.push(Line::from(spans));
        }
        
        let cursor_addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
        let ascii_indicator = if self.show_ascii { " [ASCII]" } else { "" };
        let title = format!(" Memory @ {:04X}{} (cursor: {:04X}) [{}] ", 
            self.memory_base_addr,
            ascii_indicator,
            cursor_addr,
            if self.focused_pane == FocusedPane::Memory { "ACTIVE" } else { "F3" }
        );
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused_pane == FocusedPane::Memory {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });
        
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
    
    fn draw_stack(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        // For now, we'll show the call stack based on RA/RAB
        let mut text = Vec::new();
        
        text.push(Line::from(vec![
            Span::raw("Return Address: "),
            Span::styled(
                format!("{:04X}:{:04X}", vm.registers[Register::Rab as usize], vm.registers[Register::Ra as usize]),
                Style::default().fg(Color::Yellow)
            ),
        ]));
        
        text.push(Line::from(""));
        text.push(Line::from(Span::styled("Call History:", Style::default().fg(Color::DarkGray))));
        
        // Show last few addresses from execution history
        let history_len = self.execution_history.len();
        let start = history_len.saturating_sub(10);
        for (i, &addr) in self.execution_history[start..].iter().enumerate() {
            text.push(Line::from(vec![
                Span::raw(format!("  {} ", if i == history_len - start - 1 { "→" } else { " " })),
                Span::styled(format!("{:04X}", addr), Style::default().fg(Color::White)),
            ]));
        }
        
        let title = format!(" Stack [{}] ", if self.focused_pane == FocusedPane::Stack { "ACTIVE" } else { "F4" });
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused_pane == FocusedPane::Stack {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });
        
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
    
    fn draw_watches(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut text = Vec::new();
        
        if self.memory_watches.is_empty() {
            text.push(Line::from(Span::styled(
                "No watches set",
                Style::default().fg(Color::DarkGray)
            )));
            text.push(Line::from(""));
            text.push(Line::from(Span::raw("Press 'w' to add a watch")));
        } else {
            for (i, watch) in self.memory_watches.iter().enumerate() {
                let is_selected = i == self.selected_watch && self.focused_pane == FocusedPane::Watches;
                
                let mut spans = vec![];
                if is_selected {
                    spans.push(Span::styled("→ ", Style::default().fg(Color::Yellow)));
                } else {
                    spans.push(Span::raw("  "));
                }
                
                spans.push(Span::styled(&watch.name, Style::default().fg(Color::Cyan)));
                spans.push(Span::raw(": "));
                
                // Get value from memory
                if watch.address < vm.memory.len() {
                    let value = vm.memory[watch.address];
                    let formatted = match watch.format {
                        WatchFormat::Hex => format!("0x{:04X}", value),
                        WatchFormat::Decimal => format!("{}", value),
                        WatchFormat::Char => {
                            let ch = (value & 0xFF) as u8;
                            if ch >= 0x20 && ch < 0x7F {
                                format!("'{}'", ch as char)
                            } else {
                                format!("\\x{:02X}", ch)
                            }
                        }
                        WatchFormat::Binary => format!("{:016b}", value),
                    };
                    spans.push(Span::styled(formatted, Style::default().fg(Color::White)));
                } else {
                    spans.push(Span::styled("Invalid", Style::default().fg(Color::Red)));
                }
                
                text.push(Line::from(spans));
            }
        }
        
        let title = format!(" Watches [{}] ", if self.focused_pane == FocusedPane::Watches { "ACTIVE" } else { "F5" });
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused_pane == FocusedPane::Watches {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });
        
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
    
    fn draw_output(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        // Get output from VM's buffer
        let output_bytes: Vec<u8> = vm.output_buffer.iter().cloned().collect();
        let output_text = String::from_utf8_lossy(&output_bytes);
        let lines: Vec<Line> = output_text
            .lines()
            .skip(self.output_scroll)
            .map(|line| Line::from(Span::raw(line)))
            .collect();
        
        let title = format!(" Output [{}] ", if self.focused_pane == FocusedPane::Output { "ACTIVE" } else { "F6" });
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused_pane == FocusedPane::Output {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });
        
        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
    
    fn draw_status_line(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut spans = vec![];
        
        // Show VM state
        let state_color = match vm.state {
            VMState::Running => Color::Green,
            VMState::Halted => Color::Red,
            VMState::Breakpoint => Color::Yellow,
            VMState::Error(_) => Color::Red,
            VMState::Setup => Color::Gray,
        };
        
        spans.push(Span::styled(
            format!(" {} ", match vm.state {
                VMState::Running => "RUNNING",
                VMState::Halted => "HALTED",
                VMState::Breakpoint => "BREAKPOINT",
                VMState::Error(_) => "ERROR",
                VMState::Setup => "SETUP",
            }),
            Style::default().bg(state_color).fg(Color::Black).add_modifier(Modifier::BOLD)
        ));
        
        spans.push(Span::raw(" "));
        
        // Show active pane
        spans.push(Span::styled(
            format!("Active: {}", match self.focused_pane {
                FocusedPane::Disassembly => "Disassembly",
                FocusedPane::Registers => "Registers",
                FocusedPane::Memory => "Memory",
                FocusedPane::Stack => "Stack",
                FocusedPane::Watches => "Watches",
                FocusedPane::Output => "Output",
                _ => "Unknown",
            }),
            Style::default().fg(Color::Cyan)
        ));
        
        // Show hints based on context
        let hints = match self.focused_pane {
            FocusedPane::Disassembly => " | Space:step b:breakpoint r:run",
            FocusedPane::Memory => " | g:goto e:edit",
            FocusedPane::Watches => " | w:add W:remove",
            _ => " | ?:help q:quit",
        };
        spans.push(Span::styled(hints, Style::default().fg(Color::DarkGray)));
        
        // Right-align some info
        let breakpoint_count = self.breakpoints.len();
        if breakpoint_count > 0 {
            let bp_text = format!(" {} BP ", breakpoint_count);
            let used_width = spans.iter().map(|s| s.content.len()).sum::<usize>();
            let padding = (area.width as usize).saturating_sub(used_width + bp_text.len());
            if padding > 0 {
                spans.push(Span::raw(" ".repeat(padding)));
            }
            spans.push(Span::styled(bp_text, Style::default().fg(Color::Red)));
        }
        
        let paragraph = Paragraph::new(Line::from(spans));
        frame.render_widget(paragraph, area);
    }
    
    fn draw_input_line(&self, frame: &mut Frame, area: Rect, prompt: &str) {
        let input_area = area;  // Use the provided area directly
        
        // Clear the line first
        frame.render_widget(Clear, input_area);
        
        // Create the input line with prompt
        let mut spans = vec![];
        
        // Add prompt with appropriate styling
        let prompt_text = format!("[{}] ", prompt);
        spans.push(Span::styled(prompt_text, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        
        // Show what the user is typing
        if self.mode == DebuggerMode::Command {
            spans.push(Span::styled(":", Style::default().fg(Color::Yellow)));
        }
        spans.push(Span::styled(&self.command_buffer, Style::default().fg(Color::White)));
        
        // Add blinking cursor
        spans.push(Span::styled("█", Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)));
        
        // Add hint on the right side
        let hint = " (ESC to cancel)";
        let used_width = prompt.len() + 3 + self.command_buffer.len() + 1 + hint.len();
        let padding_len = (area.width as usize).saturating_sub(used_width);
        if padding_len > 0 {
            spans.push(Span::raw(" ".repeat(padding_len)));
            spans.push(Span::styled(hint, Style::default().fg(Color::DarkGray)));
        }
        
        let paragraph = Paragraph::new(Line::from(spans));
        frame.render_widget(paragraph, input_area);
    }
    
    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(Span::styled("RVM TUI Debugger Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Navigation:", Style::default().fg(Color::Yellow))),
            Line::from("  F1-F6     Switch between panes"),
            Line::from("  Tab       Cycle through panes forward"),
            Line::from("  Shift+Tab Cycle through panes backward"),
            Line::from("  h/j/k/l   Vim-style navigation"),
            Line::from("  ↑/↓/←/→   Arrow key navigation"),
            Line::from(""),
            Line::from(Span::styled("Execution:", Style::default().fg(Color::Yellow))),
            Line::from("  Space/s   Step single instruction"),
            Line::from("  r         Run until breakpoint/halt"),
            Line::from("  c         Continue from breakpoint"),
            Line::from("  R         Restart execution from beginning"),
            Line::from(""),
            Line::from(Span::styled("Breakpoints:", Style::default().fg(Color::Yellow))),
            Line::from("  b         Toggle breakpoint at cursor"),
            Line::from("  Shift+B   Set/toggle breakpoint by number"),
            Line::from("  B         Clear all breakpoints"),
            Line::from(""),
            Line::from(Span::styled("Memory:", Style::default().fg(Color::Yellow))),
            Line::from("  g         Go to address"),
            Line::from("  a         Toggle ASCII display (in Memory pane)"),
            Line::from("  e         Edit memory (formats below)"),
            Line::from("    addr:0xFF      Hex value"),
            Line::from("    addr:255       Decimal value"),
            Line::from("    addr:'A'       Character"),
            Line::from("    addr:\"Hello\"   String"),
            Line::from("  0-9,a-f   Quick edit (in Memory pane)"),
            Line::from("  w         Add memory watch"),
            Line::from("  W         Remove selected watch"),
            Line::from(""),
            Line::from(Span::styled("Command Mode (:):", Style::default().fg(Color::Yellow))),
            Line::from("  :break <addr>    Set breakpoint"),
            Line::from("  :delete <addr>   Remove breakpoint"),
            Line::from("  :watch <name> <addr>  Add memory watch"),
            Line::from("  :mem <addr> <val>     Write to memory"),
            Line::from("  :reg <#> <val>        Set register"),
            Line::from("  :help            Show this help"),
            Line::from("  :quit/:q         Quit debugger"),
            Line::from(""),
            Line::from(Span::styled("Other:", Style::default().fg(Color::Yellow))),
            Line::from("  :         Enter command mode"),
            Line::from("  ?         Toggle this help"),
            Line::from("  q         Quit debugger"),
            Line::from(""),
            Line::from(Span::styled("Press '?' to close help", Style::default().fg(Color::DarkGray))),
        ];
        
        let help_width = 50;
        let help_height = 30;
        let help_area = Rect::new(
            (area.width - help_width) / 2,
            (area.height - help_height) / 2,
            help_width,
            help_height,
        );
        
        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        
        let paragraph = Paragraph::new(help_text)
            .block(block)
            .alignment(Alignment::Left);
        
        frame.render_widget(Clear, help_area);
        frame.render_widget(paragraph, help_area);
    }
    
    fn handle_normal_mode(&mut self, key: KeyCode, modifiers: KeyModifiers, vm: &mut VM) -> bool {
        match key {
            // Quit
            KeyCode::Char('q') if modifiers == KeyModifiers::NONE => return false,
            KeyCode::Char('Q') if modifiers == KeyModifiers::SHIFT => return false,
            
            // Help
            KeyCode::Char('?') => self.show_help = !self.show_help,
            
            // Execution control
            KeyCode::Char(' ') | KeyCode::Char('s') => {
                // If we're at a breakpoint, clear the breakpoint state and step
                if matches!(vm.state, VMState::Breakpoint) {
                    vm.state = VMState::Running;
                    self.step_vm_no_break_check(vm);
                } else {
                    self.step_vm(vm);
                }
            }
            KeyCode::Char('r') => {
                // If at breakpoint, clear state first
                if matches!(vm.state, VMState::Breakpoint) {
                    vm.state = VMState::Running;
                }
                self.run_until_break(vm);
            }
            KeyCode::Char('c') => {
                if matches!(vm.state, VMState::Breakpoint) {
                    vm.state = VMState::Running;
                    // Step once to get past the current breakpoint
                    self.step_vm_no_break_check(vm);
                    // Then continue running
                    if matches!(vm.state, VMState::Running) {
                        self.run_until_break(vm);
                    }
                }
            }
            
            // Pane switching
            KeyCode::F(1) => self.focused_pane = FocusedPane::Disassembly,
            KeyCode::F(2) => self.focused_pane = FocusedPane::Registers,
            KeyCode::F(3) => self.focused_pane = FocusedPane::Memory,
            KeyCode::F(4) => self.focused_pane = FocusedPane::Stack,
            KeyCode::F(5) => self.focused_pane = FocusedPane::Watches,
            KeyCode::F(6) => self.focused_pane = FocusedPane::Output,
            KeyCode::Tab if modifiers == KeyModifiers::NONE => self.cycle_pane(),
            KeyCode::BackTab | KeyCode::Tab if modifiers == KeyModifiers::SHIFT => self.cycle_pane_reverse(),
            
            // Navigation based on focused pane
            KeyCode::Up | KeyCode::Char('k') => self.navigate_up(vm),
            KeyCode::Down | KeyCode::Char('j') => self.navigate_down(vm),
            KeyCode::Left | KeyCode::Char('h') => self.navigate_left(vm),
            KeyCode::Right | KeyCode::Char('l') => self.navigate_right(vm),
            KeyCode::PageUp => self.page_up(vm),
            KeyCode::PageDown => self.page_down(vm),
            
            // Breakpoints
            KeyCode::Char('b') if modifiers == KeyModifiers::NONE => self.toggle_breakpoint_at_cursor(vm),
            KeyCode::Char('B') if modifiers == KeyModifiers::SHIFT => {
                // Enter breakpoint mode to set/toggle breakpoint by instruction number
                self.mode = DebuggerMode::SetBreakpoint;
                self.command_buffer.clear();
            },
            KeyCode::Char('B') if modifiers == KeyModifiers::NONE => self.breakpoints.clear(),
            
            // Memory operations
            KeyCode::Char('g') => self.mode = DebuggerMode::GotoAddress,
            KeyCode::Char('w') => self.mode = DebuggerMode::AddWatch,
            KeyCode::Char('W') => self.remove_selected_watch(),
            KeyCode::Char('e') => {
                if self.focused_pane == FocusedPane::Memory {
                    // Pre-fill with current cursor position
                    let addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                    self.command_buffer = format!("{:04x}:", addr);
                    self.mode = DebuggerMode::MemoryEdit;
                } else {
                    self.mode = DebuggerMode::MemoryEdit;
                }
            }
            
            // Command mode
            KeyCode::Char(':') => {
                self.mode = DebuggerMode::Command;
                self.command_buffer.clear();
            }
            
            // Reset
            KeyCode::Char('R') => {
                vm.reset();
                self.execution_history.clear();
                self.register_changes.clear();
            }
            
            // Toggle ASCII display in memory view
            KeyCode::Char('a') if self.focused_pane == FocusedPane::Memory => {
                self.show_ascii = !self.show_ascii;
            }
            
            // Quick memory edit - if in memory view and pressing hex digit
            KeyCode::Char(c) if self.focused_pane == FocusedPane::Memory && c.is_ascii_hexdigit() => {
                let addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                self.command_buffer = format!("{:04x}:{}", addr, c);
                self.mode = DebuggerMode::MemoryEdit;
            }
            
            _ => {}
        }
        
        true
    }
    
    fn handle_command_mode(&mut self, key: KeyCode, vm: &mut VM) -> bool {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                let command = self.command_buffer.clone();
                let should_continue = self.execute_command(&command, vm);
                if !should_continue {
                    return false; // Quit the debugger
                }
                self.command_history.push(command);
                self.command_buffer.clear();
                self.mode = DebuggerMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            KeyCode::Up => {
                if self.command_history_idx > 0 {
                    self.command_history_idx -= 1;
                    self.command_buffer = self.command_history[self.command_history_idx].clone();
                }
            }
            KeyCode::Down => {
                if self.command_history_idx < self.command_history.len() - 1 {
                    self.command_history_idx += 1;
                    self.command_buffer = self.command_history[self.command_history_idx].clone();
                }
            }
            _ => {}
        }
        true // Continue unless quit command was entered
    }
    
    fn handle_goto_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                // Parse address - handle both "0x" prefix and plain hex
                let addr_str = self.command_buffer.trim();
                let addr_result = if addr_str.starts_with("0x") || addr_str.starts_with("0X") {
                    usize::from_str_radix(&addr_str[2..], 16)
                } else {
                    usize::from_str_radix(addr_str, 16)
                };
                
                if let Ok(addr) = addr_result {
                    self.memory_base_addr = addr & !0xF; // Align to 16 bytes
                    self.memory_scroll = 0;
                }
                self.command_buffer.clear();
                self.mode = DebuggerMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) if c.is_ascii_hexdigit() || c == 'x' => {
                self.command_buffer.push(c);
            }
            _ => {}
        }
    }
    
    fn handle_add_watch_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                // Parse format: "name:address[:format]"
                let parts: Vec<&str> = self.command_buffer.split(':').collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    if let Ok(addr) = usize::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        let format = if parts.len() > 2 {
                            match parts[2] {
                                "hex" | "h" => WatchFormat::Hex,
                                "dec" | "d" => WatchFormat::Decimal,
                                "char" | "c" => WatchFormat::Char,
                                "bin" | "b" => WatchFormat::Binary,
                                _ => WatchFormat::Hex,
                            }
                        } else {
                            WatchFormat::Hex
                        };
                        
                        self.memory_watches.push(MemoryWatch {
                            name,
                            address: addr,
                            size: 1,
                            format,
                        });
                    }
                }
                self.command_buffer.clear();
                self.mode = DebuggerMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            _ => {}
        }
    }
    
    fn handle_breakpoint_mode(&mut self, key: KeyCode, vm: &VM) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                let input = self.command_buffer.trim();
                
                // Try to parse as instruction number first (decimal)
                if let Ok(instr_num) = input.parse::<usize>() {
                    // Convert instruction number to address
                    if instr_num < vm.instructions.len() {
                        let addr = instr_num; // Instruction number is the address
                        if self.breakpoints.contains(&addr) {
                            self.breakpoints.remove(&addr);
                        } else {
                            self.breakpoints.insert(addr);
                        }
                    }
                } else if input.starts_with("0x") {
                    // Parse as hex address
                    if let Ok(addr) = usize::from_str_radix(&input[2..], 16) {
                        if self.breakpoints.contains(&addr) {
                            self.breakpoints.remove(&addr);
                        } else {
                            self.breakpoints.insert(addr);
                        }
                    }
                }
                
                self.command_buffer.clear();
                self.mode = DebuggerMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) if c.is_ascii_alphanumeric() || c == 'x' => {
                self.command_buffer.push(c);
            }
            _ => {}
        }
    }
    
    fn handle_memory_edit_mode(&mut self, key: KeyCode, vm: &mut VM) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                // Parse "address:value" or just "value" if address is pre-filled
                let parts: Vec<&str> = self.command_buffer.split(':').collect();
                
                let (addr_str, value_str) = if parts.len() == 2 {
                    (parts[0], parts[1])
                } else if parts.len() == 1 && self.command_buffer.contains(':') {
                    // Address pre-filled, just value after colon
                    let colon_pos = self.command_buffer.find(':').unwrap();
                    let (addr_part, value_part) = self.command_buffer.split_at(colon_pos);
                    (addr_part, &value_part[1..])
                } else {
                    // Invalid format
                    self.command_buffer.clear();
                    self.mode = DebuggerMode::Normal;
                    return;
                };
                
                if let Ok(mut addr) = usize::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                    // Check if it's a string literal
                    if value_str.starts_with('"') && value_str.ends_with('"') {
                        // String literal - write multiple bytes
                        let string_content = &value_str[1..value_str.len()-1];
                        for ch in string_content.chars() {
                            if addr < vm.memory.len() {
                                vm.memory[addr] = ch as u16;
                                addr += 1;
                            }
                        }
                        // Jump to the modified address for visual feedback
                        if let Ok(start_addr) = usize::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                            self.memory_base_addr = start_addr & !0xF;
                            self.memory_scroll = 0;
                        }
                    } else {
                        // Parse single value - support multiple formats
                        let value = if value_str.starts_with("'") && value_str.ends_with("'") && value_str.len() == 3 {
                            // Character literal
                            value_str.chars().nth(1).map(|c| c as u16)
                        } else if value_str.starts_with("0x") {
                            // Hexadecimal
                            u16::from_str_radix(&value_str[2..], 16).ok()
                        } else if value_str.starts_with("0b") {
                            // Binary
                            u16::from_str_radix(&value_str[2..], 2).ok()
                        } else if value_str.chars().all(|c| c.is_ascii_hexdigit()) {
                            // Assume hex without 0x prefix
                            u16::from_str_radix(value_str, 16).ok()
                        } else {
                            // Decimal
                            value_str.parse::<u16>().ok()
                        };
                        
                        if let Some(val) = value {
                            if addr < vm.memory.len() {
                                vm.memory[addr] = val;
                                // Don't change the memory view position when editing
                                // User can manually navigate if needed
                            }
                        }
                    }
                }
                
                self.command_buffer.clear();
                self.mode = DebuggerMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            _ => {}
        }
    }
    
    fn step_vm(&mut self, vm: &mut VM) {
        // Check for breakpoint BEFORE executing
        let pc = vm.registers[Register::Pc as usize] as usize;
        let pcb = vm.registers[Register::Pcb as usize] as usize;
        let addr = pcb * vm.bank_size as usize + pc;
        
        // Only stop at breakpoint if we're in Running state (not already at a breakpoint)
        if self.breakpoints.contains(&addr) && matches!(vm.state, VMState::Running) {
            vm.state = VMState::Breakpoint;
            return;
        }
        
        self.step_vm_no_break_check(vm);
    }
    
    fn step_vm_no_break_check(&mut self, vm: &mut VM) {
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
    
    fn run_until_break(&mut self, vm: &mut VM) {
        while matches!(vm.state, VMState::Running) {
            // Step will check for breakpoint before executing
            self.step_vm(vm);
            
            // If we hit a breakpoint or other state change, stop
            if !matches!(vm.state, VMState::Running) {
                break;
            }
        }
    }
    
    fn toggle_breakpoint_at_cursor(&mut self, vm: &VM) {
        if self.focused_pane == FocusedPane::Disassembly {
            let addr = self.disasm_scroll;
            if self.breakpoints.contains(&addr) {
                self.breakpoints.remove(&addr);
            } else {
                self.breakpoints.insert(addr);
            }
        }
    }
    
    fn remove_selected_watch(&mut self) {
        if !self.memory_watches.is_empty() && self.selected_watch < self.memory_watches.len() {
            self.memory_watches.remove(self.selected_watch);
            if self.selected_watch > 0 && self.selected_watch >= self.memory_watches.len() {
                self.selected_watch -= 1;
            }
        }
    }
    
    fn cycle_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Disassembly => FocusedPane::Registers,
            FocusedPane::Registers => FocusedPane::Memory,
            FocusedPane::Memory => FocusedPane::Stack,
            FocusedPane::Stack => FocusedPane::Watches,
            FocusedPane::Watches => FocusedPane::Output,
            FocusedPane::Output => FocusedPane::Disassembly,
            _ => FocusedPane::Disassembly,
        };
    }
    
    fn cycle_pane_reverse(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Disassembly => FocusedPane::Output,
            FocusedPane::Registers => FocusedPane::Disassembly,
            FocusedPane::Memory => FocusedPane::Registers,
            FocusedPane::Stack => FocusedPane::Memory,
            FocusedPane::Watches => FocusedPane::Stack,
            FocusedPane::Output => FocusedPane::Watches,
            _ => FocusedPane::Output,
        };
    }
    
    fn navigate_up(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Disassembly => {
                self.disasm_scroll = self.disasm_scroll.saturating_sub(1);
            }
            FocusedPane::Memory => {
                // Calculate current absolute address
                let current_addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                if current_addr >= MEMORY_NAV_COLS {
                    // Move up by one row
                    let new_addr = current_addr - MEMORY_NAV_COLS;
                    self.memory_base_addr = new_addr & !0xF; // Align to 16 bytes
                    self.memory_scroll = (new_addr - self.memory_base_addr) / MEMORY_NAV_COLS;
                } else if current_addr > 0 {
                    // At the top, just go to 0
                    self.memory_base_addr = 0;
                    self.memory_scroll = 0;
                }
            }
            FocusedPane::Output => {
                self.output_scroll = self.output_scroll.saturating_sub(1);
            }
            FocusedPane::Watches => {
                if self.selected_watch > 0 {
                    self.selected_watch -= 1;
                }
            }
            _ => {}
        }
    }
    
    fn navigate_down(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Disassembly => {
                if self.disasm_scroll < vm.instructions.len().saturating_sub(1) {
                    self.disasm_scroll += 1;
                }
            }
            FocusedPane::Memory => {
                // Calculate current absolute address
                let current_addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                let new_addr = current_addr + MEMORY_NAV_COLS;
                if new_addr < vm.memory.len() {
                    self.memory_base_addr = new_addr & !0xF; // Align to 16 bytes
                    self.memory_scroll = (new_addr - self.memory_base_addr) / MEMORY_NAV_COLS;
                }
            }
            FocusedPane::Output => {
                self.output_scroll += 1;
            }
            FocusedPane::Watches => {
                if self.selected_watch < self.memory_watches.len().saturating_sub(1) {
                    self.selected_watch += 1;
                }
            }
            _ => {}
        }
    }
    
    fn navigate_left(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Memory => {
                // Move left by one column (1 byte)
                let current_addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                if current_addr > 0 {
                    let new_addr = current_addr - 1;
                    self.memory_base_addr = new_addr & !0xF; // Align to 16 bytes
                    self.memory_scroll = (new_addr - self.memory_base_addr) / MEMORY_NAV_COLS;
                }
            }
            _ => {}
        }
    }
    
    fn navigate_right(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Memory => {
                // Move right by one column (1 byte)
                let current_addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                if current_addr + 1 < vm.memory.len() {
                    let new_addr = current_addr + 1;
                    self.memory_base_addr = new_addr & !0xF; // Align to 16 bytes
                    self.memory_scroll = (new_addr - self.memory_base_addr) / MEMORY_NAV_COLS;
                }
            }
            _ => {}
        }
    }
    
    fn page_up(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Disassembly => {
                self.disasm_scroll = self.disasm_scroll.saturating_sub(20);
            }
            FocusedPane::Memory => {
                // Calculate current absolute address
                let current_addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                let rows_to_move = 10;
                if current_addr >= rows_to_move * MEMORY_NAV_COLS {
                    let new_addr = current_addr - (rows_to_move * MEMORY_NAV_COLS);
                    self.memory_base_addr = new_addr & !0xF; // Align to 16 bytes
                    self.memory_scroll = (new_addr - self.memory_base_addr) / MEMORY_NAV_COLS;
                } else {
                    // Jump to beginning
                    self.memory_base_addr = 0;
                    self.memory_scroll = 0;
                }
            }
            _ => {}
        }
    }
    
    fn page_down(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Disassembly => {
                self.disasm_scroll = (self.disasm_scroll + 20).min(vm.instructions.len().saturating_sub(1));
            }
            FocusedPane::Memory => {
                // Calculate current absolute address
                let current_addr = self.memory_base_addr + self.memory_scroll * MEMORY_NAV_COLS;
                let rows_to_move = 10;
                let new_addr = current_addr + (rows_to_move * MEMORY_NAV_COLS);
                if new_addr < vm.memory.len() {
                    self.memory_base_addr = new_addr & !0xF; // Align to 16 bytes
                    self.memory_scroll = (new_addr - self.memory_base_addr) / MEMORY_NAV_COLS;
                }
            }
            _ => {}
        }
    }
    
    fn execute_command(&mut self, command: &str, vm: &mut VM) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return true;
        }
        
        match parts[0] {
            // Breakpoint commands
            "b" | "break" => {
                // Usage: break <addr>  - Set breakpoint at address
                if parts.len() > 1 {
                    if let Ok(addr) = usize::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        self.breakpoints.insert(addr);
                    }
                }
            }
            "d" | "delete" => {
                // Usage: delete <addr> - Remove breakpoint at address
                if parts.len() > 1 {
                    if let Ok(addr) = usize::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        self.breakpoints.remove(&addr);
                    }
                }
            }
            
            // Watch commands
            "w" | "watch" => {
                // Usage: watch <name> <addr> - Add memory watch
                if parts.len() > 2 {
                    let name = parts[1].to_string();
                    if let Ok(addr) = usize::from_str_radix(parts[2].trim_start_matches("0x"), 16) {
                        self.memory_watches.push(MemoryWatch {
                            name,
                            address: addr,
                            size: 1,
                            format: WatchFormat::Hex,
                        });
                    }
                }
            }
            
            // Memory commands
            "m" | "mem" => {
                // Usage: mem <addr> <value> - Write value to memory
                if parts.len() > 2 {
                    if let Ok(addr) = usize::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        if let Ok(value) = u16::from_str_radix(parts[2].trim_start_matches("0x"), 16) {
                            if addr < vm.memory.len() {
                                vm.memory[addr] = value;
                            }
                        }
                    }
                }
            }
            
            // Register commands
            "reg" => {
                // Usage: reg <reg#> <value> - Set register value
                if parts.len() > 2 {
                    if let Ok(reg) = parts[1].parse::<usize>() {
                        if let Ok(value) = u16::from_str_radix(parts[2].trim_start_matches("0x"), 16) {
                            if reg < 18 {
                                vm.registers[reg] = value;
                            }
                        }
                    }
                }
            }
            
            // Help command
            "help" | "h" | "?" => {
                self.show_help = true;
            }
            
            // Quit command
            "q" | "quit" | "exit" => {
                return false; // Signal to quit
            }
            
            _ => {}
        }
        
        true // Continue running
    }
    
    fn format_instruction(&self, instr: &Instr) -> String {
        let opcode_str = Self::opcode_name(instr.opcode);
        
        match instr.opcode {
            0x00 => {
                if instr.word0 == 0 && instr.word1 == 0 && instr.word2 == 0 && instr.word3 == 0 {
                    "HALT".to_string()
                } else {
                    "NOP".to_string()
                }
            },
            0x01..=0x09 | 0x1A..=0x1C => {
                let rd = Self::register_name(instr.word1 as u8);
                let rs = Self::register_name(instr.word2 as u8);
                let rt = Self::register_name(instr.word3 as u8);
                format!("{:<6} {}, {}, {}", opcode_str, rd, rs, rt)
            },
            0x0A..=0x0D | 0x0F | 0x10 | 0x1D..=0x1F => {
                let rd = Self::register_name(instr.word1 as u8);
                let rs = Self::register_name(instr.word2 as u8);
                format!("{:<6} {}, {}, 0x{:X}", opcode_str, rd, rs, instr.word3)
            },
            0x0E => {
                let rd = Self::register_name(instr.word1 as u8);
                format!("{:<6} {}, 0x{:X}", opcode_str, rd, instr.word2)
            },
            0x11 | 0x12 => {
                let r = Self::register_name(instr.word1 as u8);
                let bank = Self::format_operand(instr.word2);
                let addr = Self::format_operand(instr.word3);
                format!("{:<6} {}, {}, {}", opcode_str, r, bank, addr)
            },
            0x13 => {
                let rd = Self::register_name(instr.word1 as u8);
                format!("{:<6} {}, 0x{:04X}", opcode_str, rd, instr.word3)
            },
            0x14 => {
                let rd = Self::register_name(instr.word1 as u8);
                let rs = Self::register_name(instr.word3 as u8);
                format!("{:<6} {}, {}", opcode_str, rd, rs)
            },
            0x15..=0x18 => {
                let rs = Self::register_name(instr.word1 as u8);
                let rt = Self::register_name(instr.word2 as u8);
                let offset = instr.word3 as i16;
                format!("{:<6} {}, {}, {}", opcode_str, rs, rt, offset)
            },
            0x19 => "BRK".to_string(),
            _ => format!("??? 0x{:02X}", instr.opcode),
        }
    }
    
    fn get_instruction_style(&self, instr: &Instr) -> Style {
        match instr.opcode {
            0x00 if instr.word1 == 0 && instr.word2 == 0 && instr.word3 == 0 => {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            },
            0x19 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            0x13..=0x18 => Style::default().fg(Color::Yellow),
            0x11 | 0x12 => Style::default().fg(Color::Blue),
            0x01..=0x09 | 0x1A..=0x1C => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::White),
        }
    }
    
    fn opcode_name(opcode: u8) -> &'static str {
        match opcode {
            0x00 => "NOP",
            0x01 => "ADD",
            0x02 => "SUB",
            0x03 => "AND",
            0x04 => "OR",
            0x05 => "XOR",
            0x06 => "SLL",
            0x07 => "SRL",
            0x08 => "SLT",
            0x09 => "SLTU",
            0x0A => "ADDI",
            0x0B => "ANDI",
            0x0C => "ORI",
            0x0D => "XORI",
            0x0E => "LI",
            0x0F => "SLLI",
            0x10 => "SRLI",
            0x11 => "LOAD",
            0x12 => "STORE",
            0x13 => "JAL",
            0x14 => "JALR",
            0x15 => "BEQ",
            0x16 => "BNE",
            0x17 => "BLT",
            0x18 => "BGE",
            0x19 => "BRK",
            0x1A => "MUL",
            0x1B => "DIV",
            0x1C => "MOD",
            0x1D => "MULI",
            0x1E => "DIVI",
            0x1F => "MODI",
            _ => "???",
        }
    }
    
    fn register_name(reg: u8) -> &'static str {
        match reg {
            0 => "R0",
            1 => "PC",
            2 => "PCB",
            3 => "RA",
            4 => "RAB",
            5 => "R3",
            6 => "R4",
            7 => "R5",
            8 => "R6",
            9 => "R7",
            10 => "R8",
            11 => "R9",
            12 => "R10",
            13 => "R11",
            14 => "R12",
            15 => "R13",
            16 => "R14",
            17 => "R15",
            _ => "??",
        }
    }
    
    fn format_operand(value: u16) -> String {
        if value < 18 {
            Self::register_name(value as u8).to_string()
        } else if value > 9 {
            format!("0x{:X}", value)
        } else {
            format!("{}", value)
        }
    }
}