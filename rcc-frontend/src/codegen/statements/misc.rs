//! Miscellaneous statement code generation (expression statements, compound blocks, inline asm)

use super::TypedStatementGenerator;
use crate::typed_ast::{TypedStmt, TypedExpr, TypedAsmOperand};
use crate::ir::{AsmOperandIR, Value};
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

pub fn generate_inline_asm_extended(
    gen: &mut TypedStatementGenerator,
    assembly: &str,
    outputs: &[TypedAsmOperand],
    inputs: &[TypedAsmOperand],
    clobbers: &[String],
) -> Result<(), CompilerError> {
    
    // If no operands, use simple version
    if outputs.is_empty() && inputs.is_empty() {
        return generate_inline_asm(gen, assembly);
    }
    
    // Generate IR values for output operands
    let mut ir_outputs = Vec::new();
    for (i, op) in outputs.iter().enumerate() {
        // For output operands, we need an lvalue (address to store to)
        let mut expr_gen = gen.create_expression_generator();
        
        // Check if it's a read-write constraint (starts with '+')
        let is_read_write = op.constraint.starts_with('+');
        
        // Generate the value for the operand
        // For outputs, we generate the expression which should be an lvalue (variable)
        // The backend will handle storing the result back to this variable
        let value = expr_gen.generate(&op.expr)?;
        
        ir_outputs.push(AsmOperandIR {
            constraint: op.constraint.clone(),
            value,
            tied_to: if is_read_write { Some(inputs.len() + i) } else { None },
        });
    }
    
    // Generate IR values for input operands  
    let mut ir_inputs = Vec::new();
    for op in inputs {
        let mut expr_gen = gen.create_expression_generator();
        let value = expr_gen.generate(&op.expr)?;
        
        ir_inputs.push(AsmOperandIR {
            constraint: op.constraint.clone(),
            value,
            tied_to: None,
        });
    }
    
    // Add read-write operands as inputs too (they appear in both lists)
    for (i, op) in outputs.iter().enumerate() {
        if op.constraint.starts_with('+') {
            let mut expr_gen = gen.create_expression_generator();
            let value = expr_gen.generate(&op.expr)?;
            
            ir_inputs.push(AsmOperandIR {
                constraint: op.constraint[1..].to_string(), // Remove '+' prefix
                value,
                tied_to: Some(i), // Tied to output operand i
            });
        }
    }
    
    // Build the inline assembly instruction
    gen.builder.build_inline_asm_extended(
        assembly.to_string(),
        ir_outputs,
        ir_inputs,
        clobbers.to_vec(),
    )?;
    
    Ok(())
}