//! Semantic Analysis for C99
//! 
//! Performs type checking, symbol resolution, and semantic validation
//! on the AST produced by the parser.

use crate::ast::*;
use rcc_common::{CompilerError, SymbolTable, SymbolId, Symbol, SourceLocation};
use std::collections::HashMap;

/// Semantic analysis errors
#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable {
        name: String,
        location: SourceLocation,
    },
    TypeMismatch {
        expected: Type,
        found: Type,
        location: SourceLocation,
    },
    RedefinedSymbol {
        name: String,
        original_location: SourceLocation,
        redefinition_location: SourceLocation,
    },
    InvalidOperation {
        operation: String,
        operand_type: Type,
        location: SourceLocation,
    },
    InvalidFunctionCall {
        function_type: Type,
        location: SourceLocation,
    },
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
        location: SourceLocation,
    },
    ReturnTypeMismatch {
        expected: Type,
        found: Type,
        location: SourceLocation,
    },
    InvalidLvalue {
        location: SourceLocation,
    },
    RedefinedType {
        name: String,
    },
}

impl From<SemanticError> for CompilerError {
    fn from(err: SemanticError) -> Self {
        match err {
            SemanticError::UndefinedVariable { name, location } => {
                CompilerError::semantic_error(
                    format!("Undefined variable: {}", name),
                    location,
                )
            }
            SemanticError::TypeMismatch { expected, found, location } => {
                CompilerError::semantic_error(
                    format!("Type mismatch: expected {}, found {}", expected, found),
                    location,
                )
            }
            SemanticError::RedefinedSymbol { name, redefinition_location, .. } => {
                CompilerError::semantic_error(
                    format!("Redefinition of symbol: {}", name),
                    redefinition_location,
                )
            }
            SemanticError::InvalidOperation { operation, operand_type, location } => {
                CompilerError::semantic_error(
                    format!("Invalid operation {} on type {}", operation, operand_type),
                    location,
                )
            }
            SemanticError::InvalidFunctionCall { function_type, location } => {
                CompilerError::semantic_error(
                    format!("Cannot call non-function type {}", function_type),
                    location,
                )
            }
            SemanticError::ArgumentCountMismatch { expected, found, location } => {
                CompilerError::semantic_error(
                    format!("Function call: expected {} arguments, found {}", expected, found),
                    location,
                )
            }
            SemanticError::ReturnTypeMismatch { expected, found, location } => {
                CompilerError::semantic_error(
                    format!("Return type mismatch: expected {}, found {}", expected, found),
                    location,
                )
            }
            SemanticError::InvalidLvalue { location } => {
                CompilerError::semantic_error(
                    "Invalid lvalue in assignment".to_string(),
                    location,
                )
            }
            SemanticError::RedefinedType { name } => {
                CompilerError::semantic_error(
                    format!("Redefinition of type: {}", name),
                    SourceLocation::new_simple(0, 0), // TODO: Track location
                )
            }
        }
    }
}

/// Semantic analyzer context
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    current_function: Option<Type>, // Current function's return type
    symbol_locations: HashMap<SymbolId, SourceLocation>, // For error reporting
    symbol_types: HashMap<SymbolId, Type>, // Type information for each symbol
    type_definitions: HashMap<String, Type>, // Named type definitions (structs, unions, enums)
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            current_function: None,
            symbol_locations: HashMap::new(),
            symbol_types: HashMap::new(),
            type_definitions: HashMap::new(),
        }
    }
    
    /// Analyze a translation unit
    pub fn analyze(&mut self, ast: &mut TranslationUnit) -> Result<(), CompilerError> {
        // First pass: collect all function and global variable declarations
        for item in &mut ast.items {
            match item {
                TopLevelItem::Function(func) => {
                    self.declare_function(func)?;
                }
                TopLevelItem::Declaration(decl) => {
                    self.declare_global_variable(decl)?;
                }
                TopLevelItem::TypeDefinition { name, type_def, .. } => {
                    self.register_type_definition(name.clone(), type_def.clone())?;
                }
            }
        }
        
        // Second pass: analyze function bodies
        for item in &mut ast.items {
            match item {
                TopLevelItem::Function(func) => {
                    self.analyze_function(func)?;
                }
                _ => {} // Already handled in first pass
            }
        }
        
        Ok(())
    }
    
    /// Declare a function in the symbol table
    fn declare_function(&mut self, func: &FunctionDefinition) -> Result<(), CompilerError> {
        if self.symbol_table.exists_in_current_scope(&func.name) {
            return Err(SemanticError::RedefinedSymbol {
                name: func.name.clone(),
                original_location: SourceLocation::new_simple(0, 0), // TODO: Track original location
                redefinition_location: func.span.start.clone(),
            }.into());
        }
        
        let func_type = Type::Function {
            return_type: Box::new(func.return_type.clone()),
            parameters: func.parameters.iter().map(|p| p.param_type.clone()).collect(),
            is_variadic: false, // TODO: Handle variadic functions
        };
        
        let symbol_id = self.symbol_table.add_symbol(func.name.clone());
        self.symbol_locations.insert(symbol_id, func.span.start.clone());
        // Store the function type
        self.symbol_types.insert(symbol_id, func_type);
        
        if let Some(symbol) = self.symbol_table.get_symbol_mut(symbol_id) {
            *symbol = symbol.clone()
                .as_function()
                .as_defined()
                .with_storage_class(
                    match func.storage_class {
                        crate::ast::StorageClass::Static => rcc_common::StorageClass::Static,
                        crate::ast::StorageClass::Extern => rcc_common::StorageClass::Extern,
                        _ => rcc_common::StorageClass::Auto,
                    }
                );
        }
        
        Ok(())
    }
    
    /// Declare a global variable in the symbol table
    fn declare_global_variable(&mut self, decl: &mut Declaration) -> Result<(), CompilerError> {
        // Handle typedef specially - it defines a type alias, not a variable
        if decl.storage_class == crate::ast::StorageClass::Typedef {
            // Resolve the type first
            decl.decl_type = self.resolve_type(&decl.decl_type);
            // Register this as a type definition
            self.type_definitions.insert(decl.name.clone(), decl.decl_type.clone());
            return Ok(());
        }
        
        if self.symbol_table.exists_in_current_scope(&decl.name) {
            return Err(SemanticError::RedefinedSymbol {
                name: decl.name.clone(),
                original_location: SourceLocation::new_simple(0, 0), // TODO: Track original location
                redefinition_location: decl.span.start.clone(),
            }.into());
        }
        
        // Resolve the type (in case it references a named struct/union/enum)
        decl.decl_type = self.resolve_type(&decl.decl_type);
        
        let symbol_id = self.symbol_table.add_symbol(decl.name.clone());
        self.symbol_locations.insert(symbol_id, decl.span.start.clone());
        // Store the global variable type
        self.symbol_types.insert(symbol_id, decl.decl_type.clone());
        
        if let Some(symbol) = self.symbol_table.get_symbol_mut(symbol_id) {
            *symbol = symbol.clone()
                .with_storage_class(
                    match decl.storage_class {
                        crate::ast::StorageClass::Static => rcc_common::StorageClass::Static,
                        crate::ast::StorageClass::Extern => rcc_common::StorageClass::Extern,
                        crate::ast::StorageClass::Register => rcc_common::StorageClass::Register,
                        _ => rcc_common::StorageClass::Auto,
                    }
                );
        }
        
        Ok(())
    }
    
    /// Register a type definition (struct, union, enum)
    fn register_type_definition(&mut self, name: String, type_def: Type) -> Result<(), CompilerError> {
        // Check if type already exists
        if self.type_definitions.contains_key(&name) {
            return Err(SemanticError::RedefinedType {
                name: name.clone(),
            }.into());
        }
        
        // Store the type definition
        self.type_definitions.insert(name, type_def);
        
        Ok(())
    }
    
    /// Resolve a type reference (e.g., struct Point -> actual struct definition)
    fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Typedef(name) => {
                // Look up typedef name
                if let Some(actual_type) = self.type_definitions.get(name) {
                    // Recursively resolve in case of typedef chains
                    self.resolve_type(actual_type)
                } else {
                    // Type not found, return as-is
                    ty.clone()
                }
            }
            Type::Struct { name: Some(name), fields } if fields.is_empty() => {
                // This is a reference to a named struct type
                if let Some(actual_type) = self.type_definitions.get(name) {
                    actual_type.clone()
                } else {
                    // Type not found, return as-is
                    ty.clone()
                }
            }
            Type::Union { name: Some(name), fields } if fields.is_empty() => {
                // This is a reference to a named union type
                if let Some(actual_type) = self.type_definitions.get(name) {
                    actual_type.clone()
                } else {
                    ty.clone()
                }
            }
            Type::Pointer(inner) => {
                // Recursively resolve pointed-to type
                Type::Pointer(Box::new(self.resolve_type(inner)))
            }
            Type::Array { element_type, size } => {
                // Recursively resolve element type
                Type::Array {
                    element_type: Box::new(self.resolve_type(element_type)),
                    size: *size,
                }
            }
            _ => ty.clone(),
        }
    }
    
    /// Analyze a function definition
    fn analyze_function(&mut self, func: &mut FunctionDefinition) -> Result<(), CompilerError> {
        // Set current function context
        self.current_function = Some(func.return_type.clone());
        
        // Enter function scope
        self.symbol_table.push_scope();
        
        // Add parameters to scope
        for param in &mut func.parameters {
            if let Some(name) = &param.name {
                let symbol_id = self.symbol_table.add_symbol(name.clone());
                param.symbol_id = Some(symbol_id);
                self.symbol_locations.insert(symbol_id, param.span.start.clone());
                // Store the parameter type
                self.symbol_types.insert(symbol_id, param.param_type.clone());
            }
        }
        
        // Analyze function body
        self.analyze_statement(&mut func.body)?;
        
        // Exit function scope
        self.symbol_table.pop_scope();
        self.current_function = None;
        
        Ok(())
    }
    
    /// Analyze a statement
    fn analyze_statement(&mut self, stmt: &mut Statement) -> Result<(), CompilerError> {
        match &mut stmt.kind {
            StatementKind::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            
            StatementKind::Compound(statements) => {
                // Enter new scope for compound statement
                self.symbol_table.push_scope();
                
                for stmt in statements {
                    self.analyze_statement(stmt)?;
                }
                
                self.symbol_table.pop_scope();
            }
            
            StatementKind::Declaration { declarations } => {
                for decl in declarations {
                    self.analyze_declaration(decl)?;
                }
            }
            
            StatementKind::If { condition, then_stmt, else_stmt } => {
                self.analyze_expression(condition)?;
                self.check_boolean_context(condition)?;
                
                self.analyze_statement(then_stmt)?;
                
                if let Some(else_stmt) = else_stmt {
                    self.analyze_statement(else_stmt)?;
                }
            }
            
            StatementKind::While { condition, body } => {
                self.analyze_expression(condition)?;
                self.check_boolean_context(condition)?;
                
                self.analyze_statement(body)?;
            }
            
            StatementKind::For { init, condition, update, body } => {
                // Enter new scope for for-loop
                self.symbol_table.push_scope();
                
                if let Some(init) = init {
                    self.analyze_statement(init)?;
                }
                
                if let Some(condition) = condition {
                    self.analyze_expression(condition)?;
                    self.check_boolean_context(condition)?;
                }
                
                if let Some(update) = update {
                    self.analyze_expression(update)?;
                }
                
                self.analyze_statement(body)?;
                
                self.symbol_table.pop_scope();
            }
            
            StatementKind::DoWhile { body, condition } => {
                self.analyze_statement(body)?;
                self.analyze_expression(condition)?;
                self.check_boolean_context(condition)?;
            }
            
            StatementKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.analyze_expression(expr)?;
                    
                    // Check return type compatibility
                    if let Some(expected_return_type) = &self.current_function {
                        if let Some(expr_type) = &expr.expr_type {
                            if !expected_return_type.is_assignable_from(expr_type) {
                                return Err(SemanticError::ReturnTypeMismatch {
                                    expected: expected_return_type.clone(),
                                    found: expr_type.clone(),
                                    location: expr.span.start.clone(),
                                }.into());
                            }
                        }
                    }
                } else {
                    // Return with no value - check if function returns void
                    if let Some(expected_return_type) = &self.current_function {
                        if !matches!(expected_return_type, Type::Void) {
                            return Err(SemanticError::ReturnTypeMismatch {
                                expected: expected_return_type.clone(),
                                found: Type::Void,
                                location: stmt.span.start.clone(),
                            }.into());
                        }
                    }
                }
            }
            
            StatementKind::Break | StatementKind::Continue | StatementKind::Empty => {
                // No semantic analysis needed
            }
            
            StatementKind::InlineAsm { assembly: _ } => {
                // Inline assembly - no semantic analysis needed for now
                // The assembly code will be passed through directly to the backend
            }
            
            // TODO: Handle other statement types
            _ => {
                // For now, skip unimplemented statement types
            }
        }
        
        Ok(())
    }
    
    /// Analyze a declaration
    fn analyze_declaration(&mut self, decl: &mut Declaration) -> Result<(), CompilerError> {
        // Handle typedef specially - it defines a type alias, not a variable
        if decl.storage_class == crate::ast::StorageClass::Typedef {
            // Resolve the type first
            decl.decl_type = self.resolve_type(&decl.decl_type);
            // Register this as a type definition
            self.type_definitions.insert(decl.name.clone(), decl.decl_type.clone());
            return Ok(());
        }
        
        // Check for redefinition in current scope
        if self.symbol_table.exists_in_current_scope(&decl.name) {
            return Err(SemanticError::RedefinedSymbol {
                name: decl.name.clone(),
                original_location: SourceLocation::new_simple(0, 0), // TODO: Track original location
                redefinition_location: decl.span.start.clone(),
            }.into());
        }
        
        // Resolve the type (in case it references a named struct/union/enum or typedef)
        decl.decl_type = self.resolve_type(&decl.decl_type);
        
        // Add to symbol table
        let symbol_id = self.symbol_table.add_symbol(decl.name.clone());
        decl.symbol_id = Some(symbol_id);
        self.symbol_locations.insert(symbol_id, decl.span.start.clone());
        // Store the variable type
        self.symbol_types.insert(symbol_id, decl.decl_type.clone());
        
        // Analyze initializer if present
        if let Some(initializer) = &mut decl.initializer {
            self.analyze_initializer(initializer, &decl.decl_type)?;
        }
        
        Ok(())
    }
    
    /// Analyze an initializer
    fn analyze_initializer(&mut self, init: &mut Initializer, expected_type: &Type) -> Result<(), CompilerError> {
        match &mut init.kind {
            InitializerKind::Expression(expr) => {
                self.analyze_expression(expr)?;
                
                // Check type compatibility
                if let Some(expr_type) = &expr.expr_type {
                    if !expected_type.is_assignable_from(expr_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: expr_type.clone(),
                            location: expr.span.start.clone(),
                        }.into());
                    }
                }
            }
            
            InitializerKind::List(initializers) => {
                // Handle array/struct initialization
                match expected_type {
                    Type::Array { element_type, .. } => {
                        for init in initializers {
                            self.analyze_initializer(init, element_type)?;
                        }
                    }
                    Type::Struct { fields, .. } => {
                        // Match initializers to fields
                        for (init, field) in initializers.iter_mut().zip(fields.iter()) {
                            self.analyze_initializer(init, &field.field_type)?;
                        }
                    }
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: expected_type.clone(),
                            found: Type::Error, // Placeholder
                            location: init.span.start.clone(),
                        }.into());
                    }
                }
            }
            
            InitializerKind::Designated { .. } => {
                // TODO: Handle designated initializers
            }
        }
        
        Ok(())
    }
    
    /// Analyze an expression and infer its type
    fn analyze_expression(&mut self, expr: &mut Expression) -> Result<(), CompilerError> {
        let expr_type = match &mut expr.kind {
            ExpressionKind::IntLiteral(_) => Type::Int,
            ExpressionKind::CharLiteral(_) => Type::Char,
            ExpressionKind::StringLiteral(_) => Type::Array { 
                element_type: Box::new(Type::Char),
                size: None, // TODO: Calculate string length
            },
            
            ExpressionKind::Identifier { name, symbol_id } => {
                // Look up in symbol table
                if let Some(id) = self.symbol_table.lookup(name) {
                    *symbol_id = Some(id);
                    // Get the actual type from our type mapping
                    self.symbol_types.get(&id).cloned().unwrap_or(Type::Int)
                } else {
                    return Err(SemanticError::UndefinedVariable {
                        name: name.clone(),
                        location: expr.span.start.clone(),
                    }.into());
                }
            }
            
            ExpressionKind::Binary { op, left, right } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)?;
                
                self.analyze_binary_operation(*op, left, right)?
            }
            
            ExpressionKind::Unary { op, operand } => {
                self.analyze_expression(operand)?;
                
                self.analyze_unary_operation(*op, operand)?
            }
            
            ExpressionKind::Call { function, arguments } => {
                self.analyze_expression(function)?;
                
                for arg in arguments.iter_mut() {
                    self.analyze_expression(arg)?;
                }
                
                // Check if function is callable
                if let Some(func_type) = &function.expr_type {
                    match func_type {
                        Type::Function { return_type, parameters, .. } => {
                            // Check argument count
                            if arguments.len() != parameters.len() {
                                return Err(SemanticError::ArgumentCountMismatch {
                                    expected: parameters.len(),
                                    found: arguments.len(),
                                    location: expr.span.start.clone(),
                                }.into());
                            }
                            
                            // Check argument types
                            for (arg, param_type) in arguments.iter().zip(parameters.iter()) {
                                if let Some(arg_type) = &arg.expr_type {
                                    if !param_type.is_assignable_from(arg_type) {
                                        return Err(SemanticError::TypeMismatch {
                                            expected: param_type.clone(),
                                            found: arg_type.clone(),
                                            location: arg.span.start.clone(),
                                        }.into());
                                    }
                                }
                            }
                            
                            *return_type.clone()
                        }
                        _ => {
                            return Err(SemanticError::InvalidFunctionCall {
                                function_type: func_type.clone(),
                                location: expr.span.start.clone(),
                            }.into());
                        }
                    }
                } else {
                    Type::Error
                }
            }
            
            ExpressionKind::Member { object, member: _, is_pointer: _ } => {
                self.analyze_expression(object)?;
                // TODO: Implement member access type checking
                Type::Int // Placeholder
            }
            
            ExpressionKind::Conditional { condition, then_expr, else_expr } => {
                self.analyze_expression(condition)?;
                self.analyze_expression(then_expr)?;
                self.analyze_expression(else_expr)?;
                
                self.check_boolean_context(condition)?;
                
                // Result type is the common type of then and else expressions
                if let (Some(then_type), Some(else_type)) = (&then_expr.expr_type, &else_expr.expr_type) {
                    self.common_type(then_type, else_type)
                } else {
                    Type::Error
                }
            }
            
            ExpressionKind::Cast { target_type, operand } => {
                self.analyze_expression(operand)?;
                target_type.clone()
            }
            
            ExpressionKind::SizeofExpr(operand) => {
                self.analyze_expression(operand)?;
                Type::UnsignedLong // sizeof returns size_t, which is unsigned long on Ripple
            }
            
            ExpressionKind::SizeofType(_) => {
                Type::UnsignedLong
            }
            
            ExpressionKind::CompoundLiteral { type_name, initializer } => {
                self.analyze_initializer(initializer, type_name)?;
                type_name.clone()
            }
        };
        
        expr.expr_type = Some(expr_type);
        Ok(())
    }
    
    /// Analyze binary operation and return result type
    fn analyze_binary_operation(&self, op: BinaryOp, left: &Expression, right: &Expression) -> Result<Type, CompilerError> {
        let left_type = left.expr_type.as_ref().unwrap_or(&Type::Error);
        let right_type = right.expr_type.as_ref().unwrap_or(&Type::Error);
        
        match op {
            // Arithmetic operations
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(self.arithmetic_result_type(left_type, right_type))
                } else if matches!(op, BinaryOp::Add | BinaryOp::Sub) {
                    // Pointer arithmetic
                    if left_type.is_pointer() && right_type.is_integer() {
                        Ok(left_type.clone())
                    } else if left_type.is_integer() && right_type.is_pointer() && matches!(op, BinaryOp::Add) {
                        Ok(right_type.clone())
                    } else {
                        Err(SemanticError::InvalidOperation {
                            operation: format!("{}", op),
                            operand_type: left_type.clone(),
                            location: left.span.start.clone(),
                        }.into())
                    }
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }.into())
                }
            }
            
            // Bitwise operations
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | 
            BinaryOp::LeftShift | BinaryOp::RightShift => {
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(self.arithmetic_result_type(left_type, right_type))
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }.into())
                }
            }
            
            // Comparison operations
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual |
            BinaryOp::Equal | BinaryOp::NotEqual => {
                // Comparisons return int (boolean)
                Ok(Type::Int)
            }
            
            // Logical operations
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                Ok(Type::Int) // Logical operations return int
            }
            
            // Assignment operations
            BinaryOp::Assign => {
                // Check if left is an lvalue
                if !self.is_lvalue(left) {
                    return Err(SemanticError::InvalidLvalue {
                        location: left.span.start.clone(),
                    }.into());
                }
                
                // Check type compatibility
                if !left_type.is_assignable_from(right_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                        location: right.span.start.clone(),
                    }.into());
                }
                
                Ok(left_type.clone())
            }
            
            // Compound assignment operations
            BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign | 
            BinaryOp::DivAssign | BinaryOp::ModAssign | BinaryOp::BitAndAssign |
            BinaryOp::BitOrAssign | BinaryOp::BitXorAssign | 
            BinaryOp::LeftShiftAssign | BinaryOp::RightShiftAssign => {
                // Check if left is an lvalue
                if !self.is_lvalue(left) {
                    return Err(SemanticError::InvalidLvalue {
                        location: left.span.start.clone(),
                    }.into());
                }
                
                // For compound assignment, the types must be compatible for the operation
                // This is a simplified check
                if left_type.is_integer() && right_type.is_integer() {
                    Ok(left_type.clone())
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: left_type.clone(),
                        found: right_type.clone(),
                        location: right.span.start.clone(),
                    }.into())
                }
            }
            
            BinaryOp::Index => {
                // Array indexing: arr[index]
                if left_type.is_pointer() && right_type.is_integer() {
                    if let Some(target_type) = left_type.pointer_target() {
                        Ok(target_type.clone())
                    } else {
                        Ok(Type::Error)
                    }
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "array indexing".to_string(),
                        operand_type: left_type.clone(),
                        location: left.span.start.clone(),
                    }.into())
                }
            }
        }
    }
    
    /// Analyze unary operation and return result type
    fn analyze_unary_operation(&self, op: UnaryOp, operand: &Expression) -> Result<Type, CompilerError> {
        let operand_type = operand.expr_type.as_ref().unwrap_or(&Type::Error);
        
        match op {
            UnaryOp::Plus | UnaryOp::Minus => {
                if operand_type.is_integer() {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::BitNot => {
                if operand_type.is_integer() {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "bitwise NOT".to_string(),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::LogicalNot => {
                Ok(Type::Int) // Logical NOT always returns int
            }
            
            UnaryOp::Dereference => {
                if let Some(target_type) = operand_type.pointer_target() {
                    Ok(target_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "dereference".to_string(),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::AddressOf => {
                if self.is_lvalue(operand) {
                    Ok(Type::Pointer(Box::new(operand_type.clone())))
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "address-of".to_string(),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::PreIncrement | UnaryOp::PostIncrement | 
            UnaryOp::PreDecrement | UnaryOp::PostDecrement => {
                if !self.is_lvalue(operand) {
                    return Err(SemanticError::InvalidLvalue {
                        location: operand.span.start.clone(),
                    }.into());
                }
                
                if operand_type.is_integer() || operand_type.is_pointer() {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", op),
                        operand_type: operand_type.clone(),
                        location: operand.span.start.clone(),
                    }.into())
                }
            }
            
            UnaryOp::Sizeof => {
                Ok(Type::UnsignedLong) // sizeof returns size_t
            }
        }
    }
    
    /// Check if an expression is a valid lvalue
    fn is_lvalue(&self, expr: &Expression) -> bool {
        match &expr.kind {
            ExpressionKind::Identifier { .. } => true,
            ExpressionKind::Unary { op: UnaryOp::Dereference, .. } => true,
            ExpressionKind::Binary { op: BinaryOp::Index, .. } => true,
            ExpressionKind::Member { .. } => true,
            _ => false,
        }
    }
    
    /// Check if expression is used in boolean context and can be converted
    fn check_boolean_context(&self, expr: &Expression) -> Result<(), CompilerError> {
        // In C, any scalar type can be used in boolean context
        if let Some(expr_type) = &expr.expr_type {
            match expr_type {
                Type::Void => {
                    Err(SemanticError::InvalidOperation {
                        operation: "boolean conversion".to_string(),
                        operand_type: expr_type.clone(),
                        location: expr.span.start.clone(),
                    }.into())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
    
    /// Get the common type for two operands (for arithmetic operations)
    fn arithmetic_result_type(&self, left: &Type, right: &Type) -> Type {
        // Simplified type promotion rules
        match (left, right) {
            (Type::Long, _) | (_, Type::Long) => Type::Long,
            (Type::UnsignedLong, _) | (_, Type::UnsignedLong) => Type::UnsignedLong,
            (Type::UnsignedInt, _) | (_, Type::UnsignedInt) => Type::UnsignedInt,
            _ => Type::Int,
        }
    }
    
    /// Get common type between two types
    fn common_type(&self, left: &Type, right: &Type) -> Type {
        if left == right {
            left.clone()
        } else if left.is_assignable_from(right) {
            left.clone()
        } else if right.is_assignable_from(left) {
            right.clone()
        } else {
            // Use arithmetic promotion rules
            self.arithmetic_result_type(left, right)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Frontend, TopLevelItem};

    #[test]
    fn test_semantic_analysis_simple_function() {
        let source = r#"
int main() {
    return 42;
}
"#;
        
        let mut ast = Frontend::parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        
        let result = analyzer.analyze(&mut ast);
        assert!(result.is_ok());
        
        // Check that function was analyzed
        match &ast.items[0] {
            TopLevelItem::Function(func) => {
                // Check that the return statement was analyzed
                match &func.body.kind {
                    StatementKind::Compound(statements) => {
                        match &statements[0].kind {
                            StatementKind::Return(Some(expr)) => {
                                assert_eq!(expr.expr_type, Some(Type::Int));
                            }
                            _ => panic!("Expected return statement"),
                        }
                    }
                    _ => panic!("Expected compound statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_semantic_analysis_undefined_variable() {
        let source = r#"
int main() {
    return x;  // undefined variable
}
"#;
        
        let mut ast = Frontend::parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        
        let result = analyzer.analyze(&mut ast);
        assert!(result.is_err());
        
        // Should be an undefined variable error
        match result.unwrap_err() {
            CompilerError::Semantic { message, .. } => {
                assert!(message.contains("Undefined variable"));
            }
            _ => panic!("Expected semantic error"),
        }
    }

    #[test]
    fn test_semantic_analysis_variable_declaration() {
        let source = r#"
int main() {
    int x = 42;
    return x;
}
"#;
        
        let mut ast = Frontend::parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        
        let result = analyzer.analyze(&mut ast);
        assert!(result.is_ok());
        
        // Variable should be resolved
        match &ast.items[0] {
            TopLevelItem::Function(func) => {
                match &func.body.kind {
                    StatementKind::Compound(statements) => {
                        // Check declaration
                        match &statements[0].kind {
                            StatementKind::Declaration { declarations } => {
                                assert!(declarations[0].symbol_id.is_some());
                            }
                            _ => panic!("Expected declaration statement"),
                        }
                        
                        // Check return statement uses the variable
                        match &statements[1].kind {
                            StatementKind::Return(Some(expr)) => {
                                match &expr.kind {
                                    ExpressionKind::Identifier { symbol_id, .. } => {
                                        assert!(symbol_id.is_some());
                                    }
                                    _ => panic!("Expected identifier in return"),
                                }
                            }
                            _ => panic!("Expected return statement"),
                        }
                    }
                    _ => panic!("Expected compound statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }
}