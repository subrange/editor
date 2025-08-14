//! Miscellaneous statement code generation (expression statements, compound blocks, inline asm)

use super::TypedStatementGenerator;
use crate::typed_ast::{TypedStmt, TypedExpr};
use crate::CompilerError;

pub fn generate_expression_stmt(
    gen: &mut TypedStatementGenerator,
    expr: &TypedExpr,
) -> Result<(), CompilerError> {
    let mut expr_gen = gen.create_expression_generator();
    expr_gen.generate(expr)?;
    Ok(())
}

pub fn generate_compound(
    gen: &mut TypedStatementGenerator,
    statements: &[TypedStmt],
) -> Result<(), CompilerError> {
    for stmt in statements {
        gen.generate(stmt)?;
    }
    Ok(())
}

pub fn generate_inline_asm(
    gen: &mut TypedStatementGenerator,
    assembly: &str,
) -> Result<(), CompilerError> {
    gen.builder.build_inline_asm(assembly.to_string())?;
    Ok(())
}