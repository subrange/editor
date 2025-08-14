//! Expression code generation modules

mod literals;
mod identifiers;
mod binary_ops;
mod unary_ops;
mod pointer_ops;
mod function_calls;
mod assignments;
mod misc_ops;

pub use literals::generate_string_literal;
pub use identifiers::generate_identifier;
pub use binary_ops::{generate_binary_operation, generate_compound_assignment};
pub use unary_ops::{generate_unary_operation, generate_lvalue_address};
pub use pointer_ops::{generate_pointer_arithmetic, generate_pointer_difference, generate_array_index, generate_pointer_compound_assignment};
pub use function_calls::generate_function_call;
pub use assignments::generate_assignment;
pub use misc_ops::{generate_sizeof_expr, generate_sizeof_type, generate_array_initializer};

use super::errors::CodegenError;
use super::types::convert_type;
use super::VarInfo;
use crate::ir::{IrBuilder, Module, Value};
use crate::typed_ast::TypedExpr;
use crate::Type;
use crate::CompilerError;
use std::collections::HashMap;

// Helper function for convert_type with default location
fn convert_type_default(ast_type: &crate::types::Type) -> Result<crate::ir::IrType, CompilerError> {
    convert_type(ast_type, rcc_common::SourceLocation::new_simple(0, 0))
}

/// Typed expression generator context
pub struct TypedExpressionGenerator<'a> {
    pub builder: &'a mut IrBuilder,
    pub module: &'a mut Module,
    pub variables: &'a HashMap<String, VarInfo>,
    pub array_variables: &'a std::collections::HashSet<String>,
    pub parameter_variables: &'a std::collections::HashSet<String>,
    pub string_literals: &'a mut HashMap<String, String>,
    pub next_string_id: &'a mut u32,
}

impl<'a> TypedExpressionGenerator<'a> {
    /// Generate IR for a typed expression
    pub fn generate(&mut self, expr: &TypedExpr) -> Result<Value, CompilerError> {
        match expr {
            TypedExpr::IntLiteral { value, .. } => Ok(Value::Constant(*value)),
            
            TypedExpr::CharLiteral { value, .. } => Ok(Value::Constant(*value as i64)),
            
            TypedExpr::StringLiteral { value, .. } => literals::generate_string_literal(self, value),
            
            TypedExpr::Variable { name, .. } => identifiers::generate_identifier(self, name),
            
            TypedExpr::Binary {
                op,
                left,
                right,
                expr_type,
            } => binary_ops::generate_binary_operation(self, *op, left, right, expr_type),
            
            TypedExpr::PointerArithmetic {
                ptr,
                offset,
                elem_type,
                is_add,
                expr_type,
            } => pointer_ops::generate_pointer_arithmetic(self, ptr, offset, elem_type, *is_add, expr_type),
            
            TypedExpr::PointerDifference {
                left,
                right,
                elem_type,
                ..
            } => pointer_ops::generate_pointer_difference(self, left, right, elem_type),
            
            TypedExpr::ArrayIndex {
                array,
                index,
                elem_type,
                ..
            } => pointer_ops::generate_array_index(self, array, index, elem_type),
            
            TypedExpr::Unary {
                op,
                operand,
                expr_type,
            } => unary_ops::generate_unary_operation(self, *op, operand, expr_type),
            
            TypedExpr::Call {
                function,
                arguments,
                ..
            } => function_calls::generate_function_call(self, function, arguments),
            
            TypedExpr::Cast {
                operand,
                target_type,
                ..
            } => {
                // Generate code for the operand
                let operand_val = self.generate(operand)?;
                let source_type = operand.get_type();
                
                // Handle different cast scenarios
                match (&source_type, target_type) {
                    // Pointer to pointer cast (including void*)
                    (Type::Pointer { .. }, Type::Pointer { .. }) => {
                        // Pointer-to-pointer casts preserve the value
                        // Fat pointer bank tags are preserved during cast
                        Ok(operand_val)
                    }
                    
                    // Integer to pointer cast
                    (source, Type::Pointer { .. }) if source.is_integer() => {
                        // Integer to pointer cast is not fully implemented for fat pointers
                        // This requires encoding the integer as a fat pointer with appropriate bank tag
                        return Err(CodegenError::UnsupportedConstruct {
                            construct: format!("cast from integer to pointer (fat pointer encoding not implemented)"),
                            location: rcc_common::SourceLocation::new_simple(0, 0),
                        }
                        .into())
                    }
                    
                    // Pointer to integer cast  
                    (Type::Pointer { .. }, target) if target.is_integer() => {
                        // Pointer to integer cast is not fully implemented for fat pointers
                        // This requires extracting just the address component from the fat pointer
                        return Err(CodegenError::UnsupportedConstruct {
                            construct: format!("cast from pointer to integer (fat pointer decoding not implemented)"),
                            location: rcc_common::SourceLocation::new_simple(0, 0),
                        }
                        .into())
                    }
                    
                    // Integer to integer cast
                    (source, target) if source.is_integer() && target.is_integer() => {
                        // Integer casts require proper sign extension/truncation
                        // This is not yet implemented
                        return Err(CodegenError::UnsupportedConstruct {
                            construct: format!("integer to integer cast (sign extension/truncation not implemented)"),
                            location: rcc_common::SourceLocation::new_simple(0, 0),
                        }
                        .into())
                    }
                    
                    // Void cast (discarding value)
                    (_, Type::Void) => {
                        // Cast to void means discard the value
                        // Return a dummy value since void has no representation
                        Ok(Value::Constant(0))
                    }
                    
                    // Array to pointer decay (implicit cast)
                    (Type::Array { element_type, .. }, Type::Pointer { target, .. })
                        if **element_type == **target => {
                        // Array decays to pointer to first element
                        Ok(operand_val)
                    }
                    
                    _ => {
                        Err(CodegenError::UnsupportedConstruct {
                            construct: format!("cast from {:?} to {:?}", source_type, target_type),
                            location: rcc_common::SourceLocation::new_simple(0, 0),
                        }
                        .into())
                    }
                }
            }
            
            TypedExpr::Assignment { lhs, rhs, .. } => assignments::generate_assignment(self, lhs, rhs),
            
            TypedExpr::CompoundAssignment {
                op,
                lhs,
                rhs,
                is_pointer,
                ..
            } => {
                if *is_pointer {
                    pointer_ops::generate_pointer_compound_assignment(self, *op, lhs, rhs)
                } else {
                    binary_ops::generate_compound_assignment(self, *op, lhs, rhs)
                }
            }
            
            TypedExpr::SizeofExpr { operand, .. } => misc_ops::generate_sizeof_expr(self, operand),
            
            TypedExpr::SizeofType { target_type, .. } => misc_ops::generate_sizeof_type(self, target_type),
            
            TypedExpr::ArrayInitializer { elements, .. } => misc_ops::generate_array_initializer(self, elements),
            
            TypedExpr::MemberAccess { .. } => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: "struct/union member access".to_string(),
                    location: rcc_common::SourceLocation::new_simple(0, 0),
                }
                .into())
            }
            
            TypedExpr::Conditional { .. } => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: "conditional expression (? :)".to_string(),
                    location: rcc_common::SourceLocation::new_simple(0, 0),
                }
                .into())
            }
        }
    }
}