pub mod types;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod source_map;
pub mod expander;
pub mod expander_helpers;
pub mod expander_utils;
pub mod preprocessor;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use types::*;
pub use expander::MacroExpander;

/// Create a new macro expander instance
pub fn create_macro_expander() -> MacroExpander {
    MacroExpander::new()
}

/// Convenience function to expand a macro string with default options
pub fn expand(input: &str) -> MacroExpanderResult {
    let mut expander = create_macro_expander();
    expander.expand(input, MacroExpanderOptions::default())
}

/// Convenience function to expand a macro string with custom options
pub fn expand_with_options(input: &str, options: MacroExpanderOptions) -> MacroExpanderResult {
    let mut expander = create_macro_expander();
    expander.expand(input, options)
}