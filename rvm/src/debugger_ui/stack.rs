use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ripple_asm::Register;
use crate::tui_debugger::{FocusedPane, TuiDebugger};
use crate::vm::VM;

impl TuiDebugger {
    pub(crate) fn draw_stack(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut text = Vec::new();

        // Show return address at top
        text.push(Line::from(vec![
            Span::raw("Return: "),
            Span::styled(
                format!("{:04X}:{:04X}", vm.registers[Register::Rab as usize], vm.registers[Register::Ra as usize]),
                Style::default().fg(Color::Yellow)
            ),
        ]));
        text.push(Line::from(""));

        // Calculate visible area for history
        let visible_lines = area.height.saturating_sub(5) as usize; // Account for title, return addr, etc.
        let history_len = self.execution_history.len();

        if history_len == 0 {
            text.push(Line::from(Span::styled(
                "No execution history",
                Style::default().fg(Color::DarkGray)
            )));
        } else {
            // Calculate scroll bounds
            let total_items = history_len;
            let max_scroll = total_items.saturating_sub(visible_lines);
            let scroll = self.stack_scroll.min(max_scroll);

            // Show scrolled portion of history
            let start = scroll;
            let end = (start + visible_lines).min(history_len);

            for (display_idx, actual_idx) in (start..end).enumerate() {
                let addr = self.execution_history[actual_idx];
                let is_current = actual_idx == history_len - 1;

                let mut spans = vec![];
                if is_current {
                    spans.push(Span::styled("► ", Style::default().fg(Color::Green)));
                } else {
                    spans.push(Span::raw("  "));
                }

                spans.push(Span::styled(
                    format!("{:04X}: ", actual_idx),
                    Style::default().fg(Color::DarkGray)
                ));

                // Try to show instruction at that address if available
                if addr < vm.instructions.len() {
                    let instr = &vm.instructions[addr];
                    spans.push(Span::styled(
                        format!("{:04X}", addr),
                        if is_current {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        }
                    ));
                } else {
                    spans.push(Span::styled(
                        format!("{:04X}", addr),
                        Style::default().fg(Color::Red)
                    ));
                }

                text.push(Line::from(spans));
            }
        }

        let scroll_indicator = if history_len > visible_lines {
            format!(" [{}/{}]", self.stack_scroll + 1, history_len)
        } else {
            String::new()
        };

        let title = format!(" Stack{} [{}] ",
                            scroll_indicator,
                            if self.focused_pane == FocusedPane::Stack { "ACTIVE" } else { "F4" }
        );
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

}