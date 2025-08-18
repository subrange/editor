use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui::app::TuiApp;

pub fn draw_delete_confirmation_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
    let test_name = app.delete_target.as_ref()
        .and_then(|f| f.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    // Check if it's an orphan test
    let is_orphan = app.delete_target.as_ref()
        .map(|f| app.orphan_tests.iter().any(|t| t.file == *f))
        .unwrap_or(false);
    
    // Calculate modal dimensions
    let modal_width = 60;
    let modal_height = 10;
    let modal_area = Rect::new(
        (area.width.saturating_sub(modal_width)) / 2,
        (area.height.saturating_sub(modal_height)) / 2,
        modal_width,
        modal_height,
    );
    
    // Clear background
    f.render_widget(Clear, modal_area);
    
    // Create the confirmation message
    let test_type = if is_orphan { "orphan test" } else { "test" };
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("⚠ Warning", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(format!("Are you sure you want to delete the {test_type}:")),
        Line::from(Span::styled(format!("  {test_name}"), Style::default().fg(Color::Cyan))),
        Line::from(""),
        Line::from("This action cannot be undone!"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm or ", Style::default().fg(Color::DarkGray)),
            Span::styled("n/Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    
    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .title(" Delete Test ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)))
        .alignment(Alignment::Center);
    
    f.render_widget(paragraph, modal_area);
}