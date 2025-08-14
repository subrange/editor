//! Type checking and conversion errors
//!
//! This module defines error types for the typed AST conversion process.

use std::fmt;

/// Type checking errors
#[derive(Debug, Clone)]
pub enum TypeError {
    TypeMismatch(String),
    UndefinedVariable(String),
    UndefinedType(String),
    UndefinedMember { struct_name: String, member_name: String },
    UnsupportedConstruct(String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            TypeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            TypeError::UndefinedType(name) => write!(f, "Undefined type: {}", name),
            TypeError::UndefinedMember { struct_name, member_name } => 
                write!(f, "Undefined member '{}' in struct '{}'", member_name, struct_name),
            TypeError::UnsupportedConstruct(construct) => write!(f, "Unsupported construct: {}", construct),
        }
    }
}

impl std::error::Error for TypeError {}