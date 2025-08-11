use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::tui_debugger::{FocusedPane, TuiDebugger};
use crate::vm::VM;

impl TuiDebugger {
    pub(crate) fn draw_memory(&self, frame: &mut Frame, area: Rect, vm: &VM) {
        let mut text = Vec::new();

        // Fixed 8 columns for cognitive consistency
        let bytes_per_row = 8;

        let visible_rows = area.height.saturating_sub(3) as usize;

        let start_addr = self.memory_base_addr;
        
        // Calculate the cursor's absolute address
        let cursor_addr = self.memory_base_addr + self.memory_scroll * bytes_per_row + self.memory_cursor_col;

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
                    
                    // Check if this is the cursor position
                    let is_cursor = idx == cursor_addr && self.focused_pane == FocusedPane::Memory;
                    
                    let style = if is_cursor {
                        // Highlight cursor position
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else if idx < 2 {
                        // Special I/O registers
                        Style::default().fg(Color::Magenta)
                    } else if value != 0 {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    
                    // Apply style only to the hex value, not the space
                    spans.push(Span::styled(format!("{:04X}", value), style));
                    spans.push(Span::raw(" ")); // Add space separately
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
                        
                        // Check if this is the cursor position
                        let is_cursor = idx == cursor_addr && self.focused_pane == FocusedPane::Memory;
                        
                        let style = if is_cursor {
                            Style::default().bg(Color::Yellow).fg(Color::Black)
                        } else if idx < 2 {
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

}