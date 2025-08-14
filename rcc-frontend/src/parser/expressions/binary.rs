//! Binary expression parsing with operator precedence

use crate::ast::*;
use crate::lexer::TokenType;
use crate::parser::Parser;
use rcc_common::{CompilerError, SourceSpan};

impl Parser {
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
}