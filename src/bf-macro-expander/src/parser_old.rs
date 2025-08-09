use crate::ast::*;
use crate::lexer::{Token, TokenType};
use crate::types::{MacroExpansionError, MacroExpansionErrorType, MacroToken, MacroTokenType, Range, SourceLocation};

pub struct ParseResult {
    pub ast: ProgramNode,
    pub errors: Vec<MacroExpansionError>,
    pub tokens: Vec<MacroToken>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<MacroExpansionError>,
    macro_tokens: Vec<MacroToken>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
            macro_tokens: Vec::new(),
        }
    }
    
    pub fn parse(&mut self) -> ParseResult {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            // Skip newlines at statement level
            while self.match_token(TokenType::Newline) {
                // Continue
            }
            
            if self.is_at_end() {
                break;
            }
            
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }
        
        let end_pos = if !self.tokens.is_empty() {
            self.tokens[self.tokens.len() - 1].position.end
        } else {
            0
        };
        
        let ast = ProgramNode {
            statements,
            position: ASTPosition {
                start: 0,
                end: end_pos,
                line: 1,
                column: 1,
            },
        };
        
        ParseResult {
            ast,
            errors: self.errors.clone(),
            tokens: self.macro_tokens.clone(),
        }
    }
    
    fn parse_statement(&mut self) -> Option<StatementNode> {
        let saved_position = self.current;
        
        // Skip whitespace to check for #define
        self.skip_whitespace();
        
        if self.check(TokenType::Define) {
            return self.parse_macro_definition();
        }
        
        // Backtrack to preserve leading whitespace
        self.current = saved_position;
        self.parse_code_line()
    }
    
    fn parse_macro_definition(&mut self) -> Option<StatementNode> {
        let start_token = self.peek();
        let start = start_token.position.start;
        let start_line = start_token.position.line;
        let start_column = start_token.position.column;
        
        self.consume(TokenType::Define);
        self.skip_whitespace();
        
        if !self.check(TokenType::Identifier) {
            self.add_error("Expected macro name after #define", self.peek().position.clone());
            self.synchronize();
            return None;
        }
        
        let name_token = self.advance();
        let name = name_token.value.clone();
        
        // Add macro definition token
        self.macro_tokens.push(MacroToken {
            token_type: MacroTokenType::MacroDefinition,
            range: Range {
                start,
                end: if self.current < self.tokens.len() {
                    self.peek().position.start
                } else {
                    name_token.position.end
                },
            },
            name: name.clone(),
        });
        
        let mut parameters = None;
        
        // Check for parameters
        if self.check(TokenType::LParen) {
            self.advance(); // consume (
            parameters = Some(self.parse_parameter_list());
            if !self.consume(TokenType::RParen) {
                self.add_error("Expected ')' after parameter list", self.peek().position.clone());
            }
        }
        
        self.skip_whitespace();
        
        // Parse body
        let body = self.parse_macro_body();
        
        let end_pos = if !body.is_empty() {
            // Safe to unwrap position since we check for non-empty body
            match body.last().unwrap() {
                ContentNode::BrainfuckCommand(node) => node.position.end,
                ContentNode::MacroInvocation(node) => node.position.end,
                ContentNode::BuiltinFunction(node) => node.position.end,
                ContentNode::Text(node) => node.position.end,
            }
        } else {
            self.current.saturating_sub(1).min(self.tokens.len() - 1)
        };
        
        Some(StatementNode::MacroDefinition(MacroDefinitionNode {
            name,
            parameters,
            body,
            position: ASTPosition {
                start,
                end: end_pos,
                line: start_line,
                column: start_column,
            },
        }))
    }
    
    fn parse_parameter_list(&mut self) -> Vec<String> {
        let mut params = Vec::new();
        
        self.skip_whitespace();
        
        if !self.check(TokenType::RParen) {
            loop {
                self.skip_whitespace();
                
                if self.check(TokenType::Identifier) {
                    params.push(self.advance().value.clone());
                } else {
                    self.add_error("Expected parameter name", self.peek().position.clone());
                    break;
                }
                
                self.skip_whitespace();
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        
        params
    }
    
    fn parse_macro_body(&mut self) -> Vec<BodyNode> {
        let mut body = Vec::new();
        
        // Handle line continuations with backslash
        loop {
            // Parse content on current line
            while !self.is_at_end() && !self.check(TokenType::Newline) && !self.check(TokenType::Backslash) {
                if let Some(content) = self.parse_content() {
                    body.push(content);
                }
            }
            
            // Check for line continuation
            if self.check(TokenType::Backslash) {
                self.advance(); // consume backslash
                if self.check(TokenType::Newline) {
                    self.advance(); // consume newline
                    self.skip_whitespace(); // skip leading whitespace on next line
                    continue; // continue parsing on next line
                }
            }
            
            break;
        }
        
        body
    }
    
    fn parse_code_line(&mut self) -> Option<StatementNode> {
        let start = self.peek().position.start;
        let start_line = self.peek().position.line;
        let start_column = self.peek().position.column;
        
        let mut content = Vec::new();
        
        while !self.is_at_end() && !self.check(TokenType::Newline) {
            if let Some(node) = self.parse_content() {
                content.push(node);
            } else {
                self.advance(); // Skip unknown tokens
            }
        }
        
        // Consume the newline if present
        self.match_token(TokenType::Newline);
        
        let end_pos = if !content.is_empty() {
            match content.last().unwrap() {
                ContentNode::BrainfuckCommand(node) => node.position.end,
                ContentNode::MacroInvocation(node) => node.position.end,
                ContentNode::BuiltinFunction(node) => node.position.end,
                ContentNode::Text(node) => node.position.end,
            }
        } else {
            start
        };
        
        Some(StatementNode::CodeLine(CodeLineNode {
            content,
            position: ASTPosition {
                start,
                end: end_pos,
                line: start_line,
                column: start_column,
            },
        }))
    }
    
    fn parse_content(&mut self) -> Option<ContentNode> {
        // Skip comments and whitespace that should be ignored
        while self.check(TokenType::Comment) || 
              (self.check(TokenType::Whitespace) && self.peek_ahead(1).map_or(false, |t| 
                  matches!(t.token_type, TokenType::Newline | TokenType::Eof))) {
            self.advance();
        }
        
        if self.is_at_end() || self.check(TokenType::Newline) {
            return None;
        }
        
        match self.peek().token_type {
            TokenType::BrainfuckCommand => {
                let token = self.advance();
                Some(ContentNode::BrainfuckCommand(BrainfuckCommandNode {
                    commands: token.value.clone(),
                    position: token.position.clone(),
                }))
            }
            TokenType::At => {
                self.advance(); // consume @
                self.parse_macro_invocation().map(ContentNode::MacroInvocation)
            }
            TokenType::LBrace => {
                self.advance(); // consume {
                self.parse_builtin_function().map(ContentNode::BuiltinFunction)
            }
            TokenType::Whitespace | TokenType::Text | TokenType::Identifier | TokenType::Number => {
                let token = self.advance();
                Some(ContentNode::Text(TextNode {
                    value: token.value.clone(),
                    position: token.position.clone(),
                }))
            }
            _ => {
                self.advance();
                None
            }
        }
    }
    
    fn parse_macro_invocation(&mut self) -> Option<MacroInvocationNode> {
        let start_pos = self.tokens[self.current.saturating_sub(1)].position.clone();
        
        if !self.check(TokenType::Identifier) {
            self.add_error("Expected macro name after @", self.peek().position.clone());
            return None;
        }
        
        let name_token = self.advance();
        let name = name_token.value.clone();
        let mut arguments = None;
        
        if self.check(TokenType::LParen) {
            self.advance(); // consume (
            arguments = Some(self.parse_argument_list());
            if !self.consume(TokenType::RParen) {
                self.add_error("Expected ')' after arguments", self.peek().position.clone());
            }
        }
        
        let end_pos = self.tokens[self.current.saturating_sub(1)].position.end;
        let macro_name = name.clone();
        
        // Add macro invocation token
        self.macro_tokens.push(MacroToken {
            token_type: MacroTokenType::MacroInvocation,
            range: Range {
                start: start_pos.start,
                end: end_pos,
            },
            name: macro_name.clone(),
        });
        
        Some(MacroInvocationNode {
            name: macro_name,
            arguments,
            position: ASTPosition {
                start: start_pos.start,
                end: end_pos,
                line: start_pos.line,
                column: start_pos.column,
            },
        })
    }
    
    fn parse_builtin_function(&mut self) -> Option<BuiltinFunctionNode> {
        let start_pos = self.tokens[self.current.saturating_sub(1)].position.clone();
        
        if !self.check(TokenType::Identifier) {
            self.add_error("Expected function name after {", self.peek().position.clone());
            return None;
        }
        
        let name_token = self.advance();
        let name_str = name_token.value.clone();
        let name_value = name_token.value.clone();
        let name = match name_str.as_str() {
            "repeat" => BuiltinFunction::Repeat,
            "if" => BuiltinFunction::If,
            "for" => BuiltinFunction::For,
            "reverse" => BuiltinFunction::Reverse,
            _ => {
                self.add_error(&format!("Unknown builtin function: {}", name_value), self.peek().position.clone());
                return None;
            }
        };
        
        if !self.consume(TokenType::LParen) {
            self.add_error("Expected '(' after function name", self.peek().position.clone());
            return None;
        }
        
        let arguments = if name == BuiltinFunction::For {
            self.parse_for_arguments()
        } else {
            self.parse_argument_list()
        };
        
        if !self.consume(TokenType::RParen) {
            self.add_error("Expected ')' after arguments", self.peek().position.clone());
        }
        
        if !self.consume(TokenType::RBrace) {
            self.add_error("Expected '}' after builtin function", self.peek().position.clone());
        }
        
        let end_pos = self.tokens[self.current.saturating_sub(1)].position.end;
        let function_name = name_value.clone();
        
        // Add builtin function token
        self.macro_tokens.push(MacroToken {
            token_type: MacroTokenType::BuiltinFunction,
            range: Range {
                start: start_pos.start,
                end: end_pos,
            },
            name: function_name,
        });
        
        Some(BuiltinFunctionNode {
            name,
            arguments,
            position: ASTPosition {
                start: start_pos.start,
                end: end_pos,
                line: start_pos.line,
                column: start_pos.column,
            },
        })
    }
    
    fn parse_for_arguments(&mut self) -> Vec<ExpressionNode> {
        let mut args = Vec::new();
        
        // Parse variable or tuple pattern
        if self.check(TokenType::LParen) {
            // Tuple pattern
            self.advance(); // consume (
            let mut elements = Vec::new();
            
            loop {
                self.skip_whitespace();
                if self.check(TokenType::Identifier) {
                    elements.push(self.advance().value.clone());
                } else {
                    break;
                }
                
                self.skip_whitespace();
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
            
            if !self.consume(TokenType::RParen) {
                self.add_error("Expected ')' after tuple pattern", self.peek().position.clone());
            }
            
            let pos = self.peek().position.clone();
            args.push(ExpressionNode::TuplePattern(TuplePatternNode {
                elements,
                position: pos,
            }));
        } else if self.check(TokenType::Identifier) {
            // Single variable
            let token = self.advance();
            args.push(ExpressionNode::Identifier(IdentifierNode {
                name: token.value.clone(),
                position: token.position.clone(),
            }));
        }
        
        self.skip_whitespace();
        
        // Expect 'in' keyword
        if !self.check(TokenType::In) {
            self.add_error("Expected 'in' in for loop", self.peek().position.clone());
        } else {
            self.advance(); // consume 'in'
        }
        
        self.skip_whitespace();
        
        // Parse array expression
        if let Some(array_expr) = self.parse_expression() {
            args.push(array_expr);
        }
        
        self.skip_whitespace();
        
        // Parse body after comma
        if self.match_token(TokenType::Comma) {
            self.skip_whitespace();
            if let Some(body_expr) = self.parse_expression() {
                args.push(body_expr);
            }
        }
        
        args
    }
    
    fn parse_argument_list(&mut self) -> Vec<ExpressionNode> {
        let mut args = Vec::new();
        
        self.skip_whitespace();
        
        if !self.check(TokenType::RParen) {
            loop {
                self.skip_whitespace();
                
                if let Some(expr) = self.parse_expression() {
                    args.push(expr);
                } else {
                    break;
                }
                
                self.skip_whitespace();
                
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        
        args
    }
    
    fn parse_expression(&mut self) -> Option<ExpressionNode> {
        self.skip_whitespace();
        
        match self.peek().token_type {
            TokenType::Number => {
                let token = self.advance();
                let value = if token.value.starts_with("0x") || token.value.starts_with("0X") {
                    i64::from_str_radix(&token.value[2..], 16).unwrap_or(0)
                } else {
                    token.value.parse().unwrap_or(0)
                };
                Some(ExpressionNode::Number(NumberNode {
                    value,
                    position: token.position.clone(),
                }))
            }
            TokenType::Identifier => {
                let token = self.advance();
                Some(ExpressionNode::Identifier(IdentifierNode {
                    name: token.value.clone(),
                    position: token.position.clone(),
                }))
            }
            TokenType::At => {
                self.advance(); // consume @
                self.parse_macro_invocation().map(ExpressionNode::MacroInvocation)
            }
            TokenType::LBrace => {
                // Could be builtin function or array literal
                let saved_pos = self.current;
                self.advance(); // consume {
                
                // Try to parse as builtin function first
                if self.check(TokenType::Identifier) {
                    let name = self.peek().value.clone();
                    if matches!(name.as_str(), "repeat" | "if" | "for" | "reverse") {
                        if let Some(builtin) = self.parse_builtin_function() {
                            return Some(ExpressionNode::BuiltinFunction(builtin));
                        }
                    }
                }
                
                // Otherwise parse as array literal
                self.current = saved_pos;
                self.advance(); // consume { again
                self.parse_array_literal().map(ExpressionNode::ArrayLiteral)
            }
            TokenType::BrainfuckCommand => {
                let token = self.advance();
                Some(ExpressionNode::BrainfuckCommand(BrainfuckCommandNode {
                    commands: token.value.clone(),
                    position: token.position.clone(),
                }))
            }
            TokenType::Text => {
                let token = self.advance();
                Some(ExpressionNode::Text(TextNode {
                    value: token.value.clone(),
                    position: token.position.clone(),
                }))
            }
            _ => {
                // Try to parse as expression list
                self.parse_expression_list().map(ExpressionNode::ExpressionList)
            }
        }
    }
    
    fn parse_array_literal(&mut self) -> Option<ArrayLiteralNode> {
        let start_pos = self.tokens[self.current.saturating_sub(1)].position.clone();
        let mut elements = Vec::new();
        
        self.skip_whitespace();
        
        while !self.check(TokenType::RBrace) && !self.is_at_end() {
            if let Some(expr) = self.parse_expression() {
                elements.push(expr);
            }
            
            self.skip_whitespace();
            
            if !self.match_token(TokenType::Comma) {
                break;
            }
            
            self.skip_whitespace();
        }
        
        if !self.consume(TokenType::RBrace) {
            self.add_error("Expected '}' after array literal", self.peek().position.clone());
        }
        
        let end_pos = self.tokens[self.current.saturating_sub(1)].position.end;
        
        Some(ArrayLiteralNode {
            elements,
            position: ASTPosition {
                start: start_pos.start,
                end: end_pos,
                line: start_pos.line,
                column: start_pos.column,
            },
        })
    }
    
    fn parse_expression_list(&mut self) -> Option<ExpressionListNode> {
        let start_pos = self.peek().position.clone();
        let mut expressions = Vec::new();
        
        while !self.is_at_end() && 
              !self.check(TokenType::Comma) && 
              !self.check(TokenType::RParen) && 
              !self.check(TokenType::RBrace) &&
              !self.check(TokenType::Newline) {
            if let Some(content) = self.parse_content() {
                expressions.push(content);
            } else {
                break;
            }
        }
        
        if expressions.is_empty() {
            return None;
        }
        
        let end_pos = match expressions.last() {
            Some(node) => match node {
                ContentNode::BrainfuckCommand(n) => n.position.end,
                ContentNode::MacroInvocation(n) => n.position.end,
                ContentNode::BuiltinFunction(n) => n.position.end,
                ContentNode::Text(n) => n.position.end,
            },
            None => start_pos.end,
        };
        
        Some(ExpressionListNode {
            expressions,
            position: ASTPosition {
                start: start_pos.start,
                end: end_pos,
                line: start_pos.line,
                column: start_pos.column,
            },
        })
    }
    
    // Helper methods
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }
    
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }
    
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn consume(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        let pos = self.current + n;
        if pos < self.tokens.len() {
            Some(&self.tokens[pos])
        } else {
            None
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().token_type == TokenType::Eof
    }
    
    fn skip_whitespace(&mut self) {
        while self.check(TokenType::Whitespace) {
            self.advance();
        }
    }
    
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.match_token(TokenType::Newline) {
                return;
            }
            self.advance();
        }
    }
    
    fn add_error(&mut self, message: &str, position: ASTPosition) {
        self.errors.push(MacroExpansionError {
            error_type: MacroExpansionErrorType::SyntaxError,
            message: message.to_string(),
            location: Some(SourceLocation {
                line: position.line.saturating_sub(1),
                column: position.column.saturating_sub(1),
                length: position.end - position.start,
            }),
        });
    }
}