use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn parse_output_with_colors(output: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    for line in output.lines() {
        let line_str = line.to_string();
        
        // Check for different patterns and apply appropriate colors
        if line_str.starts_with("  ✓") {
            // Test passed
            let parts: Vec<String> = line_str.splitn(2, " ").map(|s| s.to_string()).collect();
            if parts.len() >= 2 {
                let mut spans = vec![
                    Span::styled("  ✓ ".to_string(), Style::default().fg(Color::Rgb(0, 200, 0)).add_modifier(Modifier::BOLD)),
                ];
                
                // Split the rest by "PASSED" to color it separately
                if let Some(idx) = parts[1].find("PASSED") {
                    let (name, rest) = parts[1].split_at(idx);
                    spans.push(Span::raw(name.to_string()));
                    spans.push(Span::styled("PASSED".to_string(), Style::default().fg(Color::Rgb(0, 200, 0))));
                    spans.push(Span::raw(rest[6..].to_string())); // Skip "PASSED"
                } else {
                    spans.push(Span::raw(parts[1].clone()));
                }
                lines.push(Line::from(spans));
            } else {
                lines.push(Line::from(line_str));
            }
        } else if line_str.starts_with("  ✗") {
            // Test failed
            let parts: Vec<String> = line_str.splitn(2, " ").map(|s| s.to_string()).collect();
            if parts.len() >= 2 {
                let mut spans = vec![
                    Span::styled("  ✗ ".to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ];
                
                // Split the rest by "FAILED" to color it separately
                if let Some(idx) = parts[1].find("FAILED") {
                    let (name, rest) = parts[1].split_at(idx);
                    spans.push(Span::raw(name.to_string()));
                    spans.push(Span::styled("FAILED".to_string(), Style::default().fg(Color::Red)));
                    spans.push(Span::raw(rest[6..].to_string())); // Skip "FAILED"
                } else {
                    spans.push(Span::raw(parts[1].clone()));
                }
                lines.push(Line::from(spans));
            } else {
                lines.push(Line::from(line_str));
            }
        } else if line_str.starts_with("Running:") {
            // Running test header
            lines.push(Line::from(vec![
                Span::styled("Running: ".to_string(), Style::default().fg(Color::Yellow)),
                Span::styled(line_str[9..].to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
        } else if line_str.starts_with("Running test:") {
            // Single test run header
            lines.push(Line::from(vec![
                Span::styled("Running test: ".to_string(), Style::default().fg(Color::Yellow)),
                Span::styled(line_str[14..].to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
        } else if line_str.starts_with("Running all tests") || line_str.starts_with("Running tests") || line_str.starts_with("🚀 Running") {
            // Batch run header
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            )));
        } else if line_str.starts_with("📁 Running") {
            // Category test run header
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            )));
        } else if line_str.starts_with("Results:") {
            // Results summary
            let mut spans = vec![Span::styled("Results: ".to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))];
            
            // Parse the results line to color passed/failed counts
            let rest = line_str[9..].to_string();
            let parts: Vec<&str> = rest.split(", ").collect();
            
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(", ".to_string()));
                }
                
                if part.contains("passed") {
                    if let Some(num) = part.split_whitespace().next() {
                        spans.push(Span::styled(
                            num.to_string(),
                            if num != "0" { 
                                Style::default().fg(Color::Rgb(0, 200, 0)).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            }
                        ));
                        spans.push(Span::raw(" passed".to_string()));
                    } else {
                        spans.push(Span::raw(part.to_string()));
                    }
                } else if part.contains("failed") {
                    if let Some(num) = part.split_whitespace().next() {
                        spans.push(Span::styled(
                            num.to_string(),
                            if num != "0" { 
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            }
                        ));
                        spans.push(Span::raw(" failed".to_string()));
                    } else {
                        spans.push(Span::raw(part.to_string()));
                    }
                } else {
                    spans.push(Span::raw(part.to_string()));
                }
            }
            
            lines.push(Line::from(spans));
        } else if line_str.starts_with("Total time:") {
            // Total time
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Cyan)
            )));
        } else if line_str.starts_with("✓ Test PASSED") {
            // Single test passed
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Rgb(0, 200, 0)).add_modifier(Modifier::BOLD)
            )));
        } else if line_str.starts_with("✗ Test FAILED") {
            // Single test failed
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            )));
        } else if line_str.starts_with("✗ Error:") {
            // Error message
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Red)
            )));
        } else if line_str.starts_with("Expected:") {
            // Expected output header
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Yellow)
            )));
        } else if line_str.starts_with("Output:") || line_str.starts_with("Actual Output:") {
            // Actual output header
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Cyan)
            )));
        } else if line_str == "=".repeat(60) || line_str == "-".repeat(60) {
            // Separator lines
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::DarkGray)
            )));
        } else if line_str.starts_with("FAILED TESTS:") {
            // Failed tests header
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            )));
        } else if line_str.starts_with("🎉 ALL TESTS PASSED!") {
            // All tests passed with emoji
            lines.push(Line::from(Span::styled(
                line_str.clone(),
                Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)
            )));
        } else if line_str.starts_with("  •") {
            // Bullet point (for failed test list)
            lines.push(Line::from(vec![
                Span::styled("  • ".to_string(), Style::default().fg(Color::Red)),
                Span::raw(line_str[4..].to_string()),
            ]));
        } else {
            // Default - no special coloring
            lines.push(Line::from(line_str));
        }
    }
    
    lines
}