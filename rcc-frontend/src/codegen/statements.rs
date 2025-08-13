//! Statement code generation

use std::collections::{HashMap, HashSet};
use rcc_common::TempId;
use crate::ir::{Value, IrType, IrBuilder};
use rcc_common::LabelId as Label;
use crate::ast::{Statement, StatementKind, Declaration, Expression, BinaryOp, Initializer, InitializerKind, ExpressionKind};
use crate::{BankTag, CompilerError, Type};
use super::errors::CodegenError;
use super::types::convert_type;
use super::expressions::ExpressionGenerator;
use super::VarInfo;

/// Statement generator context
pub struct StatementGenerator<'a> {
    pub builder: &'a mut IrBuilder,
    pub module: &'a mut crate::ir::Module,
    pub variables: &'a mut HashMap<String, VarInfo>,
    pub array_variables: &'a mut HashSet<String>,
    pub parameter_variables: &'a mut HashSet<String>,
    pub string_literals: &'a mut HashMap<String, String>,
    pub next_string_id: &'a mut u32,
    pub break_labels: &'a mut Vec<Label>,
    pub continue_labels: &'a mut Vec<Label>,
}

impl<'a> StatementGenerator<'a> {
    /// Generate IR for a statement
    pub fn generate(&mut self, stmt: &Statement) -> Result<(), CompilerError> {
        match &stmt.kind {
            StatementKind::Return(expr) => {
                if let Some(expr) = expr {
                    let value = self.generate_expression(expr)?;
                    self.builder.build_return(Some(value))?;
                } else {
                    self.builder.build_return(None)?;
                }
                // Create an unreachable block for any subsequent code in same scope
                let unreachable_label = self.builder.new_label();
                self.builder.create_block(unreachable_label)?;
            }
            
            StatementKind::Break => {
                if let Some(&break_label) = self.break_labels.last() {
                    self.builder.build_branch(break_label)?;
                    // Create an unreachable block for any subsequent code in same scope
                    let unreachable_label = self.builder.new_label();
                    self.builder.create_block(unreachable_label)?;
                } else {
                    return Err(CodegenError::UnsupportedConstruct {
                        construct: "break outside of loop".to_string(),
                        location: stmt.span.start.clone(),
                    }.into());
                }
            }
            
            StatementKind::Continue => {
                if let Some(&continue_label) = self.continue_labels.last() {
                    self.builder.build_branch(continue_label)?;
                    // Create an unreachable block for any subsequent code in same scope
                    let unreachable_label = self.builder.new_label();
                    self.builder.create_block(unreachable_label)?;
                } else {
                    return Err(CodegenError::UnsupportedConstruct {
                        construct: "continue outside of loop".to_string(),
                        location: stmt.span.start.clone(),
                    }.into());
                }
            }
            
            StatementKind::Declaration { declarations } => {
                for decl in declarations {
                    self.generate_local_declaration(decl)?;
                }
            }
            
            StatementKind::Expression(expr) => {
                self.generate_expression(expr)?;
            }
            
            StatementKind::Compound(statements) => {
                for stmt in statements {
                    self.generate(stmt)?;
                }
            }
            
            StatementKind::If { condition, then_stmt, else_stmt } => {
                self.generate_if_statement(condition, then_stmt, else_stmt.as_deref())?;
            }
            
            StatementKind::While { condition, body } => {
                self.generate_while_loop(condition, body)?;
            }
            
            StatementKind::For { init, condition, update, body } => {
                self.generate_for_loop(init.as_deref(), condition.as_ref(), update.as_ref(), body)?;
            }
            
            StatementKind::DoWhile { body, condition } => {
                self.generate_do_while_loop(body, condition)?;
            }
            
            StatementKind::InlineAsm { assembly } => {
                // Pass inline assembly directly to the IR
                self.builder.build_inline_asm(assembly.clone())?;
            }
            
            _ => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: format!("statement type: {:?}", stmt.kind),
                    location: stmt.span.start.clone(),
                }.into());
            }
        }
        
        Ok(())
    }
    
    fn generate_expression(&mut self, expr: &Expression) -> Result<Value, CompilerError> {
        // Special handling for assignment
        if let ExpressionKind::Binary { op: BinaryOp::Assign, left, right } = &expr.kind {
            // Generate rvalue
            let rvalue = self.generate_expression(right)?;
            
            // Generate lvalue (address to store to)
            let mut expr_gen = self.create_expression_generator();
            let lvalue_ptr = expr_gen.generate_lvalue(left)?;
            
            // Store the value
            self.builder.build_store(rvalue.clone(), lvalue_ptr)?;
            
            // Assignment returns the assigned value
            return Ok(rvalue);
        }
        
        let mut expr_gen = self.create_expression_generator();
        expr_gen.generate(expr)
    }
    
    fn create_expression_generator(&mut self) -> ExpressionGenerator {
        ExpressionGenerator {
            builder: self.builder,
            module: self.module,
            variables: self.variables,
            array_variables: self.array_variables,
            parameter_variables: self.parameter_variables,
            string_literals: self.string_literals,
            next_string_id: self.next_string_id,
        }
    }
    
    /// Generate local variable declaration
    fn generate_local_declaration(&mut self, decl: &Declaration) -> Result<(), CompilerError> {
        // First, check if we need to infer array size from initializer
        let decl_type = if let Type::Array { element_type, size: None } = &decl.decl_type {
            // Array with unspecified size - need to infer from initializer
            if let Some(init) = &decl.initializer {
                let inferred_size = match &init.kind {
                    InitializerKind::List(items) => items.len() as u64,
                    InitializerKind::Expression(expr) => {
                        if let ExpressionKind::StringLiteral(s) = &expr.kind {
                            // String literal includes null terminator
                            (s.len() + 1) as u64
                        } else {
                            return Err(CodegenError::InternalError {
                                message: "Cannot infer array size from non-list initializer".to_string(),
                                location: init.span.start.clone(),
                            }.into());
                        }
                    }
                    _ => return Err(CodegenError::InternalError {
                        message: "Cannot infer array size from complex initializer".to_string(),
                        location: init.span.start.clone(),
                    }.into()),
                };
                Type::Array {
                    element_type: element_type.clone(),
                    size: Some(inferred_size),
                }
            } else {
                return Err(CodegenError::InternalError {
                    message: "Array with unspecified size must have initializer".to_string(),
                    location: decl.span.start.clone(),
                }.into());
            }
        } else {
            decl.decl_type.clone()
        };
        
        let ir_type = convert_type(&decl_type, decl.span.start.clone())?;
        
        // For arrays, we need to handle allocation differently
        let (alloca_temp, var_type, is_array) = match &ir_type {
            IrType::Array { size, element_type } => {
                // Allocate array - alloca returns pointer to first element
                let count = Some(Value::Constant(*size as i64));
                let alloca_val = self.builder.build_alloca((**element_type).clone(), count)?;
                // Extract the temp ID from the fat pointer
                let alloca_temp = if let Value::FatPtr(fp) = &alloca_val {
                    if let Value::Temp(id) = *fp.addr {
                        id
                    } else {
                        return Err(CodegenError::InternalError {
                            message: "Unexpected alloca result".to_string(),
                            location: decl.span.start.clone(),
                        }.into());
                    }
                } else {
                    return Err(CodegenError::InternalError {
                        message: "Alloca should return FatPtr".to_string(),
                        location: decl.span.start.clone(),
                    }.into());
                };
                // The variable type is pointer to element type (array decays to pointer)
                (alloca_temp, IrType::FatPtr(element_type.clone()), true)
            }
            _ => {
                // Regular scalar allocation
                let alloca_val = self.builder.build_alloca(ir_type.clone(), None)?;
                // Extract the temp ID from the fat pointer
                let alloca_temp = if let Value::FatPtr(fp) = &alloca_val {
                    if let Value::Temp(id) = *fp.addr {
                        id
                    } else {
                        return Err(CodegenError::InternalError {
                            message: "Unexpected alloca result".to_string(),
                            location: decl.span.start.clone(),
                        }.into());
                    }
                } else {
                    return Err(CodegenError::InternalError {
                        message: "Alloca should return FatPtr".to_string(),
                        location: decl.span.start.clone(),
                    }.into());
                };
                (alloca_temp, IrType::FatPtr(Box::new(ir_type.clone())), false)
            }
        };
        
        // If there's an initializer, handle it appropriately
        if let Some(init) = &decl.initializer {
            if is_array {
                // For arrays, we need to handle initializer lists specially
                self.generate_array_initializer(alloca_temp, &ir_type, init)?;
            } else {
                // For scalars, generate and store the value
                let init_value = self.generate_initializer(init)?;
                self.builder.build_store(init_value, Value::Temp(alloca_temp))?;
            }
        }
        
        // Add to variables map
        let var_value = Value::Temp(alloca_temp);
        self.variables.insert(decl.name.clone(), VarInfo {
            value: var_value,
            ir_type: var_type,
            bank: Some(BankTag::Stack), // Local variables are on the stack
        });
        
        // Track if this is an array
        if is_array {
            self.array_variables.insert(decl.name.clone());
        }
        
        Ok(())
    }
    
    /// Generate initializer for scalar values
    fn generate_initializer(&mut self, init: &Initializer) -> Result<Value, CompilerError> {
        match &init.kind {
            InitializerKind::Expression(expr) => {
                self.generate_expression(expr)
            }
            InitializerKind::List(items) if items.len() == 1 => {
                // Single-element list can be treated as scalar
                self.generate_initializer(&items[0])
            }
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: "complex initializer".to_string(),
                location: init.span.start.clone(),
            }.into()),
        }
    }
    
    /// Generate array initializer
    fn generate_array_initializer(&mut self, array_ptr: TempId, array_type: &IrType, init: &Initializer) 
        -> Result<(), CompilerError> {
        match &init.kind {
            InitializerKind::List(items) => {
                // Get element type
                let element_type = match array_type {
                    IrType::Array { element_type, .. } => element_type.as_ref(),
                    _ => return Err(CodegenError::InternalError {
                        message: "Expected array type for array initializer".to_string(),
                        location: init.span.start.clone(),
                    }.into()),
                };
                
                // Initialize each array element
                for (index, item) in items.iter().enumerate() {
                    // Calculate pointer to element at index
                    let index_val = Value::Constant(index as i64);
                    let elem_ptr = self.builder.build_pointer_offset(
                        Value::Temp(array_ptr),
                        index_val,
                        IrType::FatPtr(Box::new(element_type.clone()))
                    )?;
                    
                    // Generate value for this element
                    let elem_value = match &item.kind {
                        InitializerKind::Expression(expr) => self.generate_expression(expr)?,
                        _ => return Err(CodegenError::UnsupportedConstruct {
                            construct: "nested initializer list".to_string(),
                            location: item.span.start.clone(),
                        }.into()),
                    };
                    
                    // Store the value
                    self.builder.build_store(elem_value, elem_ptr)?;
                }
                
                Ok(())
            }
            InitializerKind::Expression(expr) => {
                // Handle string literal as array initializer for char arrays
                if let ExpressionKind::StringLiteral(s) = &expr.kind {
                    let element_type = match array_type {
                        IrType::Array { element_type, .. } => element_type.as_ref(),
                        _ => return Err(CodegenError::InternalError {
                            message: "Expected array type for array initializer".to_string(),
                            location: init.span.start.clone(),
                        }.into()),
                    };
                    
                    // Only support char arrays for string literals
                    if !matches!(element_type, IrType::I8) {
                        return Err(CodegenError::InternalError {
                            message: format!("String literal can only initialize char array, not {:?} array", element_type),
                            location: expr.span.start.clone(),
                        }.into());
                    }
                    
                    // Initialize char array with string contents
                    for (index, ch) in s.bytes().enumerate() {
                        let index_val = Value::Constant(index as i64);
                        let elem_ptr = self.builder.build_pointer_offset(
                            Value::Temp(array_ptr),
                            index_val,
                            IrType::FatPtr(Box::new(IrType::I8))
                        )?;
                        
                        let char_val = Value::Constant(ch as i64);
                        self.builder.build_store(char_val, elem_ptr)?;
                    }
                    
                    // Add null terminator
                    let null_index = Value::Constant(s.len() as i64);
                    let null_ptr = self.builder.build_pointer_offset(
                        Value::Temp(array_ptr),
                        null_index,
                        IrType::FatPtr(Box::new(IrType::I8))
                    )?;
                    self.builder.build_store(Value::Constant(0), null_ptr)?;
                    
                    Ok(())
                } else {
                    Err(CodegenError::UnsupportedConstruct {
                        construct: "non-list array initializer".to_string(),
                        location: expr.span.start.clone(),
                    }.into())
                }
            }
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: "unsupported array initializer".to_string(),
                location: init.span.start.clone(),
            }.into()),
        }
    }
    
    /// Generate if statement
    fn generate_if_statement(&mut self, condition: &Expression, then_stmt: &Statement, else_stmt: Option<&Statement>) 
        -> Result<(), CompilerError> {
        let cond_val = self.generate_expression(condition)?;
        
        let then_label = self.builder.new_label();
        let else_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Conditional branch
        self.builder.build_branch_cond(
            cond_val,
            then_label,
            if else_stmt.is_some() { else_label } else { end_label }
        )?;
        
        // Generate then block
        self.builder.create_block(then_label)?;
        self.generate(then_stmt)?;
        self.builder.build_branch(end_label)?;
        
        // Generate else block if present
        if let Some(else_stmt) = else_stmt {
            self.builder.create_block(else_label)?;
            self.generate(else_stmt)?;
            self.builder.build_branch(end_label)?;
        }
        
        // Continue with end block
        self.builder.create_block(end_label)?;
        
        Ok(())
    }
    
    /// Generate while loop
    fn generate_while_loop(&mut self, condition: &Expression, body: &Statement) -> Result<(), CompilerError> {
        let header_label = self.builder.new_label();
        let body_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Jump to header
        self.builder.build_branch(header_label)?;
        
        // Header: check condition
        self.builder.create_block(header_label)?;
        let cond_val = self.generate_expression(condition)?;
        self.builder.build_branch_cond(cond_val, body_label, end_label)?;
        
        // Body
        self.builder.create_block(body_label)?;
        
        // Set up break/continue labels
        self.break_labels.push(end_label);
        self.continue_labels.push(header_label);
        
        self.generate(body)?;
        
        // Clean up break/continue labels
        self.break_labels.pop();
        self.continue_labels.pop();
        
        self.builder.build_branch(header_label)?;
        
        // End
        self.builder.create_block(end_label)?;
        
        Ok(())
    }
    
    /// Generate for loop
    fn generate_for_loop(&mut self, init: Option<&Statement>, condition: Option<&Expression>, 
                         update: Option<&Expression>, body: &Statement) -> Result<(), CompilerError> {
        // Generate init if present
        if let Some(init) = init {
            self.generate(init)?;
        }
        
        let header_label = self.builder.new_label();
        let body_label = self.builder.new_label();
        let update_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Jump to header
        self.builder.build_branch(header_label)?;
        
        // Header: check condition
        self.builder.create_block(header_label)?;
        if let Some(condition) = condition {
            let cond_val = self.generate_expression(condition)?;
            self.builder.build_branch_cond(cond_val, body_label, end_label)?;
        } else {
            // No condition means infinite loop
            self.builder.build_branch(body_label)?;
        }
        
        // Body
        self.builder.create_block(body_label)?;
        
        // Set up break/continue labels
        self.break_labels.push(end_label);
        self.continue_labels.push(update_label);
        
        self.generate(body)?;
        
        // Clean up break/continue labels
        self.break_labels.pop();
        self.continue_labels.pop();
        
        self.builder.build_branch(update_label)?;
        
        // Update
        self.builder.create_block(update_label)?;
        if let Some(update) = update {
            self.generate_expression(update)?;
        }
        self.builder.build_branch(header_label)?;
        
        // End
        self.builder.create_block(end_label)?;
        
        Ok(())
    }
    
    /// Generate do-while loop
    fn generate_do_while_loop(&mut self, body: &Statement, condition: &Expression) -> Result<(), CompilerError> {
        let body_label = self.builder.new_label();
        let header_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Jump to body first
        self.builder.build_branch(body_label)?;
        
        // Body
        self.builder.create_block(body_label)?;
        
        // Set up break/continue labels
        self.break_labels.push(end_label);
        self.continue_labels.push(header_label);
        
        self.generate(body)?;
        
        // Clean up break/continue labels
        self.break_labels.pop();
        self.continue_labels.pop();
        
        self.builder.build_branch(header_label)?;
        
        // Header: check condition
        self.builder.create_block(header_label)?;
        let cond_val = self.generate_expression(condition)?;
        self.builder.build_branch_cond(cond_val, body_label, end_label)?;
        
        // End
        self.builder.create_block(end_label)?;
        
        Ok(())
    }
}

