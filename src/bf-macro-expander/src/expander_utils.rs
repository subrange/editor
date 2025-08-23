use crate::ast::*;
use crate::expander::{ExpansionContext, MacroExpander};
use crate::source_map::SourceMapBuilder;
use crate::types::*;
use std::collections::HashMap;

impl MacroExpander {
    pub fn expand_expression(&mut self, expr: &ExpressionNode, context: &mut ExpansionContext, generate_source_map: bool) {
        let source_range = PositionRange {
            start: Position {
                line: expr.position().line,
                column: expr.position().column,
                offset: Some(expr.position().start),
            },
            end: Position {
                line: expr.position().line,
                column: expr.position().column + (expr.position().end - expr.position().start),
                offset: Some(expr.position().end),
            },
        };
        
        match expr {
            ExpressionNode::Number(num) => {
                self.append_to_expanded(&num.value.to_string(), context, generate_source_map, Some(source_range));
            }
            ExpressionNode::Identifier(ident) => {
                self.append_to_expanded(&ident.name, context, generate_source_map, Some(source_range));
            }
            ExpressionNode::MacroInvocation(invocation) => {
                self.expand_macro_invocation(invocation, context, generate_source_map, source_range);
            }
            ExpressionNode::BuiltinFunction(builtin) => {
                self.expand_builtin_function(builtin, context, generate_source_map, source_range);
            }
            ExpressionNode::ExpressionList(list) => {
                for item in &list.expressions {
                    self.expand_content(item, context, generate_source_map);
                }
            }
            ExpressionNode::Text(text) => {
                self.append_to_expanded(&text.value, context, generate_source_map, Some(source_range));
            }
            ExpressionNode::BrainfuckCommand(cmd) => {
                self.append_to_expanded(&cmd.commands, context, generate_source_map, Some(source_range));
            }
            ExpressionNode::ArrayLiteral(array) => {
                self.append_to_expanded("{", context, generate_source_map, Some(source_range.clone()));
                for (i, element) in array.elements.iter().enumerate() {
                    if i > 0 {
                        self.append_to_expanded(", ", context, generate_source_map, None);
                    }
                    self.expand_expression(element, context, generate_source_map);
                }
                self.append_to_expanded("}", context, generate_source_map, Some(source_range));
            }
            _ => {}
        }
    }
    
    pub fn expand_expression_to_string(&mut self, expr: &ExpressionNode, context: &ExpansionContext) -> String {
        let mut temp_context = ExpansionContext {
            source_map_builder: SourceMapBuilder::new(),
            current_source_position: context.current_source_position.clone(),
            expansion_depth: context.expansion_depth,
            macro_call_stack: context.macro_call_stack.clone(),
            expanded_lines: vec![String::new()],
            current_expanded_line: 1,
            current_expanded_column: 1,
        };
        
        self.expand_expression(expr, &mut temp_context, false);
        temp_context.expanded_lines.join("\n").trim().to_string()
    }
    
    pub fn append_to_expanded(&mut self, text: &str, context: &mut ExpansionContext, generate_source_map: bool, source_range: Option<PositionRange>) {
        if text.is_empty() {
            return;
        }
        
        let start_line = context.current_expanded_line;
        let start_column = context.current_expanded_column;
        
        let lines: Vec<&str> = text.split('\n').collect();
        
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                // Not the first line, so we had a newline
                context.expanded_lines.push(String::new());
                context.current_expanded_line += 1;
                context.current_expanded_column = 1;
            }
            
            if !line.is_empty() {
                let current_line_index = context.current_expanded_line - 1;
                if current_line_index >= context.expanded_lines.len() {
                    context.expanded_lines.push(String::new());
                }
                
                context.expanded_lines[current_line_index].push_str(line);
                context.current_expanded_column += line.len();
            }
        }
        
        // Create source map entry if we have a source range
        if generate_source_map && source_range.is_some() {
            let source_range = source_range.unwrap();
            let expanded_range = PositionRange {
                start: Position {
                    line: start_line,
                    column: start_column,
                    offset: None,
                },
                end: Position {
                    line: context.current_expanded_line,
                    column: context.current_expanded_column,
                    offset: None,
                },
            };
            
            let macro_context = context.macro_call_stack.last();
            
            let entry = SourceMapEntry {
                expanded_range,
                source_range,
                expansion_depth: context.expansion_depth,
                macro_name: macro_context.map(|m| m.macro_name.clone()),
                macro_call_site: macro_context.map(|m| m.call_site.clone()),
                parameter_values: macro_context.and_then(|m| m.parameters.clone()),
                macro_call_stack: if context.macro_call_stack.is_empty() {
                    None
                } else {
                    Some(context.macro_call_stack.clone())
                },
            };
            
            context.source_map_builder.add_mapping(entry);
        }
    }
    
    pub fn create_invocation_signature(&self, node: &MacroInvocationNode) -> String {
        let mut signature = node.name.clone();
        
        if let Some(args) = &node.arguments {
            if !args.is_empty() {
                let args_to_include = args.len().min(3);
                let mut arg_signatures = Vec::new();
                
                for i in 0..args_to_include {
                    let arg = &args[i];
                    let arg_sig = match arg {
                        ExpressionNode::Number(num) => num.value.to_string(),
                        ExpressionNode::Identifier(ident) => ident.name.clone(),
                        ExpressionNode::Text(text) => {
                            if text.value.len() < 10 {
                                text.value.clone()
                            } else {
                                format!("{}…", &text.value[..8])
                            }
                        }
                        ExpressionNode::BrainfuckCommand(cmd) => {
                            if cmd.commands.len() < 10 {
                                cmd.commands.clone()
                            } else {
                                format!("{}…", &cmd.commands[..8])
                            }
                        }
                        ExpressionNode::MacroInvocation(inv) => format!("@{}", inv.name),
                        _ => format!("<{:?}>", arg),
                    };
                    arg_signatures.push(arg_sig);
                }
                
                if args.len() > args_to_include {
                    arg_signatures.push("...".to_string());
                }
                
                signature.push_str(&format!("({})", arg_signatures.join(", ")));
            }
        }
        
        signature
    }
    
    pub fn node_to_string(&self, nodes: &[ContentNode]) -> String {
        let mut result = String::new();
        
        for node in nodes {
            match node {
                ContentNode::BrainfuckCommand(cmd) => result.push_str(&cmd.commands),
                ContentNode::Text(text) => result.push_str(&text.value),
                ContentNode::MacroInvocation(inv) => {
                    result.push('@');
                    result.push_str(&inv.name);
                    if let Some(args) = &inv.arguments {
                        result.push('(');
                        result.push_str(&self.expressions_to_string(args));
                        result.push(')');
                    }
                }
                ContentNode::BuiltinFunction(builtin) => {
                    result.push('{');
                    result.push_str(builtin.name.to_string());
                    result.push('(');
                    result.push_str(&self.expressions_to_string(&builtin.arguments));
                    result.push_str(")}");
                }
            }
        }
        
        result
    }
    
    fn expressions_to_string(&self, expressions: &[ExpressionNode]) -> String {
        expressions.iter()
            .map(|expr| self.expression_to_string(expr))
            .collect::<Vec<_>>()
            .join(", ")
    }
    
    pub fn expression_to_string(&self, expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Number(num) => num.value.to_string(),
            ExpressionNode::Identifier(ident) => ident.name.clone(),
            ExpressionNode::Text(text) => text.value.clone(),
            ExpressionNode::BrainfuckCommand(cmd) => cmd.commands.clone(),
            ExpressionNode::MacroInvocation(inv) => {
                let mut result = format!("@{}", inv.name);
                if let Some(args) = &inv.arguments {
                    result.push('(');
                    result.push_str(&self.expressions_to_string(args));
                    result.push(')');
                }
                result
            }
            ExpressionNode::BuiltinFunction(builtin) => {
                format!("{{{}({})}}", 
                    builtin.name.to_string(),
                    self.expressions_to_string(&builtin.arguments))
            }
            ExpressionNode::ArrayLiteral(array) => {
                format!("{{{}}}", self.expressions_to_string(&array.elements))
            }
            ExpressionNode::ExpressionList(list) => {
                self.node_to_string(&list.expressions)
            }
            _ => String::new()
        }
    }
    
    pub fn collapse_empty_lines(&self, code: &str) -> String {
        let lines: Vec<&str> = code.split('\n').collect();
        let non_empty_lines: Vec<&str> = lines.into_iter()
            .filter(|line| {
                // Keep lines with brainfuck commands
                let has_bf_commands = line.chars().any(|c| matches!(c, '>' | '<' | '+' | '-' | '.' | '[' | ']' | '$'));
                // Also keep any line that isn't just whitespace (for preserved content)
                let has_content = !line.trim().is_empty();
                has_bf_commands || has_content
            })
            .collect();
        
        non_empty_lines.join("\n")
    }
    
    pub fn collapse_empty_lines_with_source_map(&self, code: &str, source_map: &SourceMap) -> (String, SourceMap) {
        let lines: Vec<&str> = code.split('\n').collect();
        let mut non_empty_lines = Vec::new();
        let mut line_mapping = HashMap::new();
        
        let mut new_line_index = 0;
        for (old_line_index, line) in lines.iter().enumerate() {
            // Keep lines with brainfuck commands OR any non-empty content (for preserved content)
            let has_bf_commands = line.chars().any(|c| matches!(c, '>' | '<' | '+' | '-' | '.' | '[' | ']' | '$'));
            let has_content = !line.trim().is_empty();
            
            if has_bf_commands || has_content {
                non_empty_lines.push(*line);
                line_mapping.insert(old_line_index + 1, new_line_index + 1); // Convert to 1-based
                new_line_index += 1;
            }
        }
        
        // Create a new source map with updated line numbers
        let mut new_source_map_builder = SourceMapBuilder::new();
        
        for entry in &source_map.entries {
            if let Some(&new_expanded_line) = line_mapping.get(&entry.expanded_range.start.line) {
                let new_entry = SourceMapEntry {
                    source_range: entry.source_range.clone(),
                    expanded_range: PositionRange {
                        start: Position {
                            line: new_expanded_line,
                            column: entry.expanded_range.start.column,
                            offset: None,
                        },
                        end: Position {
                            line: new_expanded_line,
                            column: entry.expanded_range.end.column,
                            offset: None,
                        },
                    },
                    macro_name: entry.macro_name.clone(),
                    parameter_values: entry.parameter_values.clone(),
                    expansion_depth: entry.expansion_depth,
                    macro_call_site: entry.macro_call_site.clone(),
                    macro_call_stack: entry.macro_call_stack.clone(),
                };
                
                new_source_map_builder.add_mapping(new_entry);
            }
        }
        
        (non_empty_lines.join("\n"), new_source_map_builder.build())
    }
}

// Helper trait to get position from various node types
impl ContentNode {
    pub fn position(&self) -> &ASTPosition {
        match self {
            ContentNode::BrainfuckCommand(node) => &node.position,
            ContentNode::MacroInvocation(node) => &node.position,
            ContentNode::BuiltinFunction(node) => &node.position,
            ContentNode::Text(node) => &node.position,
        }
    }
}

impl ExpressionNode {
    pub fn position(&self) -> &ASTPosition {
        match self {
            ExpressionNode::Number(node) => &node.position,
            ExpressionNode::Identifier(node) => &node.position,
            ExpressionNode::MacroInvocation(node) => &node.position,
            ExpressionNode::BuiltinFunction(node) => &node.position,
            ExpressionNode::ExpressionList(node) => &node.position,
            ExpressionNode::Text(node) => &node.position,
            ExpressionNode::BrainfuckCommand(node) => &node.position,
            ExpressionNode::ArrayLiteral(node) => &node.position,
            ExpressionNode::TuplePattern(node) => &node.position,
            ExpressionNode::ForPattern(node) => &node.position,
        }
    }
}

impl StatementNode {
    pub fn position(&self) -> &ASTPosition {
        match self {
            StatementNode::MacroDefinition(node) => &node.position,
            StatementNode::CodeLine(node) => &node.position,
        }
    }
}

impl BuiltinFunction {
    pub fn to_string(&self) -> &str {
        match self {
            BuiltinFunction::Repeat => "repeat",
            BuiltinFunction::If => "if",
            BuiltinFunction::For => "for",
            BuiltinFunction::Reverse => "reverse",
            BuiltinFunction::Preserve => "preserve",
            BuiltinFunction::Label => "label",
            BuiltinFunction::Br => "br",
        }
    }
}