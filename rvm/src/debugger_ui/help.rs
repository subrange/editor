use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use crate::tui_debugger::TuiDebugger;

impl TuiDebugger {
    pub(crate) fn draw_help(&self, frame: &mut Frame, area: Rect) {
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
            Line::from("  r         Run until breakpoint.rs/halt"),
            Line::from("  c         Continue from breakpoint.rs"),
            Line::from("  R         Restart execution from beginning"),
            Line::from(""),
            Line::from(Span::styled("Breakpoints:", Style::default().fg(Color::Yellow))),
            Line::from("  b         Toggle breakpoint.rs at cursor"),
            Line::from("  Shift+B   Set/toggle breakpoint.rs by number"),
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
            Line::from("  :break <addr>    Set breakpoint.rs"),
            Line::from("  :delete <addr>   Remove breakpoint.rs"),
            Line::from("  :watch <name> <addr>  Add memory watch"),
            Line::from("  :mem <addr> <val>     Write to memory"),
            Line::from("  :reg <#> <val>        Set register"),
            Line::from("  :help            Show this help"),
            Line::from("  :quit/:q         Quit debugger_ui"),
            Line::from(""),
            Line::from(Span::styled("Other:", Style::default().fg(Color::Yellow))),
            Line::from("  :         Enter command mode"),
            Line::from("  ?         Toggle this help"),
            Line::from("  q         Quit debugger_ui"),
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
    
}