//! Postfix expression parsing

use crate::ast::*;
use crate::lexer::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::Parser;
use rcc_common::{CompilerError, SourceSpan};

impl Parser {
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
}