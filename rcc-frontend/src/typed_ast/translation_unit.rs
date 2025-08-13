//! Typed translation unit and top-level definitions
//!
//! This module defines the top-level structure of a typed program.

use super::expressions::TypedExpr;
use super::statements::TypedStmt;
use crate::types::Type;

/// Typed function definition
#[derive(Debug, Clone)]
pub struct TypedFunction {
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<(String, Type)>,
    pub body: TypedStmt,
}

/// Typed top-level item
#[derive(Debug, Clone)]
pub enum TypedTopLevelItem {
    Function(TypedFunction),
    GlobalVariable {
        name: String,
        var_type: Type,
        initializer: Option<TypedExpr>,
    },
}

/// Typed translation unit (entire program)
#[derive(Debug, Clone)]
pub struct TypedTranslationUnit {
    pub items: Vec<TypedTopLevelItem>,
}