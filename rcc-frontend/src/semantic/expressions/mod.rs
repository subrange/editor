//! Expression semantic analysis
//!
//! This module handles type checking and semantic validation of expressions.

mod analyzer;
mod binary;
mod initializers;
mod unary;

// Re-export the main analyzer
pub use analyzer::ExpressionAnalyzer;