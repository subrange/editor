//! Typed AST representation
//! 
//! This module defines a typed version of the AST that is produced after
//! semantic analysis. The typed AST makes pointer arithmetic and other
//! type-dependent operations explicit, simplifying IR generation.

mod expressions;
mod statements;
mod translation_unit;
mod conversion;
mod errors;

// Re-export main types
pub use expressions::TypedExpr;
pub use statements::TypedStmt;
pub use translation_unit::{TypedFunction, TypedTopLevelItem, TypedTranslationUnit};
pub use conversion::{type_expression, type_statement, type_translation_unit, TypeEnvironment};
pub use errors::TypeError;