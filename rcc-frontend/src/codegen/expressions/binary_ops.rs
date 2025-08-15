//! Binary operation code generation

use super::{TypedExpressionGenerator, convert_type_default};
use crate::ast::BinaryOp;
use crate::ir::{IrBinaryOp, Value};
use crate::typed_ast::TypedExpr;
use crate::types::Type;
use crate::codegen::CodegenError;
use crate::CompilerError;

pub fn generate_binary_operation(
    gen: &mut TypedExpressionGenerator,
    op: BinaryOp,
    left: &TypedExpr,
    right: &TypedExpr,
    result_type: &Type,
) -> Result<Value, CompilerError> {
    let left_val = gen.generate(left)?;
    let right_val = gen.generate(right)?;
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
                construct: format!("binary op: {op:?}"),
                location: rcc_common::SourceLocation::new_simple(0, 0),
            }
            .into())
        }
    };
    
    let result = gen
        .builder
        .build_binary(ir_op, left_val, right_val, ir_type)?;
    Ok(Value::Temp(result))
}

pub fn generate_compound_assignment(
    gen: &mut TypedExpressionGenerator,
    op: BinaryOp,
    lhs: &TypedExpr,
    rhs: &TypedExpr,
) -> Result<Value, CompilerError> {
    use super::unary_ops::generate_lvalue_address;
    
    let lhs_addr = generate_lvalue_address(gen, lhs)?;
    let lhs_val = {
        let ir_type = convert_type_default(lhs.get_type())?;
        let temp = gen.builder.build_load(lhs_addr.clone(), ir_type.clone())?;
        Value::Temp(temp)
    };
    let rhs_val = gen.generate(rhs)?;
    
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
                construct: format!("compound assignment: {op:?}"),
                location: rcc_common::SourceLocation::new_simple(0, 0),
            }
            .into())
        }
    };
    
    let result = gen
        .builder
        .build_binary(ir_op, lhs_val, rhs_val, ir_type.clone())?;
    gen.builder.build_store(Value::Temp(result), lhs_addr)?;
    
    Ok(Value::Temp(result))
}