use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};
use crate::tui::app::{TuiApp, AppMode};

pub fn draw_category_selector(f: &mut Frame, area: Rect, app: &TuiApp) {
    // Create a list of categories with "All Tests" at the top
    let mut items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("All Tests", if app.selected_category_index == 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
            Span::raw(format!(" ({})", app.test_config.tests.len() + app.test_config.known_failures.len())),
        ]))
    ];
    
    // Add all categories
    for (idx, (name, category)) in app.categories.iter().enumerate() {
        let style = if app.selected_category_index == idx + 1 {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled(name.clone(), style),
            Span::raw(format!(" ({})", category.test_count)),
        ])));
    }

    // Calculate modal dimensions
    let modal_width = 50;
    let modal_height = (items.len() + 4).min(20) as u16; // +4 for borders and title
    let modal_area = Rect::new(
        (area.width.saturating_sub(modal_width)) / 2,
        (area.height.saturating_sub(modal_height)) / 2,
        modal_width,
        modal_height,
    );

    // Different title based on mode
    let title = if app.mode == AppMode::MovingTest {
        " Move Test to Category (↑/↓: Navigate | Enter: Move | Esc: Cancel) ".to_string()
    } else {
        " Select Category (↑/↓: Navigate | Enter: Select | Esc: Cancel) ".to_string()
    };
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .highlight_style(Style::default().bg(Color::Rgb(60, 60, 60)))
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_category_index));

    // Clear background and render modal
    f.render_widget(Clear, modal_area);
    f.render_stateful_widget(list, modal_area, &mut state);
}