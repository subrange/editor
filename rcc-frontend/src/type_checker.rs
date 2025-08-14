//! Enhanced type checking for pointer arithmetic
//!
//! This module provides detailed type checking for pointer arithmetic operations,
//! ensuring that all pointer operations are properly typed for GEP generation.

use crate::types::{Type, BankTag};
use rcc_common::SourceLocation;
use crate::{BinaryOp, Expression, ExpressionKind};

/// Result of type checking a binary operation
#[derive(Debug, Clone)]
pub enum TypedBinaryOp {
    /// Regular integer arithmetic
    IntegerArithmetic {
        op: BinaryOp,
        result_type: Type,
    },
    /// Pointer + integer
    PointerOffset {
        ptr_type: Type,
        elem_type: Type,
        elem_size: u64,
        is_add: bool, // true for add, false for sub
    },
    /// Pointer - pointer (returns integer)
    PointerDifference {
        elem_type: Type,
        elem_size: u64,
    },
    /// Array indexing (special case of pointer arithmetic)
    ArrayIndex {
        elem_type: Type,
        elem_size: u64,
    },
    /// Comparison operation
    Comparison {
        op: BinaryOp,
        is_pointer_compare: bool,
    },
    /// Logical operation
    Logical {
        op: BinaryOp,
    },
    /// Assignment
    Assignment {
        lhs_type: Type,
    },
    /// Compound assignment
    CompoundAssignment {
        op: BinaryOp,
        lhs_type: Type,
        is_pointer: bool,
    },
}

/// Enhanced type checker for expressions
pub struct TypeChecker;

impl TypeChecker {
    /// Check and classify a binary operation
    pub fn check_binary_op(
        op: BinaryOp,
        left: &Expression,
        right: &Expression,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        let left_type = left.expr_type.as_ref()
            .ok_or_else(|| format!("Left operand has no type information at {:?}", location))?;
        let right_type = right.expr_type.as_ref()
            .ok_or_else(|| format!("Right operand has no type information at {:?}", location))?;
        
        match op {
            BinaryOp::Add | BinaryOp::Sub => {
                Self::check_additive_op(op, left_type, right_type, location)
            }
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                Self::check_multiplicative_op(op, left_type, right_type, location)
            }
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor |
            BinaryOp::LeftShift | BinaryOp::RightShift => {
                Self::check_bitwise_op(op, left_type, right_type, location)
            }
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | 
            BinaryOp::GreaterEqual | BinaryOp::Equal | BinaryOp::NotEqual => {
                Self::check_comparison_op(op, left_type, right_type, location)
            }
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                Ok(TypedBinaryOp::Logical { op })
            }
            BinaryOp::Assign => {
                Self::check_assignment(left_type, right_type, location)
            }
            BinaryOp::AddAssign | BinaryOp::SubAssign => {
                Self::check_compound_additive_assignment(op, left_type, right_type, location)
            }
            BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::ModAssign |
            BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign |
            BinaryOp::LeftShiftAssign | BinaryOp::RightShiftAssign => {
                Self::check_compound_assignment(op, left_type, right_type, location)
            }
            BinaryOp::Index => {
                Self::check_array_index(left_type, right_type, location)
            }
        }
    }
    
    /// Check additive operations (+ and -)
    fn check_additive_op(
        op: BinaryOp,
        left_type: &Type,
        right_type: &Type,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        // Case 1: Integer + Integer
        if left_type.is_integer() && right_type.is_integer() {
            return Ok(TypedBinaryOp::IntegerArithmetic {
                op,
                result_type: Self::promote_integer_types(left_type, right_type),
            });
        }
        
        // Case 2: Pointer + Integer (only for Add)
        if left_type.is_pointer() && right_type.is_integer() {
            let elem_type = left_type.pointer_target()
                .ok_or_else(|| format!("Invalid pointer type at {:?}", location))?;
            
            let elem_size = elem_type.size_in_words()
                .ok_or_else(|| format!("Cannot determine size of type {} at {:?}", elem_type, location))?;
            
            return Ok(TypedBinaryOp::PointerOffset {
                ptr_type: left_type.clone(),
                elem_type: elem_type.clone(),
                elem_size,
                is_add: op == BinaryOp::Add,
            });
        }
        
        // Case 3: Integer + Pointer (only for Add, commutative)
        if left_type.is_integer() && right_type.is_pointer() && op == BinaryOp::Add {
            let elem_type = right_type.pointer_target()
                .ok_or_else(|| format!("Invalid pointer type at {:?}", location))?;
            let elem_size = elem_type.size_in_words()
                .ok_or_else(|| format!("Cannot determine size of type {} at {:?}", elem_type, location))?;
            
            return Ok(TypedBinaryOp::PointerOffset {
                ptr_type: right_type.clone(),
                elem_type: elem_type.clone(),
                elem_size,
                is_add: true,
            });
        }
        
        // Case 4: Pointer - Pointer (only for Sub)
        if left_type.is_pointer() && right_type.is_pointer() && op == BinaryOp::Sub {
            // Both pointers must point to compatible types
            let left_elem = left_type.pointer_target()
                .ok_or_else(|| format!("Invalid pointer type at {:?}", location))?;
            let right_elem = right_type.pointer_target()
                .ok_or_else(|| format!("Invalid pointer type at {:?}", location))?;
            
            // Check if element types are compatible
            if !Self::are_types_compatible(left_elem, right_elem) {
                return Err(format!(
                    "Subtracting pointers to incompatible types: {} and {} at {:?}",
                    left_elem, right_elem, location
                ));
            }
            
            let elem_size = left_elem.size_in_words()
                .ok_or_else(|| format!("Cannot determine size of type {} at {:?}", left_elem, location))?;
            
            return Ok(TypedBinaryOp::PointerDifference {
                elem_type: left_elem.clone(),
                elem_size,
            });
        }
        
        Err(format!(
            "Invalid operands for {} operation: {} and {} at {:?}",
            op, left_type, right_type, location
        ))
    }
    
    /// Check multiplicative operations (*, /, %)
    fn check_multiplicative_op(
        op: BinaryOp,
        left_type: &Type,
        right_type: &Type,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        if !left_type.is_integer() || !right_type.is_integer() {
            return Err(format!(
                "Invalid operands for {} operation: {} and {} at {:?}",
                op, left_type, right_type, location
            ));
        }
        
        Ok(TypedBinaryOp::IntegerArithmetic {
            op,
            result_type: Self::promote_integer_types(left_type, right_type),
        })
    }
    
    /// Check bitwise operations
    fn check_bitwise_op(
        op: BinaryOp,
        left_type: &Type,
        right_type: &Type,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        if !left_type.is_integer() || !right_type.is_integer() {
            return Err(format!(
                "Invalid operands for {} operation: {} and {} at {:?}",
                op, left_type, right_type, location
            ));
        }
        
        Ok(TypedBinaryOp::IntegerArithmetic {
            op,
            result_type: Self::promote_integer_types(left_type, right_type),
        })
    }
    
    /// Check comparison operations
    fn check_comparison_op(
        op: BinaryOp,
        left_type: &Type,
        right_type: &Type,
        _location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        // Pointers can be compared if they point to compatible types
        let is_pointer_compare = left_type.is_pointer() && right_type.is_pointer();
        
        Ok(TypedBinaryOp::Comparison {
            op,
            is_pointer_compare,
        })
    }
    
    /// Check assignment operation
    fn check_assignment(
        left_type: &Type,
        right_type: &Type,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        if !left_type.is_assignable_from(right_type) {
            return Err(format!(
                "Cannot assign {} to {} at {:?}",
                right_type, left_type, location
            ));
        }
        
        Ok(TypedBinaryOp::Assignment {
            lhs_type: left_type.clone(),
        })
    }
    
    /// Check compound additive assignment (+=, -=)
    fn check_compound_additive_assignment(
        op: BinaryOp,
        left_type: &Type,
        right_type: &Type,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        // For pointers, += and -= work like pointer arithmetic
        if left_type.is_pointer() && right_type.is_integer() {
            return Ok(TypedBinaryOp::CompoundAssignment {
                op,
                lhs_type: left_type.clone(),
                is_pointer: true,
            });
        }
        
        // For integers, normal compound assignment
        if left_type.is_integer() && right_type.is_integer() {
            return Ok(TypedBinaryOp::CompoundAssignment {
                op,
                lhs_type: left_type.clone(),
                is_pointer: false,
            });
        }
        
        Err(format!(
            "Invalid operands for {} operation: {} and {} at {:?}",
            op, left_type, right_type, location
        ))
    }
    
    /// Check other compound assignments
    fn check_compound_assignment(
        op: BinaryOp,
        left_type: &Type,
        right_type: &Type,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        if !left_type.is_integer() || !right_type.is_integer() {
            return Err(format!(
                "Invalid operands for {} operation: {} and {} at {:?}",
                op, left_type, right_type, location
            ));
        }
        
        Ok(TypedBinaryOp::CompoundAssignment {
            op,
            lhs_type: left_type.clone(),
            is_pointer: false,
        })
    }
    
    /// Check array indexing operation
    fn check_array_index(
        left_type: &Type,
        right_type: &Type,
        location: SourceLocation,
    ) -> Result<TypedBinaryOp, String> {
        if !left_type.is_pointer() {
            return Err(format!(
                "Cannot index non-pointer type {} at {:?}",
                left_type, location
            ));
        }
        
        if !right_type.is_integer() {
            return Err(format!(
                "Array index must be integer, found {} at {:?}",
                right_type, location
            ));
        }
        
        let elem_type = left_type.pointer_target()
            .ok_or_else(|| format!("Invalid pointer type at {:?}", location))?;
        let elem_size = elem_type.size_in_words()
            .ok_or_else(|| format!("Cannot determine size of type {} at {:?}", elem_type, location))?;
        
        Ok(TypedBinaryOp::ArrayIndex {
            elem_type: elem_type.clone(),
            elem_size,
        })
    }
    
    /// Promote integer types according to C rules (simplified)
    fn promote_integer_types(left: &Type, right: &Type) -> Type {
        // Simplified promotion rules
        match (left, right) {
            (Type::Long, _) | (_, Type::Long) => Type::Long,
            (Type::UnsignedLong, _) | (_, Type::UnsignedLong) => Type::UnsignedLong,
            (Type::UnsignedInt, _) | (_, Type::UnsignedInt) => Type::UnsignedInt,
            (Type::Int, _) | (_, Type::Int) => Type::Int,
            (Type::UnsignedShort, _) | (_, Type::UnsignedShort) => Type::UnsignedShort,
            (Type::Short, _) | (_, Type::Short) => Type::Short,
            _ => Type::Int, // Default promotion
        }
    }
    
    /// Check if two types are compatible (for pointer operations)
    fn are_types_compatible(left: &Type, right: &Type) -> bool {
        // Exact match
        if left == right {
            return true;
        }
        
        // void* is compatible with any pointer
        matches!(left, Type::Void) || matches!(right, Type::Void)
    }
    
    /// Determine result type for pointer arithmetic
    pub fn pointer_arithmetic_result_type(
        ptr_type: &Type,
        op: BinaryOp,
        _is_integer_operand: bool,
    ) -> Type {
        match op {
            BinaryOp::Add | BinaryOp::Sub => ptr_type.clone(),
            _ => Type::Error,
        }
    }
    
    /// Check if an expression needs pointer arithmetic handling
    pub fn needs_pointer_arithmetic(expr: &Expression) -> bool {
        if let ExpressionKind::Binary { op, left, right } = &expr.kind {
            let left_is_ptr = left.expr_type.as_ref().map_or(false, |t| t.is_pointer());
            let right_is_ptr = right.expr_type.as_ref().map_or(false, |t| t.is_pointer());
            
            match op {
                BinaryOp::Add | BinaryOp::Sub => left_is_ptr || right_is_ptr,
                BinaryOp::AddAssign | BinaryOp::SubAssign => left_is_ptr,
                BinaryOp::Index => true, // Always uses pointer arithmetic
                _ => false,
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcc_common::SourceLocation;
    
    fn dummy_location() -> SourceLocation {
        SourceLocation::new_simple(1, 1)
    }
    
    fn make_expr_with_type(typ: Type) -> Expression {
        Expression {
            node_id: 0,
            kind: ExpressionKind::IntLiteral(0),
            span: rcc_common::SourceSpan::new(dummy_location(), dummy_location()),
            expr_type: Some(typ),
        }
    }
    
    #[test]
    fn test_integer_arithmetic() {
        let left = make_expr_with_type(Type::Int);
        let right = make_expr_with_type(Type::Int);
        
        let result = TypeChecker::check_binary_op(
            BinaryOp::Add,
            &left,
            &right,
            dummy_location(),
        ).unwrap();
        
        match result {
            TypedBinaryOp::IntegerArithmetic { op, result_type } => {
                assert_eq!(op, BinaryOp::Add);
                assert_eq!(result_type, Type::Int);
            }
            _ => panic!("Expected IntegerArithmetic"),
        }
    }
    
    #[test]
    fn test_pointer_plus_integer() {
        let ptr_type = Type::Pointer {
            target: Box::new(Type::Int),
            bank: None,
        };
        let left = make_expr_with_type(ptr_type.clone());
        let right = make_expr_with_type(Type::Int);
        
        let result = TypeChecker::check_binary_op(
            BinaryOp::Add,
            &left,
            &right,
            dummy_location(),
        ).unwrap();
        
        match result {
            TypedBinaryOp::PointerOffset { elem_size, is_add, .. } => {
                assert_eq!(elem_size, 1); // int is 1 word (2 bytes) on Ripple VM
                assert!(is_add);
            }
            _ => panic!("Expected PointerOffset"),
        }
    }
    
    #[test]
    fn test_array_indexing() {
        let arr_type = Type::Array {
            element_type: Box::new(Type::Int),
            size: Some(10),
        };
        let left = make_expr_with_type(arr_type);
        let right = make_expr_with_type(Type::Int);
        
        let result = TypeChecker::check_binary_op(
            BinaryOp::Index,
            &left,
            &right,
            dummy_location(),
        ).unwrap();
        
        match result {
            TypedBinaryOp::ArrayIndex { elem_size, .. } => {
                assert_eq!(elem_size, 1); // int is 1 word (2 bytes) on Ripple VM
            }
            _ => panic!("Expected ArrayIndex"),
        }
    }
}