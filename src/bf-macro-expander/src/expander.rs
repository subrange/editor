use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::source_map::SourceMapBuilder;
use crate::types::*;
use std::collections::{HashMap, HashSet};

// Helper function to safely calculate position length
fn safe_position_length(start: usize, end: usize) -> usize {
    if end >= start {
        end - start
    } else {
        // If positions are swapped, return the absolute difference
        start - end
    }
}

// Helper function to replace whole words without lookahead/lookbehind
pub(crate) fn replace_whole_word(text: &str, word: &str, replacement: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let word_chars: Vec<char> = word.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        // Check if we have a potential match
        if i + word_chars.len() <= chars.len() {
            let mut matches = true;
            for j in 0..word_chars.len() {
                if chars[i + j] != word_chars[j] {
                    matches = false;
                    break;
                }
            }
            
            if matches {
                // Check word boundaries
                let at_start = i == 0 || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_';
                let at_end = i + word_chars.len() == chars.len() || 
                    !chars[i + word_chars.len()].is_alphanumeric() && chars[i + word_chars.len()] != '_';
                
                if at_start && at_end {
                    // Replace the word
                    result.push_str(replacement);
                    i += word_chars.len();
                    continue;
                }
            }
        }
        
        result.push(chars[i]);
        i += 1;
    }
    
    result
}

pub struct ExpansionContext {
    pub source_map_builder: SourceMapBuilder,
    pub current_source_position: Position,
    pub expansion_depth: usize,
    pub macro_call_stack: Vec<MacroCallStackEntry>,
    pub expanded_lines: Vec<String>,
    pub current_expanded_line: usize,
    pub current_expanded_column: usize,
}

pub struct MacroExpander {
    pub(crate) macros: HashMap<String, MacroDefinitionNode>,
    pub(crate) errors: Vec<MacroExpansionError>,
    pub(crate) tokens: Vec<MacroToken>,
    pub(crate) expansion_chain: HashSet<String>,
    pub max_expansion_depth: usize,
    pub(crate) input: String,
    pub(crate) enable_circular_dependency_detection: bool,
    pub(crate) label_counter: usize,
    pub(crate) label_map: HashMap<String, String>,
}

impl MacroExpander {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            errors: Vec::new(),
            tokens: Vec::new(),
            expansion_chain: HashSet::new(),
            max_expansion_depth: 100,
            input: String::new(),
            enable_circular_dependency_detection: false,
            label_counter: 0,
            label_map: HashMap::new(),
        }
    }
    
    pub fn expand(&mut self, input: &str, options: MacroExpanderOptions) -> MacroExpanderResult {
        // Reset state
        self.macros.clear();
        self.errors.clear();
        self.tokens.clear();
        self.input = input.to_string();
        self.expansion_chain.clear();
        self.enable_circular_dependency_detection = options.enable_circular_dependency_detection;
        self.label_counter = 0;  // Reset label counter for each expansion
        self.label_map.clear();   // Clear label map as well
        
        // Tokenize and parse
        let mut lexer = Lexer::new(input, !options.strip_comments);
        let tokens = lexer.tokenize();
        
        let mut parser = Parser::new(tokens);
        let parse_result = parser.parse();
        
        self.errors.extend(parse_result.errors);
        self.tokens.extend(parse_result.tokens);
        
        // Collect macro definitions
        self.collect_macro_definitions(&parse_result.ast);
        self.validate_all_macros();
        
        let definition_errors = self.errors.clone();
        self.errors.clear();
        
        // Create expansion context
        let mut context = ExpansionContext {
            source_map_builder: SourceMapBuilder::new(),
            current_source_position: Position { line: 1, column: 1, offset: Some(0) },
            expansion_depth: 0,
            macro_call_stack: Vec::new(),
            expanded_lines: vec![String::new()],
            current_expanded_line: 1,
            current_expanded_column: 1,
        };
        
        // Expand the AST
        self.expand_program(&parse_result.ast, &mut context, options.generate_source_map);
        
        self.errors = [definition_errors, self.errors.clone()].concat();
        
        // Join expanded lines
        let mut expanded = context.expanded_lines.join("\n");
        // Trim trailing whitespace
        expanded = expanded.trim_end().to_string();
        
        // Build source map before post-processing
        let mut source_map = None;
        if options.generate_source_map {
            source_map = Some(context.source_map_builder.build());
        }
        
        // Post-process
        if options.collapse_empty_lines {
            if let Some(map) = source_map.as_mut() {
                let result = self.collapse_empty_lines_with_source_map(&expanded, map);
                expanded = result.0;
                *map = result.1;
            } else {
                expanded = self.collapse_empty_lines(&expanded);
            }
        }
        
        // Convert macro definitions to the expected format
        let macro_definitions: Vec<MacroDefinition> = self.macros.values().map(|node| {
            MacroDefinition {
                name: node.name.clone(),
                parameters: node.parameters.clone(),
                body: self.node_to_string(&node.body),
                source_location: SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: safe_position_length(node.position.start, node.position.end),
                },
            }
        }).collect();
        
        MacroExpanderResult {
            expanded,
            errors: self.errors.clone(),
            tokens: self.tokens.clone(),
            macros: macro_definitions,
            source_map,
        }
    }
    
    fn collect_macro_definitions(&mut self, ast: &ProgramNode) {
        for statement in &ast.statements {
            if let StatementNode::MacroDefinition(def) = statement {
                if self.macros.contains_key(&def.name) {
                    self.errors.push(MacroExpansionError {
                        error_type: MacroExpansionErrorType::SyntaxError,
                        message: format!("Duplicate macro definition: '{}'", def.name),
                        location: Some(SourceLocation {
                            line: def.position.line.saturating_sub(1),
                            column: def.position.column.saturating_sub(1),
                            length: safe_position_length(def.position.start, def.position.end),
                        }),
                    });
                } else {
                    self.macros.insert(def.name.clone(), def.clone());
                }
            }
        }
    }
    
    fn validate_all_macros(&mut self) {
        let macros_clone = self.macros.clone();
        for macro_def in macros_clone.values() {
            self.validate_macro_definition(macro_def);
        }
    }
    
    fn validate_macro_definition(&mut self, macro_def: &MacroDefinitionNode) {
        let param_set: HashSet<String> = macro_def.parameters.as_ref()
            .map(|p| p.iter().cloned().collect())
            .unwrap_or_default();
        
        self.validate_nodes(&macro_def.body, &param_set, &macro_def.position);
    }
    
    fn validate_nodes(&mut self, nodes: &[BodyNode], valid_params: &HashSet<String>, _macro_position: &ASTPosition) {
        for node in nodes {
            match node {
                ContentNode::MacroInvocation(invocation) => {
                    if !self.macros.contains_key(&invocation.name) {
                        self.errors.push(MacroExpansionError {
                            error_type: MacroExpansionErrorType::Undefined,
                            message: format!("Macro '{}' is not defined", invocation.name),
                            location: Some(SourceLocation {
                                line: invocation.position.line.saturating_sub(1),
                                column: invocation.position.column.saturating_sub(1),
                                length: safe_position_length(invocation.position.start, invocation.position.end),
                            }),
                        });
                    }
                    if let Some(args) = &invocation.arguments {
                        for arg in args {
                            self.validate_expression(arg, valid_params);
                        }
                    }
                }
                ContentNode::BuiltinFunction(builtin) => {
                    for arg in &builtin.arguments {
                        self.validate_expression(arg, valid_params);
                    }
                }
                _ => {}
            }
        }
    }
    
    fn validate_expression(&mut self, expr: &ExpressionNode, valid_params: &HashSet<String>) {
        match expr {
            ExpressionNode::MacroInvocation(invocation) => {
                if !self.macros.contains_key(&invocation.name) {
                    self.errors.push(MacroExpansionError {
                        error_type: MacroExpansionErrorType::Undefined,
                        message: format!("Macro '{}' is not defined", invocation.name),
                        location: Some(SourceLocation {
                            line: invocation.position.line.saturating_sub(1),
                            column: invocation.position.column.saturating_sub(1),
                            length: safe_position_length(invocation.position.start, invocation.position.end),
                        }),
                    });
                }
            }
            ExpressionNode::BuiltinFunction(builtin) => {
                if builtin.name == BuiltinFunction::Repeat && builtin.arguments.len() == 2 {
                    if let ExpressionNode::Text(text) = &builtin.arguments[0] {
                        let value = &text.value;
                        let might_be_loop_var = value.len() == 1 && value.chars().next().unwrap().is_alphabetic();
                        
                        let is_valid_number = if value.starts_with("'") && value.ends_with("'") && value.len() == 3 {
                            true // Character literal
                        } else if value.starts_with("0x") || value.starts_with("0X") {
                            value[2..].parse::<i64>().is_ok()
                        } else {
                            value.parse::<i64>().is_ok()
                        };
                        
                        if !valid_params.contains(value) && !might_be_loop_var && !is_valid_number {
                            self.errors.push(MacroExpansionError {
                                error_type: MacroExpansionErrorType::SyntaxError,
                                message: format!("Invalid repeat count: {}", value),
                                location: Some(SourceLocation {
                                    line: builtin.position.line.saturating_sub(1),
                                    column: builtin.position.column.saturating_sub(1),
                                    length: safe_position_length(builtin.position.start, builtin.position.end),
                                }),
                            });
                        }
                    } else if let ExpressionNode::Identifier(ident) = &builtin.arguments[0] {
                        if !valid_params.contains(&ident.name) {
                            self.errors.push(MacroExpansionError {
                                error_type: MacroExpansionErrorType::SyntaxError,
                                message: format!("Undefined parameter: {}", ident.name),
                                location: Some(SourceLocation {
                                    line: builtin.position.line.saturating_sub(1),
                                    column: builtin.position.column.saturating_sub(1),
                                    length: safe_position_length(builtin.position.start, builtin.position.end),
                                }),
                            });
                        }
                    }
                }
                for arg in &builtin.arguments {
                    self.validate_expression(arg, valid_params);
                }
            }
            ExpressionNode::ExpressionList(list) => {
                for item in &list.expressions {
                    if let ContentNode::MacroInvocation(inv) = item {
                        self.validate_expression(&ExpressionNode::MacroInvocation(inv.clone()), valid_params);
                    } else if let ContentNode::BuiltinFunction(builtin) = item {
                        self.validate_expression(&ExpressionNode::BuiltinFunction(builtin.clone()), valid_params);
                    }
                }
            }
            _ => {}
        }
    }
    
    fn expand_program(&mut self, ast: &ProgramNode, context: &mut ExpansionContext, generate_source_map: bool) {
        for statement in &ast.statements {
            context.current_source_position = Position {
                line: statement.position().line,
                column: statement.position().column,
                offset: Some(statement.position().start),
            };
            
            match statement {
                StatementNode::CodeLine(code_line) => {
                    self.expand_code_line(code_line, context, generate_source_map);
                    self.append_to_expanded("\n", context, generate_source_map, None);
                }
                StatementNode::MacroDefinition(macro_def) => {
                    // Macro definitions are replaced with empty lines
                    // Ensure start and end are valid
                    let raw_start = macro_def.position.start;
                    let raw_end = macro_def.position.end;
                    
                    // Fix swapped positions if necessary
                    let (start, end) = if raw_start <= raw_end {
                        (raw_start, raw_end)
                    } else {
                        (raw_end, raw_start)
                    };
                    
                    // Clamp to input bounds
                    let start = start.min(self.input.len());
                    let end = end.min(self.input.len()).max(start);
                    let source_text = &self.input[start..end];
                    let line_count = source_text.matches('\n').count() + 1;
                    let end_line = macro_def.position.line + line_count - 1;
                    
                    for line in macro_def.position.line..=end_line {
                        if generate_source_map {
                            let source_range = PositionRange {
                                start: Position { line, column: 1, offset: None },
                                end: Position { line, column: 1000, offset: None },
                            };
                            self.append_to_expanded("", context, generate_source_map, Some(source_range));
                        }
                    }
                    self.append_to_expanded("\n", context, generate_source_map, None);
                }
            }
        }
    }
    
    fn expand_code_line(&mut self, node: &CodeLineNode, context: &mut ExpansionContext, generate_source_map: bool) {
        let line_expanded_start = Position {
            line: context.current_expanded_line,
            column: context.current_expanded_column,
            offset: None,
        };
        
        for content in &node.content {
            context.current_source_position = Position {
                line: content.position().line,
                column: content.position().column,
                offset: Some(content.position().start),
            };
            self.expand_content(content, context, generate_source_map);
        }
        
        // Create minimal source map entry if needed
        if generate_source_map && !node.content.is_empty() &&
           context.current_expanded_line == line_expanded_start.line &&
           context.current_expanded_column == line_expanded_start.column {
            
            if let Some(first_content) = node.content.iter().find(|c| {
                if let ContentNode::Text(t) = c {
                    !t.value.trim().is_empty()
                } else {
                    true
                }
            }) {
                let entry = SourceMapEntry {
                    expanded_range: PositionRange {
                        start: line_expanded_start.clone(),
                        end: line_expanded_start.clone(),
                    },
                    source_range: PositionRange {
                        start: Position {
                            line: first_content.position().line,
                            column: first_content.position().column,
                            offset: Some(first_content.position().start),
                        },
                        end: Position {
                            line: first_content.position().line,
                            column: first_content.position().column + 1,
                            offset: Some(first_content.position().start + 1),
                        },
                    },
                    expansion_depth: context.expansion_depth,
                    macro_name: context.macro_call_stack.last().map(|m| m.macro_name.clone()),
                    macro_call_site: context.macro_call_stack.last().map(|m| m.call_site.clone()),
                    parameter_values: None,
                    macro_call_stack: if context.macro_call_stack.is_empty() {
                        None
                    } else {
                        Some(context.macro_call_stack.clone())
                    },
                };
                context.source_map_builder.add_mapping(entry);
            }
        }
    }
    
    pub(crate) fn expand_content(&mut self, node: &ContentNode, context: &mut ExpansionContext, generate_source_map: bool) {
        let current_macro = context.macro_call_stack.last();
        let source_range = current_macro.as_ref().map(|m| m.call_site.clone()).unwrap_or_else(|| {
            PositionRange {
                start: Position {
                    line: node.position().line,
                    column: node.position().column,
                    offset: Some(node.position().start),
                },
                end: Position {
                    line: node.position().line,
                    column: node.position().column + (node.position().end - node.position().start),
                    offset: Some(node.position().end),
                },
            }
        });
        
        self.expand_content_with_source_range(node, context, generate_source_map, source_range);
    }
    
    fn expand_content_with_source_range(&mut self, node: &ContentNode, context: &mut ExpansionContext, generate_source_map: bool, source_range: PositionRange) {
        match node {
            ContentNode::BrainfuckCommand(cmd) => {
                self.append_to_expanded(&cmd.commands, context, generate_source_map, Some(source_range));
            }
            ContentNode::Text(text) => {
                self.append_to_expanded(&text.value, context, generate_source_map, Some(source_range));
            }
            ContentNode::MacroInvocation(invocation) => {
                self.expand_macro_invocation(invocation, context, generate_source_map, source_range);
            }
            ContentNode::BuiltinFunction(builtin) => {
                self.expand_builtin_function(builtin, context, generate_source_map, source_range);
            }
        }
    }
    
    pub(crate) fn expand_macro_invocation(&mut self, node: &MacroInvocationNode, context: &mut ExpansionContext, generate_source_map: bool, source_range: PositionRange) {
        let macro_def = self.macros.get(&node.name).cloned();
        
        if macro_def.is_none() {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::Undefined,
                message: format!("Macro '{}' is not defined", node.name),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: safe_position_length(node.position.start, node.position.end),
                }),
            });
            self.append_to_expanded(&self.node_to_string(&[ContentNode::MacroInvocation(node.clone())]), context, generate_source_map, Some(source_range));
            return;
        }
        
        let macro_def = macro_def.unwrap();
        let invocation_signature = self.create_invocation_signature(node);
        
        if self.enable_circular_dependency_detection && self.expansion_chain.contains(&invocation_signature) {
            let chain = self.expansion_chain.iter().cloned().collect::<Vec<_>>().join(" → ");
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::CircularDependency,
                message: format!("Circular macro dependency detected: {} → {}", chain, invocation_signature),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: safe_position_length(node.position.start, node.position.end),
                }),
            });
            self.append_to_expanded(&format!("@{}", node.name), context, generate_source_map, Some(source_range));
            return;
        }
        
        context.expansion_depth += 1;
        if context.expansion_depth > self.max_expansion_depth {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::SyntaxError,
                message: "Maximum macro expansion depth exceeded".to_string(),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: safe_position_length(node.position.start, node.position.end),
                }),
            });
            self.append_to_expanded(&format!("@{}", node.name), context, generate_source_map, Some(source_range));
            return;
        }
        
        if self.enable_circular_dependency_detection {
            self.expansion_chain.insert(invocation_signature.clone());
        }
        
        // Check parameter count
        let expected_params = macro_def.parameters.as_ref().map(|p| p.len()).unwrap_or(0);
        let provided_args = node.arguments.as_ref().map(|a| a.len()).unwrap_or(0);
        
        if expected_params != provided_args {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::ParameterMismatch,
                message: format!("Macro '{}' expects {} parameter(s), got {}", node.name, expected_params, provided_args),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: safe_position_length(node.position.start, node.position.end),
                }),
            });
            if self.enable_circular_dependency_detection {
                self.expansion_chain.remove(&invocation_signature);
            }
            context.expansion_depth -= 1;
            return;
        }
        
        // Prepare parameter substitutions
        let mut parameter_values = None;
        if let (Some(params), Some(args)) = (&macro_def.parameters, &node.arguments) {
            let mut values = HashMap::new();
            for (param, arg) in params.iter().zip(args.iter()) {
                values.insert(param.clone(), self.expand_expression_to_string(arg, context));
            }
            parameter_values = Some(values);
        }
        
        // Clear label map for this macro invocation to ensure unique labels
        self.label_map.clear();
        
        // Push macro context
        context.macro_call_stack.push(MacroCallStackEntry {
            macro_name: node.name.clone(),
            call_site: source_range.clone(),
            parameters: parameter_values.clone(),
        });
        
        // Expand macro body
        if let Some(params) = parameter_values {
            self.expand_body_nodes_with_substitutions(&macro_def.body, &params, context, generate_source_map);
        } else {
            self.expand_body_nodes(&macro_def.body, context, generate_source_map);
        }
        
        // Pop macro context
        context.macro_call_stack.pop();
        context.expansion_depth -= 1;
        if self.enable_circular_dependency_detection {
            self.expansion_chain.remove(&invocation_signature);
        }
    }
    
    fn expand_body_nodes(&mut self, nodes: &[BodyNode], context: &mut ExpansionContext, generate_source_map: bool) {
        for node in nodes {
            let node_source_range = PositionRange {
                start: Position {
                    line: node.position().line,
                    column: node.position().column,
                    offset: Some(node.position().start),
                },
                end: Position {
                    line: node.position().line,
                    column: node.position().column + (node.position().end - node.position().start),
                    offset: Some(node.position().end),
                },
            };
            
            context.current_source_position = Position {
                line: node.position().line,
                column: node.position().column,
                offset: Some(node.position().start),
            };
            
            self.expand_content_with_source_range(node, context, generate_source_map, node_source_range);
        }
    }
    
    fn expand_body_nodes_with_substitutions(&mut self, nodes: &[BodyNode], substitutions: &HashMap<String, String>, context: &mut ExpansionContext, generate_source_map: bool) {
        for node in nodes {
            match node {
                ContentNode::Text(text_node) => {
                    let mut text = text_node.value.clone();
                    let mut sorted_subs: Vec<_> = substitutions.iter().collect();
                    sorted_subs.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
                    
                    for (param, value) in sorted_subs {
                        // Use a simpler approach without lookahead
                        text = replace_whole_word(&text, param, value);
                    }
                    
                    let source_range = PositionRange {
                        start: Position {
                            line: text_node.position.line,
                            column: text_node.position.column,
                            offset: Some(text_node.position.start),
                        },
                        end: Position {
                            line: text_node.position.line,
                            column: text_node.position.column + safe_position_length(text_node.position.start, text_node.position.end),
                            offset: Some(text_node.position.end),
                        },
                    };
                    
                    self.append_to_expanded(&text, context, generate_source_map, Some(source_range));
                }
                ContentNode::BuiltinFunction(builtin) => {
                    let modified_args = builtin.arguments.iter()
                        .map(|arg| self.substitute_in_expression(arg, substitutions))
                        .collect();
                    
                    let modified_node = BuiltinFunctionNode {
                        name: builtin.name.clone(),
                        arguments: modified_args,
                        position: builtin.position.clone(),
                    };
                    
                    let source_range = PositionRange {
                        start: Position {
                            line: builtin.position.line,
                            column: builtin.position.column,
                            offset: Some(builtin.position.start),
                        },
                        end: Position {
                            line: builtin.position.line,
                            column: builtin.position.column + safe_position_length(builtin.position.start, builtin.position.end),
                            offset: Some(builtin.position.end),
                        },
                    };
                    
                    self.expand_builtin_function(&modified_node, context, generate_source_map, source_range);
                }
                ContentNode::MacroInvocation(invocation) => {
                    let modified_args = invocation.arguments.as_ref().map(|args| {
                        args.iter()
                            .map(|arg| self.substitute_in_expression(arg, substitutions))
                            .collect()
                    });
                    
                    let modified_node = MacroInvocationNode {
                        name: invocation.name.clone(),
                        arguments: modified_args,
                        position: invocation.position.clone(),
                    };
                    
                    let source_range = PositionRange {
                        start: Position {
                            line: invocation.position.line,
                            column: invocation.position.column,
                            offset: Some(invocation.position.start),
                        },
                        end: Position {
                            line: invocation.position.line,
                            column: invocation.position.column + safe_position_length(invocation.position.start, invocation.position.end),
                            offset: Some(invocation.position.end),
                        },
                    };
                    
                    self.expand_macro_invocation(&modified_node, context, generate_source_map, source_range);
                }
                ContentNode::BrainfuckCommand(_) => {
                    let source_range = PositionRange {
                        start: Position {
                            line: node.position().line,
                            column: node.position().column,
                            offset: Some(node.position().start),
                        },
                        end: Position {
                            line: node.position().line,
                            column: node.position().column + (node.position().end - node.position().start),
                            offset: Some(node.position().end),
                        },
                    };
                    self.expand_content_with_source_range(node, context, generate_source_map, source_range);
                }
            }
        }
    }
    
    // Part 2 continues in next message...
}