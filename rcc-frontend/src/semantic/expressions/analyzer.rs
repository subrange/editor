//! Main expression analyzer that coordinates all expression analysis

use crate::ast::*;
use crate::semantic::errors::SemanticError;
use crate::semantic::types::TypeAnalyzer;
use crate::Type;
use rcc_common::{CompilerError, SymbolId, SymbolTable};
use std::collections::HashMap;

use super::binary::BinaryOperationAnalyzer;
use super::initializers::InitializerAnalyzer;
use super::unary::UnaryOperationAnalyzer;

pub struct ExpressionAnalyzer<'a> {
    pub symbol_types: &'a HashMap<SymbolId, Type>,
    pub type_analyzer: TypeAnalyzer<'a>,
}

impl<'a> ExpressionAnalyzer<'a> {
    pub fn new(
        symbol_types: &'a HashMap<SymbolId, Type>,
        type_definitions: &'a HashMap<String, Type>,
    ) -> Self {
        Self {
            symbol_types,
            type_analyzer: TypeAnalyzer::new(type_definitions),
        }
    }

    /// Analyze an expression and infer its type
    pub fn analyze(
        &self,
        expr: &mut Expression,
        symbol_table: &mut SymbolTable,
    ) -> Result<(), CompilerError> {
        let expr_type = match &mut expr.kind {
            ExpressionKind::IntLiteral(_) => Type::Int,
            ExpressionKind::CharLiteral(_) => Type::Char,
            ExpressionKind::StringLiteral(_) => Type::Array {
                element_type: Box::new(Type::Char),
                size: None, // TODO: Calculate string length
            },

            ExpressionKind::Identifier { name, symbol_id } => {
                // Look up in symbol table
                if let Some(id) = symbol_table.lookup(name) {
                    *symbol_id = Some(id);
                    // Get the actual type from our type mapping
                    self.symbol_types.get(&id).cloned().unwrap_or(Type::Int)
                } else {
                    return Err(SemanticError::UndefinedVariable {
                        name: name.clone(),
                        location: expr.span.start.clone(),
                    }
                    .into());
                }
            }

            ExpressionKind::Binary { op, left, right } => {
                self.analyze(left, symbol_table)?;
                self.analyze(right, symbol_table)?;

                let binary_analyzer = BinaryOperationAnalyzer::new(&self.type_analyzer);
                binary_analyzer.analyze(*op, left, right)?
            }

            ExpressionKind::Unary { op, operand } => {
                self.analyze(operand, symbol_table)?;

                let unary_analyzer = UnaryOperationAnalyzer;
                unary_analyzer.analyze(*op, operand, symbol_table)?
            }

            ExpressionKind::Call {
                function,
                arguments,
            } => {
                self.analyze(function, symbol_table)?;

                for arg in arguments.iter_mut() {
                    self.analyze(arg, symbol_table)?;
                }

                // Check if function is callable
                if let Some(func_type) = &function.expr_type {
                    // Dereference function pointer to get the actual function type
                    let callable_type = match func_type {
                        Type::Pointer { target, .. } => target.as_ref(),
                        other => other,
                    };
                    
                    match callable_type {
                        Type::Function {
                            return_type,
                            parameters,
                            ..
                        } => {
                            // Check argument count
                            if arguments.len() != parameters.len() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    expected: parameters.len(),
                                    found: arguments.len(),
                                    location: expr.span.start.clone(),
                                }
                                .into());
                            }

                            // Check argument types
                            for (arg, param_type) in arguments.iter().zip(parameters.iter()) {
                                if let Some(arg_type) = &arg.expr_type {
                                    if !param_type.is_assignable_from(arg_type) {
                                        return Err(SemanticError::TypeMismatch {
                                            expected: param_type.clone(),
                                            found: arg_type.clone(),
                                            location: arg.span.start.clone(),
                                        }
                                        .into());
                                    }
                                }
                            }

                            *return_type.clone()
                        }
                        _ => {
                            return Err(SemanticError::InvalidFunctionCall {
                                function_type: func_type.clone(),
                                location: expr.span.start.clone(),
                            }
                            .into());
                        }
                    }
                } else {
                    Type::Error
                }
            }

            ExpressionKind::Member {
                object,
                member,
                is_pointer,
            } => {
                self.analyze(object, symbol_table)?;

                // Get the struct type from the object
                let struct_type = if *is_pointer {
                    // For arrow operator, dereference the pointer first
                    if let Some(obj_type) = &object.expr_type {
                        if let Some(target) = obj_type.pointer_target() {
                            target.clone()
                        } else {
                            return Err(SemanticError::InvalidOperation {
                                operation: "arrow operator on non-pointer".to_string(),
                                operand_type: obj_type.clone(),
                                location: object.span.start.clone(),
                            }
                            .into());
                        }
                    } else {
                        Type::Error
                    }
                } else {
                    // For dot operator, use the object type directly
                    object.expr_type.as_ref().unwrap_or(&Type::Error).clone()
                };

                // Look up the field type in the struct
                self.analyze_member_access(&struct_type, member, &expr.span.start)?
            }

            ExpressionKind::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.analyze(condition, symbol_table)?;
                self.analyze(then_expr, symbol_table)?;
                self.analyze(else_expr, symbol_table)?;

                self.check_boolean_context(condition)?;

                // Result type is the common type of then and else expressions
                if let (Some(then_type), Some(else_type)) =
                    (&then_expr.expr_type, &else_expr.expr_type)
                {
                    self.type_analyzer.common_type(then_type, else_type)
                } else {
                    Type::Error
                }
            }

            ExpressionKind::Cast {
                target_type,
                operand,
            } => {
                self.analyze(operand, symbol_table)?;
                target_type.clone()
            }

            ExpressionKind::SizeofExpr(operand) => {
                self.analyze(operand, symbol_table)?;
                Type::UnsignedLong // sizeof returns size_t, which is unsigned long on Ripple
            }

            ExpressionKind::SizeofType(_) => Type::UnsignedLong,

            ExpressionKind::CompoundLiteral {
                type_name,
                initializer,
            } => {
                self.analyze_initializer(initializer, type_name, symbol_table)?;
                type_name.clone()
            }
        };

        expr.expr_type = Some(expr_type);
        Ok(())
    }

    /// Analyze an initializer
    pub fn analyze_initializer(
        &self,
        init: &mut Initializer,
        expected_type: &Type,
        symbol_table: &mut SymbolTable,
    ) -> Result<(), CompilerError> {
        let initializer_analyzer = InitializerAnalyzer;
        initializer_analyzer.analyze(init, expected_type, symbol_table, &|expr, st| {
            self.analyze(expr, st)
        })
    }

    /// Check if expression is used in boolean context and can be converted
    pub fn check_boolean_context(&self, expr: &Expression) -> Result<(), CompilerError> {
        // In C, any scalar type can be used in boolean context
        if let Some(expr_type) = &expr.expr_type {
            match expr_type {
                Type::Void => Err(SemanticError::InvalidOperation {
                    operation: "boolean conversion".to_string(),
                    operand_type: expr_type.clone(),
                    location: expr.span.start.clone(),
                }
                .into()),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    /// Analyze member access on a struct type
    fn analyze_member_access(
        &self,
        struct_type: &Type,
        member: &str,
        location: &rcc_common::SourceLocation,
    ) -> Result<Type, CompilerError> {
        // First resolve the type if it's a struct reference
        let resolved_type = self.type_analyzer.resolve_type(struct_type);
        
        match &resolved_type {
            Type::Struct { fields, name, .. } => {
                // Find the field by name
                if let Some(field) = fields.iter().find(|f| f.name == member) {
                    // Resolve the field type in case it's a struct reference
                    Ok(self.type_analyzer.resolve_type(&field.field_type))
                } else {
                    // Use the actual struct name if available, otherwise use a generic description
                    let struct_name = name
                        .clone()
                        .unwrap_or_else(|| format!("{struct_type}"));
                    Err(SemanticError::UndefinedMember {
                        struct_name,
                        member_name: member.to_string(),
                        location: location.clone(),
                    }
                    .into())
                }
            }
            Type::Typedef(name) => {
                // Look up the typedef in type definitions
                let resolved_type = self.type_analyzer.resolve_type(&Type::Typedef(name.clone()));

                // Check if the type was actually resolved
                if let Type::Typedef(unresolved_name) = &resolved_type {
                    return Err(SemanticError::UndefinedType {
                        name: unresolved_name.clone(),
                        location: location.clone(),
                    }
                    .into());
                }

                match &resolved_type {
                    Type::Struct { fields, .. } => {
                        if let Some(field) = fields.iter().find(|f| f.name == member) {
                            // Resolve the field type in case it's a struct reference
                            Ok(self.type_analyzer.resolve_type(&field.field_type))
                        } else {
                            Err(SemanticError::UndefinedMember {
                                struct_name: name.clone(),
                                member_name: member.to_string(),
                                location: location.clone(),
                            }
                            .into())
                        }
                    }
                    _ => Err(SemanticError::InvalidOperation {
                        operation: "member access on non-struct".to_string(),
                        operand_type: resolved_type,
                        location: location.clone(),
                    }
                    .into()),
                }
            }
            _ => Err(SemanticError::InvalidOperation {
                operation: "member access on non-struct".to_string(),
                operand_type: struct_type.clone(),
                location: location.clone(),
            }
            .into()),
        }
    }
}