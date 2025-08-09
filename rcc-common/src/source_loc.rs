//! Source location tracking for error reporting
//! 
//! This module provides types for tracking locations in source files,
//! which is essential for good error messages and debugging support.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A location in a source file (line and column are 1-based)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub filename: String,
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    /// Create a location with filename
    pub fn new(filename: &str, line: u32, column: u32) -> Self {
        Self {
            filename: filename.to_string(),
            line,
            column,
        }
    }
    
    /// Create a dummy location for testing
    pub fn dummy() -> Self {
        Self::new("<unknown>", 0, 0)
    }
}

// Allow creating location with just line and column (common pattern in tests)
impl SourceLocation {
    pub fn new_simple(line: u32, column: u32) -> Self {
        Self {
            filename: "<input>".to_string(),
            line,
            column,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.column)
    }
}

/// A span in a source file (from start to end location)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }
    
    /// Create a span from a single location
    pub fn from_location(location: SourceLocation) -> Self {
        Self {
            end: location.clone(),
            start: location,
        }
    }
    
    /// Create a dummy span for testing
    pub fn dummy() -> Self {
        Self::from_location(SourceLocation::dummy())
    }
    
    /// Check if this span is in the same file as another
    pub fn same_file(&self, other: &SourceSpan) -> bool {
        self.start.filename == other.start.filename
    }
    
    /// Extend this span to include another span
    pub fn extend(&self, other: &SourceSpan) -> SourceSpan {
        if !self.same_file(other) {
            return self.clone();
        }
        
        let start = if self.start.line < other.start.line
            || (self.start.line == other.start.line && self.start.column <= other.start.column)
        {
            self.start.clone()
        } else {
            other.start.clone()
        };
        
        let end = if self.end.line > other.end.line
            || (self.end.line == other.end.line && self.end.column >= other.end.column)
        {
            self.end.clone()
        } else {
            other.end.clone()
        };
        
        SourceSpan::new(start, end)
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.filename != self.end.filename {
            write!(f, "{} to {}", self.start, self.end)
        } else if self.start.line == self.end.line {
            if self.start.column == self.end.column {
                write!(f, "{}:{}", self.start.filename, self.start.line)
            } else {
                write!(
                    f,
                    "{}:{}:{}-{}",
                    self.start.filename, self.start.line, self.start.column, self.end.column
                )
            }
        } else {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.start.filename, self.start.line, self.start.column, self.end.line, self.end.column
            )
        }
    }
}

/// Trait for types that have a source location
pub trait HasSpan {
    fn span(&self) -> SourceSpan;
}

/// Helper for creating source locations during parsing
#[derive(Debug, Clone)]
pub struct SourceTracker {
    filename: String,
    line: u32,
    column: u32,
}

impl SourceTracker {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            line: 1,
            column: 1,
        }
    }
    
    /// Get current location
    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(&self.filename, self.line, self.column)
    }
    
    /// Advance by one character
    pub fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
    
    /// Advance by a string
    pub fn advance_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.advance(ch);
        }
    }
    
    /// Create a span from a start location to current location
    pub fn span_from(&self, start: SourceLocation) -> SourceSpan {
        SourceSpan::new(start, self.location())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new("test.c", 42, 10);
        assert_eq!(loc.filename, "test.c");
        assert_eq!(loc.line, 42);
        assert_eq!(loc.column, 10);
        assert_eq!(format!("{}", loc), "test.c:42:10");
    }

    #[test]
    fn test_source_span_same_line() {
        let start = SourceLocation::new("test.c", 1, 5);
        let end = SourceLocation::new("test.c", 1, 10);
        let span = SourceSpan::new(start, end);
        
        assert_eq!(format!("{}", span), "test.c:1:5-10");
    }

    #[test]
    fn test_source_span_different_lines() {
        let start = SourceLocation::new("test.c", 1, 5);
        let end = SourceLocation::new("test.c", 3, 10);
        let span = SourceSpan::new(start, end);
        
        assert_eq!(format!("{}", span), "test.c:1:5-3:10");
    }

    #[test]
    fn test_source_span_extend() {
        let span1 = SourceSpan::new(
            SourceLocation::new("test.c", 1, 5),
            SourceLocation::new("test.c", 1, 10),
        );
        let span2 = SourceSpan::new(
            SourceLocation::new("test.c", 1, 8),
            SourceLocation::new("test.c", 2, 5),
        );
        
        let extended = span1.extend(&span2);
        assert_eq!(extended.start.line, 1);
        assert_eq!(extended.start.column, 5);
        assert_eq!(extended.end.line, 2);
        assert_eq!(extended.end.column, 5);
    }

    #[test]
    fn test_source_tracker() {
        let mut tracker = SourceTracker::new("test.c");
        
        let start_loc = tracker.location();
        assert_eq!(start_loc.line, 1);
        assert_eq!(start_loc.column, 1);
        
        tracker.advance('h');
        tracker.advance('i');
        tracker.advance('\n');
        tracker.advance('t');
        
        let end_loc = tracker.location();
        assert_eq!(end_loc.line, 2);
        assert_eq!(end_loc.column, 2);
        
        let span = tracker.span_from(start_loc);
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        assert_eq!(span.end.line, 2);
        assert_eq!(span.end.column, 2);
    }

    #[test]
    fn test_source_tracker_advance_str() {
        let mut tracker = SourceTracker::new("test.c");
        
        let start_loc = tracker.location();
        tracker.advance_str("hello\nworld");
        
        let end_loc = tracker.location();
        assert_eq!(end_loc.line, 2);
        assert_eq!(end_loc.column, 6); // "world" is 5 chars + 1 for 1-based
    }
}