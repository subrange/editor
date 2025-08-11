use crossterm::event::KeyCode;
use crate::tui_debugger::{DebuggerMode, TuiDebugger};
use crate::vm::VM;

impl TuiDebugger {
    pub(crate) fn handle_breakpoint_mode(&mut self, key: KeyCode, vm: &VM) {
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
    
}