//! AST to Typed AST conversion
//!
//! This module implements the conversion from the untyped AST to the typed AST.

use super::expressions::TypedExpr;
use super::statements::TypedStmt;
use super::translation_unit::{TypedFunction, TypedTopLevelItem, TypedTranslationUnit};
use super::errors::TypeError;
use crate::types::{Type, BankTag};
use crate::type_checker::{TypeChecker, TypedBinaryOp};
use crate::ast::{Initializer, InitializerKind};
use rcc_common::SymbolId;
use std::collections::HashMap;

/// Type environment for looking up variable types
pub struct TypeEnvironment {
    /// Maps symbol IDs to their types (from semantic analysis)
    pub symbol_types: HashMap<SymbolId, Type>,
    /// Maps typedef names to their underlying types
    pub type_definitions: HashMap<String, Type>,
}

impl TypeEnvironment {
    /// Create a new type environment with symbol and type information
    pub fn new(
        symbol_types: HashMap<SymbolId, Type>,
        type_definitions: HashMap<String, Type>,
    ) -> Self {
        Self {
            symbol_types,
            type_definitions,
        }
    }
    
    /// Look up the type of a symbol by its ID
    pub fn lookup_type(&self, symbol_id: SymbolId) -> Option<&Type> {
        self.symbol_types.get(&symbol_id)
    }
    
    /// Resolve a typedef name to its underlying type
    pub fn resolve_typedef(&self, name: &str) -> Option<&Type> {
        self.type_definitions.get(name)
    }
    
    /// Check if a name is a typedef
    pub fn is_typedef(&self, name: &str) -> bool {
        self.type_definitions.contains_key(name)
    }
}

/// Convert an initializer to a typed expression
fn type_initializer(
    init: &Initializer,
    expected_type: &Type,
    type_env: &TypeEnvironment,
) -> Result<TypedExpr, TypeError> {
    match &init.kind {
        InitializerKind::Expression(expr) => {
            // Single expression initializer
            // Special case: string literal initializing a char array
            if let crate::ast::ExpressionKind::StringLiteral(s) = &expr.kind {
                if let Type::Array { element_type, .. } = expected_type {
                    if matches!(**element_type, Type::Char) {
                        // Convert string to array of character literals
                        let mut chars = Vec::new();
                        for ch in s.bytes() {
                            chars.push(TypedExpr::CharLiteral {
                                value: ch,
                                expr_type: Type::Char,
                            });
                        }
                        // Add null terminator
                        chars.push(TypedExpr::CharLiteral {
                            value: 0,
                            expr_type: Type::Char,
                        });
                        
                        return Ok(TypedExpr::ArrayInitializer {
                            elements: chars,
                            expr_type: expected_type.clone(),
                        });
                    }
                }
            }
            
            // Otherwise, process as normal expression
            type_expression(expr, type_env)
        }
        InitializerKind::List(initializers) => {
            // List initializer - for arrays
            match expected_type {
                Type::Array { element_type, size } => {
                    // Check we don't have too many initializers if size is known
                    if let Some(array_size) = size {
                        if initializers.len() as u64 > *array_size {
                            return Err(TypeError::TypeMismatch(format!(
                                "Too many initializers for array of size {}", array_size
                            )));
                        }
                    }
                    
                    // Type each element
                    let mut typed_elements = Vec::new();
                    for init in initializers {
                        let typed_elem = type_initializer(init, element_type, type_env)?;
                        typed_elements.push(typed_elem);
                    }
                    
                    Ok(TypedExpr::ArrayInitializer {
                        elements: typed_elements,
                        expr_type: expected_type.clone(),
                    })
                }
                _ => {
                    Err(TypeError::TypeMismatch(format!(
                        "List initializer not supported for type {:?}", expected_type
                    )))
                }
            }
        }
        InitializerKind::Designated { .. } => {
            Err(TypeError::UnsupportedConstruct(
                "Designated initializers not yet supported".to_string()
            ))
        }
    }
}

/// Convert an untyped AST expression to a typed expression
/// This is the main entry point for the typing phase
pub fn type_expression(
    expr: &crate::ast::Expression,
    type_env: &TypeEnvironment,
) -> Result<TypedExpr, TypeError> {
    use crate::ast::ExpressionKind;
    
    match &expr.kind {
        ExpressionKind::IntLiteral(value) => {
            Ok(TypedExpr::IntLiteral {
                value: *value,
                expr_type: expr.expr_type.clone().unwrap_or(Type::Int),
            })
        }
        
        ExpressionKind::CharLiteral(value) => {
            Ok(TypedExpr::CharLiteral {
                value: *value,
                expr_type: Type::Char,
            })
        }
        
        ExpressionKind::StringLiteral(value) => {
            Ok(TypedExpr::StringLiteral {
                value: value.clone(),
                expr_type: Type::Pointer {
                    target: Box::new(Type::Char),
                    bank: Some(BankTag::Global),
                },
            })
        }
        
        ExpressionKind::Identifier { name, symbol_id } => {
            // First try to look up the type using the symbol_id from semantic analysis
            let var_type = if let Some(id) = symbol_id {
                type_env.lookup_type(*id)
                    .cloned()
                    .or_else(|| expr.expr_type.clone())
                    .ok_or_else(|| TypeError::UndefinedVariable(name.clone()))?
            } else {
                // Fallback to expr_type if no symbol_id (shouldn't happen after semantic analysis)
                expr.expr_type.clone()
                    .ok_or_else(|| TypeError::UndefinedVariable(name.clone()))?
            };
            
            Ok(TypedExpr::Variable {
                name: name.clone(),
                symbol_id: *symbol_id,
                expr_type: var_type,
            })
        }
        
        ExpressionKind::Binary { op, left, right } => {
            // Use the type checker to classify the operation
            let typed_op = TypeChecker::check_binary_op(
                *op,
                left,
                right,
                expr.span.start.clone(),
            ).map_err(|msg| TypeError::TypeMismatch(msg))?;
            
            match typed_op {
                TypedBinaryOp::IntegerArithmetic { op, result_type } => {
                    let left_typed = type_expression(left, type_env)?;
                    let right_typed = type_expression(right, type_env)?;
                    
                    Ok(TypedExpr::Binary {
                        op,
                        left: Box::new(left_typed),
                        right: Box::new(right_typed),
                        expr_type: result_type,
                    })
                }
                
                TypedBinaryOp::PointerOffset { ptr_type, elem_type, elem_size: _, is_add } => {
                    // Determine which operand is the pointer
                    let (ptr_expr, offset_expr) = if left.expr_type.as_ref()
                        .map_or(false, |t| t.is_pointer()) {
                        (left, right)
                    } else {
                        (right, left)
                    };
                    
                    let ptr_typed = type_expression(ptr_expr, type_env)?;
                    let offset_typed = type_expression(offset_expr, type_env)?;
                    
                    Ok(TypedExpr::PointerArithmetic {
                        ptr: Box::new(ptr_typed),
                        offset: Box::new(offset_typed),
                        elem_type,
                        is_add,
                        expr_type: ptr_type,
                    })
                }
                
                TypedBinaryOp::PointerDifference { elem_type, elem_size: _ } => {
                    let left_typed = type_expression(left, type_env)?;
                    let right_typed = type_expression(right, type_env)?;
                    
                    Ok(TypedExpr::PointerDifference {
                        left: Box::new(left_typed),
                        right: Box::new(right_typed),
                        elem_type,
                        expr_type: Type::Int,  // Pointer difference returns integer
                    })
                }
                
                TypedBinaryOp::ArrayIndex { elem_type, elem_size: _ } => {
                    let array_typed = type_expression(left, type_env)?;
                    let index_typed = type_expression(right, type_env)?;
                    
                    Ok(TypedExpr::ArrayIndex {
                        array: Box::new(array_typed),
                        index: Box::new(index_typed),
                        elem_type: elem_type.clone(),
                        expr_type: elem_type,
                    })
                }
                
                TypedBinaryOp::Comparison { op, is_pointer_compare: _ } => {
                    let left_typed = type_expression(left, type_env)?;
                    let right_typed = type_expression(right, type_env)?;
                    
                    Ok(TypedExpr::Binary {
                        op,
                        left: Box::new(left_typed),
                        right: Box::new(right_typed),
                        expr_type: Type::Bool,  // Comparisons return bool
                    })
                }
                
                TypedBinaryOp::Logical { op } => {
                    let left_typed = type_expression(left, type_env)?;
                    let right_typed = type_expression(right, type_env)?;
                    
                    Ok(TypedExpr::Binary {
                        op,
                        left: Box::new(left_typed),
                        right: Box::new(right_typed),
                        expr_type: Type::Bool,
                    })
                }
                
                TypedBinaryOp::Assignment { lhs_type } => {
                    let lhs_typed = type_expression(left, type_env)?;
                    let rhs_typed = type_expression(right, type_env)?;
                    
                    Ok(TypedExpr::Assignment {
                        lhs: Box::new(lhs_typed),
                        rhs: Box::new(rhs_typed),
                        expr_type: lhs_type,
                    })
                }
                
                TypedBinaryOp::CompoundAssignment { op, lhs_type, is_pointer } => {
                    let lhs_typed = type_expression(left, type_env)?;
                    let rhs_typed = type_expression(right, type_env)?;
                    
                    Ok(TypedExpr::CompoundAssignment {
                        op,
                        lhs: Box::new(lhs_typed),
                        rhs: Box::new(rhs_typed),
                        is_pointer,
                        expr_type: lhs_type,
                    })
                }
            }
        }
        
        ExpressionKind::Unary { op, operand } => {
            let operand_typed = type_expression(operand, type_env)?;
            let result_type = expr.expr_type.clone()
                .ok_or_else(|| TypeError::TypeMismatch("Unary expression has no type".to_string()))?;
            
            Ok(TypedExpr::Unary {
                op: *op,
                operand: Box::new(operand_typed),
                expr_type: result_type,
            })
        }
        
        ExpressionKind::Call { function, arguments } => {
            let func_typed = type_expression(function, type_env)?;
            let args_typed: Result<Vec<_>, _> = arguments.iter()
                .map(|arg| type_expression(arg, type_env))
                .collect();
            
            let result_type = expr.expr_type.clone()
                .ok_or_else(|| TypeError::TypeMismatch("Call expression has no type".to_string()))?;
            
            Ok(TypedExpr::Call {
                function: Box::new(func_typed),
                arguments: args_typed?,
                expr_type: result_type,
            })
        }
        
        ExpressionKind::Cast { target_type, operand } => {
            let operand_typed = type_expression(operand, type_env)?;
            
            Ok(TypedExpr::Cast {
                operand: Box::new(operand_typed),
                target_type: target_type.clone(),
                expr_type: target_type.clone(),
            })
        }
        
        ExpressionKind::SizeofExpr(operand) => {
            let operand_typed = type_expression(operand, type_env)?;
            
            Ok(TypedExpr::SizeofExpr {
                operand: Box::new(operand_typed),
                expr_type: Type::Int,  // sizeof returns size_t, which we treat as int
            })
        }
        
        ExpressionKind::SizeofType(target_type) => {
            Ok(TypedExpr::SizeofType {
                target_type: target_type.clone(),
                expr_type: Type::Int,
            })
        }
        
        ExpressionKind::Member { object, member, is_pointer } => {
            // Get the typed object expression
            let object_typed = type_expression(object, type_env)?;
            
            // Determine the struct type from the object
            let struct_type = if *is_pointer {
                // For -> operator, object should be a pointer to struct
                match object_typed.get_type() {
                    Type::Pointer { target, .. } => target.as_ref(),
                    _ => return Err(TypeError::TypeMismatch(
                        format!("-> operator requires pointer to struct, got {:?}", object_typed.get_type())
                    ))
                }
            } else {
                // For . operator, object should be a struct
                object_typed.get_type()
            };
            
            // Extract struct fields
            let fields = match struct_type {
                Type::Struct { fields, name, .. } => {
                    if fields.is_empty() && name.is_some() {
                        // This is a reference to a named struct, look it up
                        if let Some(Type::Struct { fields, .. }) = type_env.type_definitions.get(name.as_ref().unwrap()) {
                            fields
                        } else {
                            return Err(TypeError::UndefinedType(
                                format!("Struct type '{}' not found", name.as_ref().unwrap())
                            ));
                        }
                    } else {
                        fields
                    }
                }
                _ => return Err(TypeError::TypeMismatch(
                    format!("Member access requires struct type, got {:?}", struct_type)
                ))
            };
            
            // Calculate struct layout to get field offset
            // Pass type_definitions to resolve nested struct sizes
            let layout = crate::semantic::struct_layout::calculate_struct_layout_with_defs(
                fields,
                rcc_common::SourceLocation::new_simple(0, 0), // TODO: Use actual location
                Some(&type_env.type_definitions)
            ).map_err(|e| TypeError::TypeMismatch(format!("Failed to calculate struct layout: {}", e)))?;
            
            // Find the field and get its offset
            let field_layout = crate::semantic::struct_layout::find_field(&layout, member)
                .ok_or_else(|| TypeError::UndefinedMember {
                    struct_name: format!("{:?}", struct_type),
                    member_name: member.clone(),
                })?;
            
            Ok(TypedExpr::MemberAccess {
                object: Box::new(object_typed),
                member: member.clone(),
                offset: field_layout.offset,
                is_pointer: *is_pointer,
                expr_type: field_layout.field_type.clone(),
            })
        }
        
        ExpressionKind::Conditional { condition, then_expr, else_expr } => {
            let cond_typed = type_expression(condition, type_env)?;
            let then_typed = type_expression(then_expr, type_env)?;
            let else_typed = type_expression(else_expr, type_env)?;
            
            // TODO: Verify that then_expr and else_expr have compatible types
            let result_type = then_typed.get_type().clone();
            
            Ok(TypedExpr::Conditional {
                condition: Box::new(cond_typed),
                then_expr: Box::new(then_typed),
                else_expr: Box::new(else_typed),
                expr_type: result_type,
            })
        }
        
        ExpressionKind::CompoundLiteral { .. } => {
            // TODO: Implement compound literals (C99 feature)
            Err(TypeError::UnsupportedConstruct(
                "Compound literals not yet implemented".to_string()
            ))
        }
    }
}

/// Convert an untyped statement to a typed statement
pub fn type_statement(
    stmt: &crate::ast::Statement,
    type_env: &TypeEnvironment,
) -> Result<TypedStmt, TypeError> {
    use crate::ast::StatementKind;
    
    match &stmt.kind {
        StatementKind::Expression(expr) => {
            let typed_expr = type_expression(expr, type_env)?;
            Ok(TypedStmt::Expression(typed_expr))
        }
        
        StatementKind::Declaration { declarations } => {
            // For now, handle only the first declaration
            // TODO: Handle multiple declarations properly
            if let Some(decl) = declarations.first() {
                let init = match decl.initializer.as_ref() {
                    Some(init) => {
                        Some(type_initializer(init, &decl.decl_type, type_env)?)
                    },
                    None => None,
                };
                
                Ok(TypedStmt::Declaration {
                    name: decl.name.clone(),
                    decl_type: decl.decl_type.clone(),
                    initializer: init,
                    symbol_id: None,
                })
            } else {
                Ok(TypedStmt::Empty)
            }
        }
        
        StatementKind::Compound(stmts) => {
            let typed_stmts: Result<Vec<_>, _> = stmts.iter()
                .map(|s| type_statement(s, type_env))
                .collect();
            
            Ok(TypedStmt::Compound(typed_stmts?))
        }
        
        StatementKind::If { condition, then_stmt, else_stmt } => {
            let typed_condition = type_expression(condition, type_env)?;
            let typed_then = type_statement(then_stmt, type_env)?;
            let typed_else = else_stmt.as_ref()
                .map(|s| type_statement(s, type_env))
                .transpose()?;
            
            Ok(TypedStmt::If {
                condition: typed_condition,
                then_stmt: Box::new(typed_then),
                else_stmt: typed_else.map(Box::new),
            })
        }
        
        StatementKind::While { condition, body } => {
            let typed_condition = type_expression(condition, type_env)?;
            let typed_body = type_statement(body, type_env)?;
            
            Ok(TypedStmt::While {
                condition: typed_condition,
                body: Box::new(typed_body),
            })
        }
        
        StatementKind::For { init, condition, update, body } => {
            let typed_init = init.as_ref()
                .map(|s| type_statement(s, type_env))
                .transpose()?
                .map(Box::new);
            let typed_condition = condition.as_ref()
                .map(|e| type_expression(e, type_env))
                .transpose()?;
            let typed_update = update.as_ref()
                .map(|e| type_expression(e, type_env))
                .transpose()?;
            let typed_body = type_statement(body, type_env)?;
            
            Ok(TypedStmt::For {
                init: typed_init,
                condition: typed_condition,
                update: typed_update,
                body: Box::new(typed_body),
            })
        }
        
        StatementKind::Return(expr) => {
            let typed_expr = expr.as_ref()
                .map(|e| type_expression(e, type_env))
                .transpose()?;
            
            Ok(TypedStmt::Return(typed_expr))
        }
        
        StatementKind::Break => Ok(TypedStmt::Break),
        StatementKind::Continue => Ok(TypedStmt::Continue),
        StatementKind::Empty => Ok(TypedStmt::Empty),
        
        StatementKind::DoWhile { body, condition } => {
            // Transform do-while into: { body; while (condition) { body } }
            // This ensures the body executes at least once
            let typed_body = type_statement(body, type_env)?;
            let typed_condition = type_expression(condition, type_env)?;
            
            // Create the equivalent structure
            let while_loop = TypedStmt::While {
                condition: typed_condition.clone(),
                body: Box::new(typed_body.clone()),
            };
            
            // Wrap in compound statement: body followed by while loop
            Ok(TypedStmt::Compound(vec![
                typed_body,
                while_loop,
            ]))
        }
        
        StatementKind::Switch { .. } => {
            Err(TypeError::UnsupportedConstruct(
                "Switch statements not yet implemented".to_string()
            ))
        }
        
        StatementKind::Case { .. } => {
            Err(TypeError::UnsupportedConstruct(
                "Case labels not yet implemented".to_string()
            ))
        }
        
        StatementKind::Default { .. } => {
            Err(TypeError::UnsupportedConstruct(
                "Default labels not yet implemented".to_string()
            ))
        }
        
        StatementKind::Goto(_) => {
            Err(TypeError::UnsupportedConstruct(
                "Goto statements not yet implemented".to_string()
            ))
        }
        
        StatementKind::Label { .. } => {
            Err(TypeError::UnsupportedConstruct(
                "Label statements not yet implemented".to_string()
            ))
        }
        
        StatementKind::InlineAsm { assembly } => {
            Ok(TypedStmt::InlineAsm {
                assembly: assembly.clone(),
            })
        }
    }
}

/// Convert an untyped translation unit to a typed one
pub fn type_translation_unit(
    ast: &crate::ast::TranslationUnit,
    symbol_types: HashMap<SymbolId, Type>,
    type_definitions: HashMap<String, Type>,
) -> Result<TypedTranslationUnit, TypeError> {
    let type_env = TypeEnvironment::new(symbol_types, type_definitions);
    let mut typed_items = Vec::new();
    
    for item in &ast.items {
        match item {
            crate::ast::TopLevelItem::Function(func) => {
                let typed_body = type_statement(&func.body, &type_env)?;
                
                let typed_func = TypedFunction {
                    name: func.name.clone(),
                    return_type: func.return_type.clone(),
                    parameters: func.parameters.iter()
                        .map(|p| (p.name.clone().unwrap_or_else(|| "unnamed".to_string()), p.param_type.clone()))
                        .collect(),
                    body: typed_body,
                };
                
                typed_items.push(TypedTopLevelItem::Function(typed_func));
            }
            
            crate::ast::TopLevelItem::Declaration(decl) => {
                // Skip function declarations (like extern void putchar(int))
                // These are function declarations, not global variables
                if matches!(decl.decl_type, Type::Function { .. }) {
                    // Function declarations don't generate code or variables
                    // They're just forward declarations that will be resolved at link time
                    continue;
                }
                
                let init = match decl.initializer.as_ref() {
                    Some(init) => {
                        Some(type_initializer(init, &decl.decl_type, &type_env)?)
                    },
                    None => None,
                };
                
                typed_items.push(TypedTopLevelItem::GlobalVariable {
                    name: decl.name.clone(),
                    var_type: decl.decl_type.clone(),
                    initializer: init,
                });
            }
            
            crate::ast::TopLevelItem::TypeDefinition { .. } => {
                // Type definitions (struct/union/enum) don't directly generate code
                // They're already stored in the type environment during semantic analysis
                // so we can safely skip them here
                continue;
            }
        }
    }
    
    Ok(TypedTranslationUnit { items: typed_items })
}