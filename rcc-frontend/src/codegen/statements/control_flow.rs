//! Control flow statement code generation (if, while, for)

use super::TypedStatementGenerator;
use crate::typed_ast::{TypedStmt, TypedExpr};
use crate::CompilerError;

pub fn generate_if(
    gen: &mut TypedStatementGenerator,
    condition: &TypedExpr,
    then_stmt: &TypedStmt,
    else_stmt: Option<&TypedStmt>,
) -> Result<(), CompilerError> {
    let mut expr_gen = gen.create_expression_generator();
    let cond_val = expr_gen.generate(condition)?;
    
    let then_label = gen.builder.new_label();
    let else_label = gen.builder.new_label();
    let end_label = gen.builder.new_label();
    
    // Branch on condition
    gen.builder.build_branch_cond(
        cond_val,
        then_label,
        if else_stmt.is_some() { else_label } else { end_label },
    )?;
    
    // Then block
    gen.builder.create_block(then_label)?;
    gen.generate(then_stmt)?;
    gen.builder.build_branch(end_label)?;
    
    // Else block (if present)
    if let Some(else_stmt) = else_stmt {
        gen.builder.create_block(else_label)?;
        gen.generate(else_stmt)?;
        gen.builder.build_branch(end_label)?;
    }
    
    // End label
    gen.builder.create_block(end_label)?;
    
    Ok(())
}

pub fn generate_while(
    gen: &mut TypedStatementGenerator,
    condition: &TypedExpr,
    body: &TypedStmt,
) -> Result<(), CompilerError> {
    let cond_label = gen.builder.new_label();
    let body_label = gen.builder.new_label();
    let end_label = gen.builder.new_label();
    
    // Set up break/continue targets
    gen.break_labels.push(end_label);
    gen.continue_labels.push(cond_label);
    
    // Jump to condition
    gen.builder.build_branch(cond_label)?;
    
    // Condition check
    gen.builder.create_block(cond_label)?;
    let mut expr_gen = gen.create_expression_generator();
    let cond_val = expr_gen.generate(condition)?;
    gen.builder.build_branch_cond(cond_val, body_label, end_label)?;
    
    // Body
    gen.builder.create_block(body_label)?;
    gen.generate(body)?;
    gen.builder.build_branch(cond_label)?;
    
    // End
    gen.builder.create_block(end_label)?;
    
    // Clean up break/continue targets
    gen.break_labels.pop();
    gen.continue_labels.pop();
    
    Ok(())
}

pub fn generate_for(
    gen: &mut TypedStatementGenerator,
    init: Option<&TypedStmt>,
    condition: Option<&TypedExpr>,
    update: Option<&TypedExpr>,
    body: &TypedStmt,
) -> Result<(), CompilerError> {
    // Initialization
    if let Some(init_stmt) = init {
        gen.generate(init_stmt)?;
    }
    
    let cond_label = gen.builder.new_label();
    let body_label = gen.builder.new_label();
    let update_label = gen.builder.new_label();
    let end_label = gen.builder.new_label();
    
    // Set up break/continue targets
    gen.break_labels.push(end_label);
    gen.continue_labels.push(update_label);
    
    // Jump to condition
    gen.builder.build_branch(cond_label)?;
    
    // Condition check
    gen.builder.create_block(cond_label)?;
    if let Some(cond_expr) = condition {
        let mut expr_gen = gen.create_expression_generator();
        let cond_val = expr_gen.generate(cond_expr)?;
        gen.builder.build_branch_cond(cond_val, body_label, end_label)?;
    } else {
        // No condition means always true
        gen.builder.build_branch(body_label)?;
    }
    
    // Body
    gen.builder.create_block(body_label)?;
    gen.generate(body)?;
    gen.builder.build_branch(update_label)?;
    
    // Update
    gen.builder.create_block(update_label)?;
    if let Some(update_expr) = update {
        let mut expr_gen = gen.create_expression_generator();
        expr_gen.generate(update_expr)?;
    }
    gen.builder.build_branch(cond_label)?;
    
    // End
    gen.builder.create_block(end_label)?;
    
    // Clean up break/continue targets
    gen.break_labels.pop();
    gen.continue_labels.pop();
    
    Ok(())
}