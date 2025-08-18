//! AST to Typed AST conversion
//!
//! This module implements the conversion from the untyped AST to the typed AST.

use std::cell::RefCell;
use super::expressions::TypedExpr;
use super::statements::TypedStmt;
use super::translation_unit::{TypedFunction, TypedTopLevelItem, TypedTranslationUnit};
use super::errors::TypeError;
use crate::types::{Type, BankTag};
use crate::ast::{Initializer, InitializerKind, BinaryOp};
use rcc_common::SymbolId;
use std::rc::Rc;
use crate::semantic::types::TypeAnalyzer;

/// Type environment for looking up variable types
pub struct TypeEnvironment {
    type_analyzer: Rc<RefCell<TypeAnalyzer>>,
}

impl TypeEnvironment {
    /// Create a new type environment with symbol and type information
    pub fn new(
        type_analyzer: Rc<RefCell<TypeAnalyzer>>,
    ) -> Self {
        Self {
            type_analyzer,
        }
    }
    
    /// Look up the type of a symbol by its ID and resolve it
    pub fn lookup_type(&self, symbol_id: SymbolId) -> Option<Type> {
        let type_opt = self.type_analyzer.borrow().symbol_types.borrow().get(&symbol_id).cloned();
        // Resolve the type to handle typedefs
        type_opt.map(|ty| self.type_analyzer.borrow().resolve_type(&ty))
    }
    
    /// Resolve a typedef name to its underlying type
    pub fn resolve_typedef(&self, name: &str) -> Option<Type> {
        self.type_analyzer.borrow().type_definitions.borrow().get(name).cloned()
    }
    
    /// Check if a name is a typedef
    pub fn is_typedef(&self, name: &str) -> bool {
        self.type_analyzer.borrow().type_definitions.borrow().contains_key(name)
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
                                "Too many initializers for array of size {array_size}"
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
                        "List initializer not supported for type {expected_type:?}"
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
            // Trust the type that semantic analysis computed, but resolve it
            let result_type = expr.expr_type.clone()
                .ok_or_else(|| TypeError::TypeMismatch(format!(
                    "Binary expression has no type at {}:{} (operator: {:?})",
                    expr.span.start.line, expr.span.start.column, op
                )))?;
            let result_type = type_env.type_analyzer.borrow().resolve_type(&result_type);
            
            let left_typed = type_expression(left, type_env)?;
            let right_typed = type_expression(right, type_env)?;
            let left_type_unresolved = left.expr_type.as_ref().ok_or_else(|| TypeError::TypeMismatch(format!(
                "Left operand has no type at {}:{} in binary expression",
                left.span.start.line, left.span.start.column
            )))?;
            let right_type_unresolved = right.expr_type.as_ref().ok_or_else(|| TypeError::TypeMismatch(format!(
                "Right operand has no type at {}:{} in binary expression", 
                right.span.start.line, right.span.start.column
            )))?;
            // Resolve types to handle typedefs
            let left_type = &type_env.type_analyzer.borrow().resolve_type(left_type_unresolved);
            let right_type = &type_env.type_analyzer.borrow().resolve_type(right_type_unresolved);
            
            // Classify the operation based on types
            match op {
                // Special handling for pointer arithmetic operations
                BinaryOp::Add | BinaryOp::Sub if type_env.type_analyzer.borrow().is_pointer(left_type) || type_env.type_analyzer.borrow().is_pointer(right_type) => {
                    // Pointer - Pointer = integer (ptrdiff_t)
                    if type_env.type_analyzer.borrow().is_pointer(left_type) && type_env.type_analyzer.borrow().is_pointer(right_type) && matches!(op, BinaryOp::Sub) {
                        let elem_type = type_env.type_analyzer.borrow().pointer_target(left_type)
                            .ok_or_else(|| TypeError::TypeMismatch("Invalid pointer type".to_string()))?;
                        
                        Ok(TypedExpr::PointerDifference {
                            left: Box::new(left_typed),
                            right: Box::new(right_typed),
                            elem_type: elem_type.clone(),
                            expr_type: result_type,
                        })
                    }
                    // Pointer +/- Integer
                    else if type_env.type_analyzer.borrow().is_pointer(left_type) && type_env.type_analyzer.borrow().is_integer(right_type) {
                        let elem_type = type_env.type_analyzer.borrow().pointer_target(left_type)
                            .ok_or_else(|| TypeError::TypeMismatch("Invalid pointer type".to_string()))?;
                        
                        Ok(TypedExpr::PointerArithmetic {
                            ptr: Box::new(left_typed),
                            offset: Box::new(right_typed),
                            elem_type: elem_type.clone(),
                            is_add: matches!(op, BinaryOp::Add),
                            expr_type: result_type,
                        })
                    }
                    // Integer + Pointer (commutative)
                    else if type_env.type_analyzer.borrow().is_integer(left_type) && type_env.type_analyzer.borrow().is_pointer(right_type) && matches!(op, BinaryOp::Add) {
                        let elem_type = type_env.type_analyzer.borrow().pointer_target(right_type)
                            .ok_or_else(|| TypeError::TypeMismatch("Invalid pointer type".to_string()))?;
                        
                        Ok(TypedExpr::PointerArithmetic {
                            ptr: Box::new(right_typed),
                            offset: Box::new(left_typed),
                            elem_type: elem_type.clone(),
                            is_add: true,
                            expr_type: result_type,
                        })
                    }
                    else {
                        // Shouldn't happen if semantic analysis is correct
                        Ok(TypedExpr::Binary {
                            op: *op,
                            left: Box::new(left_typed),
                            right: Box::new(right_typed),
                            expr_type: result_type,
                        })
                    }
                }
                
                // Array indexing is special
                BinaryOp::Index => {
                    // Arrays decay to pointers
                    let array_type = if let Type::Array { element_type, .. } = left_type {
                        element_type.as_ref().clone()
                    } else if let Some(elem) = type_env.type_analyzer.borrow().pointer_target(left_type) {
                        elem
                    } else {
                        return Err(TypeError::TypeMismatch("Cannot index non-array/pointer type".to_string()));
                    };
                    
                    Ok(TypedExpr::ArrayIndex {
                        array: Box::new(left_typed),
                        index: Box::new(right_typed),
                        elem_type: array_type.clone(),
                        expr_type: result_type,
                    })
                }
                
                // Assignment is special
                BinaryOp::Assign => {
                    Ok(TypedExpr::Assignment {
                        lhs: Box::new(left_typed),
                        rhs: Box::new(right_typed),
                        expr_type: result_type,
                    })
                }
                
                // Compound assignments
                BinaryOp::AddAssign | BinaryOp::SubAssign if type_env.type_analyzer.borrow().is_pointer(left_type) => {
                    Ok(TypedExpr::CompoundAssignment {
                        op: *op,
                        lhs: Box::new(left_typed),
                        rhs: Box::new(right_typed),
                        is_pointer: true,
                        expr_type: result_type,
                    })
                }
                
                BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign | 
                BinaryOp::DivAssign | BinaryOp::ModAssign | BinaryOp::BitAndAssign |
                BinaryOp::BitOrAssign | BinaryOp::BitXorAssign | 
                BinaryOp::LeftShiftAssign | BinaryOp::RightShiftAssign => {
                    Ok(TypedExpr::CompoundAssignment {
                        op: *op,
                        lhs: Box::new(left_typed),
                        rhs: Box::new(right_typed),
                        is_pointer: false,
                        expr_type: result_type,
                    })
                }
                
                // All other binary operations (arithmetic, logical, comparison)
                _ => {
                    Ok(TypedExpr::Binary {
                        op: *op,
                        left: Box::new(left_typed),
                        right: Box::new(right_typed),
                        expr_type: result_type,
                    })
                }
            }
        }
        
        ExpressionKind::Unary { op, operand } => {
            let operand_typed = type_expression(operand, type_env)?;
            let result_type = expr.expr_type.clone()
                .ok_or_else(|| TypeError::TypeMismatch(format!(
                    "Unary expression has no type at {}:{} (operator: {:?})",
                    expr.span.start.line, expr.span.start.column, op
                )))?;
            let result_type = type_env.type_analyzer.borrow().resolve_type(&result_type);
            
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
            // Resolve typedef in target type
            let resolved_target = type_env.type_analyzer.borrow().resolve_type(target_type);
            
            Ok(TypedExpr::Cast {
                operand: Box::new(operand_typed),
                target_type: resolved_target.clone(),
                expr_type: resolved_target,
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
            // Resolve typedef in target type
            let resolved_target = type_env.type_analyzer.borrow().resolve_type(target_type);
            Ok(TypedExpr::SizeofType {
                target_type: resolved_target,
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
            
            // Extract struct fields - need to resolve typedef if necessary
            let resolved_struct_type = type_env.type_analyzer.borrow().resolve_type(struct_type);
            
            let fields = match &resolved_struct_type {
                Type::Struct { fields, .. } => {
                    fields.clone()
                }
                _ => return Err(TypeError::TypeMismatch(
                    format!("Member access requires struct type, got {resolved_struct_type:?}")
                ))
            };
            
            // Calculate struct layout to get field offset
            // Pass type_definitions to resolve nested struct sizes
            let layout = crate::semantic::struct_layout::calculate_struct_layout_with_defs(
                &fields,
                rcc_common::SourceLocation::new_simple(0, 0), // TODO: Use actual location
                Some(&type_env.type_analyzer.borrow().type_definitions.borrow())
            ).map_err(|e| TypeError::TypeMismatch(format!("Failed to calculate struct layout: {e}")))?;
            
            // Find the field and get its offset
            let field_layout = crate::semantic::struct_layout::find_field(&layout, member)
                .ok_or_else(|| TypeError::UndefinedMember {
                    struct_name: format!("{struct_type:?}"),
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
        
        ExpressionKind::CompoundLiteral { type_name, initializer } => {
            // Compound literal creates a temporary object of the specified type
            // initialized with the given initializer
            
            // Process the initializer to get typed expressions
            let typed_elements = match &initializer.kind {
                crate::ast::InitializerKind::Expression(expr) => {
                    // Single expression initializer
                    vec![type_expression(expr, type_env)?]
                }
                crate::ast::InitializerKind::List(init_list) => {
                    // List initializer - convert each element
                    let mut elements = Vec::new();
                    for init in init_list {
                        match &init.kind {
                            crate::ast::InitializerKind::Expression(expr) => {
                                elements.push(type_expression(expr, type_env)?);
                            }
                            _ => {
                                // Designated initializers in compound literals not yet supported
                                return Err(TypeError::UnsupportedConstruct(
                                    "Designated initializers in compound literals not yet implemented".to_string()
                                ));
                            }
                        }
                    }
                    elements
                }
                crate::ast::InitializerKind::Designated { .. } => {
                    return Err(TypeError::UnsupportedConstruct(
                        "Designated initializers in compound literals not yet implemented".to_string()
                    ));
                }
            };
            
            // Resolve typedef in type name
            let resolved_type = type_env.type_analyzer.borrow().resolve_type(type_name);
            // Return the typed compound literal
            Ok(TypedExpr::CompoundLiteral {
                initializer: typed_elements,
                expr_type: resolved_type,
            })
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
            // Handle multiple declarations by creating a compound statement
            if declarations.is_empty() {
                return Ok(TypedStmt::Empty);
            }
            
            if declarations.len() == 1 {
                // Single declaration - handle directly
                let decl = &declarations[0];
                let init = match decl.initializer.as_ref() {
                    Some(init) => {
                        Some(type_initializer(init, &decl.decl_type, type_env)?)
                    },
                    None => None,
                };
                
                // Resolve the type to handle typedefs
                let resolved_type = type_env.type_analyzer.borrow().resolve_type(&decl.decl_type);
                Ok(TypedStmt::Declaration {
                    name: decl.name.clone(),
                    decl_type: resolved_type,
                    initializer: init,
                    symbol_id: decl.symbol_id,
                })
            } else {
                // Multiple declarations - create a compound statement
                let mut typed_decls = Vec::new();
                for decl in declarations {
                    let init = match decl.initializer.as_ref() {
                        Some(init) => {
                            Some(type_initializer(init, &decl.decl_type, type_env)?)
                        },
                        None => None,
                    };
                    
                    // Resolve the type to handle typedefs
                    let resolved_type = type_env.type_analyzer.borrow().resolve_type(&decl.decl_type);
                    typed_decls.push(TypedStmt::Declaration {
                        name: decl.name.clone(),
                        decl_type: resolved_type,
                        initializer: init,
                        symbol_id: decl.symbol_id,
                    });
                }
                
                Ok(TypedStmt::Compound(typed_decls))
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
        
        StatementKind::InlineAsm { assembly, outputs, inputs, clobbers } => {
            use crate::typed_ast::statements::TypedAsmOperand;
            
            // Type check output operands
            // For outputs, we need to handle them as lvalues (write destinations)
            let mut typed_outputs = Vec::new();
            for op in outputs {
                // For output operands, we need to ensure the expression is an lvalue
                // For now, we'll create a typed expression that represents the lvalue
                // without evaluating it (which would require the variable to be initialized)
                
                // Check if it's a simple identifier (most common case)
                let typed_expr = match &op.expr.kind {
                    crate::ast::ExpressionKind::Identifier { name, symbol_id } => {
                        // For output operands, we just need to verify the variable exists
                        // and get its type, but not evaluate it (no initialization required)
                        let var_type = if let Some(id) = symbol_id {
                            type_env.lookup_type(*id)
                                .or_else(|| op.expr.expr_type.clone())
                                .ok_or_else(|| TypeError::UndefinedVariable(name.clone()))?
                        } else {
                            // Fallback to expr_type if no symbol_id
                            op.expr.expr_type.clone()
                                .ok_or_else(|| TypeError::UndefinedVariable(name.clone()))?
                        };
                        
                        // Create a typed variable expression (represents the lvalue)
                        TypedExpr::Variable {
                            name: name.clone(),
                            symbol_id: *symbol_id,
                            expr_type: var_type,
                        }
                    }
                    _ => {
                        // For other expressions (like array elements, struct fields),
                        // we still need to type them normally
                        type_expression(&op.expr, type_env)?
                    }
                };
                
                typed_outputs.push(TypedAsmOperand {
                    constraint: op.constraint.clone(),
                    expr: typed_expr,
                });
            }
            
            // Type check input operands (these are read, so normal evaluation)
            let mut typed_inputs = Vec::new();
            for op in inputs {
                let typed_expr = type_expression(&op.expr, type_env)?;
                typed_inputs.push(TypedAsmOperand {
                    constraint: op.constraint.clone(),
                    expr: typed_expr,
                });
            }
            
            Ok(TypedStmt::InlineAsm {
                assembly: assembly.clone(),
                outputs: typed_outputs,
                inputs: typed_inputs,
                clobbers: clobbers.clone(),
            })
        }
    }
}

/// Convert an untyped translation unit to a typed one
pub fn type_translation_unit(
    ast: &crate::ast::TranslationUnit,
    type_analyzer: Rc<RefCell<TypeAnalyzer>>,
) -> Result<TypedTranslationUnit, TypeError> {
    let type_env = TypeEnvironment::new(Rc::clone(&type_analyzer));
    let mut typed_items = Vec::new();
    
    for item in &ast.items {
        match item {
            crate::ast::TopLevelItem::Function(func) => {
                let typed_body = type_statement(&func.body, &type_env)?;
                
                // Resolve return type and parameter types to handle typedefs
                let resolved_return_type = type_env.type_analyzer.borrow().resolve_type(&func.return_type);
                let resolved_parameters: Vec<(String, Type)> = func.parameters.iter()
                    .map(|p| {
                        let name = p.name.clone().unwrap_or_else(|| "unnamed".to_string());
                        let resolved_type = type_env.type_analyzer.borrow().resolve_type(&p.param_type);
                        (name, resolved_type)
                    })
                    .collect();
                
                let typed_func = TypedFunction {
                    name: func.name.clone(),
                    return_type: resolved_return_type,
                    parameters: resolved_parameters,
                    body: typed_body,
                };
                
                typed_items.push(TypedTopLevelItem::Function(typed_func));
            }
            
            crate::ast::TopLevelItem::Declarations(decls) => {
                for decl in decls {
                    // Skip function declarations (like extern void putchar(int))
                    // These are function declarations, not global variables
                    if matches!(decl.decl_type, Type::Function { .. }) {
                        // Function declarations don't generate code or variables
                        // They're just forward declarations that will be resolved at link time
                        continue;
                    }
                    
                    // Skip typedef declarations - they don't generate code
                    if decl.storage_class == crate::StorageClass::Typedef {
                        continue;
                    }
                    
                    let init = match decl.initializer.as_ref() {
                        Some(init) => {
                            Some(type_initializer(init, &decl.decl_type, &type_env)?)
                        },
                        None => None,
                    };
                    
                    // Resolve the type to handle typedefs
                    let resolved_type = type_env.type_analyzer.borrow().resolve_type(&decl.decl_type);
                    typed_items.push(TypedTopLevelItem::GlobalVariable {
                        name: decl.name.clone(),
                        var_type: resolved_type,
                        initializer: init,
                    });
                }
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