use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};
use crate::tui::app::{TuiApp, AppMode, FocusedPane};
use crate::tui::ui::tabs::{draw_source_code, draw_asm_code, draw_ir_code, draw_output, draw_test_details};

pub fn draw_test_list(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut test_index = 0;
    
    // If a specific category is selected, show only that category
    if let Some(ref selected_cat) = app.selected_category {
        if let Some(category) = app.categories.get(selected_cat) {
            // Add category header
            items.push(ListItem::new(Line::from(vec![
                Span::styled("▼ ", if category.expanded { 
                    Style::default().fg(Color::Yellow) 
                } else { 
                    Style::default().fg(Color::DarkGray) 
                }),
                Span::styled(selected_cat.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({} tests)", category.test_count), Style::default().fg(Color::DarkGray)),
            ])));
            
            // Add tests in this category
            if category.expanded {
                for test in &category.tests {
                    items.push(create_test_item(test, test_index, app));
                    test_index += 1;
                }
            }
        }
    } else {
        // Show all categories with their tests
        for (category_name, category) in &app.categories {
            // Add category header
            items.push(ListItem::new(Line::from(vec![
                Span::styled(if category.expanded { "▼ " } else { "▶ " }, 
                    Style::default().fg(Color::Yellow)),
                Span::styled(category_name.clone(), 
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({} tests)", category.test_count), 
                    Style::default().fg(Color::DarkGray)),
            ])));
            
            // Add tests if category is expanded
            if category.expanded {
                for test in &category.tests {
                    items.push(create_test_item(test, test_index, app));
                    test_index += 1;
                }
            }
        }
    }

    let selected_style = if app.focused_pane == FocusedPane::TestList {
        Style::default().bg(Color::Rgb(60, 60, 60)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::DIM)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Tests ({}/{}) ", 
                    app.filtered_tests.len() + app.filtered_failures.len(),
                    app.test_config.tests.len() + app.test_config.known_failures.len()
                ))
                .border_style(if app.focused_pane == FocusedPane::TestList {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                })
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_item));
    
    f.render_stateful_widget(list, area, &mut state);

    // TODO: Add scrollbar widget when available
}

fn create_test_item<'a>(test: &'a crate::config::TestCase, _index: usize, app: &'a TuiApp) -> ListItem<'a> {
    let test_name = test.file.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let mut spans = vec![
        Span::raw("  "),  // Indent for tests under categories
    ];

    // Check if this is an orphan test
    let is_orphan = app.orphan_tests.iter().any(|orphan| orphan.file == test.file);
    
    // Add test result indicator or orphan indicator
    if test.skipped {
        spans.push(Span::styled("⊘ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    } else if is_orphan {
        spans.push(Span::styled("⚠ ", Style::default().fg(Color::Magenta)));
    } else if let Some(result) = app.test_results.get(test_name) {
        if result.passed {
            spans.push(Span::styled("✓ ", Style::default().fg(Color::Rgb(0, 200, 0))));
        } else {
            spans.push(Span::styled("✗ ", Style::default().fg(Color::Red)));
        }
    } else if app.running_test.as_deref() == Some(test_name) {
        spans.push(Span::styled("⟳ ", Style::default().fg(Color::Yellow)));
    } else {
        spans.push(Span::raw("  "));
    }
    
    // Style test name differently if skipped
    if test.skipped {
        spans.push(Span::styled(test_name, Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)));
        spans.push(Span::styled(" [SKIP]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    } else {
        spans.push(Span::raw(test_name));
        if is_orphan {
            spans.push(Span::styled(" [no metadata]", Style::default().fg(Color::DarkGray)));
        }
    }
    
    ListItem::new(Line::from(spans))
}

pub fn draw_details_panel(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    // Tab headers for different views
    let tab_titles = vec!["Source", "ASM", "IR", "Output", "Details", "AST", "Symbols", "TypedAST"];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(5),     // Content
        ])
        .split(area);

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(app.selected_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, chunks[0]);

    // Draw content based on selected tab
    match app.selected_tab {
        0 => draw_source_code(f, chunks[1], app),
        1 => draw_asm_code(f, chunks[1], app),
        2 => draw_ir_code(f, chunks[1], app),
        3 => draw_output(f, chunks[1], app),
        4 => draw_test_details(f, chunks[1], app),
        5 => crate::tui::ui::trace_viewer::draw_ast_tree(f, chunks[1], app),
        6 => crate::tui::ui::trace_viewer::draw_symbols_table(f, chunks[1], app),
        7 => crate::tui::ui::trace_viewer::draw_typed_ast_tree(f, chunks[1], app),
        _ => {}
    }
}

pub fn draw_status_bar(f: &mut Frame, area: Rect, app: &TuiApp) {
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(20),
            Constraint::Length(30),
        ])
        .split(Rect::new(0, area.height - 1, area.width, 1));

    // Mode indicator
    let (mode_text, mode_style) = match app.mode {
        AppMode::Normal => ("NORMAL", Style::default().bg(Color::Blue).fg(Color::Black)),
        AppMode::FindTest => ("FIND", Style::default().bg(Color::Magenta).fg(Color::Black)),
        AppMode::Running => ("RUNNING", Style::default().bg(Color::Green).fg(Color::Black)),
        AppMode::SelectCategory => ("CATEGORY", Style::default().bg(Color::Cyan).fg(Color::Black)),
        AppMode::AddingMetadata => ("METADATA", Style::default().bg(Color::Yellow).fg(Color::Black)),
        AppMode::ConfirmDelete => ("DELETE", Style::default().bg(Color::Red).fg(Color::Black)),
        AppMode::EditingExpected => ("EDIT EXP", Style::default().bg(Color::Magenta).fg(Color::Black)),
        AppMode::RenamingTest => ("RENAME", Style::default().bg(Color::Yellow).fg(Color::Black)),
        AppMode::MovingTest => ("MOVE", Style::default().bg(Color::Cyan).fg(Color::Black)),
        AppMode::CreatingTest => ("CREATE", Style::default().bg(Color::Green).fg(Color::Black)),
    };
    
    let mode_spans = vec![
        Span::styled(format!(" {mode_text} "), mode_style),
        Span::raw(" "),  // Add spacing after mode
    ];
    let mode = Paragraph::new(Line::from(mode_spans))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(mode, status_chunks[0]);

    // Current action or info
    let info = if let Some(ref test) = app.running_test {
        format!("Running: {test}")
    } else if let Some(test_name) = app.get_selected_test_name() {
        format!("Selected: {test_name}")
    } else {
        "No test selected".to_string()
    };
    
    let info_widget = Paragraph::new(info)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(info_widget, status_chunks[1]);

    // Quick help
    let help = Paragraph::new(" ? for help | q to quit ")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Right);
    f.render_widget(help, status_chunks[2]);
}