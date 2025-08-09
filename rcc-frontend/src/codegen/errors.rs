//! Code generation error types

use rcc_common::SourceLocation;
use crate::ast::Type;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("Unsupported construct '{construct}' at {location}")]
    UnsupportedConstruct {
        construct: String,
        location: SourceLocation,
    },
    
    #[error("Invalid type at {location}")]
    InvalidType {
        ast_type: Type,
        location: SourceLocation,
    },
    
    #[error("Undefined function '{name}' at {location}")]
    UndefinedFunction {
        name: String,
        location: SourceLocation,
    },
    
    #[error("Undefined variable '{name}' at {location}")]
    UndefinedVariable {
        name: String,
        location: SourceLocation,
    },
    
    #[error("Internal error at {location}: {message}")]
    InternalError {
        message: String,
        location: SourceLocation,
    },
    
    #[error("Invalid lvalue at {location}")]
    InvalidLvalue {
        location: SourceLocation,
    },
}