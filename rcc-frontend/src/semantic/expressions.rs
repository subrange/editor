//! Expression semantic analysis
//! 
//! This module handles type checking and semantic validation of expressions.

use crate::ast::*;
use crate::semantic::errors::SemanticError;
use crate::semantic::types::TypeAnalyzer;
use rcc_common::{CompilerError, SymbolTable, SymbolId, SourceLocation, StorageClass as CommonStorageClass};
use std::collections::HashMap;
use crate::{BankTag, Type};

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
    pub fn analyze(&self, expr: &mut Expression, symbol_table: &mut SymbolTable) -> Result<(), CompilerError> {
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
                    }.into());
                }
            }
            
            ExpressionKind::Binary { op, left, right } => {
                self.analyze(left, symbol_table)?;
                self.analyze(right, symbol_table)?;
                
                self.analyze_binary_operation(*op, left, right)?
            }
            
            ExpressionKind::Unary { op, operand } => {
                self.analyze(operand, symbol_table)?;
                
                self.analyze_unary_operation(*op, operand, symbol_table)?
            }
            
            ExpressionKind::Call { function, arguments } => {
                self.analyze(function, symbol_table)?;
                
                for arg in arguments.iter_mut() {
                    self.analyze(arg, symbol_table)?;
                }
                
                // Check if function is callable
                if let Some(func_type) = &function.expr_type {
                    match func_type {
                        Type::Function { return_type, parameters, .. } => {
                            // Check argument count
                            if arguments.len() != parameters.len() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    expected: parameters.len(),
                                    found: arguments.len(),
                                    location: expr.span.start.clone(),
                                }.into());
                            }
                            
                            // Check argument types
                            for (arg, param_type) in arguments.iter().zip(parameters.iter()) {
                                if let Some(arg_type) = &arg.expr_type {
                                    if !param_type.is_assignable_from(arg_type) {
                                        return Err(SemanticError::TypeMismatch {
                                            expected: param_type.clone(),
                                            found: arg_type.clone(),
                                            location: arg.span.start.clone(),
                                        }.into());
                                    }
                                }
                            }
                            
                            *return_type.clone()
                        }
                        _ => {
                            return Err(SemanticError::InvalidFunctionCall {
                                function_type: func_type.clone(),
                                location: expr.span.start.clone(),
                            }.into());
                        }
                    }
                } else {
                    Type::Error
                }
            }
            
            ExpressionKind::Member { object, member: _, is_pointer: _ } => {
                self.analyze(object, symbol_table)?;
                // TODO: Implement member access type checking
                Type::Int // Placeholder
            }
            
            ExpressionKind::Conditional { condition, then_expr, else_expr } => {
                self.analyze(condition, symbol_table)?;
                self.analyze(then_expr, symbol_table)?;
                self.analyze(else_expr, symbol_table)?;
                
                self.check_boolean_context(condition)?;
                
                // Result type is the common type of then and else expressions
                if let (Some(then_type), Some(else_type)) = (&then_expr.expr_type, &else_expr.expr_type) {
                    self.type_analyzer.common_type(then_type, else_type)
                } else {
                    Type::Error
                }
            }
            
            ExpressionKind::Cast { target_type, operand } => {
                self.analyze(operand, symbol_table)?;
                target_type.clone()
            }
            
            ExpressionKind::SizeofExpr(operand) => {
                self.analyze(operand, symbol_table)?;
                Type::UnsignedLong // sizeof returns size_t, which is unsigned long on Ripple
            }
            
            ExpressionKind::SizeofType(_) => {
                Type::UnsignedLong
            }
            
            ExpressionKind::CompoundLiteral { type_name, initializer } => {
                self.analyze_initializer(initializer, type_name, symbol_table)?;
                type_name.clone()
            }
        };
        
        expr.expr_type = Some(expr_type);
        Ok(())
    }
    
    /// Analyze an initializer
    pub fn analyze_initializer(&self, init: &mut Initializer, expected_type: &Type, symbol_table: &mut SymbolTable) -> Result<(), CompilerError> {
        match &mut init.kind {
            InitializerKind::Expression(expr) => {
                self.analyze(expr, symbol_table)?;
                
                // Check type compatibility
                if let Some(expr_type) = &expr.expr_type {
                    if !expected_type.is_assignable_from(expr_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: expr_type.clone(),
                            location: expr.span.start.clone(),
                        }.into());
                    }
                }
            }
            
            InitializerKind::List(initializers) => {
                // Handle array/struct initialization
                match expected_type {
                    Type::Array { element_type, .. } => {
                        for init in initializers {
                            self.analyze_initializer(init, element_type, symbol_table)?;
                        }
                    }
                    Type::Struct { fields, .. } => {
                        // Match initializers to fields
                        for (init, field) in initializers.iter_mut().zip(fields.iter()) {
                            self.analyze_initializer(init, &field.field_type, symbol_table)?;
                        }
                    }
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: Type::Error, // Placeholder
                            location: init.span.start.clone(),
                        }.into());
                    }
                }
            }
            
            InitializerKind::Designated { .. } => {
                // TODO: Handle designated initializers
            }
        }
        
        Ok(())
    }
    
    /// Analyze binary operation and return result type
    fn analyze_binary_operation(&self, op: BinaryOp, left: &Expression, right: &Expression) -> Result<Type, CompilerError> {
        let left_type = left.expr_type.as_ref().unwrap_or(&Type::Error);
        let right_type = right.expr_type.as_ref().unwrap_or(&Type::Error);
        
        match op {
            // Arithmetic operations
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(self.type_analyzer.arithmetic_result_type(left_type, right_type))
                } else if matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                    // Pointer arithmetic
                    if left_type.is_pointer() && right_type.is_integer() {
                        Ok(left_type.clone())
                    } else if left_type.is_integer() && right_type.is_pointer() && matches!(op, BinaryOp::Add) {
                        Ok(right_type.clone())
                    } else {
                        Err(SemanticError::InvalidOperation {
                            operation: format!("{}", op),
                            operand_type: left_type.clone(),
                            location: left.span.start.clone(),
                        }.into())
                    }
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }.into())
                }
            }
            
            // Bitwise operations
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | 
            BinaryOp::LeftShift | BinaryOp::RightShift => {
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(self.type_analyzer.arithmetic_result_type(left_type, right_type))
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }.into())
                }
            }
            
            // Comparison operations
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual |
            BinaryOp::Equal | BinaryOp::NotEqual => {
                // Comparisons return int (boolean)
                Ok(Type::Int)
            }
            
            // Logical operations
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                Ok(Type::Int) // Logical operations return int
            }
            
            // Assignment operations
            BinaryOp::Assign => {
                // Check if left is an lvalue
                if !TypeAnalyzer::is_lvalue(left) {
                    return Err(SemanticError::InvalidLvalue {
                        location: left.span.start.clone(),
                    }.into());
                }
                
                // Check type compatibility
                if !left_type.is_assignable_from(right_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                        location: right.span.start.clone(),
                    }.into());
                }
                
                Ok(left_type.clone())
            }
            
            // Compound assignment operations
            BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign | 
            BinaryOp::DivAssign | BinaryOp::ModAssign | BinaryOp::BitAndAssign |
            BinaryOp::BitOrAssign | BinaryOp::BitXorAssign | 
            BinaryOp::LeftShiftAssign | BinaryOp::RightShiftAssign => {
                // Check if left is an lvalue
                if !TypeAnalyzer::is_lvalue(left) {
                    return Err(SemanticError::InvalidLvalue {
                        location: left.span.start.clone(),
                    }.into());
                }
                
                // For compound assignment, the types must be compatible for the operation
                // This is a simplified check
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(left_type.clone())
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                        location: right.span.start.clone(),
                    }.into())
                }
            }
            
            BinaryOp::Index => {
                // Array indexing: arr[index]
                if left_type.is_pointer() && right_type.is_integer() {
                    if let Some(target_type) = left_type.pointer_target() {
                        Ok(target_type.clone())
                    } else {
                        Ok(Type::Error)
                    }
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "array indexing".to_string(),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }.into())
                }
            }
        }
    }
    
    /// Analyze unary operation and return result type
    fn analyze_unary_operation(&self, op: UnaryOp, operand: &Expression, symbol_table: &SymbolTable) -> Result<Type, CompilerError> {
        let operand_type = operand.expr_type.as_ref().unwrap_or(&Type::Error);
        
        match op {
            UnaryOp::Plus | UnaryOp::Minus => {
                if operand_type.is_integer() {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::BitNot => {
                if operand_type.is_integer() {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "bitwise NOT".to_string(),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::LogicalNot => {
                Ok(Type::Int) // Logical NOT always returns int
            }
            
            UnaryOp::Dereference => {
                if let Some(target_type) = operand_type.pointer_target() {
                    Ok(target_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "dereference".to_string(),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::AddressOf => {
                if TypeAnalyzer::is_lvalue(operand) {
                    // Determine bank based on operand
                    let bank = self.determine_bank_for_address_of(operand, symbol_table);
                    Ok(Type::Pointer { 
                        target: Box::new(operand_type.clone()),
                        bank,
                    })
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "address-of".to_string(),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::PreIncrement | UnaryOp::PostIncrement | 
            UnaryOp::PreDecrement | UnaryOp::PostDecrement => {
                if !TypeAnalyzer::is_lvalue(operand) {
                    return Err(SemanticError::InvalidLvalue {
                        location: operand.span.start.clone(),
                    }.into());
                }
                
                if operand_type.is_integer() || operand_type.is_pointer() {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::Sizeof => {
                Ok(Type::UnsignedLong) // sizeof returns size_t
            }
        }
    }
    
    /// Check if expression is used in boolean context and can be converted
    pub fn check_boolean_context(&self, expr: &Expression) -> Result<(), CompilerError> {
        // In C, any scalar type can be used in boolean context
        if let Some(expr_type) = &expr.expr_type {
            match expr_type {
                Type::Void => {
                    Err(SemanticError::InvalidOperation {
                        operation: "boolean conversion".to_string(),
                        operand_type: expr_type.clone(),
                        location: expr.span.start.clone(),
                    }.into())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
    
    /// Determine the bank tag for address-of operations
    fn determine_bank_for_address_of(&self, operand: &Expression, symbol_table: &SymbolTable) -> Option<BankTag> {
        match &operand.kind {
            ExpressionKind::Identifier { symbol_id, .. } => {
                if let Some(id) = symbol_id {
                    if let Some(symbol) = symbol_table.get_symbol(*id) {
                        // Local variables are on the stack
                        if matches!(symbol.storage_class, CommonStorageClass::Auto | CommonStorageClass::Register) {
                            return Some(BankTag::Stack);
                        }
                        // Static and extern variables are global
                        if matches!(symbol.storage_class, CommonStorageClass::Static | CommonStorageClass::Extern) {
                            return Some(BankTag::Global);
                        }
                    }
                }
                None
            }
            ExpressionKind::Member { .. } => {
                // For struct members, inherit the bank from the struct
                None // We'll need more context to determine this
            }
            ExpressionKind::Binary { op: BinaryOp::Index, left, .. } => {
                // Array indexing inherits bank from array
                if let Some(arr_type) = &left.expr_type {
                    if let Type::Pointer { bank, .. } = arr_type {
                        return *bank;
                    }
                }
                None
            }
            ExpressionKind::Unary { op: UnaryOp::Dereference, operand } => {
                // Dereferencing a pointer - bank depends on the pointer's bank
                if let Some(ptr_type) = &operand.expr_type {
                    if let Type::Pointer { bank, .. } = ptr_type {
                        return *bank;
                    }
                }
                None
            }
            _ => None,
        }
    }
}