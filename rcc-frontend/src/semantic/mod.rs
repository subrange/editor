//! Semantic Analysis for C99
//! 
//! Performs type checking, symbol resolution, and semantic validation
//! on the AST produced by the parser.

pub mod errors;
pub mod types;
pub mod expressions;
pub mod statements;
pub mod symbols;

use crate::ast::*;
use rcc_common::{CompilerError, SymbolTable, SymbolId, SourceLocation};
use std::collections::HashMap;

pub use errors::SemanticError;
use crate::Type;

/// Semantic analyzer context
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    current_function: Option<Type>, // Current function's return type
    symbol_locations: HashMap<SymbolId, SourceLocation>, // For error reporting
    symbol_types: HashMap<SymbolId, Type>, // Type information for each symbol
    type_definitions: HashMap<String, Type>, // Named type definitions (structs, unions, enums)
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            current_function: None,
            symbol_locations: HashMap::new(),
            symbol_types: HashMap::new(),
            type_definitions: HashMap::new(),
        }
    }
    
    /// Analyze a translation unit
    pub fn analyze(&mut self, ast: &mut TranslationUnit) -> Result<(), CompilerError> {
        // First pass: collect all function and global variable declarations
        for item in &mut ast.items {
            match item {
                TopLevelItem::Function(func) => {
                    let mut symbol_manager = symbols::SymbolManager {
                        symbol_table: &mut self.symbol_table,
                        symbol_locations: &mut self.symbol_locations,
                        symbol_types: &mut self.symbol_types,
                        type_definitions: &mut self.type_definitions,
                    };
                    symbol_manager.declare_function(func)?;
                }
                TopLevelItem::Declaration(decl) => {
                    let mut symbol_manager = symbols::SymbolManager {
                        symbol_table: &mut self.symbol_table,
                        symbol_locations: &mut self.symbol_locations,
                        symbol_types: &mut self.symbol_types,
                        type_definitions: &mut self.type_definitions,
                    };
                    symbol_manager.declare_global_variable(decl)?;
                }
                TopLevelItem::TypeDefinition { name, type_def, .. } => {
                    let mut symbol_manager = symbols::SymbolManager {
                        symbol_table: &mut self.symbol_table,
                        symbol_locations: &mut self.symbol_locations,
                        symbol_types: &mut self.symbol_types,
                        type_definitions: &mut self.type_definitions,
                    };
                    symbol_manager.register_type_definition(name.clone(), type_def.clone())?;
                }
            }
        }
        
        // Second pass: analyze function bodies
        for item in &mut ast.items {
            match item {
                TopLevelItem::Function(func) => {
                    self.analyze_function(func)?;
                }
                _ => {} // Already handled in first pass
            }
        }
        
        Ok(())
    }
    
    /// Analyze a function definition
    fn analyze_function(&mut self, func: &mut FunctionDefinition) -> Result<(), CompilerError> {
        // Set current function context
        self.current_function = Some(func.return_type.clone());
        
        // Enter function scope
        self.symbol_table.push_scope();
        
        // Add parameters to scope
        let mut symbol_manager = symbols::SymbolManager {
            symbol_table: &mut self.symbol_table,
            symbol_locations: &mut self.symbol_locations,
            symbol_types: &mut self.symbol_types,
            type_definitions: &mut self.type_definitions,
        };
        symbol_manager.add_function_parameters(&mut func.parameters)?;
        
        // Analyze function body
        let mut stmt_analyzer = statements::StatementAnalyzer {
            current_function: self.current_function.clone(),
            symbol_locations: &mut self.symbol_locations,
            symbol_types: &mut self.symbol_types,
            type_definitions: &mut self.type_definitions,
        };
        stmt_analyzer.analyze_statement(&mut func.body, &mut self.symbol_table)?;
        
        // Exit function scope
        self.symbol_table.pop_scope();
        self.current_function = None;
        
        Ok(())
    }
    
    /// Get the symbol types map (for typed AST conversion)
    pub fn into_symbol_types(self) -> HashMap<SymbolId, Type> {
        self.symbol_types
    }
    
    /// Get the type definitions map (for typed AST conversion)
    pub fn into_type_definitions(self) -> HashMap<String, Type> {
        self.type_definitions
    }
    
    /// Get both symbol types and type definitions (consumes the analyzer)
    pub fn into_type_info(self) -> (HashMap<SymbolId, Type>, HashMap<String, Type>) {
        (self.symbol_types, self.type_definitions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Frontend, TopLevelItem};

    #[test]
    fn test_semantic_analysis_simple_function() {
        let source = r#"
int main() {
    return 42;
}
"#;
        
        let mut ast = Frontend::parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        
        let result = analyzer.analyze(&mut ast);
        assert!(result.is_ok());
        
        // Check that function was analyzed
        match &ast.items[0] {
            TopLevelItem::Function(func) => {
                // Check that the return statement was analyzed
                match &func.body.kind {
                    StatementKind::Compound(statements) => {
                        match &statements[0].kind {
                            StatementKind::Return(Some(expr)) => {
                                assert_eq!(expr.expr_type, Some(Type::Int));
                            }
                            _ => panic!("Expected return statement"),
                        }
                    }
                    _ => panic!("Expected compound statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_semantic_analysis_undefined_variable() {
        let source = r#"
int main() {
    return x;  // undefined variable
}
"#;
        
        let mut ast = Frontend::parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        
        let result = analyzer.analyze(&mut ast);
        assert!(result.is_err());
        
        // Should be an undefined variable error
        match result.unwrap_err() {
            CompilerError::Semantic { message, .. } => {
                assert!(message.contains("Undefined variable"));
            }
            _ => panic!("Expected semantic error"),
        }
    }

    #[test]
    fn test_semantic_analysis_variable_declaration() {
        let source = r#"
int main() {
    int x = 42;
    return x;
}
"#;
        
        let mut ast = Frontend::parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        
        let result = analyzer.analyze(&mut ast);
        assert!(result.is_ok());
        
        // Variable should be resolved
        match &ast.items[0] {
            TopLevelItem::Function(func) => {
                match &func.body.kind {
                    StatementKind::Compound(statements) => {
                        // Check declaration
                        match &statements[0].kind {
                            StatementKind::Declaration { declarations } => {
                                assert!(declarations[0].symbol_id.is_some());
                            }
                            _ => panic!("Expected declaration statement"),
                        }
                        
                        // Check return statement uses the variable
                        match &statements[1].kind {
                            StatementKind::Return(Some(expr)) => {
                                match &expr.kind {
                                    ExpressionKind::Identifier { symbol_id, .. } => {
                                        assert!(symbol_id.is_some());
                                    }
                                    _ => panic!("Expected identifier in return"),
                                }
                            }
                            _ => panic!("Expected return statement"),
                        }
                    }
                    _ => panic!("Expected compound statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }
}