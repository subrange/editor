use crate::ast::*;
use crate::expander::{ExpansionContext, MacroExpander, replace_whole_word};
use crate::types::*;
use std::collections::HashMap;

impl MacroExpander {
    pub fn substitute_in_expression(&self, expr: &ExpressionNode, substitutions: &HashMap<String, String>) -> ExpressionNode {
        match expr {
            ExpressionNode::Identifier(ident) => {
                if let Some(value) = substitutions.get(&ident.name) {
                    let trimmed = value.trim();
                    if let Ok(num) = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                        i64::from_str_radix(&trimmed[2..], 16)
                    } else {
                        trimmed.parse()
                    } {
                        return ExpressionNode::Number(NumberNode {
                            value: num,
                            position: expr.position().clone(),
                        });
                    }
                    return ExpressionNode::Text(TextNode {
                        value: value.clone(),
                        position: expr.position().clone(),
                    });
                }
                expr.clone()
            }
            ExpressionNode::Text(text_node) => {
                let text = &text_node.value;
                
                if let Some(value) = substitutions.get(text) {
                    let trimmed = value.trim();
                    if let Ok(num) = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                        i64::from_str_radix(&trimmed[2..], 16)
                    } else {
                        trimmed.parse()
                    } {
                        return ExpressionNode::Number(NumberNode {
                            value: num,
                            position: text_node.position.clone(),
                        });
                    }
                    return ExpressionNode::Text(TextNode {
                        value: value.clone(),
                        position: text_node.position.clone(),
                    });
                }
                
                let mut result_text = text.clone();
                let mut sorted_subs: Vec<_> = substitutions.iter().collect();
                sorted_subs.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
                
                for (param, value) in sorted_subs {
                    result_text = replace_whole_word(&result_text, param, value);
                }
                
                if result_text != text_node.value {
                    ExpressionNode::Text(TextNode {
                        value: result_text,
                        position: text_node.position.clone(),
                    })
                } else {
                    expr.clone()
                }
            }
            ExpressionNode::ExpressionList(list) => {
                let expressions = list.expressions.iter().map(|item| {
                    match item {
                        ContentNode::MacroInvocation(inv) => {
                            let substituted = self.substitute_in_expression(
                                &ExpressionNode::MacroInvocation(inv.clone()),
                                substitutions
                            );
                            if let ExpressionNode::MacroInvocation(new_inv) = substituted {
                                ContentNode::MacroInvocation(new_inv)
                            } else {
                                item.clone()
                            }
                        }
                        ContentNode::BuiltinFunction(builtin) => {
                            let substituted = self.substitute_in_expression(
                                &ExpressionNode::BuiltinFunction(builtin.clone()),
                                substitutions
                            );
                            if let ExpressionNode::BuiltinFunction(new_builtin) = substituted {
                                ContentNode::BuiltinFunction(new_builtin)
                            } else {
                                item.clone()
                            }
                        }
                        ContentNode::Text(text) => {
                            let substituted = self.substitute_in_expression(
                                &ExpressionNode::Text(text.clone()),
                                substitutions
                            );
                            if let ExpressionNode::Text(new_text) = substituted {
                                ContentNode::Text(new_text)
                            } else {
                                item.clone()
                            }
                        }
                        _ => item.clone()
                    }
                }).collect();
                
                ExpressionNode::ExpressionList(ExpressionListNode {
                    expressions,
                    position: list.position.clone(),
                })
            }
            ExpressionNode::BuiltinFunction(builtin) => {
                let arguments = builtin.arguments.iter()
                    .map(|arg| self.substitute_in_expression(arg, substitutions))
                    .collect();
                ExpressionNode::BuiltinFunction(BuiltinFunctionNode {
                    name: builtin.name.clone(),
                    arguments,
                    position: builtin.position.clone(),
                })
            }
            ExpressionNode::MacroInvocation(invocation) => {
                let arguments = invocation.arguments.as_ref().map(|args| {
                    args.iter()
                        .map(|arg| self.substitute_in_expression(arg, substitutions))
                        .collect()
                });
                ExpressionNode::MacroInvocation(MacroInvocationNode {
                    name: invocation.name.clone(),
                    arguments,
                    position: invocation.position.clone(),
                })
            }
            ExpressionNode::ArrayLiteral(array) => {
                let elements = array.elements.iter()
                    .map(|el| self.substitute_in_expression(el, substitutions))
                    .collect();
                ExpressionNode::ArrayLiteral(ArrayLiteralNode {
                    elements,
                    position: array.position.clone(),
                })
            }
            _ => expr.clone()
        }
    }
    
    pub fn expand_builtin_function(&mut self, node: &BuiltinFunctionNode, context: &mut ExpansionContext, generate_source_map: bool, source_range: PositionRange) {
        match node.name {
            BuiltinFunction::Repeat => self.expand_repeat(node, context, generate_source_map, source_range),
            BuiltinFunction::If => self.expand_if(node, context, generate_source_map, source_range),
            BuiltinFunction::For => self.expand_for(node, context, generate_source_map, source_range),
            BuiltinFunction::Reverse => self.expand_reverse(node, context, generate_source_map, source_range),
        }
    }
    
    fn expand_repeat(&mut self, node: &BuiltinFunctionNode, context: &mut ExpansionContext, generate_source_map: bool, source_range: PositionRange) {
        if node.arguments.len() != 2 {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::SyntaxError,
                message: format!("repeat() expects exactly 2 arguments, got {}", node.arguments.len()),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: node.position.end - node.position.start,
                }),
            });
            self.append_to_expanded(&self.node_to_string(&[ContentNode::BuiltinFunction(node.clone())]), context, generate_source_map, Some(source_range));
            return;
        }
        
        let count_expr = self.expand_expression_to_string(&node.arguments[0], context);
        let trimmed = count_expr.trim();
        
        let count = if trimmed.starts_with("'") && trimmed.ends_with("'") && trimmed.len() == 3 {
            trimmed.chars().nth(1).unwrap() as i64
        } else if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            i64::from_str_radix(&trimmed[2..], 16).unwrap_or(-1)
        } else {
            trimmed.parse().unwrap_or(-1)
        };
        
        if count < 0 {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::SyntaxError,
                message: format!("Invalid repeat count: {}", count_expr),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: node.position.end - node.position.start,
                }),
            });
            self.append_to_expanded(&self.node_to_string(&[ContentNode::BuiltinFunction(node.clone())]), context, generate_source_map, Some(source_range));
            return;
        }
        
        let current_macro = context.macro_call_stack.last();
        let effective_source_range = current_macro.as_ref()
            .map(|m| m.call_site.clone())
            .unwrap_or(source_range);
        
        for _ in 0..count {
            if let ExpressionNode::BrainfuckCommand(cmd) = &node.arguments[1] {
                self.append_to_expanded(&cmd.commands, context, generate_source_map, Some(effective_source_range.clone()));
            } else {
                self.expand_expression(&node.arguments[1], context, generate_source_map);
            }
        }
    }
    
    fn expand_if(&mut self, node: &BuiltinFunctionNode, context: &mut ExpansionContext, generate_source_map: bool, source_range: PositionRange) {
        if node.arguments.len() != 3 {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::SyntaxError,
                message: format!("if() expects exactly 3 arguments, got {}", node.arguments.len()),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: node.position.end - node.position.start,
                }),
            });
            self.append_to_expanded(&self.node_to_string(&[ContentNode::BuiltinFunction(node.clone())]), context, generate_source_map, Some(source_range));
            return;
        }
        
        let condition_expr = self.expand_expression_to_string(&node.arguments[0], context);
        let trimmed = condition_expr.trim();
        
        let condition = if trimmed.starts_with("'") && trimmed.ends_with("'") && trimmed.len() == 3 {
            trimmed.chars().nth(1).unwrap() as i64
        } else if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            i64::from_str_radix(&trimmed[2..], 16).unwrap_or(0)
        } else {
            trimmed.parse().unwrap_or(0)
        };
        
        let selected_branch = if condition != 0 { &node.arguments[1] } else { &node.arguments[2] };
        self.expand_expression(selected_branch, context, generate_source_map);
    }
    
    fn expand_for(&mut self, node: &BuiltinFunctionNode, context: &mut ExpansionContext, generate_source_map: bool, _source_range: PositionRange) {
        if node.arguments.len() != 3 {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::SyntaxError,
                message: format!("for() expects exactly 3 arguments, got {}", node.arguments.len()),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: node.position.end - node.position.start,
                }),
            });
            return;
        }
        
        let var_node = &node.arguments[0];
        let array_node = &node.arguments[1];
        let body_node = &node.arguments[2];
        
        let var_names = match var_node {
            ExpressionNode::Identifier(ident) => vec![ident.name.clone()],
            ExpressionNode::TuplePattern(tuple) => tuple.elements.clone(),
            _ => {
                self.errors.push(MacroExpansionError {
                    error_type: MacroExpansionErrorType::SyntaxError,
                    message: format!("Expected variable name or tuple pattern in for loop, got {:?}", var_node),
                    location: Some(SourceLocation {
                        line: node.position.line.saturating_sub(1),
                        column: node.position.column.saturating_sub(1),
                        length: node.position.end - node.position.start,
                    }),
                });
                return;
            }
        };
        
        let is_tuple_pattern = matches!(var_node, ExpressionNode::TuplePattern(_));
        
        let values = self.extract_array_values(array_node, context, is_tuple_pattern);
        
        for value in values {
            let mut temp_substitutions = HashMap::new();
            
            if is_tuple_pattern {
                let tuple_elements = self.parse_tuple_elements(&value);
                for (i, var_name) in var_names.iter().enumerate() {
                    temp_substitutions.insert(
                        var_name.clone(),
                        tuple_elements.get(i).cloned().unwrap_or_default()
                    );
                }
            } else {
                temp_substitutions.insert(var_names[0].clone(), value);
            }
            
            match body_node {
                ExpressionNode::ExpressionList(list) => {
                    for expr in &list.expressions {
                        let substituted = match expr {
                            ContentNode::MacroInvocation(inv) => {
                                self.substitute_in_expression(
                                    &ExpressionNode::MacroInvocation(inv.clone()),
                                    &temp_substitutions
                                )
                            }
                            ContentNode::BuiltinFunction(builtin) => {
                                self.substitute_in_expression(
                                    &ExpressionNode::BuiltinFunction(builtin.clone()),
                                    &temp_substitutions
                                )
                            }
                            ContentNode::Text(text) => {
                                self.substitute_in_expression(
                                    &ExpressionNode::Text(text.clone()),
                                    &temp_substitutions
                                )
                            }
                            ContentNode::BrainfuckCommand(cmd) => {
                                ExpressionNode::BrainfuckCommand(cmd.clone())
                            }
                        };
                        self.expand_expression(&substituted, context, generate_source_map);
                    }
                }
                _ => {
                    let substituted = self.substitute_in_expression(body_node, &temp_substitutions);
                    self.expand_expression(&substituted, context, generate_source_map);
                }
            }
        }
    }
    
    fn expand_reverse(&mut self, node: &BuiltinFunctionNode, context: &mut ExpansionContext, generate_source_map: bool, source_range: PositionRange) {
        if node.arguments.len() != 1 {
            self.errors.push(MacroExpansionError {
                error_type: MacroExpansionErrorType::SyntaxError,
                message: format!("reverse() expects exactly 1 argument, got {}", node.arguments.len()),
                location: Some(SourceLocation {
                    line: node.position.line.saturating_sub(1),
                    column: node.position.column.saturating_sub(1),
                    length: node.position.end - node.position.start,
                }),
            });
            self.append_to_expanded(&self.node_to_string(&[ContentNode::BuiltinFunction(node.clone())]), context, generate_source_map, Some(source_range));
            return;
        }
        
        let array_arg = &node.arguments[0];
        
        match array_arg {
            ExpressionNode::ArrayLiteral(array_literal) => {
                let mut reversed_elements: Vec<_> = array_literal.elements.clone();
                reversed_elements.reverse();
                let expanded_elements: Vec<String> = reversed_elements.iter()
                    .map(|el| self.expand_expression_to_string(el, context))
                    .collect();
                let result = format!("{{{}}}", expanded_elements.join(", "));
                self.append_to_expanded(&result, context, generate_source_map, Some(source_range));
            }
            _ => {
                let expanded = self.expand_expression_to_string(array_arg, context).trim().to_string();
                
                if expanded.starts_with('{') && expanded.ends_with('}') {
                    let inner = &expanded[1..expanded.len()-1];
                    let mut values: Vec<_> = inner.split(',').map(|v| v.trim()).collect();
                    values.reverse();
                    let result = format!("{{{}}}", values.join(", "));
                    self.append_to_expanded(&result, context, generate_source_map, Some(source_range));
                } else if expanded.contains(',') {
                    let mut values: Vec<_> = expanded.split(',').map(|v| v.trim()).collect();
                    values.reverse();
                    let result = format!("{{{}}}", values.join(", "));
                    self.append_to_expanded(&result, context, generate_source_map, Some(source_range));
                } else {
                    self.errors.push(MacroExpansionError {
                        error_type: MacroExpansionErrorType::SyntaxError,
                        message: format!("reverse() expects an array literal, got {:?}", array_arg),
                        location: Some(SourceLocation {
                            line: node.position.line.saturating_sub(1),
                            column: node.position.column.saturating_sub(1),
                            length: node.position.end - node.position.start,
                        }),
                    });
                    self.append_to_expanded(&self.node_to_string(&[ContentNode::BuiltinFunction(node.clone())]), context, generate_source_map, Some(source_range));
                }
            }
        }
    }
    
    fn extract_array_values(&mut self, array_node: &ExpressionNode, context: &mut ExpansionContext, is_tuple_pattern: bool) -> Vec<String> {
        match array_node {
            ExpressionNode::ArrayLiteral(array_literal) => {
                array_literal.elements.iter()
                    .map(|el| self.expand_expression_to_string(el, context).trim().to_string())
                    .collect()
            }
            _ => {
                let expanded = self.expand_expression_to_string(array_node, context).trim().to_string();
                
                if is_tuple_pattern && expanded.starts_with('{') && expanded.ends_with('}') && !self.looks_like_array_of_arrays(&expanded) {
                    // Single tuple to destructure
                    vec![expanded]
                } else if self.looks_like_array_of_arrays(&expanded) {
                    self.parse_array_elements(&expanded)
                } else if expanded.starts_with('{') && expanded.ends_with('}') {
                    let inner = &expanded[1..expanded.len()-1];
                    let values = self.parse_array_elements(inner);
                    
                    if is_tuple_pattern && values.len() == 1 && values[0].starts_with('{') && values[0].ends_with('}') {
                        values
                    } else {
                        values
                    }
                } else if expanded.contains(',') {
                    self.parse_array_elements(&expanded)
                } else {
                    vec![expanded]
                }
            }
        }
    }
    
    fn parse_tuple_elements(&self, value: &str) -> Vec<String> {
        if value.starts_with('{') && value.ends_with('}') {
            self.parse_array_elements(&value[1..value.len()-1])
        } else {
            value.split(',').map(|v| v.trim().to_string()).collect()
        }
    }
    
    fn looks_like_array_of_arrays(&self, s: &str) -> bool {
        let trimmed = s.trim();
        trimmed.contains(',') && trimmed.contains("},{")
    }
    
    fn parse_array_elements(&self, inner: &str) -> Vec<String> {
        let mut elements = Vec::new();
        let mut current = String::new();
        let mut brace_depth = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;
        
        for ch in inner.chars() {
            if escaped {
                current.push(ch);
                escaped = false;
                continue;
            }
            
            match ch {
                '\\' => {
                    escaped = true;
                    current.push(ch);
                }
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                    current.push(ch);
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                    current.push(ch);
                }
                '{' if !in_single_quote && !in_double_quote => {
                    brace_depth += 1;
                    current.push(ch);
                }
                '}' if !in_single_quote && !in_double_quote => {
                    brace_depth -= 1;
                    current.push(ch);
                }
                ',' if brace_depth == 0 && !in_single_quote && !in_double_quote => {
                    if !current.trim().is_empty() {
                        elements.push(current.trim().to_string());
                    }
                    current.clear();
                }
                _ => current.push(ch)
            }
        }
        
        if !current.trim().is_empty() {
            elements.push(current.trim().to_string());
        }
        
        elements
    }
}