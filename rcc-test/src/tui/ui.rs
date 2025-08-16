use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};
use crate::tui::app::{TuiApp, AppMode, FocusedPane, TestCategory};
use rcc_frontend::c_formatter;

pub fn draw(f: &mut Frame, app: &mut TuiApp) {
    let size = f.size();

    // Main layout - vertical split for optional help
    let main_chunks = if app.show_help {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(15),
                Constraint::Length(12), // Help window
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(size)
    };

    // Top area layout - horizontal split
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Test list
            Constraint::Percentage(65), // Details/Output
        ])
        .split(main_chunks[0]);

    // Left side - test list with categories
    let left_chunks = if app.show_categories {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Category tabs
                Constraint::Min(5),     // Test list
            ])
            .split(top_chunks[0])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(top_chunks[0])
    };

    // Draw category tabs if visible
    if app.show_categories {
        draw_categories(f, left_chunks[0], app);
        draw_test_list(f, left_chunks[1], app);
    } else {
        draw_test_list(f, left_chunks[0], app);
    }

    // Right side - tabbed view for different content
    draw_details_panel(f, top_chunks[1], app);

    // Draw help if visible
    if app.show_help && main_chunks.len() > 1 {
        draw_help(f, main_chunks[1]);
    }

    // Draw filter overlay if in filter mode
    if app.mode == AppMode::Filter {
        draw_filter_overlay(f, size, app);
    }

    // Draw status bar at bottom
    draw_status_bar(f, size, app);
}

fn draw_categories(f: &mut Frame, area: Rect, app: &TuiApp) {
    let categories: Vec<String> = TestCategory::all()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    let selected_idx = TestCategory::all()
        .iter()
        .position(|c| *c == app.selected_category)
        .unwrap_or(0);

    let tabs = Tabs::new(categories)
        .block(Block::default().borders(Borders::ALL).title("Categories"))
        .select(selected_idx)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, area);
}

fn draw_test_list(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    let mut items: Vec<ListItem> = Vec::new();
    
    // Add regular tests
    for (idx, test) in app.filtered_tests.iter().enumerate() {
        let test_name = test.file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let mut spans = vec![
            Span::raw(format!("{:3} ", idx + 1)),
        ];

        // Add test result indicator
        if let Some(result) = app.test_results.get(test_name) {
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

        // Add category indicator
        let category_indicator = if test.file.to_string_lossy().contains("/core/") {
            "C"
        } else if test.file.to_string_lossy().contains("/advanced/") {
            "A"
        } else if test.file.to_string_lossy().contains("/memory/") {
            "M"
        } else if test.file.to_string_lossy().contains("/integration/") {
            "I"
        } else if test.file.to_string_lossy().contains("/runtime/") {
            "R"
        } else if test.file.to_string_lossy().contains("/experimental/") {
            "E"
        } else {
            "?"
        };
        
        spans.push(Span::styled(
            format!("[{}] ", category_indicator),
            Style::default().fg(Color::Cyan),
        ));
        
        spans.push(Span::raw(test_name));
        
        items.push(ListItem::new(Line::from(spans)));
    }

    // Add known failures
    for failure in &app.filtered_failures {
        let test_name = failure.file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let spans = vec![
            Span::raw("    "),
            Span::styled("[F] ", Style::default().fg(Color::DarkGray)),
            Span::styled(test_name, Style::default().fg(Color::DarkGray)),
        ];
        
        items.push(ListItem::new(Line::from(spans)));
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
    state.select(Some(app.selected_test));
    
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
                    let paragraph = Paragraph::new(format!("Error reading file: {}", e))
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
        let asm_path = app.tools.build_dir.join(format!("{}.asm", test_name));
        
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
                                .title(format!(" ASM: {}.asm ", test_name))
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
                    let paragraph = Paragraph::new(format!("Error reading ASM file: {}", e))
                        .block(Block::default().borders(Borders::ALL).title(format!(" ASM: {}.asm ", test_name))
                            .border_style(Style::default().fg(Color::Gray)));
                    f.render_widget(paragraph, area);
                }
            }
        } else {
            let paragraph = Paragraph::new("ASM file not found. Run the test first to generate it.")
                .block(Block::default().borders(Borders::ALL).title(format!(" ASM: {}.asm ", test_name))
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
        let ir_path = app.tools.build_dir.join(format!("{}.ir", test_name));
        
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
                Err(e) => format!("Error reading IR file: {}", e),
            }
        } else {
            "IR file not found. Run the test first to generate it.".to_string()
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" IR: {}.ir ", test_name))
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
                lines.push(Line::from(format!("  {}", line)));
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
                        lines.push(Line::from(format!("  {}", line)));
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

fn draw_filter_overlay(f: &mut Frame, area: Rect, app: &TuiApp) {
    let popup_area = centered_rect(60, 20, area);
    
    let block = Block::default()
        .title(" Filter Tests ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let input = Paragraph::new(app.filter_text.as_str())
        .style(Style::default())
        .block(block);

    f.render_widget(input, popup_area);
}


fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![Span::styled("Test Navigation", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  j/↓     Move down"),
        Line::from("  k/↑     Move up"),
        Line::from("  Enter   Run selected test"),
        Line::from("  d       Debug selected test"),
        Line::from("  r       Run all visible tests"),
        Line::from(""),
        Line::from(vec![Span::styled("View Controls", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  Tab     Switch between panes"),
        Line::from("  1-5     Switch tabs (Source/ASM/IR/Output/Details)"),
        Line::from("  c       Toggle categories"),
        Line::from("  /       Filter tests"),
        Line::from("  Esc     Clear filter/Exit mode"),
        Line::from(""),
        Line::from(vec![Span::styled("Other", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("  ?       Toggle this help"),
        Line::from("  q       Quit"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Cyan))
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
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
        AppMode::Filter => ("FILTER", Style::default().bg(Color::Magenta).fg(Color::Black)),
        AppMode::Running => ("RUNNING", Style::default().bg(Color::Green).fg(Color::Black)),
        AppMode::SelectCategory => ("CATEGORY", Style::default().bg(Color::Cyan).fg(Color::Black)),
    };
    
    let mode = Paragraph::new(format!(" {} ", mode_text))
        .style(mode_style);
    f.render_widget(mode, status_chunks[0]);

    // Current action or info
    let info = if let Some(ref test) = app.running_test {
        format!("Running: {}", test)
    } else if let Some(test_name) = app.get_selected_test_name() {
        format!("Selected: {}", test_name)
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