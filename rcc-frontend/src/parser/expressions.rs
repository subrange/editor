//! Expression parsing for C99
//! 
//! This module handles parsing of all expression types using operator precedence parsing.

use crate::ast::*;
use crate::lexer::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::Parser;
use rcc_common::{CompilerError, SourceSpan};
use crate::Type;

impl Parser {
    /// Parse expression (top level)
    pub fn parse_expression(&mut self) -> Result<Expression, CompilerError> {
        self.parse_assignment_expression()
    }
    
    /// Parse assignment expression
    pub fn parse_assignment_expression(&mut self) -> Result<Expression, CompilerError> {
        let left = self.parse_conditional_expression()?;
        
        // Check for assignment operators
        if let Some(op) = self.parse_assignment_operator() {
            let right = self.parse_assignment_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            return Ok(Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse assignment operator
    fn parse_assignment_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Equal) => { self.advance(); Some(BinaryOp::Assign) }
            Some(TokenType::PlusEqual) => { self.advance(); Some(BinaryOp::AddAssign) }
            Some(TokenType::MinusEqual) => { self.advance(); Some(BinaryOp::SubAssign) }
            Some(TokenType::StarEqual) => { self.advance(); Some(BinaryOp::MulAssign) }
            Some(TokenType::SlashEqual) => { self.advance(); Some(BinaryOp::DivAssign) }
            Some(TokenType::PercentEqual) => { self.advance(); Some(BinaryOp::ModAssign) }
            _ => None,
        }
    }
    
    /// Parse conditional expression (ternary operator)
    pub fn parse_conditional_expression(&mut self) -> Result<Expression, CompilerError> {
        let condition = self.parse_logical_or_expression()?;
        
        if self.match_token(&TokenType::Question) {
            let then_expr = self.parse_expression()?;
            self.expect(TokenType::Colon, "conditional expression")?;
            let else_expr = self.parse_conditional_expression()?;
            
            let span = SourceSpan::new(condition.span.start.clone(), else_expr.span.end.clone());
            
            Ok(Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Conditional {
                    condition: Box::new(condition),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                },
                span,
                expr_type: None,
            })
        } else {
            Ok(condition)
        }
    }
    
    /// Parse logical OR expression
    pub fn parse_logical_or_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_logical_and_expression()?;
        
        while self.match_token(&TokenType::PipePipe) {
            let right = self.parse_logical_and_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::LogicalOr,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse logical AND expression
    pub fn parse_logical_and_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_bitwise_or_expression()?;
        
        while self.match_token(&TokenType::AmpersandAmpersand) {
            let right = self.parse_bitwise_or_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::LogicalAnd,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse bitwise OR expression
    pub fn parse_bitwise_or_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_bitwise_xor_expression()?;
        
        while self.match_token(&TokenType::Pipe) {
            let right = self.parse_bitwise_xor_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::BitOr,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse bitwise XOR expression
    pub fn parse_bitwise_xor_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_bitwise_and_expression()?;
        
        while self.match_token(&TokenType::Caret) {
            let right = self.parse_bitwise_and_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::BitXor,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse bitwise AND expression
    pub fn parse_bitwise_and_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_equality_expression()?;
        
        while self.match_token(&TokenType::Ampersand) {
            let right = self.parse_equality_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::BitAnd,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse equality expression
    pub fn parse_equality_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_relational_expression()?;
        
        while let Some(op) = self.parse_equality_operator() {
            let right = self.parse_relational_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse equality operator
    fn parse_equality_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::EqualEqual) => { self.advance(); Some(BinaryOp::Equal) }
            Some(TokenType::BangEqual) => { self.advance(); Some(BinaryOp::NotEqual) }
            _ => None,
        }
    }
    
    /// Parse relational expression
    pub fn parse_relational_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_shift_expression()?;
        
        while let Some(op) = self.parse_relational_operator() {
            let right = self.parse_shift_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse relational operator
    fn parse_relational_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Less) => { self.advance(); Some(BinaryOp::Less) }
            Some(TokenType::Greater) => { self.advance(); Some(BinaryOp::Greater) }
            Some(TokenType::LessEqual) => { self.advance(); Some(BinaryOp::LessEqual) }
            Some(TokenType::GreaterEqual) => { self.advance(); Some(BinaryOp::GreaterEqual) }
            _ => None,
        }
    }
    
    /// Parse shift expression
    pub fn parse_shift_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_additive_expression()?;
        
        while let Some(op) = self.parse_shift_operator() {
            let right = self.parse_additive_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse shift operator
    fn parse_shift_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::LeftShift) => { self.advance(); Some(BinaryOp::LeftShift) }
            Some(TokenType::RightShift) => { self.advance(); Some(BinaryOp::RightShift) }
            _ => None,
        }
    }
    
    /// Parse additive expression
    pub fn parse_additive_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_multiplicative_expression()?;
        
        while let Some(op) = self.parse_additive_operator() {
            let right = self.parse_multiplicative_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse additive operator
    fn parse_additive_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Plus) => { self.advance(); Some(BinaryOp::Add) }
            Some(TokenType::Minus) => { self.advance(); Some(BinaryOp::Sub) }
            _ => None,
        }
    }
    
    /// Parse multiplicative expression
    pub fn parse_multiplicative_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_unary_expression()?;
        
        while let Some(op) = self.parse_multiplicative_operator() {
            let right = self.parse_unary_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse multiplicative operator
    fn parse_multiplicative_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Star) => { self.advance(); Some(BinaryOp::Mul) }
            Some(TokenType::Slash) => { self.advance(); Some(BinaryOp::Div) }
            Some(TokenType::Percent) => { self.advance(); Some(BinaryOp::Mod) }
            _ => None,
        }
    }
    
    /// Parse unary expression
    pub fn parse_unary_expression(&mut self) -> Result<Expression, CompilerError> {
        // Special handling for sizeof
        if self.peek().map(|t| &t.token_type) == Some(&TokenType::Sizeof) {
            let start = self.current_location();
            self.advance(); // consume 'sizeof'
            
            // Check if next token is '(' to determine if it's sizeof(type) or sizeof expr
            if self.peek().map(|t| &t.token_type) == Some(&TokenType::LeftParen) {
                // Could be sizeof(type) or sizeof(expr)
                // Save state for potential backtracking
                let saved_tokens = self.tokens.clone();
                self.advance(); // consume '('
                
                // Try to parse as type - check for type keywords
                let is_type = match self.peek().map(|t| &t.token_type) {
                    Some(TokenType::Void) | Some(TokenType::Char) | Some(TokenType::Short) |
                    Some(TokenType::Int) | Some(TokenType::Long) | Some(TokenType::Float) |
                    Some(TokenType::Double) | Some(TokenType::Signed) | Some(TokenType::Unsigned) |
                    Some(TokenType::Struct) | Some(TokenType::Union) | Some(TokenType::Enum) => true,
                    _ => false,
                };
                
                if is_type {
                    // Parse as type
                    if let Ok(mut type_spec) = self.parse_type_specifier() {
                        // Handle pointer types
                        while self.peek().map(|t| &t.token_type) == Some(&TokenType::Star) {
                            self.advance();
                            type_spec = Type::Pointer { target: Box::new(type_spec), bank: None };
                        }
                        
                        if self.peek().map(|t| &t.token_type) == Some(&TokenType::RightParen) {
                            self.advance(); // consume ')'
                            let end = self.current_location();
                            
                            return Ok(Expression {
                                node_id: self.node_id_gen.next(),
                                kind: ExpressionKind::SizeofType(type_spec),
                                span: SourceSpan::new(start, end),
                                expr_type: None,
                            });
                        }
                    }
                }
                
                // Not a type, restore and parse as expression
                self.tokens = saved_tokens;
                self.advance(); // re-consume '('
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen, "sizeof expression")?;
                
                let end = self.current_location();
                return Ok(Expression {
                    node_id: self.node_id_gen.next(),
                    kind: ExpressionKind::SizeofExpr(Box::new(expr)),
                    span: SourceSpan::new(start, end),
                    expr_type: None,
                });
            }
            
            // Parse as sizeof expression (without parens)
            let operand = self.parse_unary_expression()?;
            let span = SourceSpan::new(start, operand.span.end.clone());
            
            return Ok(Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::SizeofExpr(Box::new(operand)),
                span,
                expr_type: None,
            });
        }
        
        // Normal unary operators
        if let Some(op) = self.parse_unary_operator() {
            let operand = self.parse_unary_expression()?;
            let span = SourceSpan::new(self.current_location(), operand.span.end.clone());
            
            Ok(Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
                span,
                expr_type: None,
            })
        } else {
            self.parse_postfix_expression()
        }
    }
    
    /// Parse unary operator
    fn parse_unary_operator(&mut self) -> Option<UnaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Plus) => { self.advance(); Some(UnaryOp::Plus) }
            Some(TokenType::Minus) => { self.advance(); Some(UnaryOp::Minus) }
            Some(TokenType::Bang) => { self.advance(); Some(UnaryOp::LogicalNot) }
            Some(TokenType::Tilde) => { self.advance(); Some(UnaryOp::BitNot) }
            Some(TokenType::Star) => { self.advance(); Some(UnaryOp::Dereference) }
            Some(TokenType::Ampersand) => { self.advance(); Some(UnaryOp::AddressOf) }
            Some(TokenType::PlusPlus) => { self.advance(); Some(UnaryOp::PreIncrement) }
            Some(TokenType::MinusMinus) => { self.advance(); Some(UnaryOp::PreDecrement) }
            // Sizeof is handled specially in parse_unary_expression
            _ => None,
        }
    }
    
    /// Parse postfix expression
    pub fn parse_postfix_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.parse_primary_expression()?;
        
        loop {
            match self.peek().map(|t| &t.token_type) {
                Some(TokenType::LeftBracket) => {
                    // Array indexing
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(TokenType::RightBracket, "array index")?;
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Binary {
                            op: BinaryOp::Index,
                            left: Box::new(expr),
                            right: Box::new(index),
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::LeftParen) => {
                    // Function call
                    self.advance();
                    let mut arguments = Vec::new();
                    
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            arguments.push(self.parse_assignment_expression()?);
                            
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.expect(TokenType::RightParen, "function call")?;
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Call {
                            function: Box::new(expr),
                            arguments,
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::Dot) => {
                    // Member access
                    self.advance();
                    let member = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.advance() {
                        name
                    } else {
                        return Err(ParseError::InvalidExpression {
                            message: "Expected member name after '.'".to_string(),
                            location: self.current_location(),
                        }.into());
                    };
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Member {
                            object: Box::new(expr),
                            member,
                            is_pointer: false,
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::Arrow) => {
                    // Pointer member access
                    self.advance();
                    let member = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.advance() {
                        name
                    } else {
                        return Err(ParseError::InvalidExpression {
                            message: "Expected member name after '->'".to_string(),
                            location: self.current_location(),
                        }.into());
                    };
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Member {
                            object: Box::new(expr),
                            member,
                            is_pointer: true,
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::PlusPlus) => {
                    // Postfix increment
                    self.advance();
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Unary {
                            op: UnaryOp::PostIncrement,
                            operand: Box::new(expr),
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::MinusMinus) => {
                    // Postfix decrement
                    self.advance();
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Unary {
                            op: UnaryOp::PostDecrement,
                            operand: Box::new(expr),
                        },
                        span,
                        expr_type: None,
                    };
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    /// Parse primary expression
    pub fn parse_primary_expression(&mut self) -> Result<Expression, CompilerError> {
        let start_location = self.current_location();
        
        let kind = match self.advance() {
            Some(Token { token_type: TokenType::IntLiteral(value), .. }) => {
                ExpressionKind::IntLiteral(value)
            }
            Some(Token { token_type: TokenType::CharLiteral(value), .. }) => {
                ExpressionKind::CharLiteral(value)
            }
            Some(Token { token_type: TokenType::StringLiteral(value), .. }) => {
                ExpressionKind::StringLiteral(value)
            }
            Some(Token { token_type: TokenType::Identifier(name), .. }) => {
                ExpressionKind::Identifier { name, symbol_id: None }
            }
            Some(Token { token_type: TokenType::LeftParen, .. }) => {
                // Parenthesized expression
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen, "parenthesized expression")?;
                return Ok(expr);
            }
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "primary expression".to_string(),
                    found: token,
                }.into());
            }
            None => {
                return Err(ParseError::UnexpectedEndOfFile {
                    expected: "primary expression".to_string(),
                    location: start_location,
                }.into());
            }
        };
        
        let end_location = self.current_location();
        
        Ok(Expression {
            node_id: self.node_id_gen.next(),
            kind,
            span: SourceSpan::new(start_location, end_location),
            expr_type: None,
        })
    }
}