//! Typed statement code generation

use std::collections::{HashMap, HashSet};
use crate::ir::{IrType, IrBuilder, Module};
use crate::typed_ast::{TypedStmt, TypedExpr};
use crate::types::{Type, BankTag};
use crate::CompilerError;
use rcc_common::LabelId as Label;
use super::errors::CodegenError;
use super::types::convert_type;
use super::VarInfo;

use super::typed_expressions::TypedExpressionGenerator;

// Helper function for convert_type with default location
fn convert_type_default(ast_type: &Type) -> Result<IrType, CompilerError> {
    convert_type(ast_type, rcc_common::SourceLocation::new_simple(0, 0))
}

/// Typed statement generator context
pub struct TypedStatementGenerator<'a> {
    pub builder: &'a mut IrBuilder,
    pub module: &'a mut Module,
    pub variables: &'a mut HashMap<String, VarInfo>,
    pub array_variables: &'a mut HashSet<String>,
    pub parameter_variables: &'a HashSet<String>,
    pub string_literals: &'a mut HashMap<String, String>,
    pub next_string_id: &'a mut u32,
    pub break_labels: &'a mut Vec<Label>,
    pub continue_labels: &'a mut Vec<Label>,
}

impl<'a> TypedStatementGenerator<'a> {
    /// Generate IR for a typed statement
    pub fn generate(&mut self, stmt: &TypedStmt) -> Result<(), CompilerError> {
        match stmt {
            TypedStmt::Expression(expr) => {
                let mut expr_gen = self.create_expression_generator();
                expr_gen.generate(expr)?;
                Ok(())
            }
            
            TypedStmt::Declaration { name, decl_type, initializer, symbol_id: _ } => {
                self.generate_declaration(name, decl_type, initializer.as_ref())
            }
            
            TypedStmt::Compound(statements) => {
                for stmt in statements {
                    self.generate(stmt)?;
                }
                Ok(())
            }
            
            TypedStmt::If { condition, then_stmt, else_stmt } => {
                self.generate_if(condition, then_stmt, else_stmt.as_deref())
            }
            
            TypedStmt::While { condition, body } => {
                self.generate_while(condition, body)
            }
            
            TypedStmt::For { init, condition, update, body } => {
                self.generate_for(
                    init.as_deref(),
                    condition.as_ref(),
                    update.as_ref(),
                    body,
                )
            }
            
            TypedStmt::Return(expr) => {
                self.generate_return(expr.as_ref())
            }
            
            TypedStmt::Break => {
                if let Some(label) = self.break_labels.last() {
                    self.builder.build_branch(*label)?;
                    Ok(())
                } else {
                    Err(CodegenError::InvalidBreak {
                        location: rcc_common::SourceLocation::new_simple(0, 0),
                    }.into())
                }
            }
            
            TypedStmt::Continue => {
                if let Some(label) = self.continue_labels.last() {
                    self.builder.build_branch(*label)?;
                    Ok(())
                } else {
                    Err(CodegenError::InvalidContinue {
                        location: rcc_common::SourceLocation::new_simple(0, 0),
                    }.into())
                }
            }
            
            TypedStmt::InlineAsm { assembly } => {
                // Generate inline assembly instruction
                self.builder.build_inline_asm(assembly.clone())?;
                Ok(())
            }
            
            TypedStmt::Empty => Ok(()),
        }
    }
    
    fn create_expression_generator(&mut self) -> TypedExpressionGenerator {
        TypedExpressionGenerator {
            builder: self.builder,
            module: self.module,
            variables: self.variables,
            array_variables: self.array_variables,
            parameter_variables: self.parameter_variables,
            string_literals: self.string_literals,
            next_string_id: self.next_string_id,
        }
    }
    
    fn generate_declaration(
        &mut self,
        name: &str,
        var_type: &Type,
        initializer: Option<&TypedExpr>,
    ) -> Result<(), CompilerError> {
        let ir_type = convert_type_default(var_type)?;
        
        // Allocate stack space for the variable
        let var_addr = self.builder.build_alloca(ir_type.clone(), None)?;
        
        // Track the variable
        let var_info = VarInfo {
            value: var_addr.clone(),
            ir_type: ir_type.clone(),
            bank: Some(BankTag::Stack),
        };
        self.variables.insert(name.to_string(), var_info);
        
        // Handle array variables specially
        if matches!(var_type, Type::Array { .. }) {
            self.array_variables.insert(name.to_string());
        }
        
        // Initialize if needed
        if let Some(init_expr) = initializer {
            match init_expr {
                TypedExpr::ArrayInitializer { elements, .. } => {
                    // For array initializers, store each element individually
                    for (i, elem_expr) in elements.iter().enumerate() {
                        let mut expr_gen = self.create_expression_generator();
                        let elem_val = expr_gen.generate(elem_expr)?;
                        
                        // Calculate element address using GEP
                        let index_val = crate::ir::Value::Constant(i as i64);
                        let elem_addr = self.builder.build_pointer_offset(
                            var_addr.clone(),
                            index_val,
                            ir_type.clone(),
                        )?;
                        
                        // Store the element
                        self.builder.build_store(elem_val, elem_addr)?;
                    }
                }
                _ => {
                    // For non-array initializers, generate and store normally
                    let mut expr_gen = self.create_expression_generator();
                    let init_val = expr_gen.generate(init_expr)?;
                    self.builder.build_store(init_val, var_addr)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_if(
        &mut self,
        condition: &TypedExpr,
        then_stmt: &TypedStmt,
        else_stmt: Option<&TypedStmt>,
    ) -> Result<(), CompilerError> {
        let mut expr_gen = self.create_expression_generator();
        let cond_val = expr_gen.generate(condition)?;
        
        let then_label = self.builder.new_label();
        let else_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Branch on condition
        self.builder.build_branch_cond(
            cond_val,
            then_label,
            if else_stmt.is_some() { else_label } else { end_label },
        )?;
        
        // Then block
        self.builder.create_block(then_label)?;
        self.generate(then_stmt)?;
        self.builder.build_branch(end_label)?;
        
        // Else block (if present)
        if let Some(else_stmt) = else_stmt {
            self.builder.create_block(else_label)?;
            self.generate(else_stmt)?;
            self.builder.build_branch(end_label)?;
        }
        
        // End label
        self.builder.create_block(end_label)?;
        
        Ok(())
    }
    
    fn generate_while(
        &mut self,
        condition: &TypedExpr,
        body: &TypedStmt,
    ) -> Result<(), CompilerError> {
        let cond_label = self.builder.new_label();
        let body_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Set up break/continue targets
        self.break_labels.push(end_label);
        self.continue_labels.push(cond_label);
        
        // Jump to condition
        self.builder.build_branch(cond_label)?;
        
        // Condition check
        self.builder.create_block(cond_label)?;
        let mut expr_gen = self.create_expression_generator();
        let cond_val = expr_gen.generate(condition)?;
        self.builder.build_branch_cond(cond_val, body_label, end_label)?;
        
        // Body
        self.builder.create_block(body_label)?;
        self.generate(body)?;
        self.builder.build_branch(cond_label)?;
        
        // End
        self.builder.create_block(end_label)?;
        
        // Clean up break/continue targets
        self.break_labels.pop();
        self.continue_labels.pop();
        
        Ok(())
    }
    
    fn generate_for(
        &mut self,
        init: Option<&TypedStmt>,
        condition: Option<&TypedExpr>,
        update: Option<&TypedExpr>,
        body: &TypedStmt,
    ) -> Result<(), CompilerError> {
        // Initialization
        if let Some(init_stmt) = init {
            self.generate(init_stmt)?;
        }
        
        let cond_label = self.builder.new_label();
        let body_label = self.builder.new_label();
        let update_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Set up break/continue targets
        self.break_labels.push(end_label);
        self.continue_labels.push(update_label);
        
        // Jump to condition
        self.builder.build_branch(cond_label)?;
        
        // Condition check
        self.builder.create_block(cond_label)?;
        if let Some(cond_expr) = condition {
            let mut expr_gen = self.create_expression_generator();
            let cond_val = expr_gen.generate(cond_expr)?;
            self.builder.build_branch_cond(cond_val, body_label, end_label)?;
        } else {
            // No condition means always true
            self.builder.build_branch(body_label)?;
        }
        
        // Body
        self.builder.create_block(body_label)?;
        self.generate(body)?;
        self.builder.build_branch(update_label)?;
        
        // Update
        self.builder.create_block(update_label)?;
        if let Some(update_expr) = update {
            let mut expr_gen = self.create_expression_generator();
            expr_gen.generate(update_expr)?;
        }
        self.builder.build_branch(cond_label)?;
        
        // End
        self.builder.create_block(end_label)?;
        
        // Clean up break/continue targets
        self.break_labels.pop();
        self.continue_labels.pop();
        
        Ok(())
    }
    
    fn generate_return(&mut self, expr: Option<&TypedExpr>) -> Result<(), CompilerError> {
        if let Some(ret_expr) = expr {
            let mut expr_gen = self.create_expression_generator();
            let ret_val = expr_gen.generate(ret_expr)?;
            self.builder.build_return(Some(ret_val))?;
        } else {
            self.builder.build_return(None)?;
        }
        Ok(())
    }
}