//! Statement semantic analysis
//! 
//! This module handles semantic analysis of statements and declarations.

use crate::StorageClass;
use crate::ast::*;
use crate::semantic::errors::SemanticError;
use crate::semantic::expressions::ExpressionAnalyzer;
use rcc_common::{CompilerError, SymbolTable, SymbolId, SourceLocation};
use std::collections::HashMap;
use crate::Type;

pub struct StatementAnalyzer<'a> {
    pub current_function: Option<Type>,
    pub symbol_locations: &'a mut HashMap<SymbolId, SourceLocation>,
    pub symbol_types: &'a mut HashMap<SymbolId, Type>,
    pub type_definitions: &'a mut HashMap<String, Type>,
}

impl<'a> StatementAnalyzer<'a> {
    /// Analyze a statement
    pub fn analyze_statement(
        &mut self,
        stmt: &mut Statement,
        symbol_table: &mut SymbolTable,
    ) -> Result<(), CompilerError> {
        match &mut stmt.kind {
            StatementKind::Expression(expr) => {
                let analyzer = ExpressionAnalyzer::new(
                    self.symbol_types,
                    self.type_definitions,
                );
                analyzer.analyze(expr, symbol_table)?;
            }
            
            StatementKind::Compound(statements) => {
                // Enter new scope for compound statement
                symbol_table.push_scope();
                
                for stmt in statements {
                    self.analyze_statement(stmt, symbol_table)?;
                }
                
                symbol_table.pop_scope();
            }
            
            StatementKind::Declaration { declarations } => {
                for decl in declarations {
                    self.analyze_declaration(decl, symbol_table)?;
                }
            }
            
            StatementKind::If { condition, then_stmt, else_stmt } => {
                let analyzer = ExpressionAnalyzer::new(
                    self.symbol_types,
                    self.type_definitions,
                );
                analyzer.analyze(condition, symbol_table)?;
                analyzer.check_boolean_context(condition)?;
                
                self.analyze_statement(then_stmt, symbol_table)?;
                
                if let Some(else_stmt) = else_stmt {
                    self.analyze_statement(else_stmt, symbol_table)?;
                }
            }
            
            StatementKind::While { condition, body } => {
                let analyzer = ExpressionAnalyzer::new(
                    self.symbol_types,
                    self.type_definitions,
                );
                analyzer.analyze(condition, symbol_table)?;
                analyzer.check_boolean_context(condition)?;
                
                self.analyze_statement(body, symbol_table)?;
            }
            
            StatementKind::For { init, condition, update, body } => {
                // Enter new scope for for-loop
                symbol_table.push_scope();
                
                if let Some(init) = init {
                    self.analyze_statement(init, symbol_table)?;
                }
                
                if let Some(condition) = condition {
                    let analyzer = ExpressionAnalyzer::new(
                        self.symbol_types,
                        self.type_definitions,
                    );
                    analyzer.analyze(condition, symbol_table)?;
                    analyzer.check_boolean_context(condition)?;
                }
                
                if let Some(update) = update {
                    let analyzer = ExpressionAnalyzer::new(
                        self.symbol_types,
                        self.type_definitions,
                    );
                    analyzer.analyze(update, symbol_table)?;
                }
                
                self.analyze_statement(body, symbol_table)?;
                
                symbol_table.pop_scope();
            }
            
            StatementKind::DoWhile { body, condition } => {
                self.analyze_statement(body, symbol_table)?;
                
                let analyzer = ExpressionAnalyzer::new(
                    self.symbol_types,
                    self.type_definitions,
                );
                analyzer.analyze(condition, symbol_table)?;
                analyzer.check_boolean_context(condition)?;
            }
            
            StatementKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let analyzer = ExpressionAnalyzer::new(
                        self.symbol_types,
                        self.type_definitions,
                    );
                    analyzer.analyze(expr, symbol_table)?;
                    
                    // Check return type compatibility
                    if let Some(expected_return_type) = &self.current_function {
                        if let Some(expr_type) = &expr.expr_type {
                            if !expected_return_type.is_assignable_from(expr_type) {
                                return Err(SemanticError::ReturnTypeMismatch {
                                    expected: expected_return_type.clone(),
                                    found: expr_type.clone(),
                                    location: expr.span.start.clone(),
                                }.into());
                            }
                        }
                    }
                } else {
                    // Return with no value - check if function returns void
                    if let Some(expected_return_type) = &self.current_function {
                        if !matches!(expected_return_type, Type::Void) {
                            return Err(SemanticError::ReturnTypeMismatch {
                                expected: expected_return_type.clone(),
                                found: Type::Void,
                                location: stmt.span.start.clone(),
                            }.into());
                        }
                    }
                }
            }
            
            StatementKind::Break | StatementKind::Continue | StatementKind::Empty => {
                // No semantic analysis needed
            }
            
            StatementKind::InlineAsm { assembly: _ } => {
                // Inline assembly - no semantic analysis needed for now
                // The assembly code will be passed through directly to the backend
            }
            
            // TODO: Handle other statement types
            _ => {
                panic!("Unhandled statement kind: {:?}", stmt.kind);
            }
        }
        
        Ok(())
    }
    
    /// Analyze a declaration
    pub fn analyze_declaration(
        &mut self,
        decl: &mut Declaration,
        symbol_table: &mut SymbolTable,
    ) -> Result<(), CompilerError> {
        // Handle typedef specially - it defines a type alias, not a variable
        if decl.storage_class == StorageClass::Typedef {
            // Resolve the type first
            let analyzer = crate::semantic::types::TypeAnalyzer::new(self.type_definitions);
            decl.decl_type = analyzer.resolve_type(&decl.decl_type);
            // Register this as a type definition
            self.type_definitions.insert(decl.name.clone(), decl.decl_type.clone());
            return Ok(());
        }
        
        // Check for redefinition in current scope
        if symbol_table.exists_in_current_scope(&decl.name) {
            return Err(SemanticError::RedefinedSymbol {
                name: decl.name.clone(),
                original_location: SourceLocation::new_simple(0, 0), // TODO: Track original location
                redefinition_location: decl.span.start.clone(),
            }.into());
        }
        
        // Resolve the type (in case it references a named struct/union/enum or typedef)
        let analyzer = crate::semantic::types::TypeAnalyzer::new(self.type_definitions);
        decl.decl_type = analyzer.resolve_type(&decl.decl_type);
        
        // Add to symbol table
        let symbol_id = symbol_table.add_symbol(decl.name.clone());
        decl.symbol_id = Some(symbol_id);
        self.symbol_locations.insert(symbol_id, decl.span.start.clone());
        // Store the variable type
        self.symbol_types.insert(symbol_id, decl.decl_type.clone());
        
        // Analyze initializer if present
        if let Some(initializer) = &mut decl.initializer {
            let expr_analyzer = ExpressionAnalyzer::new(
                self.symbol_types,
                self.type_definitions,
            );
            expr_analyzer.analyze_initializer(initializer, &decl.decl_type, symbol_table)?;
        }
        
        Ok(())
    }
}