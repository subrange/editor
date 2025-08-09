//! AST to IR Lowering (Code Generation)
//! 
//! This module translates the analyzed AST into IR for the backend.
//! It handles the conversion from high-level C constructs to low-level IR.

use crate::ast::*;
use rcc_ir::{Module, Function, BasicBlock, Instruction, Value, IrType, IrBinaryOp, IrUnaryOp, IrBuilder, GlobalVariable, Linkage};
use rcc_common::{CompilerError, TempId, LabelId, SourceLocation};
use std::collections::{HashMap, HashSet};

/// Code generation errors
#[derive(Debug, Clone)]
pub enum CodegenError {
    UnsupportedConstruct {
        construct: String,
        location: SourceLocation,
    },
    InvalidType {
        ast_type: Type,
        location: SourceLocation,
    },
    UndefinedFunction {
        name: String,
        location: SourceLocation,
    },
    UndefinedVariable {
        name: String,
        location: SourceLocation,
    },
    InternalError {
        message: String,
        location: SourceLocation,
    },
}

impl From<CodegenError> for CompilerError {
    fn from(err: CodegenError) -> Self {
        match err {
            CodegenError::UnsupportedConstruct { construct, location } => {
                CompilerError::codegen_error(
                    format!("Unsupported construct: {}", construct),
                    location,
                )
            }
            CodegenError::InvalidType { ast_type, location } => {
                CompilerError::codegen_error(
                    format!("Cannot convert AST type to IR: {}", ast_type),
                    location,
                )
            }
            CodegenError::UndefinedFunction { name, location } => {
                CompilerError::codegen_error(
                    format!("Undefined function: {}", name),
                    location,
                )
            }
            CodegenError::UndefinedVariable { name, location } => {
                CompilerError::codegen_error(
                    format!("Undefined variable: {}", name),
                    location,
                )
            }
            CodegenError::InternalError { message, location } => {
                CompilerError::codegen_error(
                    format!("Internal codegen error: {}", message),
                    location,
                )
            }
        }
    }
}

/// Code generator context
pub struct CodeGenerator {
    module: Module,
    builder: IrBuilder,
    
    // Symbol mapping from AST to IR
    variables: HashMap<String, (Value, IrType)>, // Variable name -> (IR value, IR type)
    functions: HashMap<String, Value>, // Function name -> IR function value
    
    // Track which variables are arrays (for array-to-pointer decay)
    array_variables: HashSet<String>, // Variable names that are arrays
    
    // String literals
    string_literals: HashMap<String, String>, // String ID -> string content
    next_string_id: u32,
    
    // Current function context
    current_function: Option<String>,
    current_return_type: Option<IrType>,
    
    // Control flow context
    break_labels: Vec<LabelId>,
    continue_labels: Vec<LabelId>,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new(module_name: String) -> Self {
        Self {
            module: Module::new(module_name),
            builder: IrBuilder::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            array_variables: HashSet::new(),
            string_literals: HashMap::new(),
            next_string_id: 0,
            current_function: None,
            current_return_type: None,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
        }
    }
    
    /// Generate IR from AST
    pub fn generate(&mut self, ast: &TranslationUnit) -> Result<Module, CompilerError> {
        // First pass: collect function declarations and global variables
        for item in &ast.items {
            match item {
                TopLevelItem::Function(func) => {
                    self.declare_function(func)?;
                }
                TopLevelItem::Declaration(decl) => {
                    // Check if this is a function declaration (extern function)
                    if matches!(&decl.decl_type, Type::Function { .. }) {
                        self.declare_extern_function(decl)?;
                    } else {
                        self.generate_global_variable(decl)?;
                    }
                }
                TopLevelItem::TypeDefinition { .. } => {
                    // TODO: Handle type definitions
                }
            }
        }
        
        // Second pass: generate function bodies
        for item in &ast.items {
            match item {
                TopLevelItem::Function(func) => {
                    self.generate_function(func)?;
                }
                _ => {} // Already handled in first pass
            }
        }
        
        Ok(self.module.clone())
    }
    
    /// Declare a function (create IR function signature)
    fn declare_function(&mut self, func: &FunctionDefinition) -> Result<(), CompilerError> {
        let return_type = self.convert_type(&func.return_type, func.span.start.clone())?;
        let ir_func = Function::new(func.name.clone(), return_type);
        
        // Add to functions map
        let func_value = Value::Function(func.name.clone());
        self.functions.insert(func.name.clone(), func_value);
        
        Ok(())
    }
    
    /// Declare an extern function (function declaration without body)
    fn declare_extern_function(&mut self, decl: &Declaration) -> Result<(), CompilerError> {
        // Extract function type information
        let (return_type, _params) = match &decl.decl_type {
            Type::Function { return_type, parameters, .. } => {
                let ret_type = self.convert_type(return_type, decl.span.start.clone())?;
                // TODO: Store parameter types if needed for type checking calls
                (ret_type, parameters)
            }
            _ => {
                return Err(CodegenError::InternalError {
                    message: "Expected function type".to_string(),
                    location: decl.span.start.clone(),
                }.into());
            }
        };
        
        // For now, just register the function as available for calls
        // The actual implementation will be provided externally (e.g., runtime library)
        let func_value = Value::Function(decl.name.clone());
        self.functions.insert(decl.name.clone(), func_value);
        
        // Note: We don't add extern functions to the module's function list
        // They are expected to be provided by the runtime or linked separately
        
        Ok(())
    }
    
    /// Generate a global variable
    fn generate_global_variable(&mut self, decl: &Declaration) -> Result<(), CompilerError> {
        let ir_type = self.convert_type(&decl.decl_type, decl.span.start.clone())?;
        
        let initializer = if let Some(init) = &decl.initializer {
            Some(self.generate_constant_initializer(init)?)
        } else {
            None
        };
        
        let linkage = match decl.storage_class {
            StorageClass::Static => Linkage::Internal,
            StorageClass::Extern => Linkage::External,
            _ => Linkage::External,
        };
        
        let global = GlobalVariable {
            name: decl.name.clone(),
            var_type: ir_type.clone(),
            is_constant: false, // TODO: Handle const qualifier
            initializer,
            linkage,
            symbol_id: decl.symbol_id,
        };
        
        self.module.add_global(global);
        
        // Add to variables map (global variables are referenced by name)
        let global_value = Value::Global(decl.name.clone());
        self.variables.insert(decl.name.clone(), (global_value, IrType::Ptr(Box::new(ir_type))));
        
        Ok(())
    }
    
    /// Generate IR for a function
    fn generate_function(&mut self, func: &FunctionDefinition) -> Result<(), CompilerError> {
        let return_type = self.convert_type(&func.return_type, func.span.start.clone())?;
        
        // Set current function context
        self.current_function = Some(func.name.clone());
        self.current_return_type = Some(return_type.clone());
        
        // Convert parameter types first
        let mut param_info = Vec::new();
        for param in &func.parameters {
            if let Some(param_name) = &param.name {
                let param_type = self.convert_type(&param.param_type, param.span.start.clone())?;
                param_info.push((param_name.clone(), param_type));
            }
        }
        
        // Create IR function and add parameters
        {
            let ir_func = self.builder.create_function(func.name.clone(), return_type);
            
            // Add parameters 
            for (i, (param_name, param_type)) in param_info.iter().enumerate() {
                let temp_id = i as TempId; // Simple parameter numbering
                ir_func.add_parameter(temp_id, param_type.clone());
            }
        }
        
        // Register parameters as variables
        for (i, (param_name, param_type)) in param_info.into_iter().enumerate() {
            let temp_id = i as TempId;
            let param_value = Value::Temp(temp_id);
            self.variables.insert(param_name, (param_value, param_type));
        }
        
        // Create entry block
        let entry_label = self.builder.new_label();
        self.builder.create_block(entry_label)?;
        
        // Generate function body
        self.generate_statement(&func.body)?;
        
        // Ensure function has a return
        self.ensure_function_return()?;
        
        // Finish function and add to module
        let completed_function = self.builder.finish_function()
            .ok_or_else(|| CodegenError::InternalError {
                message: "Failed to finish function".to_string(),
                location: func.span.start.clone(),
            })?;
        
        self.module.add_function(completed_function);
        
        // Clear function context
        self.current_function = None;
        self.current_return_type = None;
        self.variables.clear(); // TODO: Only clear local variables
        self.array_variables.clear(); // Clear array tracking
        
        Ok(())
    }
    
    /// Generate IR for a statement
    fn generate_statement(&mut self, stmt: &Statement) -> Result<(), CompilerError> {
        match &stmt.kind {
            StatementKind::Expression(expr) => {
                // Generate expression and discard result
                self.generate_expression(expr)?;
                Ok(())
            }
            
            StatementKind::Compound(statements) => {
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }
                Ok(())
            }
            
            StatementKind::Declaration { declarations } => {
                for decl in declarations {
                    self.generate_local_declaration(decl)?;
                }
                Ok(())
            }
            
            StatementKind::If { condition, then_stmt, else_stmt } => {
                self.generate_if_statement(condition, then_stmt, else_stmt.as_deref())
            }
            
            StatementKind::While { condition, body } => {
                self.generate_while_loop(condition, body)
            }
            
            StatementKind::For { init, condition, update, body } => {
                self.generate_for_loop(init.as_deref(), condition.as_ref(), update.as_ref(), body)
            }
            
            StatementKind::DoWhile { body, condition } => {
                self.generate_do_while_loop(body, condition)
            }
            
            StatementKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let value = self.generate_expression(expr)?;
                    self.builder.build_return(Some(value))?;
                } else {
                    self.builder.build_return(None)?;
                }
                Ok(())
            }
            
            StatementKind::Break => {
                if let Some(&break_label) = self.break_labels.last() {
                    self.builder.build_branch(break_label)?;
                    Ok(())
                } else {
                    Err(CodegenError::UnsupportedConstruct {
                        construct: "break outside loop".to_string(),
                        location: stmt.span.start.clone(),
                    }.into())
                }
            }
            
            StatementKind::Continue => {
                if let Some(&continue_label) = self.continue_labels.last() {
                    self.builder.build_branch(continue_label)?;
                    Ok(())
                } else {
                    Err(CodegenError::UnsupportedConstruct {
                        construct: "continue outside loop".to_string(),
                        location: stmt.span.start.clone(),
                    }.into())
                }
            }
            
            StatementKind::Empty => Ok(()),
            
            // TODO: Implement other statement types
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: format!("statement type: {:?}", stmt.kind),
                location: stmt.span.start.clone(),
            }.into()),
        }
    }
    
    /// Generate IR for an expression
    fn generate_expression(&mut self, expr: &Expression) -> Result<Value, CompilerError> {
        match &expr.kind {
            ExpressionKind::IntLiteral(value) => {
                Ok(Value::Constant(*value))
            }
            
            ExpressionKind::CharLiteral(value) => {
                Ok(Value::Constant(*value as i64))
            }
            
            ExpressionKind::StringLiteral(s) => {
                // Create a unique name for this string literal
                let string_id = self.next_string_id;
                self.next_string_id += 1;
                let name = format!("__str_{}", string_id);
                
                // Add string as a global with the string data
                // Encode the string bytes in the variable name since we don't have
                // a proper .data section yet - the lowering phase will decode and emit it
                let encoded_name = format!("__str_{}_{}", string_id, 
                    s.bytes().map(|b| format!("{:02x}", b)).collect::<String>());
                
                let global = GlobalVariable {
                    name: encoded_name.clone(),
                    var_type: IrType::Array {
                        element_type: Box::new(IrType::I8),
                        size: (s.len() + 1) as u64, // +1 for null terminator
                    },
                    is_constant: true,
                    initializer: None, // String data is encoded in the name
                    linkage: Linkage::Internal,
                    symbol_id: None,
                };
                
                // Add to module
                self.module.add_global(global);
                
                // Store the string data for later
                self.string_literals.insert(encoded_name.clone(), s.clone());
                
                // Return a pointer to the string
                Ok(Value::Global(encoded_name))
            }
            
            ExpressionKind::Identifier { name, .. } => {
                if let Some((value, var_type)) = self.variables.get(name) {
                    // Check if this is a pointer type (which means it's a variable that needs to be loaded)
                    match var_type {
                        IrType::Ptr(element_type) => {
                            // Check if this is an array variable
                            if self.array_variables.contains(name) {
                                // Arrays decay to pointers when used as rvalues
                                // Return the address directly, don't load
                                Ok(value.clone())
                            } else {
                                // Regular variable, load its value
                                let temp = self.builder.build_load(value.clone(), *element_type.clone())?;
                                Ok(Value::Temp(temp))
                            }
                        }
                        _ => {
                            // This is a direct value (like function parameters), use as is
                            Ok(value.clone())
                        }
                    }
                } else {
                    Err(CodegenError::UndefinedVariable {
                        name: name.clone(),
                        location: expr.span.start.clone(),
                    }.into())
                }
            }
            
            ExpressionKind::Binary { op, left, right } => {
                self.generate_binary_operation(*op, left, right)
            }
            
            ExpressionKind::Unary { op, operand } => {
                self.generate_unary_operation(*op, operand)
            }
            
            ExpressionKind::Call { function, arguments } => {
                self.generate_function_call(function, arguments)
            }
            
            ExpressionKind::SizeofExpr(operand) => {
                // sizeof(expression) - get size of expression's type
                // Note: sizeof doesn't evaluate the expression
                if let Some(ref expr_type) = operand.expr_type {
                    let size = self.get_ast_type_size(expr_type);
                    Ok(Value::Constant(size as i64))
                } else {
                    // Fallback if no type info
                    Ok(Value::Constant(2))
                }
            }
            
            ExpressionKind::SizeofType(ast_type) => {
                // sizeof(type) - get size of the type
                let size = self.get_ast_type_size(ast_type);
                Ok(Value::Constant(size as i64))
            }
            
            ExpressionKind::Member { object, member, is_pointer } => {
                self.generate_member_access(object, member, *is_pointer)
            }
            
            // TODO: Implement other expression types
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: format!("expression type: {:?}", expr.kind),
                location: expr.span.start.clone(),
            }.into()),
        }
    }
    
    /// Generate binary operation
    fn generate_binary_operation(&mut self, op: BinaryOp, left: &Expression, right: &Expression) -> Result<Value, CompilerError> {
        match op {
            BinaryOp::Assign => {
                // For assignment, left should be an lvalue
                let rvalue = self.generate_expression(right)?;
                let lvalue_ptr = self.generate_lvalue(left)?;
                
                self.builder.build_store(rvalue.clone(), lvalue_ptr)?;
                Ok(rvalue)
            }
            
            BinaryOp::Index => {
                // Array indexing: arr[idx] = *(arr + idx)
                // First, get the base pointer
                let base_ptr = self.generate_expression(left)?;
                let index = self.generate_expression(right)?;
                
                // Calculate the address: ptr + index
                // For now, assume char arrays (1 byte per element)
                let addr = self.builder.build_binary(
                    IrBinaryOp::Add,
                    base_ptr,
                    index,
                    IrType::Ptr(Box::new(IrType::I8))
                )?;
                
                // Load from the computed address
                let result = self.builder.build_load(Value::Temp(addr), IrType::I8)?;
                Ok(Value::Temp(result))
            }
            
            _ => {
                let left_val = self.generate_expression(left)?;
                let right_val = self.generate_expression(right)?;
                let result_type = self.get_expression_ir_type(left)?; // Simplified
                
                let ir_op = self.convert_binary_op(op)?;
                let temp = self.builder.build_binary(ir_op, left_val, right_val, result_type)?;
                Ok(Value::Temp(temp))
            }
        }
    }
    
    /// Generate unary operation
    fn generate_unary_operation(&mut self, op: UnaryOp, operand: &Expression) -> Result<Value, CompilerError> {
        match op {
            UnaryOp::AddressOf => {
                // Return the address of the lvalue
                self.generate_lvalue(operand)
            }
            
            UnaryOp::Dereference => {
                // Load from the pointer
                let ptr = self.generate_expression(operand)?;
                let result_type = self.get_expression_ir_type(operand)?; // TODO: Get dereferenced type
                let temp = self.builder.build_load(ptr, result_type)?;
                Ok(Value::Temp(temp))
            }
            
            _ => {
                let operand_val = self.generate_expression(operand)?;
                let result_type = self.get_expression_ir_type(operand)?;
                
                match op {
                    UnaryOp::Minus => {
                        // Generate 0 - operand
                        let zero = Value::Constant(0);
                        let temp = self.builder.build_binary(IrBinaryOp::Sub, zero, operand_val, result_type)?;
                        Ok(Value::Temp(temp))
                    }
                    
                    UnaryOp::LogicalNot => {
                        // Generate operand == 0
                        let zero = Value::Constant(0);
                        let temp = self.builder.build_binary(IrBinaryOp::Eq, operand_val, zero, IrType::I1)?;
                        Ok(Value::Temp(temp))
                    }
                    
                    UnaryOp::Sizeof => {
                        // For sizeof(expression), we need the type of the expression
                        // Note: sizeof doesn't evaluate the expression, just gets its type
                        if let Some(ref expr_type) = operand.expr_type {
                            let size = self.get_ast_type_size(expr_type);
                            Ok(Value::Constant(size as i64))
                        } else {
                            // If no type info, try to infer from the expression
                            // This is a fallback, normally semantic analysis should provide the type
                            Ok(Value::Constant(2)) // Default to word size
                        }
                    }
                    
                    _ => Err(CodegenError::UnsupportedConstruct {
                        construct: format!("unary operator: {:?}", op),
                        location: operand.span.start.clone(),
                    }.into()),
                }
            }
        }
    }
    
    /// Generate function call
    fn generate_function_call(&mut self, function: &Expression, arguments: &[Expression]) -> Result<Value, CompilerError> {
        // Generate function reference and get function info
        let (func_val, func_name) = match &function.kind {
            ExpressionKind::Identifier { name, .. } => {
                if let Some(func_val) = self.functions.get(name) {
                    (func_val.clone(), name.clone())
                } else {
                    return Err(CodegenError::UndefinedFunction {
                        name: name.clone(),
                        location: function.span.start.clone(),
                    }.into());
                }
            }
            _ => {
                return Err(CodegenError::UnsupportedConstruct {
                    construct: "indirect function calls".to_string(),
                    location: function.span.start.clone(),
                }.into());
            }
        };
        
        // Generate arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            arg_values.push(self.generate_expression(arg)?);
        }
        
        // Get return type from the function type
        let return_type = if let Some(expr_type) = &function.expr_type {
            match expr_type {
                Type::Function { return_type, .. } => {
                    self.convert_type(return_type, function.span.start.clone())?
                }
                _ => {
                    // If not a function type, assume void for now
                    IrType::Void
                }
            }
        } else {
            // If type is unknown, assume void for now
            IrType::Void
        };
        
        if let Some(result_temp) = self.builder.build_call(func_val, arg_values, return_type)? {
            Ok(Value::Temp(result_temp))
        } else {
            Ok(Value::Constant(0)) // Void functions don't return a value
        }
    }
    
    /// Generate lvalue (address that can be assigned to)
    fn generate_lvalue(&mut self, expr: &Expression) -> Result<Value, CompilerError> {
        match &expr.kind {
            ExpressionKind::Identifier { name, .. } => {
                if let Some((value, _)) = self.variables.get(name) {
                    match value {
                        Value::Global(_) => Ok(value.clone()), // Global variables are already addresses
                        Value::Temp(_) => {
                            // Local variables should have been allocated
                            // For now, return the temp (this needs improvement)
                            Ok(value.clone())
                        }
                        _ => Err(CodegenError::InternalError {
                            message: "Invalid lvalue".to_string(),
                            location: expr.span.start.clone(),
                        }.into()),
                    }
                } else {
                    Err(CodegenError::UndefinedVariable {
                        name: name.clone(),
                        location: expr.span.start.clone(),
                    }.into())
                }
            }
            
            ExpressionKind::Unary { op: UnaryOp::Dereference, operand } => {
                // *ptr - the pointer itself is the address
                self.generate_expression(operand)
            }
            
            ExpressionKind::Binary { op: BinaryOp::Index, left, right } => {
                // arr[idx] as lvalue - compute the address
                let base_ptr = self.generate_expression(left)?;
                let index = self.generate_expression(right)?;
                
                // Calculate the address: ptr + index
                let addr = self.builder.build_binary(
                    IrBinaryOp::Add,
                    base_ptr,
                    index,
                    IrType::Ptr(Box::new(IrType::I8))
                )?;
                
                Ok(Value::Temp(addr))
            }
            
            ExpressionKind::Member { object, member, is_pointer } => {
                // Get struct type
                let obj_type = if let Some(ref expr_type) = object.expr_type {
                    expr_type
                } else {
                    return Err(CodegenError::InternalError {
                        message: "Object has no type information".to_string(),
                        location: object.span.start.clone(),
                    }.into());
                };
                
                // Handle pointer dereferencing if needed
                let struct_type = if *is_pointer {
                    match obj_type {
                        Type::Pointer(inner) => &**inner,
                        _ => {
                            return Err(CodegenError::InternalError {
                                message: format!("Expected pointer type for -> operator, found {:?}", obj_type),
                                location: object.span.start.clone(),
                            }.into());
                        }
                    }
                } else {
                    obj_type
                };
                
                // Extract struct fields
                let fields = match struct_type {
                    Type::Struct { fields, .. } => fields,
                    Type::Union { fields, .. } => fields,
                    _ => {
                        return Err(CodegenError::InternalError {
                            message: format!("Expected struct or union type, found {:?}", struct_type),
                            location: object.span.start.clone(),
                        }.into());
                    }
                };
                
                // Find the field and calculate offset
                let mut offset = 0u64;
                let mut found = false;
                for field in fields {
                    if field.name == *member {
                        found = true;
                        break;
                    }
                    // For struct, add field size to offset (no alignment for simplicity)
                    // For union, offset is always 0
                    if matches!(struct_type, Type::Struct { .. }) {
                        if let Some(size) = field.field_type.size_in_words() {
                            offset += size;
                        } else {
                            return Err(CodegenError::InternalError {
                                message: format!("Cannot compute size of field {}", field.name),
                                location: object.span.start.clone(),
                            }.into());
                        }
                    }
                }
                
                if !found {
                    return Err(CodegenError::UndefinedVariable {
                        name: format!("field '{}'", member),
                        location: object.span.start.clone(),
                    }.into());
                }
                
                // Generate the base address
                let base_addr = if *is_pointer {
                    // For pointer->member, load the pointer value
                    self.generate_expression(object)?
                } else {
                    // For object.member, get the address of the object
                    self.generate_lvalue(object)?
                };
                
                // Add offset if non-zero
                if offset > 0 {
                    let offset_val = Value::Constant(offset as i64);
                    let temp = self.builder.build_binary(IrBinaryOp::Add, base_addr, offset_val, IrType::Ptr(Box::new(IrType::I16)))?;
                    Ok(Value::Temp(temp))
                } else {
                    Ok(base_addr)
                }
            }
            
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: "complex lvalue".to_string(),
                location: expr.span.start.clone(),
            }.into()),
        }
    }
    
    /// Generate local variable declaration
    fn generate_local_declaration(&mut self, decl: &Declaration) -> Result<(), CompilerError> {
        let ir_type = self.convert_type(&decl.decl_type, decl.span.start.clone())?;
        
        // For arrays, we need to handle allocation differently
        let (alloca_temp, var_type, is_array) = match &ir_type {
            IrType::Array { size, element_type } => {
                // Allocate array - alloca returns pointer to first element
                let count = Some(Value::Constant(*size as i64));
                let alloca_temp = self.builder.build_alloca((**element_type).clone(), count)?;
                // The variable type is pointer to element type (array decays to pointer)
                (alloca_temp, IrType::Ptr(element_type.clone()), true)
            }
            _ => {
                // Regular scalar allocation
                let alloca_temp = self.builder.build_alloca(ir_type.clone(), None)?;
                (alloca_temp, IrType::Ptr(Box::new(ir_type.clone())), false)
            }
        };
        
        // If there's an initializer, store it
        if let Some(init) = &decl.initializer {
            let init_value = self.generate_initializer(init)?;
            self.builder.build_store(init_value, Value::Temp(alloca_temp))?;
        }
        
        // Add to variables map
        let var_value = Value::Temp(alloca_temp);
        self.variables.insert(decl.name.clone(), (var_value, var_type));
        
        // Track if this is an array
        if is_array {
            self.array_variables.insert(decl.name.clone());
        }
        
        Ok(())
    }
    
    /// Generate if statement
    fn generate_if_statement(&mut self, condition: &Expression, then_stmt: &Statement, else_stmt: Option<&Statement>) -> Result<(), CompilerError> {
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
        self.generate_statement(then_stmt)?;
        self.builder.build_branch(end_label)?;
        
        // Generate else block if present
        if let Some(else_stmt) = else_stmt {
            self.builder.create_block(else_label)?;
            self.generate_statement(else_stmt)?;
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
        
        self.generate_statement(body)?;
        
        // Clean up break/continue labels
        self.break_labels.pop();
        self.continue_labels.pop();
        
        self.builder.build_branch(header_label)?; // Loop back
        
        // End
        self.builder.create_block(end_label)?;
        
        Ok(())
    }
    
    /// Generate for loop
    fn generate_for_loop(
        &mut self, 
        init: Option<&Statement>, 
        condition: Option<&Expression>, 
        update: Option<&Expression>, 
        body: &Statement
    ) -> Result<(), CompilerError> {
        let header_label = self.builder.new_label();
        let body_label = self.builder.new_label();
        let update_label = self.builder.new_label();
        let end_label = self.builder.new_label();
        
        // Init
        if let Some(init) = init {
            self.generate_statement(init)?;
        }
        
        // Jump to header
        self.builder.build_branch(header_label)?;
        
        // Header: check condition
        self.builder.create_block(header_label)?;
        if let Some(condition) = condition {
            let cond_val = self.generate_expression(condition)?;
            self.builder.build_branch_cond(cond_val, body_label, end_label)?;
        } else {
            self.builder.build_branch(body_label)?;
        }
        
        // Body
        self.builder.create_block(body_label)?;
        
        // Set up break/continue labels
        self.break_labels.push(end_label);
        self.continue_labels.push(update_label);
        
        self.generate_statement(body)?;
        
        // Clean up break/continue labels
        self.break_labels.pop();
        self.continue_labels.pop();
        
        self.builder.build_branch(update_label)?;
        
        // Update
        self.builder.create_block(update_label)?;
        if let Some(update) = update {
            self.generate_expression(update)?;
        }
        self.builder.build_branch(header_label)?; // Loop back
        
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
        
        self.generate_statement(body)?;
        
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
    
    /// Generate member access (object.member or pointer->member)
    fn generate_member_access(&mut self, object: &Expression, member: &str, is_pointer: bool) -> Result<Value, CompilerError> {
        // Get the object type
        let obj_type = if let Some(ref expr_type) = object.expr_type {
            expr_type
        } else {
            return Err(CodegenError::InternalError {
                message: "Object has no type information".to_string(),
                location: object.span.start.clone(),
            }.into());
        };
        
        // Handle pointer dereferencing if needed
        let struct_type = if is_pointer {
            // For pointer->member, we need to get the pointed-to type
            match obj_type {
                Type::Pointer(inner) => &**inner,
                _ => {
                    return Err(CodegenError::InternalError {
                        message: format!("Expected pointer type for -> operator, found {:?}", obj_type),
                        location: object.span.start.clone(),
                    }.into());
                }
            }
        } else {
            // For object.member, use the type directly
            obj_type
        };
        
        // Extract struct fields
        let fields = match struct_type {
            Type::Struct { fields, .. } => fields,
            Type::Union { fields, .. } => fields,
            _ => {
                return Err(CodegenError::InternalError {
                    message: format!("Expected struct or union type, found {:?}", struct_type),
                    location: object.span.start.clone(),
                }.into());
            }
        };
        
        // Find the field and calculate offset
        let mut offset = 0u64;
        let mut field_type = None;
        for field in fields {
            if field.name == member {
                field_type = Some(&field.field_type);
                break;
            }
            // For struct, add field size to offset (no alignment for simplicity)
            // For union, offset is always 0
            if matches!(struct_type, Type::Struct { .. }) {
                if let Some(size) = field.field_type.size_in_words() {
                    offset += size;
                } else {
                    return Err(CodegenError::InternalError {
                        message: format!("Cannot compute size of field {}", field.name),
                        location: object.span.start.clone(),
                    }.into());
                }
            }
        }
        
        let field_type = field_type.ok_or_else(|| {
            CodegenError::UndefinedVariable {
                name: format!("field '{}'", member),
                location: object.span.start.clone(),
            }
        })?;
        
        // Generate the base address
        let base_addr = if is_pointer {
            // For pointer->member, load the pointer value
            self.generate_expression(object)?
        } else {
            // For object.member, get the address of the object
            self.generate_lvalue(object)?
        };
        
        // Add offset if non-zero
        let field_addr = if offset > 0 {
            let offset_val = Value::Constant(offset as i64);
            let temp = self.builder.build_binary(IrBinaryOp::Add, base_addr, offset_val, IrType::Ptr(Box::new(IrType::I16)))?;
            Value::Temp(temp)
        } else {
            base_addr
        };
        
        // For member access in rvalue context, load the value
        // The field address is what we need for lvalue context
        // Since we're in generate_expression (rvalue context), load the value
        let ir_field_type = self.convert_type(field_type, object.span.start.clone())?;
        let result = self.builder.build_load(field_addr, ir_field_type)?;
        Ok(Value::Temp(result))
    }
    
    /// Generate constant initializer
    fn generate_constant_initializer(&mut self, init: &Initializer) -> Result<Value, CompilerError> {
        match &init.kind {
            InitializerKind::Expression(expr) => {
                match &expr.kind {
                    ExpressionKind::IntLiteral(val) => Ok(Value::Constant(*val)),
                    ExpressionKind::CharLiteral(val) => Ok(Value::Constant(*val as i64)),
                    _ => Err(CodegenError::UnsupportedConstruct {
                        construct: "non-constant initializer".to_string(),
                        location: init.span.start.clone(),
                    }.into()),
                }
            }
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: "complex initializer".to_string(),
                location: init.span.start.clone(),
            }.into()),
        }
    }
    
    /// Generate initializer for local variables
    fn generate_initializer(&mut self, init: &Initializer) -> Result<Value, CompilerError> {
        match &init.kind {
            InitializerKind::Expression(expr) => {
                self.generate_expression(expr)
            }
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: "complex initializer".to_string(),
                location: init.span.start.clone(),
            }.into()),
        }
    }
    
    /// Convert AST type to IR type
    fn convert_type(&self, ast_type: &Type, location: SourceLocation) -> Result<IrType, CompilerError> {
        match ast_type {
            Type::Void => Ok(IrType::Void),
            Type::Bool => Ok(IrType::I1),
            Type::Char | Type::SignedChar | Type::UnsignedChar => Ok(IrType::I8),
            Type::Short | Type::UnsignedShort => Ok(IrType::I16),
            Type::Int | Type::UnsignedInt => Ok(IrType::I16), // 16-bit int on Ripple
            Type::Long | Type::UnsignedLong => Ok(IrType::I32),
            Type::Pointer(target) => {
                let target_type = self.convert_type(target, location)?;
                Ok(IrType::Ptr(Box::new(target_type)))
            }
            Type::Array { element_type, size } => {
                let elem_type = self.convert_type(element_type, location)?;
                if let Some(size) = size {
                    Ok(IrType::Array { size: *size, element_type: Box::new(elem_type) })
                } else {
                    // Incomplete array type - treat as pointer for now
                    Ok(IrType::Ptr(Box::new(elem_type)))
                }
            }
            Type::Struct { fields, .. } => {
                // For structs, allocate as array of words
                // Calculate total size in words
                let mut total_words = 0u64;
                for field in fields {
                    if let Some(size) = field.field_type.size_in_words() {
                        total_words += size;
                    } else {
                        return Err(CodegenError::InternalError {
                            message: format!("Cannot compute size of struct field: {}", field.name),
                            location,
                        }.into());
                    }
                }
                // Return as array of I16 (words)
                Ok(IrType::Array { size: total_words, element_type: Box::new(IrType::I16) })
            }
            Type::Union { fields, .. } => {
                // For unions, allocate the size of the largest field
                let mut max_words = 0u64;
                for field in fields {
                    if let Some(size) = field.field_type.size_in_words() {
                        max_words = max_words.max(size);
                    } else {
                        return Err(CodegenError::InternalError {
                            message: format!("Cannot compute size of union field: {}", field.name),
                            location,
                        }.into());
                    }
                }
                // Return as array of I16 (words)
                Ok(IrType::Array { size: max_words, element_type: Box::new(IrType::I16) })
            }
            _ => Err(CodegenError::InvalidType {
                ast_type: ast_type.clone(),
                location,
            }.into()),
        }
    }
    
    /// Get the size of an AST type in bytes
    fn get_ast_type_size(&self, ast_type: &Type) -> u64 {
        match ast_type {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::SignedChar | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt => 2, // 16-bit int on Ripple
            Type::Long | Type::UnsignedLong => 4,
            Type::Pointer(_) => 2, // 16-bit pointers
            Type::Array { element_type, size } => {
                let elem_size = self.get_ast_type_size(element_type);
                if let Some(size) = size {
                    elem_size * size
                } else {
                    // Incomplete array type
                    0
                }
            }
            Type::Function { .. } => 0, // Functions don't have size
            Type::Struct { .. } | Type::Union { .. } => {
                // Use the size_in_bytes method from Type
                ast_type.size_in_bytes().unwrap_or(0)
            }
            _ => 0, // TODO: Handle other types
        }
    }
    
    /// Get the size of an IR type in bytes
    fn get_ir_type_size(&self, ir_type: &IrType) -> u64 {
        match ir_type {
            IrType::Void => 0,
            IrType::I1 => 1,
            IrType::I8 => 1,
            IrType::I16 => 2,
            IrType::I32 => 4,
            IrType::I64 => 8,
            IrType::Ptr(_) => 2, // 16-bit pointers
            IrType::Array { element_type, size } => {
                let elem_size = self.get_ir_type_size(element_type);
                elem_size * size
            }
            IrType::Function { .. } => 0,
            IrType::Struct { .. } => 0, // TODO: Calculate struct size
            IrType::Label => 0,
        }
    }
    
    /// Convert AST binary operator to IR binary operator
    fn convert_binary_op(&self, op: BinaryOp) -> Result<IrBinaryOp, CompilerError> {
        match op {
            BinaryOp::Add => Ok(IrBinaryOp::Add),
            BinaryOp::Sub => Ok(IrBinaryOp::Sub),
            BinaryOp::Mul => Ok(IrBinaryOp::Mul),
            BinaryOp::Div => Ok(IrBinaryOp::SDiv), // Assume signed for now
            BinaryOp::Mod => Ok(IrBinaryOp::SRem),
            BinaryOp::BitAnd => Ok(IrBinaryOp::And),
            BinaryOp::BitOr => Ok(IrBinaryOp::Or),
            BinaryOp::BitXor => Ok(IrBinaryOp::Xor),
            BinaryOp::LeftShift => Ok(IrBinaryOp::Shl),
            BinaryOp::RightShift => Ok(IrBinaryOp::AShr), // Assume arithmetic for now
            BinaryOp::Equal => Ok(IrBinaryOp::Eq),
            BinaryOp::NotEqual => Ok(IrBinaryOp::Ne),
            BinaryOp::Less => Ok(IrBinaryOp::Slt),
            BinaryOp::Greater => Ok(IrBinaryOp::Sgt),
            BinaryOp::LessEqual => Ok(IrBinaryOp::Sle),
            BinaryOp::GreaterEqual => Ok(IrBinaryOp::Sge),
            _ => Err(CodegenError::UnsupportedConstruct {
                construct: format!("binary operator: {:?}", op),
                location: SourceLocation::new_simple(0, 0), // TODO: Better location tracking
            }.into()),
        }
    }
    
    /// Get the IR type for an expression
    fn get_expression_ir_type(&self, expr: &Expression) -> Result<IrType, CompilerError> {
        if let Some(ast_type) = &expr.expr_type {
            self.convert_type(ast_type, expr.span.start.clone())
        } else {
            // Default to int if type is unknown
            Ok(IrType::I16)
        }
    }
    
    /// Ensure function has a proper return statement
    fn ensure_function_return(&mut self) -> Result<(), CompilerError> {
        // Only add a default return if the current block doesn't already have a terminator
        if !self.builder.current_block_has_terminator() {
            if let Some(return_type) = &self.current_return_type {
                match return_type {
                    IrType::Void => {
                        self.builder.build_return(None)?;
                    }
                    _ => {
                        // Return 0 for non-void functions without explicit return
                        self.builder.build_return(Some(Value::Constant(0)))?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frontend;

    #[test]
    fn test_codegen_simple_function() {
        let source = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let mut codegen = CodeGenerator::new("test".to_string());
        
        let module = codegen.generate(&ast).unwrap();
        assert_eq!(module.functions.len(), 1);
        
        let function = &module.functions[0];
        assert_eq!(function.name, "add");
        assert_eq!(function.return_type, IrType::I16);
        assert_eq!(function.parameters.len(), 2);
    }

    #[test]
    fn test_codegen_global_variable() {
        let source = r#"
int global_var = 42;

int main() {
    return global_var;
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let mut codegen = CodeGenerator::new("test".to_string());
        
        let module = codegen.generate(&ast).unwrap();
        assert_eq!(module.globals.len(), 1);
        assert_eq!(module.functions.len(), 1);
        
        let global = &module.globals[0];
        assert_eq!(global.name, "global_var");
        assert_eq!(global.var_type, IrType::I16);
        assert_eq!(global.initializer, Some(Value::Constant(42)));
    }

    #[test]
    fn test_codegen_local_variables() {
        let source = r#"
int main() {
    int x = 10;
    int y = 20;
    return x + y;
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let mut codegen = CodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert_eq!(module.functions.len(), 1);
        
        let function = &module.functions[0];
        assert_eq!(function.name, "main");
        assert!(!function.blocks.is_empty());
    }

    #[test]
    fn test_codegen_if_statement() {
        let source = r#"
int abs(int x) {
    if (x < 0) {
        return -x;
    } else {
        return x;
    }
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let mut codegen = CodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = &module.functions[0];
        assert_eq!(function.name, "abs");
        
        // Should have multiple basic blocks for if-else
        assert!(function.blocks.len() >= 3);
    }

    #[test]
    fn test_codegen_while_loop() {
        let source = r#"
int factorial(int n) {
    int result = 1;
    while (n > 0) {
        result = result * n;
        n = n - 1;
    }
    return result;
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let mut codegen = CodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = &module.functions[0];
        assert_eq!(function.name, "factorial");
        
        // Should have multiple basic blocks for loop
        assert!(function.blocks.len() >= 3);
    }
}