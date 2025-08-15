//! Semantic analysis error definitions
//! 
//! This module defines all error types that can occur during semantic analysis.

use crate::types::Type;
use rcc_common::{CompilerError, SourceLocation};

/// Semantic analysis errors
#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable {
        name: String,
        location: SourceLocation,
    },
    TypeMismatch {
        expected: Type,
        found: Type,
        location: SourceLocation,
    },
    RedefinedSymbol {
        name: String,
        original_location: SourceLocation,
        redefinition_location: SourceLocation,
    },
    InvalidOperation {
        operation: String,
        operand_type: Type,
        location: SourceLocation,
    },
    InvalidFunctionCall {
        function_type: Type,
        location: SourceLocation,
    },
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
        location: SourceLocation,
    },
    ReturnTypeMismatch {
        expected: Type,
        found: Type,
        location: SourceLocation,
    },
    InvalidLvalue {
        location: SourceLocation,
    },
    RedefinedType {
        name: String,
    },
    IncompleteType {
        type_name: String,
        location: SourceLocation,
    },
    StructTooLarge {
        struct_name: String,
        location: SourceLocation,
    },
    UndefinedMember {
        struct_name: String,
        member_name: String,
        location: SourceLocation,
    },
    MemberAccessOnNonStruct {
        type_name: Type,
        location: SourceLocation,
    },
    UndefinedType {
        name: String,
        location: SourceLocation,
    },
}

impl From<SemanticError> for CompilerError {
    fn from(err: SemanticError) -> Self {
        match err {
            SemanticError::UndefinedVariable { name, location } => {
                CompilerError::semantic_error(
                    format!("Undefined variable: {}", name),
                    location,
                )
            }
            SemanticError::TypeMismatch { expected, found, location } => {
                CompilerError::semantic_error(
                    format!("Type mismatch: expected {}, found {}", expected, found),
                    location,
                )
            }
            SemanticError::RedefinedSymbol { name, redefinition_location, .. } => {
                CompilerError::semantic_error(
                    format!("Redefinition of symbol: {}", name),
                    redefinition_location,
                )
            }
            SemanticError::InvalidOperation { operation, operand_type, location } => {
                CompilerError::semantic_error(
                    format!("Invalid operation {} on type {}", operation, operand_type),
                    location,
                )
            }
            SemanticError::InvalidFunctionCall { function_type, location } => {
                CompilerError::semantic_error(
                    format!("Cannot call non-function type {}", function_type),
                    location,
                )
            }
            SemanticError::ArgumentCountMismatch { expected, found, location } => {
                CompilerError::semantic_error(
                    format!("Function call: expected {} arguments, found {}", expected, found),
                    location,
                )
            }
            SemanticError::ReturnTypeMismatch { expected, found, location } => {
                CompilerError::semantic_error(
                    format!("Return type mismatch: expected {}, found {}", expected, found),
                    location,
                )
            }
            SemanticError::InvalidLvalue { location } => {
                CompilerError::semantic_error(
                    "Invalid lvalue in assignment".to_string(),
                    location,
                )
            }
            SemanticError::RedefinedType { name } => {
                CompilerError::semantic_error(
                    format!("Redefinition of type: {}", name),
                    SourceLocation::new_simple(0, 0), // TODO: Track location
                )
            }
            SemanticError::IncompleteType { type_name, location } => {
                CompilerError::semantic_error(
                    format!("Incomplete type: {}", type_name),
                    location,
                )
            }
            SemanticError::StructTooLarge { struct_name, location } => {
                CompilerError::semantic_error(
                    format!("Struct '{}' is too large (overflow)", struct_name),
                    location,
                )
            }
            SemanticError::UndefinedMember { struct_name, member_name, location } => {
                CompilerError::semantic_error(
                    format!("Struct '{}' has no member '{}'", struct_name, member_name),
                    location,
                )
            }
            SemanticError::MemberAccessOnNonStruct { type_name, location } => {
                CompilerError::semantic_error(
                    format!("Member access on non-struct type: {}", type_name),
                    location,
                )
            }
            SemanticError::UndefinedType { name, location } => {
                CompilerError::semantic_error(
                    format!("Undefined type: '{}'", name),
                    location,
                )
            }
        }
    }
}