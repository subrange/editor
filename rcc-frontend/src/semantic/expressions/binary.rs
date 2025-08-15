//! Binary expression operations and type checking

use crate::ast::*;
use crate::semantic::errors::SemanticError;
use crate::semantic::types::TypeAnalyzer;
use crate::Type;
use rcc_common::{CompilerError, SourceLocation};

pub struct BinaryOperationAnalyzer<'a> {
    pub type_analyzer: &'a TypeAnalyzer<'a>,
}

impl<'a> BinaryOperationAnalyzer<'a> {
    pub fn new(type_analyzer: &'a TypeAnalyzer<'a>) -> Self {
        Self { type_analyzer }
    }

    /// Analyze binary operation and return result type
    pub fn analyze(
        &self,
        op: BinaryOp,
        left: &Expression,
        right: &Expression,
    ) -> Result<Type, CompilerError> {
        let left_type = left.expr_type.as_ref().unwrap_or(&Type::Error);
        let right_type = right.expr_type.as_ref().unwrap_or(&Type::Error);

        match op {
            // Arithmetic operations
            BinaryOp::Add | BinaryOp::Sub => {
                self.analyze_additive_operation(op, left_type, right_type, &left.span.start)
            }

            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(self.type_analyzer.arithmetic_result_type(left_type, right_type))
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }
                    .into())
                }
            }

            // Bitwise operations
            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::LeftShift
            | BinaryOp::RightShift => {
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(self.type_analyzer.arithmetic_result_type(left_type, right_type))
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }
                    .into())
                }
            }

            // Comparison operations
            BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual
            | BinaryOp::Equal
            | BinaryOp::NotEqual => {
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
                    }
                    .into());
                }

                // Check type compatibility
                if !left_type.is_assignable_from(right_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                        location: right.span.start.clone(),
                    }
                    .into());
                }

                Ok(left_type.clone())
            }

            // Compound assignment operations
            BinaryOp::AddAssign | BinaryOp::SubAssign => {
                // Check if left is an lvalue
                if !TypeAnalyzer::is_lvalue(left) {
                    return Err(SemanticError::InvalidLvalue {
                        location: left.span.start.clone(),
                    }
                    .into());
                }

                // For pointers, += and -= work like pointer arithmetic
                if left_type.is_pointer() && right_type.is_integer() {
                    Ok(left_type.clone())
                } else if left_type.is_integer() && right_type.is_integer() {
                    Ok(left_type.clone())
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                        location: right.span.start.clone(),
                    }
                    .into())
                }
            }

            BinaryOp::MulAssign
            | BinaryOp::DivAssign
            | BinaryOp::ModAssign
            | BinaryOp::BitAndAssign
            | BinaryOp::BitOrAssign
            | BinaryOp::BitXorAssign
            | BinaryOp::LeftShiftAssign
            | BinaryOp::RightShiftAssign => {
                // Check if left is an lvalue
                if !TypeAnalyzer::is_lvalue(left) {
                    return Err(SemanticError::InvalidLvalue {
                        location: left.span.start.clone(),
                    }
                    .into());
                }

                // These operations only work on integers
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(left_type.clone())
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                        location: right.span.start.clone(),
                    }
                    .into())
                }
            }

            BinaryOp::Index => {
                // Array indexing: arr[index]
                // Arrays decay to pointers for indexing
                let array_type = if let Type::Array { element_type, .. } = left_type {
                    Type::Pointer {
                        target: element_type.clone(),
                        bank: None, // Arrays are typically on stack or global
                    }
                } else {
                    left_type.clone()
                };

                if array_type.is_pointer() && right_type.is_integer() {
                    if let Some(target_type) = array_type.pointer_target() {
                        Ok(target_type.clone())
                    } else {
                        Ok(Type::Error)
                    }
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "array indexing".to_string(),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }
                    .into())
                }
            }
        }
    }

    /// Analyze additive operations (+ and -) with proper pointer arithmetic handling
    fn analyze_additive_operation(
        &self,
        op: BinaryOp,
        left_type: &Type,
        right_type: &Type,
        location: &SourceLocation,
    ) -> Result<Type, CompilerError> {
        // Case 1: Integer + Integer
        if left_type.is_integer() && right_type.is_integer() {
            return Ok(self.type_analyzer.arithmetic_result_type(left_type, right_type));
        }

        // Case 2: Pointer + Integer
        if left_type.is_pointer() && right_type.is_integer() {
            // Verify the pointer target has a known size
            if let Some(target) = left_type.pointer_target() {
                if target.size_in_words().is_none() {
                    return Err(SemanticError::InvalidOperation {
                        operation: format!("pointer arithmetic on incomplete type"),
                        operand_type: left_type.clone(),
                        location: location.clone(),
                    }
                    .into());
                }
            }
            return Ok(left_type.clone());
        }

        // Case 3: Integer + Pointer (only for Add, commutative)
        if left_type.is_integer() && right_type.is_pointer() && op == BinaryOp::Add {
            // Verify the pointer target has a known size
            if let Some(target) = right_type.pointer_target() {
                if target.size_in_words().is_none() {
                    return Err(SemanticError::InvalidOperation {
                        operation: format!("pointer arithmetic on incomplete type"),
                        operand_type: right_type.clone(),
                        location: location.clone(),
                    }
                    .into());
                }
            }
            return Ok(right_type.clone());
        }

        // Case 4: Pointer - Pointer (only for Sub)
        if left_type.is_pointer() && right_type.is_pointer() && op == BinaryOp::Sub {
            // Both pointers must point to compatible types
            let left_target = left_type.pointer_target();
            let right_target = right_type.pointer_target();

            if let (Some(left_elem), Some(right_elem)) = (left_target, right_target) {
                // Check if element types are compatible
                if !self.are_types_compatible(left_elem, right_elem) {
                    return Err(SemanticError::InvalidOperation {
                        operation: format!("subtracting pointers to incompatible types"),
                        operand_type: left_type.clone(),
                        location: location.clone(),
                    }
                    .into());
                }

                // Verify the types have known sizes
                if left_elem.size_in_words().is_none() {
                    return Err(SemanticError::InvalidOperation {
                        operation: format!("pointer difference on incomplete type"),
                        operand_type: left_type.clone(),
                        location: location.clone(),
                    }
                    .into());
                }
            }

            // Pointer difference returns ptrdiff_t (we use int)
            return Ok(Type::Int);
        }

        Err(SemanticError::InvalidOperation {
            operation: format!("{}", op),
            operand_type: left_type.clone(),
            location: location.clone(),
        }
        .into())
    }

    /// Check if two types are compatible (for pointer operations)
    fn are_types_compatible(&self, left: &Type, right: &Type) -> bool {
        // Exact match
        if left == right {
            return true;
        }

        // void* is compatible with any pointer
        matches!(left, Type::Void) || matches!(right, Type::Void)
    }
}