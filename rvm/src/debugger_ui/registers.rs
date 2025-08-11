use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ripple_asm::Register;
use crate::tui_debugger::{FocusedPane, TuiDebugger};
use crate::vm::{VMState, VM};

impl TuiDebugger {
    pub(crate) fn draw_registers(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut text = Vec::new();

        // Special registers
        text.push(Line::from(vec![
            Span::styled("PC", Style::default().fg(Color::Gray)),
            Span::styled("=", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:04X}:{:04X}", vm.registers[Register::Pcb as usize], vm.registers[Register::Pc as usize]),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            ),
            Span::raw("  "),
            Span::styled("RA", Style::default().fg(Color::Gray)),
            Span::styled("=", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:04X}:{:04X}", vm.registers[Register::Rab as usize], vm.registers[Register::Ra as usize]),
                Style::default().fg(Color::Magenta)
            ),
        ]));

        text.push(Line::from(""));
        
        // R0 (always zero) on its own line
        text.push(Line::from(vec![
            Span::styled("R0", Style::default().fg(Color::Gray)),
            Span::styled("=", Style::default().fg(Color::DarkGray)),
            Span::styled("0000", Style::default().fg(Color::DarkGray)),
            Span::styled(" (always zero)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]));
        
        text.push(Line::from(""));

        // General purpose registers in grid (R3-R15)
        for row in 0..3 {
            let mut spans = Vec::new();
            for col in 0..5 {
                let reg_idx = 5 + row * 5 + col;
                if reg_idx <= 17 {
                    let value = vm.registers[reg_idx];
                    let name = format!("R{:<2}", reg_idx - 2);

                    // Check if register was recently changed
                    let was_changed = self.register_changes.contains_key(&reg_idx);
                    
                    // Style for register name
                    let name_style = if was_changed {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    
                    // Style for value
                    let value_style = if was_changed {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else if value != 0 {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    spans.push(Span::styled(name, name_style));
                    spans.push(Span::styled("=", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled(format!("{:04X}", value), value_style));
                    
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

}