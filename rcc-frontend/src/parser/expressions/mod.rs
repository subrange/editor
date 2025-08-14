//! Expression parsing for C99
//! 
//! This module handles parsing of all expression types using operator precedence parsing.

mod primary;
mod postfix;
mod unary;
mod binary;
mod assignment;

use crate::ast::*;
use crate::parser::Parser;
use rcc_common::CompilerError;

impl Parser {
    /// Parse expression (top level)
    pub fn parse_expression(&mut self) -> Result<Expression, CompilerError> {
        self.parse_assignment_expression()
    }
}