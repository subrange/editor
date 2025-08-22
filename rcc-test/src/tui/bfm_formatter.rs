//! BFM (Brainfuck Macro) syntax highlighting formatter for TUI display

use ratatui::prelude::{Color, Modifier, Span, Style};
use regex::Regex;

/// Formats a line of BFM code with syntax highlighting
pub fn format_bfm_line(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut pos = 0;
    let line_bytes = line.as_bytes();

    // Patterns for different BFM elements
    let patterns = vec![
        // Comments
        (r"//.*$", Style::default().fg(Color::DarkGray)),
        // #define directive
        (r"#define\s+\w+", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        // Macro invocations with @
        (r"@\w+", Style::default().fg(Color::Yellow)),
        // Preserve blocks {:...}
        (r"\{:[^}]*\}", Style::default().fg(Color::Rgb(0, 200, 0))), // Green
        // Labels {label ...}
        (r"\{label\s+\w+\}", Style::default().fg(Color::Magenta)),
        // Built-in functions {br}, {repeat(...)}, etc.
        (r"\{(br|repeat|if|for|reverse|preserve)\b[^}]*\}", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        // Numbers
        (r"\b\d+\b", Style::default().fg(Color::Rgb(255, 140, 0))), // Orange
        // Hex numbers
        (r"\b0x[0-9a-fA-F]+\b", Style::default().fg(Color::Rgb(255, 140, 0))),
        // Brainfuck operators
        (r"[+\-<>.,\[\]$]", Style::default().fg(Color::Blue)),
        // Parentheses and braces
        (r"[(){}]", Style::default().fg(Color::Gray)),
        // Commas
        (r",", Style::default().fg(Color::Gray)),
        // Identifiers (catch-all for variable names)
        (r"\b[a-zA-Z_]\w*\b", Style::default().fg(Color::White)),
    ];

    while pos < line.len() {
        let remaining = &line[pos..];
        let mut matched = false;
        
        for (pattern_str, style) in &patterns {
            let re = Regex::new(pattern_str).unwrap();
            
            if let Some(mat) = re.find(remaining) {
                if mat.start() == 0 {
                    // Add the matched portion with style
                    let matched_text = mat.as_str().to_string();
                    spans.push(Span::styled(matched_text, *style));
                    
                    // Move past this match
                    pos += mat.end();
                    matched = true;
                    break;
                }
            }
        }
        
        if !matched {
            // No pattern matched at the beginning, take one character as-is
            if pos < line_bytes.len() {
                let ch_len = if line_bytes[pos] & 0x80 == 0 {
                    1  // ASCII
                } else if line_bytes[pos] & 0xe0 == 0xc0 {
                    2  // 2-byte UTF-8
                } else if line_bytes[pos] & 0xf0 == 0xe0 {
                    3  // 3-byte UTF-8
                } else {
                    4  // 4-byte UTF-8
                };
                
                let ch_end = (pos + ch_len).min(line_bytes.len());
                if let Ok(ch) = std::str::from_utf8(&line_bytes[pos..ch_end]) {
                    spans.push(Span::raw(ch.to_string()));
                    pos = ch_end;
                } else {
                    pos += 1;  // Skip invalid UTF-8
                }
            }
        }
    }

    spans
}

/// Formats BFM code with line numbers and syntax highlighting
pub fn format_bfm_code_with_line_numbers(code: &str) -> Vec<Vec<Span<'static>>> {
    code.lines()
        .enumerate()
        .map(|(i, line)| {
            let mut spans = vec![
                // Line number in dark gray
                Span::styled(
                    format!("{:4} | ", i + 1),
                    Style::default().fg(Color::DarkGray)
                ),
            ];
            
            // Add the highlighted line
            spans.extend(format_bfm_line(line));
            spans
        })
        .collect()
}