//! Initializer and compound literal analysis

use crate::ast::*;
use crate::semantic::errors::SemanticError;
use crate::Type;
use rcc_common::{CompilerError, SymbolTable};

pub struct InitializerAnalyzer;

impl InitializerAnalyzer {
    /// Analyze an initializer
    pub fn analyze<F>(
        &self,
        init: &mut Initializer,
        expected_type: &Type,
        symbol_table: &mut SymbolTable,
        analyze_expr: &F,
    ) -> Result<(), CompilerError>
    where
        F: Fn(&mut Expression, &mut SymbolTable) -> Result<(), CompilerError>,
    {
        match &mut init.kind {
            InitializerKind::Expression(expr) => {
                analyze_expr(expr, symbol_table)?;

                // Check type compatibility
                if let Some(expr_type) = &expr.expr_type {
                    // Special case: Allow 0 to initialize pointers (NULL)
                    let is_null_init = matches!(expected_type, Type::Pointer { .. })
                        && expr_type.is_integer()
                        && matches!(expr.kind, ExpressionKind::IntLiteral(0));
                    
                    if !is_null_init && !expected_type.is_assignable_from(expr_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: expr_type.clone(),
                            location: expr.span.start.clone(),
                        }
                        .into());
                    }
                }
            }

            InitializerKind::List(initializers) => {
                // Handle array/struct initialization
                match expected_type {
                    Type::Array { element_type, .. } => {
                        for init in initializers {
                            self.analyze(init, element_type, symbol_table, analyze_expr)?;
                        }
                    }
                    Type::Struct { fields, .. } => {
                        // Match initializers to fields
                        for (init, field) in initializers.iter_mut().zip(fields.iter()) {
                            self.analyze(init, &field.field_type, symbol_table, analyze_expr)?;
                        }
                    }
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: Type::Error, // Placeholder
                            location: init.span.start.clone(),
                        }
                        .into());
                    }
                }
            }

            InitializerKind::Designated { .. } => {
                // TODO: Handle designated initializers
            }
        }

        Ok(())
    }
}