//! Error handling for the Ripple C99 compiler
//! 
//! This module defines common error types and error reporting utilities
//! used throughout the compiler.

use crate::source_loc::{SourceLocation, SourceSpan};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Main compiler error type that encompasses all phases of compilation
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CompilerError {
    #[error("Lexical error at {location}: {message}")]
    LexError {
        location: SourceLocation,
        message: String,
    },

    #[error("Parse error at {location}: {message}")]
    ParseError {
        location: SourceLocation,
        message: String,
    },

    #[error("Semantic error at {span}: {message}")]
    SemanticError {
        span: SourceSpan,
        message: String,
    },

    #[error("Type error at {span}: {message}")]
    TypeError {
        span: SourceSpan,
        message: String,
    },

    #[error("Code generation error at {location}: {message}")]
    CodegenError {
        location: SourceLocation,
        message: String,
    },
    
    #[error("Semantic error at {location}: {message}")]
    Semantic {
        location: SourceLocation,
        message: String,
    },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Internal compiler error: {message}")]
    InternalError { message: String },
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Note => write!(f, "note"),
        }
    }
}

/// A diagnostic message with location and severity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: SourceSpan,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(message: String, span: SourceSpan) -> Self {
        Self {
            severity: Severity::Error,
            message,
            span,
            notes: Vec::new(),
        }
    }

    pub fn warning(message: String, span: SourceSpan) -> Self {
        Self {
            severity: Severity::Warning,
            message,
            span,
            notes: Vec::new(),
        }
    }

    pub fn note(message: String, span: SourceSpan) -> Self {
        Self {
            severity: Severity::Note,
            message,
            span,
            notes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.severity, self.message)?;
        
        if !self.notes.is_empty() {
            for note in &self.notes {
                write!(f, "\n  note: {}", note)?;
            }
        }
        
        Ok(())
    }
}

/// Error reporter for collecting and displaying diagnostics
pub struct ErrorReporter {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
    warning_count: usize,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Report an error diagnostic
    pub fn error(&mut self, message: String, span: SourceSpan) -> &mut Diagnostic {
        let diagnostic = Diagnostic::error(message, span);
        self.diagnostics.push(diagnostic);
        self.error_count += 1;
        self.diagnostics.last_mut().unwrap()
    }

    /// Report a warning diagnostic
    pub fn warning(&mut self, message: String, span: SourceSpan) -> &mut Diagnostic {
        let diagnostic = Diagnostic::warning(message, span);
        self.diagnostics.push(diagnostic);
        self.warning_count += 1;
        self.diagnostics.last_mut().unwrap()
    }

    /// Report a note diagnostic
    pub fn note(&mut self, message: String, span: SourceSpan) -> &mut Diagnostic {
        let diagnostic = Diagnostic::note(message, span);
        self.diagnostics.push(diagnostic);
        self.diagnostics.last_mut().unwrap()
    }

    /// Check if any errors have been reported
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
        self.warning_count = 0;
    }

    /// Print all diagnostics to stderr
    pub fn print_diagnostics(&self) {
        for diagnostic in &self.diagnostics {
            eprintln!("{}", diagnostic);
        }
    }

    /// Create a summary string
    pub fn summary(&self) -> String {
        match (self.error_count, self.warning_count) {
            (0, 0) => "No errors or warnings".to_string(),
            (0, w) => format!("{} warning{}", w, if w == 1 { "" } else { "s" }),
            (e, 0) => format!("{} error{}", e, if e == 1 { "" } else { "s" }),
            (e, w) => format!(
                "{} error{} and {} warning{}",
                e,
                if e == 1 { "" } else { "s" },
                w,
                if w == 1 { "" } else { "s" }
            ),
        }
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerError {
    /// Create a lexer error
    pub fn lexer_error(message: String, location: SourceLocation) -> Self {
        CompilerError::LexError { location, message }
    }
    
    /// Create a parse error
    pub fn parse_error(message: String, location: SourceLocation) -> Self {
        CompilerError::ParseError { location, message }
    }
    
    /// Create a semantic error
    pub fn semantic_error(message: String, location: SourceLocation) -> Self {
        CompilerError::Semantic { location, message }
    }
    
    /// Create a codegen error
    pub fn codegen_error(message: String, location: SourceLocation) -> Self {
        CompilerError::CodegenError { location, message }
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for CompilerError {
    fn from(err: std::io::Error) -> Self {
        CompilerError::IoError {
            message: err.to_string(),
        }
    }
}

/// Convert from String (for simple error cases)
impl From<String> for CompilerError {
    fn from(message: String) -> Self {
        CompilerError::InternalError { message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let span = SourceSpan::new(
            SourceLocation::new("test.c", 1, 1),
            SourceLocation::new("test.c", 1, 5),
        );

        let diag = Diagnostic::error("Test error".to_string(), span.clone());
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Test error");
        assert_eq!(diag.span, span);
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();
        let span = SourceSpan::new(
            SourceLocation::new("test.c", 1, 1),
            SourceLocation::new("test.c", 1, 5),
        );

        assert!(!reporter.has_errors());
        assert_eq!(reporter.error_count(), 0);

        reporter.error("Test error".to_string(), span);
        assert!(reporter.has_errors());
        assert_eq!(reporter.error_count(), 1);
    }

    #[test]
    fn test_diagnostic_with_notes() {
        let span = SourceSpan::new(
            SourceLocation::new("test.c", 1, 1),
            SourceLocation::new("test.c", 1, 5),
        );

        let diag = Diagnostic::error("Test error".to_string(), span)
            .with_note("This is a note".to_string())
            .with_note("This is another note".to_string());

        assert_eq!(diag.notes.len(), 2);
        assert_eq!(diag.notes[0], "This is a note");
        assert_eq!(diag.notes[1], "This is another note");
    }

    #[test]
    fn test_summary() {
        let mut reporter = ErrorReporter::new();
        assert_eq!(reporter.summary(), "No errors or warnings");

        let span = SourceSpan::new(
            SourceLocation::new("test.c", 1, 1),
            SourceLocation::new("test.c", 1, 5),
        );

        reporter.error("Error 1".to_string(), span.clone());
        assert_eq!(reporter.summary(), "1 error");

        reporter.error("Error 2".to_string(), span.clone());
        assert_eq!(reporter.summary(), "2 errors");

        reporter.warning("Warning 1".to_string(), span);
        assert_eq!(reporter.summary(), "2 errors and 1 warning");
    }
}