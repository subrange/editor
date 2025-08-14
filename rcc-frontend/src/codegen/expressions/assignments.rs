//! Assignment operation code generation

use super::TypedExpressionGenerator;
use super::unary_ops::generate_lvalue_address;
use crate::ir::Value;
use crate::typed_ast::TypedExpr;
use crate::CompilerError;

pub fn generate_assignment(
    gen: &mut TypedExpressionGenerator,
    lhs: &TypedExpr,
    rhs: &TypedExpr,
) -> Result<Value, CompilerError> {
    let lhs_addr = generate_lvalue_address(gen, lhs)?;
    let rhs_val = gen.generate(rhs)?;
    
    gen.builder.build_store(rhs_val.clone(), lhs_addr)?;
    Ok(rhs_val)
}