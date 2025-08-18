use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui::app::TuiApp;

pub fn draw_create_test_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
    let current_category = app.get_current_category_name()
        .unwrap_or_else(|| "Uncategorized".to_string());
    
    // Calculate modal dimensions
    let modal_width = 70;
    let modal_height = 12;
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
            Constraint::Length(3),  // Test name input
            Constraint::Length(3),  // Description input
            Constraint::Length(3),  // Help text
        ])
        .split(modal_area);
    
    // Draw title
    let title = Paragraph::new(format!("Create New Test in: {current_category}"))
        .block(Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(title, chunks[0]);
    
    // Draw test name input field
    let name_text = if app.new_test_name.is_empty() {
        "(enter test name)".to_string()
    } else {
        app.new_test_name.clone()
    };
    
    let name_prefix = if !app.new_test_focused_field { "▶ Name: " } else { "  Name: " };
    let name_field = Paragraph::new(format!("{name_prefix}{name_text}"))
        .block(Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(if app.new_test_name.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else if !app.new_test_focused_field {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(name_field, chunks[1]);
    
    // Draw description field
    let desc_prefix = if app.new_test_focused_field { "▶ Description: " } else { "  Description: " };
    let desc_field = Paragraph::new(format!("{}{}", desc_prefix, app.new_test_description))
        .block(Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(if app.new_test_focused_field {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(desc_field, chunks[2]);
    
    // Draw help text
    let help = Paragraph::new("Tab: Switch fields | Enter: Create | Esc: Cancel")
        .block(Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    f.render_widget(help, chunks[3]);
}