use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Option<Vec<String>>,
    pub body: String,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MacroExpansionErrorType {
    Undefined,
    ParameterMismatch,
    CircularDependency,
    SyntaxError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroExpansionError {
    #[serde(rename = "type")]
    pub error_type: MacroExpansionErrorType,
    pub message: String,
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MacroTokenType {
    MacroDefinition,
    MacroInvocation,
    BuiltinFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroToken {
    #[serde(rename = "type")]
    pub token_type: MacroTokenType,
    pub range: Range,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacroExpanderOptions {
    #[serde(default = "default_true")]
    pub strip_comments: bool,
    #[serde(default)]
    pub collapse_empty_lines: bool,
    #[serde(default)]
    pub generate_source_map: bool,
    #[serde(default)]
    pub enable_circular_dependency_detection: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroExpanderResult {
    pub expanded: String,
    pub errors: Vec<MacroExpansionError>,
    pub tokens: Vec<MacroToken>,
    pub macros: Vec<MacroDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<SourceMap>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionRange {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapEntry {
    pub expanded_range: PositionRange,
    pub source_range: PositionRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macro_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macro_call_site: Option<PositionRange>,
    pub expansion_depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macro_call_stack: Option<Vec<MacroCallStackEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroCallStackEntry {
    pub macro_name: String,
    pub call_site: PositionRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub version: u32,
    pub entries: Vec<SourceMapEntry>,
    #[serde(skip)]
    pub expanded_to_source: HashMap<String, Vec<SourceMapEntry>>,
    #[serde(skip)]
    pub source_to_expanded: HashMap<String, Vec<SourceMapEntry>>,
}