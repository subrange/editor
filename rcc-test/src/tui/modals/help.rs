use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui::app::TuiApp;

pub fn draw_help_modal(f: &mut Frame, area: Rect, app: &mut TuiApp) {
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
        Line::from("  t       Open terminal shell"),
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
        Line::from("  ⊘ [SKIP] Test skipped (yellow, will not run)"),
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
        Line::from("  s       Toggle skip status for selected test"),
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