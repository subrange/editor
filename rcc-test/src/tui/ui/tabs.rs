use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::path::Path;
use crate::tui::app::{TuiApp, FocusedPane};
use crate::tui::ui::colors::parse_output_with_colors;
use rcc_frontend::c_formatter;

pub fn draw_source_code(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test_path) = app.get_selected_test_path() {
        // Build full path
        let full_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
            Path::new("c-test").join(&test_path)
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

pub fn draw_asm_code(f: &mut Frame, area: Rect, app: &TuiApp) {
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

pub fn draw_ir_code(f: &mut Frame, area: Rect, app: &TuiApp) {
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

pub fn draw_output(f: &mut Frame, area: Rect, app: &TuiApp) {
    // Parse the output buffer to create colored lines
    let lines = if app.output_buffer.is_empty() {
        vec![Line::from(Span::styled(
            "No output yet. Run a test to see results.",
            Style::default().fg(Color::DarkGray)
        ))]
    } else {
        parse_output_with_colors(&app.output_buffer)
    };

    // Calculate total lines for scroll indicator
    let total_lines = lines.len();
    let visible_lines = area.height.saturating_sub(2) as usize; // Subtract borders
    
    // Create title with scroll info if content is scrollable
    let title = if total_lines > visible_lines && !app.output_buffer.is_empty() {
        format!(" Output [Line {}/{}] (j/k to scroll) ", 
            app.output_scroll + 1, 
            total_lines)
    } else {
        " Output ".to_string()
    };

    let paragraph = Paragraph::new(lines)
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

pub fn draw_test_details(f: &mut Frame, area: Rect, app: &TuiApp) {
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
            Line::from(vec![
                Span::styled("Skipped: ", Style::default().add_modifier(Modifier::BOLD)),
                if test.skipped {
                    Span::styled("Yes (Test will not run)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("No", Style::default().fg(Color::Green))
                },
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