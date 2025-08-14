//! Typed expression code generation
//!
//! This module generates IR from typed AST expressions, ensuring that
//! pointer arithmetic generates proper GEP instructions.

use super::errors::CodegenError;
use super::types::convert_type;
use super::VarInfo;
use crate::ast::{BinaryOp, UnaryOp};
use crate::ir::FatPointer;
use crate::ir::{
    GlobalVariable, Instruction, IrBinaryOp, IrBuilder, IrType, IrUnaryOp, Linkage, Module, Value,
};
use crate::typed_ast::{TypedExpr, TypedStmt};
use crate::types::{BankTag, Type};
use crate::CompilerError;
use std::collections::HashMap;

// Helper function for convert_type with default location
fn convert_type_default(ast_type: &Type) -> Result<IrType, CompilerError> {
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

            TypedExpr::StringLiteral { value, .. } => self.generate_string_literal(value),

            TypedExpr::Variable { name, .. } => self.generate_identifier(name),

            TypedExpr::Binary {
                op,
                left,
                right,
                expr_type,
            } => self.generate_binary_operation(*op, left, right, expr_type),

            TypedExpr::PointerArithmetic {
                ptr,
                offset,
                elem_type,
                is_add,
                expr_type,
            } => {
                // THIS IS THE KEY PART: Generate GEP for pointer arithmetic!
                let ptr_val = self.generate(ptr)?;
                let offset_val = self.generate(offset)?;

                // For pointer arithmetic, use GEP instruction
                let ir_type = convert_type_default(expr_type)?;

                // Handle add vs subtract
                let final_offset = if *is_add {
                    offset_val
                } else {
                    // For subtraction, negate the offset
                    let neg_offset = self.builder.build_binary(
                        IrBinaryOp::Sub,
                        Value::Constant(0),
                        offset_val,
                        IrType::I16,
                    )?;
                    Value::Temp(neg_offset)
                };

                // Generate GEP instruction - this handles bank overflow!
                let result = self
                    .builder
                    .build_pointer_offset(ptr_val, final_offset, ir_type)?;

                Ok(result)
            }

            TypedExpr::PointerDifference {
                left,
                right,
                elem_type,
                ..
            } => {
                // Generate pointer difference (returns number of elements)
                let left_val = self.generate(left)?;
                let right_val = self.generate(right)?;

                // Calculate byte difference
                let byte_diff =
                    self.builder
                        .build_binary(IrBinaryOp::Sub, left_val, right_val, IrType::I16)?;

                // Divide by element size to get element count
                let elem_size = super::types::get_ast_type_size(elem_type) as i64;
                let result = self.builder.build_binary(
                    IrBinaryOp::UDiv,
                    Value::Temp(byte_diff),
                    Value::Constant(elem_size),
                    IrType::I16,
                )?;

                Ok(Value::Temp(result))
            }

            TypedExpr::ArrayIndex {
                array,
                index,
                elem_type,
                ..
            } => {
                // Array indexing is pointer arithmetic followed by load
                let array_val = self.generate(array)?;
                let index_val = self.generate(index)?;

                // Generate GEP for the element address
                let elem_ptr_type = convert_type_default(&Type::Pointer {
                    target: Box::new(elem_type.clone()),
                    bank: None,
                })?;

                let elem_ptr =
                    self.builder
                        .build_pointer_offset(array_val, index_val, elem_ptr_type)?;

                // Load the value from that address
                let elem_ir_type = convert_type_default(elem_type)?;
                let result = self.builder.build_load(elem_ptr, elem_ir_type)?;

                Ok(Value::Temp(result))
            }

            TypedExpr::Unary {
                op,
                operand,
                expr_type,
            } => self.generate_unary_operation(*op, operand, expr_type),

            TypedExpr::Call {
                function,
                arguments,
                ..
            } => self.generate_function_call(function, arguments),

            TypedExpr::Cast {
                operand,
                target_type,
                ..
            } => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: format!("type cast to {:?}", target_type),
                    location: rcc_common::SourceLocation::new_simple(0, 0),
                }
                .into())
            }

            TypedExpr::Assignment { lhs, rhs, .. } => self.generate_assignment(lhs, rhs),

            TypedExpr::CompoundAssignment {
                op,
                lhs,
                rhs,
                is_pointer,
                ..
            } => {
                if *is_pointer {
                    // For pointer compound assignment (e.g., p += 5)
                    // This should generate GEP
                    self.generate_pointer_compound_assignment(*op, lhs, rhs)
                } else {
                    // Regular compound assignment
                    self.generate_compound_assignment(*op, lhs, rhs)
                }
            }

            TypedExpr::SizeofExpr { operand, .. } => {
                let size = super::types::get_ast_type_size(operand.get_type());
                Ok(Value::Constant(size as i64))
            }

            TypedExpr::SizeofType { target_type, .. } => {
                let size = super::types::get_ast_type_size(target_type);
                Ok(Value::Constant(size as i64))
            }

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

    fn generate_string_literal(&mut self, s: &str) -> Result<Value, CompilerError> {
        // Create a unique name for this string literal
        let string_id = *self.next_string_id;
        *self.next_string_id += 1;

        // Encode the string bytes in the variable name
        let encoded_name = format!(
            "__str_{}_{}",
            string_id,
            s.bytes().map(|b| format!("{:02x}", b)).collect::<String>()
        );

        let global = GlobalVariable {
            name: encoded_name.clone(),
            var_type: IrType::Array {
                element_type: Box::new(IrType::I8),
                size: (s.len() + 1) as u64, // +1 for null terminator
            },
            is_constant: true, // String literals are constant
            initializer: None,
            linkage: Linkage::Internal,
            symbol_id: None,
        };

        self.module.add_global(global);
        self.string_literals
            .insert(encoded_name.clone(), s.to_string());

        // Return a fat pointer to the string
        Ok(Value::FatPtr(FatPointer {
            addr: Box::new(Value::Global(encoded_name)),
            bank: BankTag::Global,
        }))
    }

    fn generate_identifier(&mut self, name: &str) -> Result<Value, CompilerError> {
        if let Some(var_info) = self.variables.get(name) {
            if self.array_variables.contains(name) {
                // For arrays, return the address (as fat pointer if needed)
                Ok(var_info.as_fat_ptr())
            } else if self.parameter_variables.contains(name) {
                // For parameters, load the value
                let result = self
                    .builder
                    .build_load(var_info.value.clone(), var_info.ir_type.clone())?;
                
                // If it's a pointer type, we need to wrap the loaded value as a FatPtr
                // For pointer parameters, we use Mixed bank to indicate runtime-determined bank
                if var_info.ir_type.is_pointer() {
                    Ok(Value::FatPtr(FatPointer {
                        addr: Box::new(Value::Temp(result)),
                        bank: BankTag::Mixed,  // Runtime-determined bank
                    }))
                } else {
                    Ok(Value::Temp(result))
                }
            } else {
                // For regular variables, load the value
                let result = self
                    .builder
                    .build_load(var_info.value.clone(), var_info.ir_type.clone())?;
                
                // If it's a pointer type, wrap it in FatPtr to preserve bank information
                if var_info.ir_type.is_pointer() {
                    // For local pointer variables, they point to stack-allocated data
                    // unless they've been assigned a value with a different bank
                    Ok(Value::FatPtr(FatPointer {
                        addr: Box::new(Value::Temp(result)),
                        bank: BankTag::Mixed,  // Runtime-determined bank
                    }))
                } else {
                    Ok(Value::Temp(result))
                }
            }
        } else {
            // Check if it's a function
            // TODO: Add has_function method to Module or track functions differently
            // For now, assume it's a global function if not a variable
            Ok(Value::Global(name.to_string()))
        }
    }

    fn generate_binary_operation(
        &mut self,
        op: BinaryOp,
        left: &TypedExpr,
        right: &TypedExpr,
        result_type: &Type,
    ) -> Result<Value, CompilerError> {
        let left_val = self.generate(left)?;
        let right_val = self.generate(right)?;
        let ir_type = convert_type_default(result_type)?;

        let ir_op = match op {
            BinaryOp::Add => IrBinaryOp::Add,
            BinaryOp::Sub => IrBinaryOp::Sub,
            BinaryOp::Mul => IrBinaryOp::Mul,
            BinaryOp::Div => IrBinaryOp::SDiv,
            BinaryOp::Mod => IrBinaryOp::SRem,
            BinaryOp::BitAnd => IrBinaryOp::And,
            BinaryOp::BitOr => IrBinaryOp::Or,
            BinaryOp::BitXor => IrBinaryOp::Xor,
            BinaryOp::LeftShift => IrBinaryOp::Shl,
            BinaryOp::RightShift => IrBinaryOp::AShr,
            BinaryOp::Less => IrBinaryOp::Slt,
            BinaryOp::Greater => IrBinaryOp::Sgt,
            BinaryOp::LessEqual => IrBinaryOp::Sle,
            BinaryOp::GreaterEqual => IrBinaryOp::Sge,
            BinaryOp::Equal => IrBinaryOp::Eq,
            BinaryOp::NotEqual => IrBinaryOp::Ne,
            BinaryOp::LogicalAnd => IrBinaryOp::And,
            BinaryOp::LogicalOr => IrBinaryOp::Or,
            _ => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: format!("binary op: {:?}", op),
                    location: rcc_common::SourceLocation::new_simple(0, 0),
                }
                .into())
            }
        };

        let result = self
            .builder
            .build_binary(ir_op, left_val, right_val, ir_type)?;
        Ok(Value::Temp(result))
    }

    fn generate_unary_operation(
        &mut self,
        op: UnaryOp,
        operand: &TypedExpr,
        result_type: &Type,
    ) -> Result<Value, CompilerError> {
        match op {
            UnaryOp::AddressOf => {
                // For address-of, we need to get the lvalue address
                self.generate_lvalue_address(operand)
            }
            UnaryOp::Dereference => {
                // For dereference, generate the pointer and load from it
                let ptr = self.generate(operand)?;
                let ir_type = convert_type_default(result_type)?;
                let result = self.builder.build_load(ptr, ir_type)?;
                Ok(Value::Temp(result))
            }
            UnaryOp::LogicalNot => {
                // Generate operand == 0
                let operand_val = self.generate(operand)?;
                let zero = Value::Constant(0);
                let temp = self.builder.build_binary(IrBinaryOp::Eq, operand_val, zero, IrType::I1)?;
                Ok(Value::Temp(temp))
            }
            _ => {
                let operand_val = self.generate(operand)?;
                let ir_type = convert_type_default(result_type)?;

                let ir_op = match op {
                    UnaryOp::Plus => return Ok(operand_val), // No-op
                    UnaryOp::Minus => IrUnaryOp::Neg,
                    UnaryOp::BitNot => IrUnaryOp::Not,
                    _ => {
                        return Err(CodegenError::UnsupportedConstruct {
                            construct: format!("unary op: {:?}", op),
                            location: rcc_common::SourceLocation::new_simple(0, 0),
                        }
                        .into())
                    }
                };

                // Build unary operation using binary with 0 or -1
                let result = match ir_op {
                    IrUnaryOp::Neg => self.builder.build_binary(
                        IrBinaryOp::Sub,
                        Value::Constant(0),
                        operand_val,
                        ir_type,
                    )?,
                    IrUnaryOp::Not => self.builder.build_binary(
                        IrBinaryOp::Xor,
                        operand_val,
                        Value::Constant(-1),
                        ir_type,
                    )?,
                    // These are cast operations that we don't support yet
                    IrUnaryOp::ZExt
                    | IrUnaryOp::SExt
                    | IrUnaryOp::Trunc
                    | IrUnaryOp::PtrToInt
                    | IrUnaryOp::IntToPtr => {
                        return Err(CodegenError::UnsupportedConstruct {
                            construct: format!("unary op: {:?}", ir_op),
                            location: rcc_common::SourceLocation::new_simple(0, 0),
                        }
                        .into())
                    }
                };
                Ok(Value::Temp(result))
            }
        }
    }

    fn generate_lvalue_address(&mut self, expr: &TypedExpr) -> Result<Value, CompilerError> {
        match expr {
            TypedExpr::Variable { name, .. } => {
                if let Some(var_info) = self.variables.get(name) {
                    Ok(var_info.as_fat_ptr())
                } else {
                    // If not a local variable, it might be a global
                    // Return a FatPtr to the global with Global bank
                    Ok(Value::FatPtr(FatPointer {
                        addr: Box::new(Value::Global(name.to_string())),
                        bank: BankTag::Global,
                    }))
                }
            }
            TypedExpr::ArrayIndex {
                array,
                index,
                elem_type,
                ..
            } => {
                // For array indexing lvalue, generate GEP
                let array_val = self.generate(array)?;
                let index_val = self.generate(index)?;

                let elem_ptr_type = convert_type_default(&Type::Pointer {
                    target: Box::new(elem_type.clone()),
                    bank: None,
                })?;

                let elem_ptr =
                    self.builder
                        .build_pointer_offset(array_val, index_val, elem_ptr_type)?;

                Ok(elem_ptr)
            }
            TypedExpr::Unary {
                op: UnaryOp::Dereference,
                operand,
                ..
            } => {
                // For *ptr, the address is just ptr
                self.generate(operand)
            }
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: format!("lvalue: {:?}", expr),
                location: rcc_common::SourceLocation::new_simple(0, 0),
            }
            .into()),
        }
    }

    fn generate_assignment(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
    ) -> Result<Value, CompilerError> {
        let lhs_addr = self.generate_lvalue_address(lhs)?;
        let rhs_val = self.generate(rhs)?;

        self.builder.build_store(rhs_val.clone(), lhs_addr)?;
        Ok(rhs_val)
    }

    fn generate_compound_assignment(
        &mut self,
        op: BinaryOp,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
    ) -> Result<Value, CompilerError> {
        let lhs_addr = self.generate_lvalue_address(lhs)?;
        let lhs_val = {
            let ir_type = convert_type_default(lhs.get_type())?;
            let temp = self.builder.build_load(lhs_addr.clone(), ir_type.clone())?;
            Value::Temp(temp)
        };
        let rhs_val = self.generate(rhs)?;

        let ir_type = convert_type_default(lhs.get_type())?;

        let ir_op = match op {
            BinaryOp::AddAssign => IrBinaryOp::Add,
            BinaryOp::SubAssign => IrBinaryOp::Sub,
            BinaryOp::MulAssign => IrBinaryOp::Mul,
            BinaryOp::DivAssign => IrBinaryOp::SDiv,
            BinaryOp::ModAssign => IrBinaryOp::SRem,
            BinaryOp::BitAndAssign => IrBinaryOp::And,
            BinaryOp::BitOrAssign => IrBinaryOp::Or,
            BinaryOp::BitXorAssign => IrBinaryOp::Xor,
            BinaryOp::LeftShiftAssign => IrBinaryOp::Shl,
            BinaryOp::RightShiftAssign => IrBinaryOp::AShr,
            _ => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: format!("compound assignment: {:?}", op),
                    location: rcc_common::SourceLocation::new_simple(0, 0),
                }
                .into())
            }
        };

        let result = self
            .builder
            .build_binary(ir_op, lhs_val, rhs_val, ir_type.clone())?;
        self.builder.build_store(Value::Temp(result), lhs_addr)?;

        Ok(Value::Temp(result))
    }

    fn generate_pointer_compound_assignment(
        &mut self,
        op: BinaryOp,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
    ) -> Result<Value, CompilerError> {
        // For pointer compound assignment (p += n), use GEP
        let lhs_addr = self.generate_lvalue_address(lhs)?;
        let lhs_val = {
            let ir_type = convert_type_default(lhs.get_type())?;
            let temp = self.builder.build_load(lhs_addr.clone(), ir_type)?;
            Value::Temp(temp)
        };
        let rhs_val = self.generate(rhs)?;

        // Handle add vs subtract
        let final_offset = match op {
            BinaryOp::AddAssign => rhs_val,
            BinaryOp::SubAssign => {
                // Negate for subtraction
                let neg = self.builder.build_binary(
                    IrBinaryOp::Sub,
                    Value::Constant(0),
                    rhs_val,
                    IrType::I16,
                )?;
                Value::Temp(neg)
            }
            _ => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: format!("pointer compound assignment: {:?}", op),
                    location: rcc_common::SourceLocation::new_simple(0, 0),
                }
                .into())
            }
        };

        // Generate GEP for pointer arithmetic
        let ir_type = convert_type_default(lhs.get_type())?;
        let result = self
            .builder
            .build_pointer_offset(lhs_val, final_offset, ir_type.clone())?;

        self.builder.build_store(result.clone(), lhs_addr)?;
        Ok(result)
    }

    fn generate_function_call(
        &mut self,
        function: &TypedExpr,
        arguments: &[TypedExpr],
    ) -> Result<Value, CompilerError> {
        // For function calls, we need the function name directly, not its loaded value
        let func_val = match function {
            TypedExpr::Variable { name, .. } => {
                // Check if it's a known variable (function pointer) or a direct function name
                if self.variables.contains_key(name) {
                    // It's a function pointer variable, load it
                    self.generate(function)?
                } else {
                    // It's a direct function name
                    Value::Global(name.to_string())
                }
            }
            _ => {
                // For other expressions (like function pointers), generate normally
                self.generate(function)?
            }
        };

        let mut arg_vals = Vec::new();
        for arg in arguments {
            arg_vals.push(self.generate(arg)?);
        }

        // TODO: Get proper return type
        let result = self.builder.build_call(func_val, arg_vals, IrType::I16)?;
        Ok(result.map(Value::Temp).unwrap_or(Value::Constant(0)))
    }
}
