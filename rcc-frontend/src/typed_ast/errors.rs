//! Type checking and conversion errors
//!
//! This module defines error types for the typed AST conversion process.

use std::fmt;

/// Type checking errors
#[derive(Debug, Clone)]
pub enum TypeError {
    TypeMismatch(String),
    UndefinedVariable(String),
    UnsupportedConstruct(String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            TypeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            TypeError::UnsupportedConstruct(construct) => write!(f, "Unsupported construct: {}", construct),
        }
    }
}

impl std::error::Error for TypeError {}