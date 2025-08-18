use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use serde_json::Value;
use std::collections::HashMap;
use crate::tui::app::{TuiApp, FocusedPane};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub value: Option<String>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Object,
    Array,
    Property,
    Value,
    AstNode,
    Symbol,
    Type,
}

impl TreeNode {
    pub fn from_json(value: &Value, label: String, node_type: NodeType) -> Self {
        match value {
            Value::Object(map) => {
                let mut node = TreeNode {
                    label,
                    value: None,
                    children: Vec::new(),
                    expanded: true,
                    node_type,
                    metadata: HashMap::new(),
                };
                
                // Extract special fields for metadata
                if let Some(node_id) = map.get("node_id").and_then(|v| v.as_u64()) {
                    node.metadata.insert("node_id".to_string(), node_id.to_string());
                }
                if let Some(span) = map.get("span").and_then(|v| v.as_object()) {
                    if let (Some(start), Some(end)) = (span.get("start"), span.get("end")) {
                        if let (Some(start_obj), Some(end_obj)) = (start.as_object(), end.as_object()) {
                            let start_line = start_obj.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                            let start_col = start_obj.get("column").and_then(|v| v.as_u64()).unwrap_or(0);
                            let end_line = end_obj.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                            let end_col = end_obj.get("column").and_then(|v| v.as_u64()).unwrap_or(0);
                            node.metadata.insert("location".to_string(), 
                                format!("{}:{}-{}:{}", start_line, start_col, end_line, end_col));
                        }
                    }
                }
                
                // Add children for object properties
                for (key, val) in map {
                    if key != "span" && key != "node_id" && key != "symbol_id" {
                        let child_type = match key.as_str() {
                            "kind" | "decl_type" | "param_type" | "return_type" => NodeType::AstNode,
                            _ => NodeType::Property,
                        };
                        node.children.push(Self::from_json(val, key.clone(), child_type));
                    }
                }
                node
            }
            Value::Array(arr) => {
                let mut node = TreeNode {
                    label: format!("{} [{}]", label, arr.len()),
                    value: None,
                    children: Vec::new(),
                    expanded: true,
                    node_type,
                    metadata: HashMap::new(),
                };
                for (i, val) in arr.iter().enumerate() {
                    node.children.push(Self::from_json(val, format!("[{}]", i), NodeType::AstNode));
                }
                node
            }
            Value::String(s) => TreeNode {
                label,
                value: Some(s.clone()),
                children: Vec::new(),
                expanded: false,
                node_type: NodeType::Value,
                metadata: HashMap::new(),
            },
            Value::Number(n) => TreeNode {
                label,
                value: Some(n.to_string()),
                children: Vec::new(),
                expanded: false,
                node_type: NodeType::Value,
                metadata: HashMap::new(),
            },
            Value::Bool(b) => TreeNode {
                label,
                value: Some(b.to_string()),
                children: Vec::new(),
                expanded: false,
                node_type: NodeType::Value,
                metadata: HashMap::new(),
            },
            Value::Null => TreeNode {
                label,
                value: Some("null".to_string()),
                children: Vec::new(),
                expanded: false,
                node_type: NodeType::Value,
                metadata: HashMap::new(),
            },
        }
    }
    
    pub fn flatten(&self, depth: usize, selected_path: &[usize]) -> Vec<(Vec<Span<'static>>, Vec<usize>)> {
        let mut lines = Vec::new();
        self.flatten_recursive(&mut lines, depth, selected_path, vec![]);
        lines
    }
    
    fn flatten_recursive(&self, lines: &mut Vec<(Vec<Span<'static>>, Vec<usize>)>, depth: usize, _selected_path: &[usize], current_path: Vec<usize>) {
        let mut spans = vec![];
        
        // Indentation
        if depth > 0 {
            spans.push(Span::raw("  ".repeat(depth)));
        }
        
        // Expansion indicator for nodes with children
        if !self.children.is_empty() {
            let indicator = if self.expanded { "▼ " } else { "▶ " };
            spans.push(Span::styled(indicator.to_string(), Style::default().fg(Color::Yellow)));
        } else {
            spans.push(Span::raw("  "));
        }
        
        // Node label with color based on type
        let label_style = match self.node_type {
            NodeType::AstNode => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            NodeType::Symbol => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            NodeType::Type => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            NodeType::Property => Style::default().fg(Color::Blue),
            NodeType::Object | NodeType::Array => Style::default().fg(Color::White),
            NodeType::Value => Style::default().fg(Color::Gray),
        };
        
        spans.push(Span::styled(self.label.clone(), label_style));
        
        // Add value if present
        if let Some(ref val) = self.value {
            spans.push(Span::raw(": "));
            spans.push(Span::styled(val.clone(), Style::default().fg(Color::Green)));
        }
        
        // Add metadata (location, node_id)
        if let Some(location) = self.metadata.get("location") {
            spans.push(Span::styled(
                format!(" @ {}", location),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            ));
        } else if let Some(node_id) = self.metadata.get("node_id") {
            spans.push(Span::styled(
                format!(" #{}", node_id),
                Style::default().fg(Color::DarkGray),
            ));
        }
        
        lines.push((spans, current_path.clone()));
        
        // Add children if expanded
        if self.expanded {
            for (i, child) in self.children.iter().enumerate() {
                let mut child_path = current_path.clone();
                child_path.push(i);
                child.flatten_recursive(lines, depth + 1, _selected_path, child_path);
            }
        }
    }
    
    pub fn toggle_at_path(&mut self, path: &[usize]) -> bool {
        if path.is_empty() {
            self.expanded = !self.expanded;
            return true;
        }
        
        if let Some(&index) = path.first() {
            if let Some(child) = self.children.get_mut(index) {
                return child.toggle_at_path(&path[1..]);
            }
        }
        false
    }
    
    pub fn expand_all(&mut self) {
        self.expanded = true;
        for child in &mut self.children {
            child.expand_all();
        }
    }
    
    pub fn collapse_all(&mut self) {
        self.expanded = false;
        for child in &mut self.children {
            child.collapse_all();
        }
    }
}

pub fn draw_ast_tree(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test_name) = app.get_selected_test_name() {
        let ast_path = app.tools.build_dir.join(format!("{}.pp.ast.json", test_name));
        
        if ast_path.exists() {
            match std::fs::read_to_string(&ast_path) {
                Ok(content) => {
                    match serde_json::from_str::<Value>(&content) {
                        Ok(json) => {
                            let tree = TreeNode::from_json(&json, "AST Root".to_string(), NodeType::AstNode);
                            let lines = tree.flatten(0, &app.ast_selected_path);
                            
                            // Convert to Line objects
                            let mut display_lines: Vec<Line> = Vec::new();
                            let visible_start = app.ast_scroll;
                            let visible_end = (app.ast_scroll + area.height.saturating_sub(2) as usize).min(lines.len());
                            
                            for (_i, (spans, path)) in lines.iter().enumerate().skip(visible_start).take(visible_end - visible_start) {
                                let is_selected = path == &app.ast_selected_path;
                                if is_selected {
                                    let mut styled_spans = vec![];
                                    for span in spans {
                                        styled_spans.push(Span::styled(
                                            span.content.to_string(),
                                            span.style.bg(Color::Rgb(60, 60, 60))
                                        ));
                                    }
                                    display_lines.push(Line::from(styled_spans));
                                } else {
                                    display_lines.push(Line::from(spans.clone()));
                                }
                            }
                            
                            let title = format!(" AST: {}.pp.ast.json [{}] (←→ expand/collapse, ↑↓ navigate) ", 
                                test_name, 
                                if app.ast_expanded { "expanded" } else { "collapsed" }
                            );
                            
                            let paragraph = Paragraph::new(display_lines)
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title(title)
                                        .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 5 {
                                            Style::default().fg(Color::Cyan)
                                        } else {
                                            Style::default().fg(Color::Gray)
                                        })
                                )
                                .wrap(Wrap { trim: false });
                            
                            f.render_widget(paragraph, area);
                        }
                        Err(e) => {
                            render_error(f, area, &format!("Error parsing AST JSON: {}", e), 5, app);
                        }
                    }
                }
                Err(e) => {
                    render_error(f, area, &format!("Error reading AST file: {}", e), 5, app);
                }
            }
        } else {
            render_error(f, area, "AST file not found. Run test with --trace flag first.", 5, app);
        }
    } else {
        render_error(f, area, "No test selected", 5, app);
    }
}

pub fn draw_symbols_table(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test_name) = app.get_selected_test_name() {
        let sem_path = app.tools.build_dir.join(format!("{}.pp.sem.json", test_name));
        
        if sem_path.exists() {
            match std::fs::read_to_string(&sem_path) {
                Ok(content) => {
                    match serde_json::from_str::<Value>(&content) {
                        Ok(json) => {
                            let mut lines: Vec<Line> = Vec::new();
                            
                            // Parse symbols
                            if let Some(symbols) = json.get("symbols").and_then(|v| v.as_array()) {
                                lines.push(Line::from(vec![
                                    Span::styled("═══ SYMBOLS ═══", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                                ]));
                                lines.push(Line::from(""));
                                
                                for symbol in symbols {
                                    if let Some(obj) = symbol.as_object() {
                                        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                                        let sym_type = obj.get("symbol_type").and_then(|v| v.as_str()).unwrap_or("?");
                                        let scope = obj.get("scope_level").and_then(|v| v.as_u64()).unwrap_or(0);
                                        
                                        lines.push(Line::from(vec![
                                            Span::styled("  ", Style::default()),
                                            Span::styled(name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                                            Span::raw(" : "),
                                            Span::styled(sym_type, Style::default().fg(Color::Cyan)),
                                            Span::raw(" "),
                                            Span::styled(format!("[scope {}]", scope), Style::default().fg(Color::DarkGray)),
                                        ]));
                                    }
                                }
                            }
                            
                            // Parse type definitions
                            if let Some(types) = json.get("type_definitions").and_then(|v| v.as_array()) {
                                if !types.is_empty() {
                                    lines.push(Line::from(""));
                                    lines.push(Line::from(vec![
                                        Span::styled("═══ TYPE DEFINITIONS ═══", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                                    ]));
                                    lines.push(Line::from(""));
                                    
                                    for type_def in types {
                                        if let Some(obj) = type_def.as_object() {
                                            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                                            lines.push(Line::from(vec![
                                                Span::raw("  "),
                                                Span::styled(name, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                                            ]));
                                            
                                            // Show type details
                                            if let Some(kind) = obj.get("kind") {
                                                let type_str = format!("{}", kind);
                                                lines.push(Line::from(vec![
                                                    Span::raw("    "),
                                                    Span::styled(type_str, Style::default().fg(Color::Blue)),
                                                ]));
                                            }
                                        }
                                    }
                                }
                            }
                            
                            let paragraph = Paragraph::new(lines)
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title(format!(" Symbols: {}.pp.sem.json ", test_name))
                                        .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 6 {
                                            Style::default().fg(Color::Cyan)
                                        } else {
                                            Style::default().fg(Color::Gray)
                                        })
                                )
                                .scroll((app.symbols_scroll as u16, 0))
                                .wrap(Wrap { trim: false });
                            
                            f.render_widget(paragraph, area);
                        }
                        Err(e) => {
                            render_error(f, area, &format!("Error parsing symbols JSON: {}", e), 6, app);
                        }
                    }
                }
                Err(e) => {
                    render_error(f, area, &format!("Error reading symbols file: {}", e), 6, app);
                }
            }
        } else {
            render_error(f, area, "Symbols file not found. Run test with --trace flag first.", 6, app);
        }
    } else {
        render_error(f, area, "No test selected", 6, app);
    }
}

pub fn draw_typed_ast_tree(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(test_name) = app.get_selected_test_name() {
        let tast_path = app.tools.build_dir.join(format!("{}.pp.tast.json", test_name));
        
        if tast_path.exists() {
            match std::fs::read_to_string(&tast_path) {
                Ok(content) => {
                    match serde_json::from_str::<Value>(&content) {
                        Ok(json) => {
                            let tree = TreeNode::from_json(&json, "Typed AST".to_string(), NodeType::AstNode);
                            let lines = tree.flatten(0, &app.typed_ast_selected_path);
                            
                            // Convert to Line objects with type highlighting
                            let mut display_lines: Vec<Line> = Vec::new();
                            let visible_start = app.typed_ast_scroll;
                            let visible_end = (app.typed_ast_scroll + area.height.saturating_sub(2) as usize).min(lines.len());
                            
                            for (_i, (spans, path)) in lines.iter().enumerate().skip(visible_start).take(visible_end - visible_start) {
                                // Clone spans to make them mutable for highlighting
                                let mut highlighted_spans = spans.clone();
                                
                                // Highlight type information
                                for span in &mut highlighted_spans {
                                    if span.content.contains("type") || span.content.contains("Type") {
                                        *span = Span::styled(
                                            span.content.to_string(),
                                            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                                        );
                                    }
                                }
                                
                                let is_selected = path == &app.typed_ast_selected_path;
                                if is_selected {
                                    let mut styled_spans = vec![];
                                    for span in highlighted_spans {
                                        styled_spans.push(Span::styled(
                                            span.content.to_string(),
                                            span.style.bg(Color::Rgb(60, 60, 60))
                                        ));
                                    }
                                    display_lines.push(Line::from(styled_spans));
                                } else {
                                    display_lines.push(Line::from(highlighted_spans));
                                }
                            }
                            
                            let title = format!(" Typed AST: {}.pp.tast.json (←→ expand/collapse, ↑↓ navigate) ", test_name);
                            
                            let paragraph = Paragraph::new(display_lines)
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title(title)
                                        .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == 7 {
                                            Style::default().fg(Color::Cyan)
                                        } else {
                                            Style::default().fg(Color::Gray)
                                        })
                                )
                                .wrap(Wrap { trim: false });
                            
                            f.render_widget(paragraph, area);
                        }
                        Err(e) => {
                            render_error(f, area, &format!("Error parsing Typed AST JSON: {}", e), 7, app);
                        }
                    }
                }
                Err(e) => {
                    render_error(f, area, &format!("Error reading Typed AST file: {}", e), 7, app);
                }
            }
        } else {
            render_error(f, area, "Typed AST file not found. Run test with --trace flag first.", 7, app);
        }
    } else {
        render_error(f, area, "No test selected", 7, app);
    }
}

fn render_error(f: &mut Frame, area: Rect, message: &str, tab_index: usize, app: &TuiApp) {
    let paragraph = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL).title(" Error ")
            .border_style(if app.focused_pane == FocusedPane::RightPanel && app.selected_tab == tab_index {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Gray)
            }));
    f.render_widget(paragraph, area);
}