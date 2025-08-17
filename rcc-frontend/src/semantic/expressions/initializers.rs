//! Initializer and compound literal analysis

use std::cell::RefCell;
use crate::ast::*;
use crate::semantic::errors::SemanticError;
use crate::Type;
use rcc_common::{CompilerError};
use std::rc::Rc;
use crate::semantic::expressions::ExpressionAnalyzer;
use crate::semantic::types::TypeAnalyzer;

pub struct InitializerAnalyzer {
    expression_analyzer: Rc<RefCell<ExpressionAnalyzer>>,
    type_analyzer: Rc<RefCell<TypeAnalyzer>>
}

impl InitializerAnalyzer {
    pub fn new(
        expression_analyzer: Rc<RefCell<ExpressionAnalyzer>>,
        type_analyzer: Rc<RefCell<TypeAnalyzer>>) -> Self {
        Self {
            expression_analyzer,
            type_analyzer
        }
    }
    
    /// Analyze an initializer
    pub fn analyze(
        &self,
        init: &mut Initializer,
        expected_type: &Type,
    ) -> Result<(), CompilerError>
    {
        match &mut init.kind {
            InitializerKind::Expression(expr) => {
                self.expression_analyzer.borrow().analyze(expr)?;
        
                // Check type compatibility with typedef awareness
                if let Some(expr_type) = &expr.expr_type {
                    // Special case: Allow 0 to initialize pointers (NULL)
                    let is_null_init = matches!(expected_type, Type::Pointer { .. })
                        && self.type_analyzer.borrow().is_integer(expr_type)
                        && matches!(expr.kind, ExpressionKind::IntLiteral(0));
                    
                    // Use typedef-aware type compatibility checking
                    if !is_null_init && !self.type_analyzer.borrow().is_assignable(expected_type, expr_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: expr_type.clone(),
                            location: expr.span.start.clone(),
                        }
                        .into());
                    }
                }
                
                Ok(())
            }
        
            InitializerKind::List(initializers) => {
                // Handle array/struct initialization
                match expected_type {
                    Type::Array { element_type, .. } => {
                        for init in initializers {
                            self.analyze(init, element_type)?;
                        }
                        Ok(())
                    }
                    Type::Struct { fields, .. } => {
                        // Match initializers to fields
                        for (init, field) in initializers.iter_mut().zip(fields.iter()) {
                            self.analyze(init, &field.field_type)?;
                        }
                        Ok(())
                    }
                    _ => {
                        Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: Type::Error, // Placeholder
                            location: init.span.start.clone(),
                        }
                        .into())
                    }
                }
            }
        
            InitializerKind::Designated { .. } => {
                // TODO: Handle designated initializers
                Err(CompilerError::from(SemanticError::NotImplemented {
                    feature: "designated initializers".to_string(),
                    location: init.span.start.clone(),
                }))
            }
        }
    }
}