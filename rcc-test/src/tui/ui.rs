use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};
use crate::tui::app::{TuiApp, AppMode, FocusedPane, MetadataField};
use rcc_frontend::c_formatter;

pub fn draw(f: &mut Frame, app: &mut TuiApp) {
    let size = f.size();

    // Main layout - always use full screen, help will be a modal
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    // Top area layout - horizontal split
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Test list
            Constraint::Percentage(65), // Details/Output
        ])
        .split(main_chunks[0]);

    // Draw test list
    draw_test_list(f, top_chunks[0], app);

    // Right side - tabbed view for different content
    draw_details_panel(f, top_chunks[1], app);

    // Draw help modal if visible
    if app.show_help {
        draw_help_modal(f, size, app);
    }

    // Draw category selector modal if visible (for both category selection and move mode)
    if app.show_categories {
        draw_category_selector(f, size, app);
    }

    // Draw find test modal if in find mode
    if app.mode == AppMode::FindTest {
        draw_find_test_modal(f, size, app);
    }

    // Draw metadata input modal if adding metadata
    if app.mode == AppMode::AddingMetadata {
        draw_metadata_input_modal(f, size, app);
    }

    // Draw delete confirmation modal if confirming deletion
    if app.mode == AppMode::ConfirmDelete {
        draw_delete_confirmation_modal(f, size, app);
    }

    // Draw edit expected output modal if editing
    if app.mode == AppMode::EditingExpected {
        draw_edit_expected_modal(f, size, app);
    }
    
    // Draw rename test modal if renaming
    if app.mode == AppMode::RenamingTest {
        draw_rename_test_modal(f, size, app);
    }
    
    // Draw create test modal if creating
    if app.mode == AppMode::CreatingTest {
        draw_create_test_modal(f, size, app);
    }

    // Draw status bar at bottom
    draw_status_bar(f, size, app);
}

fn draw_category_selector(f: &mut Frame, area: Rect, app: &TuiApp) {
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

fn create_test_item<'a>(test: &'a crate::config::TestCase, index: usize, app: &'a TuiApp) -> ListItem<'a> {
    let test_name = test.file.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let mut spans = vec![
        Span::raw("  "),  // Indent for tests under categories
    ];

    // Check if this is an orphan test
    let is_orphan = app.orphan_tests.iter().any(|orphan| orphan.file == test.file);
    
    // Add test result indicator or orphan indicator
    if is_orphan {
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
    
    spans.push(Span::raw(test_name));
    
    // Add orphan indicator text
    if is_orphan {
        spans.push(Span::styled(" [no metadata]", Style::default().fg(Color::DarkGray)));
    }
    
    ListItem::new(Line::from(spans))
}

fn draw_test_list(f: &mut Frame, area: Rect, app: &mut TuiApp) {
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

fn draw_details_panel(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    // Tab headers for different views
    let tab_titles = vec!["Source", "ASM", "IR", "Output", "Details"];

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
        _ => {}
    }
}

fn draw_source_code(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test_path) = app.get_selected_test_path() {
        // Build full path
        let full_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
            std::path::Path::new("c-test").join(&test_path)
        } else {
            test_path.clone()
        };

        if full_path.exists() {
            match std::fs::read_to_string(&full_path) {
                Ok(code) => {
                    // Create colored lines with line numbers
                    let lines: Vec<Line> = c_formatter::format_c_code_with_line_numbers(&code)
                        .into_iter()
                        .map(Line::from)
                        .collect();
                    
                    let paragraph = Paragraph::new(lines)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(format!(" Source: {} ", test_path.display()))
                                .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 0 {
                                    Style::default().fg(Color::Cyan)
                                } else {
                                    Style::default().fg(Color::Gray)
                                })
                        )
                        .scroll((app.source_scroll as u16, 0))
                        .wrap(Wrap { trim: false });

                    f.render_widget(paragraph, area);
                }
                Err(e) => {
                    let paragraph = Paragraph::new(format!("Error reading file: {e}"))
                        .block(Block::default().borders(Borders::ALL).title(format!(" Source: {} ", test_path.display()))
                            .border_style(Style::default().fg(Color::Gray)));
                    f.render_widget(paragraph, area);
                }
            }
        } else {
            let paragraph = Paragraph::new(format!("File not found: {}", full_path.display()))
                .block(Block::default().borders(Borders::ALL).title(format!(" Source: {} ", test_path.display()))
                    .border_style(Style::default().fg(Color::Gray)));
            f.render_widget(paragraph, area);
        }
    } else {
        let paragraph = Paragraph::new("No test selected")
            .block(Block::default().borders(Borders::ALL).title(" Source ")
                .border_style(Style::default().fg(Color::Gray)));
        f.render_widget(paragraph, area);
    }
}

fn draw_asm_code(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test_name) = app.get_selected_test_name() {
        let asm_path = app.tools.build_dir.join(format!("{test_name}.asm"));
        
        if asm_path.exists() {
            match std::fs::read_to_string(&asm_path) {
                Ok(code) => {
                    // Create colored lines with line numbers
                    let mut lines: Vec<Line> = Vec::new();
                    
                    for (i, line) in code.lines().enumerate() {
                        let mut spans = vec![
                            // Line number in dark gray
                            Span::styled(
                                format!("{:4} | ", i + 1),
                                Style::default().fg(Color::DarkGray)
                            ),
                        ];
                        
                        // Format the assembly line with colors
                        let colored_spans = rvm::format_asm_line(line);
                        spans.extend(colored_spans);
                        
                        lines.push(Line::from(spans));
                    }
                    
                    let paragraph = Paragraph::new(lines)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(format!(" ASM: {test_name}.asm "))
                                .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 1 {
                                    Style::default().fg(Color::Cyan)
                                } else {
                                    Style::default().fg(Color::Gray)
                                })
                        )
                        .scroll((app.asm_scroll as u16, 0))
                        .wrap(Wrap { trim: false });

                    f.render_widget(paragraph, area);
                }
                Err(e) => {
                    let paragraph = Paragraph::new(format!("Error reading ASM file: {e}"))
                        .block(Block::default().borders(Borders::ALL).title(format!(" ASM: {test_name}.asm "))
                            .border_style(Style::default().fg(Color::Gray)));
                    f.render_widget(paragraph, area);
                }
            }
        } else {
            let paragraph = Paragraph::new("ASM file not found. Run the test first to generate it.")
                .block(Block::default().borders(Borders::ALL).title(format!(" ASM: {test_name}.asm "))
                    .border_style(Style::default().fg(Color::Gray)));
            f.render_widget(paragraph, area);
        }
    } else {
        let paragraph = Paragraph::new("No test selected")
            .block(Block::default().borders(Borders::ALL).title(" ASM ")
                .border_style(Style::default().fg(Color::Gray)));
        f.render_widget(paragraph, area);
    }
}

fn draw_ir_code(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test_name) = app.get_selected_test_name() {
        let ir_path = app.tools.build_dir.join(format!("{test_name}.ir"));
        
        let content = if ir_path.exists() {
            match std::fs::read_to_string(&ir_path) {
                Ok(code) => {
                    // Add line numbers
                    code.lines()
                        .enumerate()
                        .map(|(i, line)| format!("{:4} | {}", i + 1, line))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                Err(e) => format!("Error reading IR file: {e}"),
            }
        } else {
            "IR file not found. Run the test first to generate it.".to_string()
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" IR: {test_name}.ir "))
                    .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 2 {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Gray)
                    })
            )
            .scroll((app.ir_scroll as u16, 0))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No test selected")
            .block(Block::default().borders(Borders::ALL).title(" IR ")
                .border_style(Style::default().fg(Color::Gray)));
        f.render_widget(paragraph, area);
    }
}

fn draw_output(f: &mut Frame, area: Rect, app: &TuiApp) {
    let content = if app.output_buffer.is_empty() {
        "No output yet. Run a test to see results.".to_string()
    } else {
        app.output_buffer.clone()
    };

    // Calculate total lines for scroll indicator
    let total_lines = content.lines().count();
    let visible_lines = area.height.saturating_sub(2) as usize; // Subtract borders
    
    // Create title with scroll info if content is scrollable
    let title = if total_lines > visible_lines && !app.output_buffer.is_empty() {
        format!(" Output [Line {}/{}] (j/k to scroll) ", 
            app.output_scroll + 1, 
            total_lines)
    } else {
        " Output ".to_string()
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 3 {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                })
        )
        .scroll((app.output_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_test_details(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test) = app.get_selected_test_details() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(test.file.display().to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Runtime: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if test.use_runtime { "Yes" } else { "No" }),
            ]),
        ];

        if let Some(desc) = &test.description {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(desc.as_str()));
        }

        if let Some(expected) = &test.expected {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Expected Output: ", Style::default().add_modifier(Modifier::BOLD)),
            ]));
            for line in expected.lines() {
                lines.push(Line::from(format!("  {line}")));
            }
        }

        // Add test result if available
        if let Some(test_name) = app.get_selected_test_name() {
            if let Some(result) = app.test_results.get(&test_name) {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Result: ", Style::default().add_modifier(Modifier::BOLD)),
                    if result.passed {
                        Span::styled("PASSED", Style::default().fg(Color::Rgb(0, 200, 0)))
                    } else {
                        Span::styled("FAILED", Style::default().fg(Color::Red))
                    },
                ]));
                
                lines.push(Line::from(vec![
                    Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{}ms", result.duration_ms)),
                ]));

                if !result.output.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("Actual Output: ", Style::default().add_modifier(Modifier::BOLD)),
                    ]));
                    for line in result.output.lines() {
                        lines.push(Line::from(format!("  {line}")));
                    }
                }
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Test Details ")
                .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 4 {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                }))
            .scroll((app.details_scroll as u16, 0))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No test selected")
            .block(Block::default().borders(Borders::ALL).title(" Test Details ")
                .border_style(Style::default().fg(Color::Gray)));
        f.render_widget(paragraph, area);
    }
}

fn draw_delete_confirmation_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
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

fn draw_edit_expected_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
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

fn draw_metadata_input_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
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

fn draw_find_test_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
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
            let name_lower = test_name.to_lowercase();
            
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


fn draw_help_modal(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    let help_content = vec![
        Line::from(""),
        Line::from(Span::styled("── Test Navigation ──", Style::default().fg(Color::Yellow))),
        Line::from("  j/↓     Move down in test list"),
        Line::from("  k/↑     Move up in test list"),
        Line::from("  g       Go to first test"),
        Line::from("  G       Go to last test"),
        Line::from("  o       Jump to first orphan test"),
        Line::from("  Enter   Run selected test"),
        Line::from("  Shift+R Run all tests in category"),
        Line::from("  r       Run all visible tests"),
        Line::from("  d       Debug selected test (interactive)"),
        Line::from("  e       Edit selected test in vim"),
        Line::from(""),
        Line::from(Span::styled("── View Controls ──", Style::default().fg(Color::Yellow))),
        Line::from("  Tab     Switch between panes"),
        Line::from("  1-5     Switch tabs:"),
        Line::from("          1=Source 2=ASM 3=IR 4=Output 5=Details"),
        Line::from("  h/←     Focus test list"),
        Line::from("  l/→     Focus right panel"),
        Line::from(""),
        Line::from(Span::styled("── Categories & Filtering ──", Style::default().fg(Color::Yellow))),
        Line::from("  c       Toggle category selector"),
        Line::from("  /       Enter filter mode"),
        Line::from("  Esc     Clear filter/Exit mode"),
        Line::from("  Ctrl+L  Clear all filters"),
        Line::from(""),
        Line::from(Span::styled("── Scrolling ──", Style::default().fg(Color::Yellow))),
        Line::from("  j/k     Scroll down/up in focused panel"),
        Line::from("  Ctrl+D  Page down"),
        Line::from("  Ctrl+U  Page up"),
        Line::from(""),
        Line::from(Span::styled("── Test Results ──", Style::default().fg(Color::Yellow))),
        Line::from("  ✓       Test passed"),
        Line::from("  ✗       Test failed"),
        Line::from("  ⟳       Test running"),
        Line::from("  ⚠       Orphan test (no metadata)"),
        Line::from("  [C]     Core category"),
        Line::from("  [M]     Memory category"),
        Line::from("  [A]     Advanced category"),
        Line::from("  [I]     Integration category"),
        Line::from("  [R]     Runtime category"),
        Line::from("  [F]     Known failure"),
        Line::from(""),
        Line::from(Span::styled("── Test Management ──", Style::default().fg(Color::Yellow))),
        Line::from("  a       Add new test from template"),
        Line::from("  e       Edit selected test in vim"),
        Line::from("  E       Edit expected output (Shift+E)"),
        Line::from("  g       Golden update (apply actual as expected)"),
        Line::from("  n       Rename selected test"),
        Line::from("  M       Move test to category (uses selector)"),
        Line::from("  x       Delete selected test"),
        Line::from("  o       Jump to first orphan test"),
        Line::from("  m       Add metadata to orphan test (with modal)"),
        Line::from("  A       Quick add orphan metadata (Shift+A)"),
        Line::from("          Uses current output as expected"),
        Line::from(""),
        Line::from(Span::styled("── Other Commands ──", Style::default().fg(Color::Yellow))),
        Line::from("  F5      Force UI refresh & reload tests"),
        Line::from("  ?       Toggle this help"),
        Line::from("  q       Quit application"),
        Line::from("  Ctrl+C  Force quit"),
    ];

    // Calculate modal dimensions
    let help_width = 60;
    let help_height = area.height.min(35);
    let help_area = Rect::new(
        (area.width.saturating_sub(help_width)) / 2,
        (area.height.saturating_sub(help_height)) / 2,
        help_width,
        help_height,
    );

    // Calculate scrolling
    let visible_lines = help_height.saturating_sub(6) as usize; // Account for borders and header
    let total_lines = help_content.len();
    let max_scroll = total_lines.saturating_sub(visible_lines);
    
    // Ensure scroll is within bounds
    if app.help_scroll > max_scroll {
        app.help_scroll = max_scroll;
    }

    // Build display content
    let mut display_lines = Vec::new();
    
    // Header
    display_lines.push(Line::from(Span::styled(
        "RCT Test Runner - Help", 
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    )));
    
    // Scroll indicator
    if total_lines > visible_lines {
        display_lines.push(Line::from(vec![
            Span::styled("[↑/↓ scroll] ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("Lines {}-{}/{}", 
                app.help_scroll + 1, 
                (app.help_scroll + visible_lines).min(total_lines),
                total_lines
            ), Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        display_lines.push(Line::from(""));
    }
    
    display_lines.push(Line::from(Span::styled(
        "Press ? or ESC to close", 
        Style::default().fg(Color::Green)
    )));
    display_lines.push(Line::from(""));  // Separator
    
    // Add visible content
    let end = (app.help_scroll + visible_lines).min(total_lines);
    display_lines.extend(help_content[app.help_scroll..end].to_vec());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Help ");

    let paragraph = Paragraph::new(display_lines)
        .block(block)
        .alignment(Alignment::Left);

    // Clear background and render modal
    f.render_widget(Clear, help_area);
    f.render_widget(paragraph, help_area);
}

fn draw_rename_test_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
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

fn draw_create_test_modal(f: &mut Frame, area: Rect, app: &TuiApp) {
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

fn draw_status_bar(f: &mut Frame, area: Rect, app: &TuiApp) {
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

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}