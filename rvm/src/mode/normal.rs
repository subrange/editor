use crossterm::event::{KeyCode, KeyModifiers};
use crate::tui_debugger::{DebuggerMode, FocusedPane, TuiDebugger, MEMORY_NAV_COLS};
use crate::vm::{VMState, VM};

impl TuiDebugger {
    pub(crate) fn handle_normal_mode(&mut self, key: KeyCode, modifiers: KeyModifiers, vm: &mut VM) -> bool {
        match key {
            // Quit
            KeyCode::Char('q') if modifiers == KeyModifiers::NONE => return false,
            KeyCode::Char('Q') if modifiers == KeyModifiers::SHIFT => return false,

            // Help
            KeyCode::Char('?') => self.show_help = !self.show_help,

            // Execution control
            KeyCode::Char(' ') | KeyCode::Char('s') => {
                // Special handling for Breakpoints panel - toggle enable/disable
                if self.focused_pane == FocusedPane::Breakpoints {
                    self.toggle_selected_breakpoint();
                } else {
                    // When stepping manually, we don't need to check for breakpoints
                    // If we're at a breakpoint state, clear it first
                    if matches!(vm.state, VMState::Breakpoint) {
                        vm.state = VMState::Running;
                    }
                    self.step_vm_no_break_check(vm);
                }
            }
            KeyCode::Char('r') => {
                // If at breakpoint.rs, clear state first
                if matches!(vm.state, VMState::Breakpoint) {
                    vm.state = VMState::Running;
                }
                self.run_until_break(vm);
            }
            KeyCode::Char('c') => {
                if matches!(vm.state, VMState::Breakpoint) {
                    vm.state = VMState::Running;
                    // Step once to get past the current breakpoint.rs
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
            KeyCode::F(6) => self.focused_pane = FocusedPane::Breakpoints,
            KeyCode::F(7) => self.focused_pane = FocusedPane::Output,
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
                // Enter breakpoint.rs mode to set/toggle breakpoint.rs by instruction number
                self.mode = DebuggerMode::SetBreakpoint;
                self.command_buffer.clear();
            },
            KeyCode::Char('B') if modifiers == KeyModifiers::NONE => self.breakpoints.clear(),
            
            // Breakpoint panel operations when focused
            KeyCode::Char('d') | KeyCode::Delete if self.focused_pane == FocusedPane::Breakpoints => {
                self.delete_selected_breakpoint();
            },

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


    pub(crate) fn toggle_breakpoint_at_cursor(&mut self, _vm: &VM) {
        if self.focused_pane == FocusedPane::Disassembly {
            let addr = self.disasm_scroll;
            if self.breakpoints.contains_key(&addr) {
                self.breakpoints.remove(&addr);
            } else {
                self.breakpoints.insert(addr, true); // New breakpoints are enabled by default
            }
        }
    }

    pub(crate) fn remove_selected_watch(&mut self) {
        if !self.memory_watches.is_empty() && self.selected_watch < self.memory_watches.len() {
            self.memory_watches.remove(self.selected_watch);
            if self.selected_watch > 0 && self.selected_watch >= self.memory_watches.len() {
                self.selected_watch -= 1;
            }
        }
    }
    
    pub(crate) fn toggle_selected_breakpoint(&mut self) {
        if !self.breakpoints.is_empty() {
            // Get sorted breakpoints to find the selected one
            let mut sorted_breakpoints: Vec<_> = self.breakpoints.keys().cloned().collect();
            sorted_breakpoints.sort();
            
            if self.selected_breakpoint < sorted_breakpoints.len() {
                let addr = sorted_breakpoints[self.selected_breakpoint];
                // Toggle the enabled state
                if let Some(enabled) = self.breakpoints.get_mut(&addr) {
                    *enabled = !*enabled;
                }
            }
        }
    }
    
    pub(crate) fn delete_selected_breakpoint(&mut self) {
        if !self.breakpoints.is_empty() {
            // Get sorted breakpoints to find the selected one
            let mut sorted_breakpoints: Vec<_> = self.breakpoints.keys().cloned().collect();
            sorted_breakpoints.sort();
            
            if self.selected_breakpoint < sorted_breakpoints.len() {
                let addr = sorted_breakpoints[self.selected_breakpoint];
                self.breakpoints.remove(&addr);
                
                // Adjust selected index if needed
                if self.selected_breakpoint > 0 && self.selected_breakpoint >= self.breakpoints.len() {
                    self.selected_breakpoint -= 1;
                }
            }
        }
    }

    pub(crate) fn cycle_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Disassembly => FocusedPane::Registers,
            FocusedPane::Registers => FocusedPane::Memory,
            FocusedPane::Memory => FocusedPane::Stack,
            FocusedPane::Stack => FocusedPane::Watches,
            FocusedPane::Watches => FocusedPane::Breakpoints,
            FocusedPane::Breakpoints => FocusedPane::Output,
            FocusedPane::Output => FocusedPane::Disassembly,
            _ => FocusedPane::Disassembly,
        };
    }

    pub(crate) fn cycle_pane_reverse(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Disassembly => FocusedPane::Output,
            FocusedPane::Registers => FocusedPane::Disassembly,
            FocusedPane::Memory => FocusedPane::Registers,
            FocusedPane::Stack => FocusedPane::Memory,
            FocusedPane::Watches => FocusedPane::Stack,
            FocusedPane::Breakpoints => FocusedPane::Watches,
            FocusedPane::Output => FocusedPane::Breakpoints,
            _ => FocusedPane::Output,
        };
    }

    pub(crate) fn navigate_up(&mut self, _vm: &VM) {
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
            FocusedPane::Stack => {
                self.stack_scroll = self.stack_scroll.saturating_sub(1);
            }
            FocusedPane::Watches => {
                if self.selected_watch > 0 {
                    self.selected_watch -= 1;
                    // Auto-scroll will happen in draw_watches
                }
            }
            FocusedPane::Breakpoints => {
                if self.selected_breakpoint > 0 {
                    self.selected_breakpoint -= 1;
                    // Auto-scroll will happen in draw_breakpoints
                }
            }
            _ => {}
        }
    }

    pub(crate) fn navigate_down(&mut self, vm: &VM) {
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
            FocusedPane::Stack => {
                // Allow scrolling through execution history
                if self.stack_scroll < self.execution_history.len().saturating_sub(1) {
                    self.stack_scroll += 1;
                }
            }
            FocusedPane::Watches => {
                if self.selected_watch < self.memory_watches.len().saturating_sub(1) {
                    self.selected_watch += 1;
                    // Auto-scroll will happen in draw_watches
                }
            }
            FocusedPane::Breakpoints => {
                // Get sorted breakpoints count
                let breakpoint_count = self.breakpoints.len();
                if self.selected_breakpoint < breakpoint_count.saturating_sub(1) {
                    self.selected_breakpoint += 1;
                    // Auto-scroll will happen in draw_breakpoints
                }
            }
            _ => {}
        }
    }

    pub(crate) fn navigate_left(&mut self, _vm: &VM) {
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

    pub(crate) fn navigate_right(&mut self, vm: &VM) {
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

    pub(crate) fn page_up(&mut self, _vm: &VM) {
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
            FocusedPane::Stack => {
                self.stack_scroll = self.stack_scroll.saturating_sub(10);
            }
            _ => {}
        }
    }

    pub(crate) fn page_down(&mut self, vm: &VM) {
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
            FocusedPane::Stack => {
                let new_scroll = self.stack_scroll + 10;
                if new_scroll < self.execution_history.len() {
                    self.stack_scroll = new_scroll;
                } else if self.execution_history.len() > 0 {
                    self.stack_scroll = self.execution_history.len() - 1;
                }
            }
            _ => {}
        }
    }

}