use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::Paragraph;
use crate::tui_debugger::{FocusedPane, TuiDebugger};
use crate::vm::{VMState, VM};

impl TuiDebugger {
    pub(crate) fn draw_status_line(&self, frame: &mut Frame, area: Rect, vm: &VM) {
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
            FocusedPane::Disassembly => " | Space:step b:breakpoint.rs r:run",
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

}