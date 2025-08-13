//! Type analysis and resolution
//! 
//! This module handles type checking, type resolution (typedef, struct references),
//! and type compatibility checks.

use crate::types::{Type, BankTag};
use crate::ast::{Expression, ExpressionKind, BinaryOp, UnaryOp};
use std::collections::HashMap;

/// Type context for semantic analysis
pub struct TypeAnalyzer<'a> {
    pub type_definitions: &'a HashMap<String, Type>,
}

impl<'a> TypeAnalyzer<'a> {
    pub fn new(type_definitions: &'a HashMap<String, Type>) -> Self {
        Self { type_definitions }
    }
    
    /// Resolve a type reference (e.g., struct Point -> actual struct definition)
    pub fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Typedef(name) => {
                // Look up typedef name
                if let Some(actual_type) = self.type_definitions.get(name) {
                    // Recursively resolve in case of typedef chains
                    self.resolve_type(actual_type)
                } else {
                    // Type not found, return as-is
                    ty.clone()
                }
            }
            Type::Struct { name: Some(name), fields } if fields.is_empty() => {
                // This is a reference to a named struct type
                if let Some(actual_type) = self.type_definitions.get(name) {
                    actual_type.clone()
                } else {
                    // Type not found, return as-is
                    ty.clone()
                }
            }
            Type::Union { name: Some(name), fields } if fields.is_empty() => {
                // This is a reference to a named union type
                if let Some(actual_type) = self.type_definitions.get(name) {
                    actual_type.clone()
                } else {
                    ty.clone()
                }
            }
            Type::Pointer { target, bank } => {
                // Recursively resolve pointed-to type
                Type::Pointer { 
                    target: Box::new(self.resolve_type(target)),
                    bank: *bank,
                }
            }
            Type::Array { element_type, size } => {
                // Recursively resolve element type
                Type::Array {
                    element_type: Box::new(self.resolve_type(element_type)),
                    size: *size,
                }
            }
            _ => ty.clone(),
        }
    }
    
    /// Get the common type for two operands (for arithmetic operations)
    pub fn arithmetic_result_type(&self, left: &Type, right: &Type) -> Type {
        // Simplified type promotion rules
        match (left, right) {
            (Type::Long, _) | (_, Type::Long) => Type::Long,
            (Type::UnsignedLong, _) | (_, Type::UnsignedLong) => Type::UnsignedLong,
            (Type::UnsignedInt, _) | (_, Type::UnsignedInt) => Type::UnsignedInt,
            _ => Type::Int,
        }
    }
    
    /// Get common type between two types
    pub fn common_type(&self, left: &Type, right: &Type) -> Type {
        if left == right {
            left.clone()
        } else if left.is_assignable_from(right) {
            left.clone()
        } else if right.is_assignable_from(left) {
            right.clone()
        } else {
            // Use arithmetic promotion rules
            self.arithmetic_result_type(left, right)
        }
    }
    
    /// Check if an expression is a valid lvalue
    pub fn is_lvalue(expr: &Expression) -> bool {
        match &expr.kind {
            ExpressionKind::Identifier { .. } => true,
            ExpressionKind::Unary { op: UnaryOp::Dereference, .. } => true,
            ExpressionKind::Binary { op: BinaryOp::Index, .. } => true,
            ExpressionKind::Member { .. } => true,
            _ => false,
        }
    }
}