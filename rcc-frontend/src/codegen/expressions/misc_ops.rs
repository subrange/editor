//! Miscellaneous operation code generation (sizeof, array initializers, etc.)

use super::TypedExpressionGenerator;
use crate::ir::Value;
use crate::typed_ast::TypedExpr;
use crate::types::Type;
use crate::codegen::CodegenError;
use crate::CompilerError;

pub fn generate_sizeof_expr(
    _gen: &mut TypedExpressionGenerator,
    operand: &TypedExpr,
) -> Result<Value, CompilerError> {
    let size = super::super::types::get_ast_type_size(operand.get_type());
    Ok(Value::Constant(size as i64))
}

pub fn generate_sizeof_type(
    _gen: &mut TypedExpressionGenerator,
    target_type: &Type,
) -> Result<Value, CompilerError> {
    let size = super::super::types::get_ast_type_size(target_type);
    Ok(Value::Constant(size as i64))
}

pub fn generate_array_initializer(
    gen: &mut TypedExpressionGenerator,
    elements: &[TypedExpr],
) -> Result<Value, CompilerError> {
    // For array initializers, we need to evaluate each element
    // and collect them into a ConstantArray value
    // This only works for constant expressions
    let mut values = Vec::new();
    for elem in elements {
        match gen.generate(elem)? {
            Value::Constant(val) => values.push(val),
            _ => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: "non-constant array initializer".to_string(),
                    location: rcc_common::SourceLocation::new_simple(0, 0),
                }
                .into())
            }
        }
    }
    Ok(Value::ConstantArray(values))
}