use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui::app::TuiApp;

pub fn draw_rename_test_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
    let test_name = app.editing_test_file.as_ref()
        .and_then(|f| f.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
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
    
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // New name input
            Constraint::Length(3),  // Help text
        ])
        .split(modal_area);
    
    // Draw title
    let title = Paragraph::new(format!("Rename Test: {test_name}"))
        .block(Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(title, chunks[0]);
    
    // Draw new name input field
    let new_name_text = if app.rename_new_name.is_empty() {
        "(enter new test name)".to_string()
    } else {
        app.rename_new_name.clone()
    };
    
    let new_name = Paragraph::new(new_name_text)
        .block(Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(if app.rename_new_name.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(new_name, chunks[1]);
    
    // Draw help text
    let help = Paragraph::new("Enter: Save | Esc: Cancel")
        .block(Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    f.render_widget(help, chunks[2]);
}