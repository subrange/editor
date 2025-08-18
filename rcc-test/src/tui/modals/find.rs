use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::tui::app::TuiApp;

pub fn draw_find_test_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
    // Calculate modal size based on results
    let modal_width = 70;
    let max_results_shown = 15;
    let results_to_show = app.search_results.len().min(max_results_shown);
    let modal_height = (results_to_show + 6).max(8) as u16; // +6 for borders, input, help
    
    let modal_area = Rect::new(
        (area.width.saturating_sub(modal_width)) / 2,
        (area.height.saturating_sub(modal_height)) / 2,
        modal_width,
        modal_height,
    );
    
    // Clear background
    f.render_widget(Clear, modal_area);
    
    // Create layout for input and results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(3),    // Results
            Constraint::Length(1), // Help text
        ])
        .split(modal_area);
    
    // Draw search input
    let input_block = Block::default()
        .title(" Find Test (fuzzy search) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let input = Paragraph::new(app.search_query.as_str())
        .style(Style::default().fg(Color::White))
        .block(input_block);
    
    f.render_widget(input, chunks[0]);
    
    // Draw search results
    if !app.search_results.is_empty() {
        let mut items: Vec<ListItem> = Vec::new();
        
        for (idx, test) in app.search_results.iter().take(max_results_shown).enumerate() {
            let test_name = test.file.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            let style = if idx == app.search_selected_index {
                Style::default().bg(Color::Rgb(60, 60, 60)).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let mut spans = vec![
                Span::raw("  "),
            ];
            
            // Highlight matching characters
            let query_lower = app.search_query.to_lowercase();
            
            let mut highlighted_name = String::new();
            let mut query_chars = query_lower.chars().peekable();
            
            for ch in test_name.chars() {
                if query_chars.peek() == Some(&ch.to_ascii_lowercase()) {
                    highlighted_name.push(ch);
                    query_chars.next();
                } else {
                    highlighted_name.push(ch);
                }
            }
            
            spans.push(Span::styled(test_name, style));
            
            // Add path info
            let path_str = test.file.to_string_lossy();
            spans.push(Span::styled(
                format!("  ({path_str})"),
                style.fg(Color::DarkGray)
            ));
            
            items.push(ListItem::new(Line::from(spans)));
        }
        
        let results_list = List::new(items)
            .block(Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan)))
            .highlight_style(Style::default())
            .highlight_symbol("> ");
        
        let mut state = ListState::default();
        state.select(Some(app.search_selected_index.min(results_to_show - 1)));
        
        f.render_stateful_widget(results_list, chunks[1], &mut state);
    } else if !app.search_query.is_empty() {
        let no_results = Paragraph::new("No tests found")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(no_results, chunks[1]);
    } else {
        let hint = Paragraph::new("Start typing to search...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(hint, chunks[1]);
    }
    
    // Draw help text
    let help = Paragraph::new("↑/↓: Navigate | Enter: Jump to test | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan)));
    
    f.render_widget(help, chunks[2]);
}