//! Ripple C99 Compiler - Common Types and Utilities
//! 
//! This crate contains shared types, error definitions, and utilities
//! used across all components of the Ripple C99 compiler.

pub mod error;
pub mod types;
pub mod source_loc;

pub use error::{CompilerError, ErrorReporter};
pub use types::*;
pub use source_loc::{SourceLocation, SourceSpan};