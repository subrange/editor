use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ripple_asm::Register;
use crate::tui_debugger::TuiDebugger;
use crate::vm::{Instr, VM};

impl TuiDebugger {
    pub(crate) fn draw_disassembly(&self, frame: &mut Frame, area: Rect, vm: &VM) {
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

        let title = format!(" Disassembly [{}] ", if self.focused_pane == crate::tui_debugger::FocusedPane::Disassembly { "ACTIVE" } else { "F1" });
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused_pane == crate::tui_debugger::FocusedPane::Disassembly {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
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