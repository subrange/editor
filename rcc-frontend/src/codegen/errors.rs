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

use rcc_common::CompilerError;

impl From<CodegenError> for CompilerError {
    fn from(err: CodegenError) -> Self {
        match err {
            CodegenError::UnsupportedConstruct { construct, location } => {
                CompilerError::codegen_error(
                    format!("Unsupported construct: {}", construct),
                    location,
                )
            }
            CodegenError::InvalidType { ast_type, location } => {
                CompilerError::codegen_error(
                    format!("Invalid type: {:?}", ast_type),
                    location,
                )
            }
            CodegenError::UndefinedFunction { name, location } => {
                CompilerError::codegen_error(
                    format!("Undefined function: {}", name),
                    location,
                )
            }
            CodegenError::UndefinedVariable { name, location } => {
                CompilerError::codegen_error(
                    format!("Undefined variable: {}", name),
                    location,
                )
            }
            CodegenError::InternalError { message, location } => {
                CompilerError::codegen_error(
                    format!("Internal error: {}", message),
                    location,
                )
            }
            CodegenError::InvalidLvalue { location } => {
                CompilerError::codegen_error(
                    "Invalid lvalue".to_string(),
                    location,
                )
            }
        }
    }
}