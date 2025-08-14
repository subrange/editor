//! Typed code generation from TypedAST to IR
//!
//! This module transforms the typed AST into IR, ensuring proper GEP generation
//! for all pointer arithmetic operations.

use std::collections::{HashMap, HashSet};
use crate::ir::{Module, Function, Value, IrType, IrBuilder, GlobalVariable, Linkage, FatPointer};
use crate::typed_ast::{TypedTranslationUnit, TypedTopLevelItem, TypedFunction, TypedExpr};
use crate::types::{Type, BankTag};
use crate::CompilerError;
use super::errors::CodegenError;
use super::types::convert_type;
use super::VarInfo;

use super::typed_statements::TypedStatementGenerator;
use super::typed_expressions::TypedExpressionGenerator;

// Helper function for convert_type with default location
fn convert_type_default(ast_type: &Type) -> Result<IrType, CompilerError> {
    convert_type(ast_type, rcc_common::SourceLocation::new_simple(0, 0))
}

/// Typed code generator - transforms TypedAST to IR
pub struct TypedCodeGenerator {
    module: Module,
    builder: IrBuilder,
    variables: HashMap<String, VarInfo>,
    array_variables: HashSet<String>,
    parameter_variables: HashSet<String>,
    string_literals: HashMap<String, String>,
    next_string_id: u32,
    break_labels: Vec<rcc_common::LabelId>,
    continue_labels: Vec<rcc_common::LabelId>,
}

impl TypedCodeGenerator {
    /// Create a new typed code generator
    pub fn new(module_name: String) -> Self {
        Self {
            module: Module::new(module_name),
            builder: IrBuilder::new(),
            variables: HashMap::new(),
            array_variables: HashSet::new(),
            parameter_variables: HashSet::new(),
            string_literals: HashMap::new(),
            next_string_id: 0,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
        }
    }
    
    /// Generate IR module from typed AST
    pub fn generate(mut self, ast: &TypedTranslationUnit) -> Result<Module, CompilerError> {
        // Process all top-level items
        for item in &ast.items {
            self.generate_top_level_item(item)?;
        }
        
        // Add string literal initializers
        self.add_string_literal_initializers()?;
        
        Ok(self.module)
    }
    
    /// Generate IR for a top-level item
    fn generate_top_level_item(&mut self, item: &TypedTopLevelItem) -> Result<(), CompilerError> {
        match item {
            TypedTopLevelItem::Function(func) => {
                self.generate_function(func)?;
            }
            TypedTopLevelItem::GlobalVariable { name, var_type, initializer } => {
                self.generate_global_variable(name, var_type, initializer.as_ref())?;
            }
        }
        Ok(())
    }
    
    /// Generate IR for a function
    fn generate_function(&mut self, func: &TypedFunction) -> Result<(), CompilerError> {
        // Save global variables before clearing
        let globals: Vec<_> = self.variables.iter()
            .filter(|(_, v)| matches!(v.value, Value::Global(_)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // Clear local state
        self.variables.clear();
        self.array_variables.clear();
        self.parameter_variables.clear();
        
        // Restore globals
        for (k, v) in globals {
            self.variables.insert(k, v);
        }
        
        // Convert return type
        let ret_type = convert_type_default(&func.return_type)?;
        
        // Convert parameter types
        let mut param_types = Vec::new();
        for (_, param_type) in &func.parameters {
            param_types.push(convert_type_default(param_type)?);
        }
        
        // Create the function
        self.builder.create_function(func.name.clone(), ret_type);
        
        // Create entry block
        let entry_label = self.builder.new_label();
        self.builder.create_block(entry_label)?;
        
        // First, add all parameters to the function
        // This must be done before any allocas to ensure temp IDs don't conflict
        for (i, (_, param_type)) in func.parameters.iter().enumerate() {
            let ir_type = convert_type_default(param_type)?;
            self.builder.add_parameter(i as rcc_common::TempId, ir_type);
        }
        
        // Now handle parameter storage (allocas and stores)
        for (i, (param_name, param_type)) in func.parameters.iter().enumerate() {
            let ir_type = convert_type_default(param_type)?;
            
            // Allocate space for parameter
            let param_addr = self.builder.build_alloca(ir_type.clone(), None)?;
            
            // Store parameter value (parameters are passed as temporaries)
            let param_value = Value::Temp(i as rcc_common::TempId);
            self.builder.build_store(param_value, param_addr.clone())?;
            
            // Track the parameter
            let var_info = VarInfo {
                value: param_addr,
                ir_type,
                bank: Some(BankTag::Stack),
            };
            self.variables.insert(param_name.clone(), var_info);
            self.parameter_variables.insert(param_name.clone());
        }
        
        // Generate function body
        let mut stmt_gen = TypedStatementGenerator {
            builder: &mut self.builder,
            module: &mut self.module,
            variables: &mut self.variables,
            array_variables: &mut self.array_variables,
            parameter_variables: &self.parameter_variables,
            string_literals: &mut self.string_literals,
            next_string_id: &mut self.next_string_id,
            break_labels: &mut self.break_labels,
            continue_labels: &mut self.continue_labels,
        };
        
        stmt_gen.generate(&func.body)?;
        
        // Add implicit return if needed
        if !self.builder.current_block_has_terminator() {
            if func.return_type == Type::Void {
                self.builder.build_return(None)?;
            } else {
                // For non-void functions, add return 0
                self.builder.build_return(Some(Value::Constant(0)))?;
            }
        }
        
        // Finalize the function
        if let Some(final_function) = self.builder.finish_function() {
            self.module.add_function(final_function);
        }
        
        Ok(())
    }
    
    /// Generate IR for a global variable
    fn generate_global_variable(
        &mut self,
        name: &str,
        var_type: &Type,
        initializer: Option<&TypedExpr>,
    ) -> Result<(), CompilerError> {
        let ir_type = convert_type_default(var_type)?;
        
        // Handle initializer if present
        let init_value = if let Some(init_expr) = initializer {
            // We need to generate the initializer value
            // For global variables, only constant initializers are allowed
            let mut expr_gen = TypedExpressionGenerator {
                builder: &mut self.builder,
                module: &mut self.module,
                variables: &self.variables,
                array_variables: &self.array_variables,
                parameter_variables: &self.parameter_variables,
                string_literals: &mut self.string_literals,
                next_string_id: &mut self.next_string_id,
            };
            
            match expr_gen.generate(init_expr) {
                Ok(value) => match value {
                    Value::Constant(_) | Value::ConstantArray(_) => Some(value),
                    _ => None, // Non-constant initializers not supported for globals
                }
                Err(_) => None,
            }
        } else {
            None
        };
        
        let global = GlobalVariable {
            name: name.to_string(),
            var_type: ir_type.clone(),
            is_constant: false,
            initializer: init_value,
            linkage: Linkage::External,
            symbol_id: None,
        };
        
        self.module.add_global(global);
        
        // Add to variables map for later reference
        self.variables.insert(name.to_string(), VarInfo {
            value: Value::Global(name.to_string()),
            ir_type,
            bank: Some(BankTag::Global),
        });
        
        // Track global arrays
        if matches!(var_type, Type::Array { .. }) {
            self.array_variables.insert(name.to_string());
        }
        
        Ok(())
    }
    
    /// Add string literal initializers to the module
    fn add_string_literal_initializers(&mut self) -> Result<(), CompilerError> {
        // String literals are already added as globals during expression generation
        // This is where we could add their initializers if needed
        Ok(())
    }
}