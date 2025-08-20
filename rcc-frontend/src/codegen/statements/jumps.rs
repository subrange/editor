//! Jump statement code generation (break, continue, return)

use super::TypedStatementGenerator;
use crate::typed_ast::TypedExpr;
use crate::codegen::CodegenError;
use crate::CompilerError;

pub fn generate_break(gen: &mut TypedStatementGenerator) -> Result<(), CompilerError> {
    if let Some(label) = gen.break_labels.last() {
        gen.builder.build_branch(*label)?;
        Ok(())
    } else {
        Err(CodegenError::InvalidBreak {
            location: rcc_common::SourceLocation::new_simple(0, 0),
        }.into())
    }
}

pub fn generate_continue(gen: &mut TypedStatementGenerator) -> Result<(), CompilerError> {
    if let Some(label) = gen.continue_labels.last() {
        gen.builder.build_branch(*label)?;
        Ok(())
    } else {
        Err(CodegenError::InvalidContinue {
            location: rcc_common::SourceLocation::new_simple(0, 0),
        }.into())
    }
}

pub fn generate_return(
    gen: &mut TypedStatementGenerator,
    expr: Option<&TypedExpr>,
) -> Result<(), CompilerError> {
    use crate::types::Type;
    
    if let Some(ret_expr) = expr {
        let mut expr_gen = gen.create_expression_generator();
        let ret_type = ret_expr.get_type();
        
        // Check if we're returning a struct by value
        if matches!(ret_type, Type::Struct { .. }) {
            // For struct returns, the expression generator returns a pointer to the struct
            // We need to load the struct value to return it by value
            let struct_ptr = expr_gen.generate(ret_expr)?;
            
            // For struct returns, just return the pointer
            // The calling convention will handle copying the struct
            gen.builder.build_return(Some(struct_ptr))?;
        } else {
            // For non-struct types, generate and return normally
            let ret_val = expr_gen.generate(ret_expr)?;
            gen.builder.build_return(Some(ret_val))?;
        }
    } else {
        gen.builder.build_return(None)?;
    }
    Ok(())
}