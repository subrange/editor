use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::{TuiApp, MetadataField};

pub fn draw_metadata_input_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
    // Calculate modal dimensions
    let modal_width = 80;
    let modal_height = 20;
    let modal_area = Rect::new(
        (area.width.saturating_sub(modal_width)) / 2,
        (area.height.saturating_sub(modal_height)) / 2,
        modal_width,
        modal_height,
    );
    
    // Clear background
    f.render_widget(Clear, modal_area);
    
    // Create layout for the modal
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Expected output
            Constraint::Length(3),  // Description
            Constraint::Length(3),  // Options (runtime, known failure)
            Constraint::Min(1),     // Help text
        ])
        .split(modal_area);
    
    // Draw title
    let test_name = app.metadata_input.test_file.as_ref()
        .and_then(|f| f.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let title = Paragraph::new(format!("Add Metadata for: {test_name}"))
        .block(Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(title, chunks[0]);
    
    // Draw expected output field
    let expected_style = if app.metadata_input.focused_field == MetadataField::ExpectedOutput {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    let expected_text = if app.metadata_input.expected_output.is_empty() {
        "(enter expected output, use \\n for newlines)".to_string()
    } else {
        app.metadata_input.expected_output.clone()
    };
    
    let expected = Paragraph::new(expected_text)
        .block(Block::default()
            .title("Expected Output")
            .borders(Borders::ALL)
            .border_style(expected_style))
        .wrap(Wrap { trim: false });
    f.render_widget(expected, chunks[1]);
    
    // Draw description field
    let desc_style = if app.metadata_input.focused_field == MetadataField::Description {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    let desc_text = if app.metadata_input.description.is_empty() {
        "(optional description)".to_string()
    } else {
        app.metadata_input.description.clone()
    };
    
    let description = Paragraph::new(desc_text)
        .block(Block::default()
            .title("Description")
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .border_style(desc_style));
    f.render_widget(description, chunks[2]);
    
    // Draw options
    let runtime_check = if app.metadata_input.use_runtime { "[✓]" } else { "[ ]" };
    let failure_check = if app.metadata_input.is_known_failure { "[✓]" } else { "[ ]" };
    
    let runtime_style = if app.metadata_input.focused_field == MetadataField::UseRuntime {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let failure_style = if app.metadata_input.focused_field == MetadataField::IsKnownFailure {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let options = vec![
        Line::from(vec![
            Span::styled(format!("{runtime_check} Use Runtime"), runtime_style),
            Span::raw("    "),
            Span::styled(format!("{failure_check} Known Failure"), failure_style),
        ])
    ];
    
    let options_widget = Paragraph::new(options)
        .block(Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(options_widget, chunks[3]);
    
    // Draw help text
    let help = Paragraph::new(vec![
        Line::from("Tab: Next field | Shift+Tab: Previous field | Space: Toggle checkbox"),
        Line::from("Enter: Save metadata | Esc: Cancel | A: Quick add with current output"),
    ])
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[4]);
}