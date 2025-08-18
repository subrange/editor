use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::TuiApp;

pub fn draw_edit_expected_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
    let test_name = app.editing_test_file.as_ref()
        .and_then(|f| f.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    // Calculate modal dimensions
    let modal_width = 80;
    let modal_height = 15;
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
            Constraint::Min(5),     // Expected output
            Constraint::Length(3),  // Help text
        ])
        .split(modal_area);
    
    // Draw title
    let title = Paragraph::new(format!("Edit Expected Output for: {test_name}"))
        .block(Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(title, chunks[0]);
    
    // Draw expected output field
    let expected_text = if app.editing_expected.is_empty() {
        "(enter expected output, use \\n for newlines)".to_string()
    } else {
        // Show the raw string with escape sequences visible
        app.editing_expected
            .replace('\n', "\\n")
            .replace('\t', "\\t")
            .replace('\r', "\\r")
    };
    
    let expected = Paragraph::new(expected_text)
        .block(Block::default()
            .title("Expected Output")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .wrap(Wrap { trim: false });
    f.render_widget(expected, chunks[1]);
    
    // Draw help text
    let help = Paragraph::new(vec![
        Line::from("Enter: Save | Esc: Cancel | Backspace: Delete char"),
        Line::from("Use \\n for newlines, \\t for tabs"),
    ])
        .block(Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}