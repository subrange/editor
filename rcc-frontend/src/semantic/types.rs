//! Type analysis and resolution
//! 
//! This module handles type checking, type resolution (typedef, struct references),
//! and type compatibility checks.

use crate::types::Type;
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
    /// Returns the resolved type, or the original type if it cannot be resolved
    pub fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Typedef(name) => {
                // Look up typedef name
                if let Some(actual_type) = self.type_definitions.get(name) {
                    // Recursively resolve in case of typedef chains
                    self.resolve_type(actual_type)
                } else {
                    // Unresolved typedef - return as-is and let caller handle it
                    ty.clone()
                }
            }
            Type::Struct { name, fields } => {
                // Only resolve if it's a named struct with no fields (a reference)
                if let Some(name) = name {
                    if fields.is_empty() {
                        // This is a reference to a named struct type
                        if let Some(actual_type) = self.type_definitions.get(name) {
                            return actual_type.clone();
                        } else {
                            // Unresolved struct reference - incomplete type
                            // Return as-is to preserve the name for error reporting
                            // The caller should check for incomplete types
                            return ty.clone();
                        }
                    }
                }
                // Otherwise return as-is (already complete or anonymous)
                ty.clone()
            }
            Type::Union { name, fields } => {
                // Only resolve if it's a named union with no fields (a reference)
                if let Some(name) = name {
                    if fields.is_empty() {
                        // This is a reference to a named union type
                        if let Some(actual_type) = self.type_definitions.get(name) {
                            return actual_type.clone();
                        } else {
                            // Unresolved union reference - incomplete type
                            // Return as-is to preserve the name for error reporting
                            // The caller should check for incomplete types
                            return ty.clone();
                        }
                    }
                }
                // Otherwise return as-is (already complete or anonymous)
                ty.clone()
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
            // Basic types that don't need resolution
            Type::Void | Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar |
            Type::Short | Type::UnsignedShort | Type::Int | Type::UnsignedInt |
            Type::Long | Type::UnsignedLong | Type::Error => ty.clone(),
            
            // Function types don't need resolution (parameters are already resolved)
            Type::Function { .. } => ty.clone(),
            
            // Enum types don't need resolution
            Type::Enum { .. } => ty.clone(),
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
    
    /// Check if a type is unresolved (e.g., undefined typedef or incomplete struct)
    pub fn is_unresolved_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Typedef(name) => {
                // A typedef is unresolved if it's not in our definitions
                !self.type_definitions.contains_key(name)
            }
            Type::Struct { name: Some(name), fields } if fields.is_empty() => {
                // A named struct reference is unresolved if not in definitions
                !self.type_definitions.contains_key(name)
            }
            Type::Union { name: Some(name), fields } if fields.is_empty() => {
                // A named union reference is unresolved if not in definitions
                !self.type_definitions.contains_key(name)
            }
            _ => false,
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