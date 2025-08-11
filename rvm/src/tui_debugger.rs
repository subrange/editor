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
    memory_cols: usize,
    
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
            memory_cols: 16,
            
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
        loop {
            // Draw UI
            terminal.draw(|f| self.draw_ui(f, vm))?;
            
            // Handle input
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    match self.mode {
                        DebuggerMode::Normal => {
                            if !self.handle_normal_mode(key.code, key.modifiers, vm) {
                                break; // Exit requested
                            }
                        }
                        DebuggerMode::Command => {
                            self.handle_command_mode(key.code, vm);
                        }
                        DebuggerMode::GotoAddress => {
                            self.handle_goto_mode(key.code);
                        }
                        DebuggerMode::AddWatch => {
                            self.handle_add_watch_mode(key.code);
                        }
                        DebuggerMode::SetBreakpoint => {
                            self.handle_breakpoint_mode(key.code);
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
        
        // Main layout: 3 columns
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Left: Disassembly
                Constraint::Percentage(35), // Middle: Registers + Stack
                Constraint::Percentage(25), // Right: Memory + Watches
            ])
            .split(size);
        
        // Left column: Disassembly + Output
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70), // Disassembly
                Constraint::Percentage(30), // Output
            ])
            .split(main_chunks[0]);
        
        // Middle column: Registers + Stack
        let middle_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12), // Registers
                Constraint::Min(10),    // Stack
            ])
            .split(main_chunks[1]);
        
        // Right column: Memory + Watches
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Memory
                Constraint::Percentage(40), // Watches
            ])
            .split(main_chunks[2]);
        
        // Draw each component
        self.draw_disassembly(frame, left_chunks[0], vm);
        self.draw_output(frame, left_chunks[1], vm);
        self.draw_registers(frame, middle_chunks[0], vm);
        self.draw_stack(frame, middle_chunks[1], vm);
        self.draw_memory(frame, right_chunks[0], vm);
        self.draw_watches(frame, right_chunks[1], vm);
        
        // Draw command line at bottom if in command mode
        if self.mode == DebuggerMode::Command {
            self.draw_command_line(frame, size);
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
        let bytes_per_row = self.memory_cols;
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
            
            spans.push(Span::raw(" | "));
            
            // ASCII representation
            for col in 0..bytes_per_row {
                let idx = addr + col;
                if idx < vm.memory.len() {
                    let value = (vm.memory[idx] & 0xFF) as u8;
                    let ch = if value >= 0x20 && value < 0x7F {
                        value as char
                    } else {
                        '.'
                    };
                    spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Cyan)));
                }
            }
            
            text.push(Line::from(spans));
        }
        
        let title = format!(" Memory @ {:04X} [{}] ", self.memory_base_addr, if self.focused_pane == FocusedPane::Memory { "ACTIVE" } else { "F3" });
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
    
    fn draw_command_line(&self, frame: &mut Frame, area: Rect) {
        let command_area = Rect::new(0, area.height - 1, area.width, 1);
        
        let mut spans = vec![
            Span::styled(":", Style::default().fg(Color::Yellow)),
            Span::raw(&self.command_buffer),
        ];
        
        // Add cursor
        spans.push(Span::styled("█", Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)));
        
        let paragraph = Paragraph::new(Line::from(spans));
        frame.render_widget(Clear, command_area);
        frame.render_widget(paragraph, command_area);
    }
    
    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(Span::styled("RVM TUI Debugger Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Navigation:", Style::default().fg(Color::Yellow))),
            Line::from("  F1-F6     Switch between panes"),
            Line::from("  Tab       Cycle through panes"),
            Line::from("  h/j/k/l   Vim-style navigation"),
            Line::from("  ↑/↓/←/→   Arrow key navigation"),
            Line::from(""),
            Line::from(Span::styled("Execution:", Style::default().fg(Color::Yellow))),
            Line::from("  Space/s   Step single instruction"),
            Line::from("  r         Run until breakpoint/halt"),
            Line::from("  c         Continue from breakpoint"),
            Line::from("  n         Step over (skip calls)"),
            Line::from("  o         Step out (finish function)"),
            Line::from(""),
            Line::from(Span::styled("Breakpoints:", Style::default().fg(Color::Yellow))),
            Line::from("  b         Toggle breakpoint at cursor"),
            Line::from("  B         Clear all breakpoints"),
            Line::from(""),
            Line::from(Span::styled("Memory:", Style::default().fg(Color::Yellow))),
            Line::from("  g         Go to address"),
            Line::from("  w         Add memory watch"),
            Line::from("  W         Remove selected watch"),
            Line::from("  e         Edit memory at cursor"),
            Line::from("  m         Change memory display format"),
            Line::from(""),
            Line::from(Span::styled("Other:", Style::default().fg(Color::Yellow))),
            Line::from("  :         Enter command mode"),
            Line::from("  ?         Toggle this help"),
            Line::from("  q         Quit debugger"),
            Line::from("  R         Restart execution from beginning"),
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
                self.step_vm(vm);
            }
            KeyCode::Char('r') => {
                self.run_until_break(vm);
            }
            KeyCode::Char('c') => {
                if matches!(vm.state, VMState::Breakpoint) {
                    vm.state = VMState::Running;
                    self.run_until_break(vm);
                }
            }
            
            // Pane switching
            KeyCode::F(1) => self.focused_pane = FocusedPane::Disassembly,
            KeyCode::F(2) => self.focused_pane = FocusedPane::Registers,
            KeyCode::F(3) => self.focused_pane = FocusedPane::Memory,
            KeyCode::F(4) => self.focused_pane = FocusedPane::Stack,
            KeyCode::F(5) => self.focused_pane = FocusedPane::Watches,
            KeyCode::F(6) => self.focused_pane = FocusedPane::Output,
            KeyCode::Tab => self.cycle_pane(),
            
            // Navigation based on focused pane
            KeyCode::Up | KeyCode::Char('k') => self.navigate_up(vm),
            KeyCode::Down | KeyCode::Char('j') => self.navigate_down(vm),
            KeyCode::Left | KeyCode::Char('h') => self.navigate_left(vm),
            KeyCode::Right | KeyCode::Char('l') => self.navigate_right(vm),
            KeyCode::PageUp => self.page_up(vm),
            KeyCode::PageDown => self.page_down(vm),
            
            // Breakpoints
            KeyCode::Char('b') => self.toggle_breakpoint_at_cursor(vm),
            KeyCode::Char('B') => self.breakpoints.clear(),
            
            // Memory operations
            KeyCode::Char('g') => self.mode = DebuggerMode::GotoAddress,
            KeyCode::Char('w') => self.mode = DebuggerMode::AddWatch,
            KeyCode::Char('W') => self.remove_selected_watch(),
            KeyCode::Char('e') => self.mode = DebuggerMode::MemoryEdit,
            
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
            
            _ => {}
        }
        
        true
    }
    
    fn handle_command_mode(&mut self, key: KeyCode, vm: &mut VM) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                let command = self.command_buffer.clone();
                self.execute_command(&command, vm);
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
    }
    
    fn handle_goto_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                if let Ok(addr) = usize::from_str_radix(&self.command_buffer.trim_start_matches("0x"), 16) {
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
    
    fn handle_breakpoint_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                if let Ok(addr) = usize::from_str_radix(&self.command_buffer.trim_start_matches("0x"), 16) {
                    if self.breakpoints.contains(&addr) {
                        self.breakpoints.remove(&addr);
                    } else {
                        self.breakpoints.insert(addr);
                    }
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
    
    fn handle_memory_edit_mode(&mut self, key: KeyCode, vm: &mut VM) {
        match key {
            KeyCode::Esc => {
                self.mode = DebuggerMode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                // Parse "address:value"
                let parts: Vec<&str> = self.command_buffer.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(addr) = usize::from_str_radix(parts[0].trim_start_matches("0x"), 16) {
                        if let Ok(value) = u16::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                            if addr < vm.memory.len() {
                                vm.memory[addr] = value;
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
        // Save current registers for change detection
        let old_registers = vm.registers.clone();
        
        // Record execution history
        let pc = vm.registers[Register::Pc as usize] as usize;
        let pcb = vm.registers[Register::Pcb as usize] as usize;
        let addr = pcb * vm.bank_size as usize + pc;
        
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
            let pc = vm.registers[Register::Pc as usize] as usize;
            let pcb = vm.registers[Register::Pcb as usize] as usize;
            let addr = pcb * vm.bank_size as usize + pc;
            
            if self.breakpoints.contains(&addr) {
                vm.state = VMState::Breakpoint;
                break;
            }
            
            self.step_vm(vm);
            
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
    
    fn navigate_up(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Disassembly => {
                self.disasm_scroll = self.disasm_scroll.saturating_sub(1);
            }
            FocusedPane::Memory => {
                self.memory_scroll = self.memory_scroll.saturating_sub(1);
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
                let max_scroll = (vm.memory.len() / self.memory_cols).saturating_sub(10);
                if self.memory_scroll < max_scroll {
                    self.memory_scroll += 1;
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
                if self.memory_base_addr >= 16 {
                    self.memory_base_addr -= 16;
                }
            }
            _ => {}
        }
    }
    
    fn navigate_right(&mut self, vm: &VM) {
        match self.focused_pane {
            FocusedPane::Memory => {
                if self.memory_base_addr + 16 < vm.memory.len() {
                    self.memory_base_addr += 16;
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
                self.memory_scroll = self.memory_scroll.saturating_sub(10);
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
                let max_scroll = (vm.memory.len() / self.memory_cols).saturating_sub(10);
                self.memory_scroll = (self.memory_scroll + 10).min(max_scroll);
            }
            _ => {}
        }
    }
    
    fn execute_command(&mut self, command: &str, vm: &mut VM) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        
        match parts[0] {
            "b" | "break" => {
                if parts.len() > 1 {
                    if let Ok(addr) = usize::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        self.breakpoints.insert(addr);
                    }
                }
            }
            "d" | "delete" => {
                if parts.len() > 1 {
                    if let Ok(addr) = usize::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        self.breakpoints.remove(&addr);
                    }
                }
            }
            "w" | "watch" => {
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
            "m" | "mem" => {
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
            "reg" => {
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
            _ => {}
        }
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