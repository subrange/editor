use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use crate::tui_debugger::TuiDebugger;

impl TuiDebugger {
    pub(crate) fn draw_help(&mut self, frame: &mut Frame, area: Rect) {
        // Compact help text - fits better in window
        let all_help = vec![
            Line::from(Span::styled("RVM Debugger (↑/↓ scroll, ? close)", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("── Navigation ──", Style::default().fg(Color::Yellow))),
            Line::from("F1-F7,Tab  Panes | hjkl/↑↓←→  Move"),
            Line::from(""),
            Line::from(Span::styled("── Execution ──", Style::default().fg(Color::Yellow))),
            Line::from("Space/s  Step | r  Run | c  Continue"),
            Line::from("R  Restart | b  Breakpoint"),
            Line::from(""),
            Line::from(Span::styled("── Disassembly ──", Style::default().fg(Color::Yellow))),
            Line::from("Shift+H  Hex view | 0-9,a-f  Edit"),
            Line::from(""),
            Line::from(Span::styled("── Memory ──", Style::default().fg(Color::Yellow))),
            Line::from("g  Go addr | Shift+G  Stack"),
            Line::from("a  ASCII | e  Edit | w/W  Watch"),
            Line::from(""),
            Line::from(Span::styled("── Panels (T+#) ──", Style::default().fg(Color::Yellow))),
            Line::from("Shift+T then: 2-7 to toggle"),
            Line::from("2 Reg | 3 Mem | 4 Stack"),
            Line::from("5 Watch | 6 Break | 7 Output"),
            Line::from(""),
            Line::from(Span::styled("── Commands (:) ──", Style::default().fg(Color::Yellow))),
            Line::from(":break <a> | :mem <a> <v>"),
            Line::from(":reg <#> <v> | :q  Quit"),
            Line::from(""),
            Line::from(Span::styled("── Edit Formats ──", Style::default().fg(Color::Yellow))),
            Line::from("addr:0xFF  Hex | addr:255  Dec"),
            Line::from("addr:'A'  Char | addr:\"hi\"  Str"),
            Line::from(""),
            Line::from("?  Help | q  Quit"),
        ];

        // Calculate visible area
        let help_width = 40;
        let help_height = area.height.min(25);
        let help_area = Rect::new(
            (area.width.saturating_sub(help_width)) / 2,
            (area.height.saturating_sub(help_height)) / 2,
            help_width,
            help_height,
        );

        // Calculate scrolling
        let visible_lines = help_height.saturating_sub(2) as usize; // Account for borders
        let total_lines = all_help.len();
        let max_scroll = total_lines.saturating_sub(visible_lines);
        
        // Ensure scroll is within bounds
        if self.help_scroll > max_scroll {
            self.help_scroll = max_scroll;
        }

        // Get visible portion
        let end = (self.help_scroll + visible_lines).min(total_lines);
        let visible_help: Vec<Line> = all_help[self.help_scroll..end].to_vec();

        // Add scroll indicator if needed
        let title = if total_lines > visible_lines {
            format!(" Help [{}/{}] ", self.help_scroll + 1, total_lines)
        } else {
            " Help ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(visible_help)
            .block(block)
            .alignment(Alignment::Left);

        frame.render_widget(Clear, help_area);
        frame.render_widget(paragraph, help_area);
    }
    
}