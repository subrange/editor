//! Unary operation code generation

use super::{TypedExpressionGenerator, convert_type_default};
use crate::ast::UnaryOp;
use crate::ir::{IrBinaryOp, IrUnaryOp, IrType, Value, FatPointer};
use crate::typed_ast::TypedExpr;
use crate::types::{Type, BankTag};
use crate::codegen::CodegenError;
use crate::CompilerError;

pub fn generate_unary_operation(
    gen: &mut TypedExpressionGenerator,
    op: UnaryOp,
    operand: &TypedExpr,
    result_type: &Type,
) -> Result<Value, CompilerError> {
    match op {
        UnaryOp::AddressOf => {
            // For address-of, we need to get the lvalue address
            generate_lvalue_address(gen, operand)
        }
        UnaryOp::Dereference => {
            // For dereference, generate the pointer and load from it
            let ptr = gen.generate(operand)?;
            let ir_type = convert_type_default(result_type)?;
            let result = gen.builder.build_load(ptr, ir_type)?;
            Ok(Value::Temp(result))
        }
        UnaryOp::LogicalNot => {
            // Generate operand == 0
            let operand_val = gen.generate(operand)?;
            let zero = Value::Constant(0);
            let temp = gen.builder.build_binary(IrBinaryOp::Eq, operand_val, zero, IrType::I1)?;
            Ok(Value::Temp(temp))
        }
        UnaryOp::PreIncrement | UnaryOp::PreDecrement => {
            // Pre-increment/decrement: ++x or --x
            // 1. Get the address of the operand
            let addr = generate_lvalue_address(gen, operand)?;
            let ir_type = convert_type_default(result_type)?;
            
            // 2. Load the current value
            let current_val = gen.builder.build_load(addr.clone(), ir_type.clone())?;
            
            // 3. Calculate the increment/decrement amount
            let amount = if operand.get_type().is_pointer() {
                // For pointers, increment by the size of the pointed-to type
                if let Type::Pointer { target, .. } = operand.get_type() {
                    Value::Constant(target.size_in_words().unwrap_or(1) as i64)
                } else {
                    Value::Constant(1)
                }
            } else {
                // For integers, increment by 1
                Value::Constant(1)
            };
            
            // 4. Perform the operation
            let binary_op = if matches!(op, UnaryOp::PreIncrement) {
                IrBinaryOp::Add
            } else {
                IrBinaryOp::Sub
            };
            let new_val_temp = gen.builder.build_binary(binary_op, Value::Temp(current_val), amount, ir_type)?;
            
            // 5. Store the new value back
            gen.builder.build_store(Value::Temp(new_val_temp), addr)?;
            
            // 6. Return the new value (for pre-increment/decrement)
            Ok(Value::Temp(new_val_temp))
        }
        UnaryOp::PostIncrement | UnaryOp::PostDecrement => {
            // Post-increment/decrement: x++ or x--
            // 1. Get the address of the operand
            let addr = generate_lvalue_address(gen, operand)?;
            let ir_type = convert_type_default(result_type)?;
            
            // 2. Load the current value
            let old_value = gen.builder.build_load(addr.clone(), ir_type.clone())?;
            
            // 3. Make a copy of the old value to return later (add 0 to copy it)
            let saved_old = gen.builder.build_binary(
                IrBinaryOp::Add,
                Value::Temp(old_value),
                Value::Constant(0),
                ir_type.clone()
            )?;
            
            // 4. Calculate the increment/decrement amount
            let amount = if operand.get_type().is_pointer() {
                // For pointers, increment by the size of the pointed-to type
                if let Type::Pointer { target, .. } = operand.get_type() {
                    Value::Constant(target.size_in_words().unwrap_or(1) as i64)
                } else {
                    Value::Constant(1)
                }
            } else {
                // For integers, increment by 1
                Value::Constant(1)
            };
            
            // 5. Load the value again for the increment operation
            let current = gen.builder.build_load(addr.clone(), ir_type.clone())?;
            
            // 6. Perform the increment/decrement
            let binary_op = if matches!(op, UnaryOp::PostIncrement) {
                IrBinaryOp::Add
            } else {
                IrBinaryOp::Sub
            };
            let new_value = gen.builder.build_binary(binary_op, Value::Temp(current), amount, ir_type)?;
            
            // 7. Store the new value back to memory
            gen.builder.build_store(Value::Temp(new_value), addr)?;
            
            // 8. Return the saved old value (this is what makes it post-increment)
            Ok(Value::Temp(saved_old))
        }
        _ => {
            let operand_val = gen.generate(operand)?;
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
                IrUnaryOp::Neg => gen.builder.build_binary(
                    IrBinaryOp::Sub,
                    Value::Constant(0),
                    operand_val,
                    ir_type,
                )?,
                IrUnaryOp::Not => gen.builder.build_binary(
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

pub fn generate_lvalue_address(
    gen: &mut TypedExpressionGenerator,
    expr: &TypedExpr,
) -> Result<Value, CompilerError> {
    match expr {
        TypedExpr::Variable { name, .. } => {
            if let Some(var_info) = gen.variables.get(name) {
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
            let array_val = gen.generate(array)?;
            let index_val = gen.generate(index)?;
            
            let elem_ptr_type = convert_type_default(&Type::Pointer {
                target: Box::new(elem_type.clone()),
                bank: None,
            })?;
            
            let elem_ptr =
                gen.builder
                    .build_pointer_offset(array_val, index_val, elem_ptr_type)?;
            
            Ok(elem_ptr)
        }
        TypedExpr::Unary {
            op: UnaryOp::Dereference,
            operand,
            ..
        } => {
            // For *ptr, the address is just ptr
            gen.generate(operand)
        }
        TypedExpr::MemberAccess {
            object,
            offset,
            is_pointer,
            expr_type,
            ..
        } => {
            // For struct member lvalue, calculate the field address using GEP
            
            // Get pointer to the struct
            let struct_ptr = if *is_pointer {
                // Object is already a pointer (-> operator)
                gen.generate(object)?
            } else {
                // Object is a struct value (. operator)
                // Need to get its address recursively
                generate_lvalue_address(gen, object)?
            };
            
            // Field offset is a compile-time constant (in words)
            let offset_val = Value::Constant(*offset as i64);
            
            // Generate GEP for field address
            let field_type_ir = convert_type_default(expr_type)?;
            Ok(gen.builder.build_pointer_offset(
                struct_ptr,
                offset_val,
                field_type_ir
            )?)
        }
        _ => Err(CodegenError::UnsupportedConstruct {
            construct: format!("lvalue: {:?}", expr),
            location: rcc_common::SourceLocation::new_simple(0, 0),
        }
        .into()),
    }
}