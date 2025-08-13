//! Parse error types for the C99 parser
//! 
//! This module defines all error types that can occur during parsing.

use crate::lexer::Token;
use rcc_common::{CompilerError, SourceLocation};

/// Parse error types specific to the parser
#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: Token,
    },
    UnexpectedEndOfFile {
        expected: String,
        location: SourceLocation,
    },
    InvalidExpression {
        message: String,
        location: SourceLocation,
    },
    InvalidType {
        message: String,
        location: SourceLocation,
    },
}

impl From<ParseError> for CompilerError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::UnexpectedToken { expected, found } => {
                CompilerError::parse_error(
                    format!("Expected {}, found {}", expected, found.token_type),
                    found.span.start,
                )
            }
            ParseError::UnexpectedEndOfFile { expected, location } => {
                CompilerError::parse_error(
                    format!("Unexpected end of file, expected {}", expected),
                    location,
                )
            }
            ParseError::InvalidExpression { message, location } => {
                CompilerError::parse_error(message, location)
            }
            ParseError::InvalidType { message, location } => {
                CompilerError::parse_error(message, location)
            }
        }
    }
}