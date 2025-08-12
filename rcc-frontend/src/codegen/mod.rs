//! Code generation from AST to IR
//!
//! This module transforms the type-checked AST into the intermediate representation (IR)
//! that can be lowered to assembly code.

mod errors;
mod types;
mod expressions;
mod statements;

pub use errors::CodegenError;

use std::collections::{HashMap, HashSet};
use rcc_ir::{Module, Function, Value, IrType, IrBuilder, GlobalVariable, Linkage, LabelId as Label};
use crate::ast::{TranslationUnit, TopLevelItem, FunctionDefinition, Declaration, Type, 
                 InitializerKind, ExpressionKind, Statement, StatementKind, Expression, BinaryOp};
use crate::CompilerError;
use self::statements::StatementGenerator;
use self::types::{convert_type, get_ast_type_size};

/// Code generator - transforms AST to IR
pub struct CodeGenerator {
    module: Module,
    builder: IrBuilder,
    variables: HashMap<String, (Value, IrType)>,
    array_variables: HashSet<String>,
    parameter_variables: HashSet<String>,
    string_literals: HashMap<String, String>,
    next_string_id: u32,
    break_labels: Vec<Label>,
    continue_labels: Vec<Label>,
}

impl CodeGenerator {
    /// Create a new code generator
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
    
    /// Generate IR module from AST
    pub fn generate(mut self, ast: &TranslationUnit) -> Result<Module, CompilerError> {
        // Process all top-level items
        for item in &ast.items {
            self.generate_top_level_item(item)?;
        }
        
        Ok(self.module)
    }
    
    /// Generate IR for a top-level item
    fn generate_top_level_item(&mut self, item: &TopLevelItem) -> Result<(), CompilerError> {
        match item {
            TopLevelItem::Function(func_def) => {
                self.generate_function(func_def)?;
            }
            TopLevelItem::Declaration(decl) => {
                self.generate_global_declaration(decl)?;
            }
            TopLevelItem::TypeDefinition { .. } => {
                // Type definitions are handled during semantic analysis
                // No code generation needed
            }
        }
        Ok(())
    }
    
    /// Find parameters that are reassigned in a statement
    fn find_reassigned_params(&self, stmt: &Statement, reassigned: &mut HashSet<String>) {
        match &stmt.kind {
            StatementKind::Expression(expr) => {
                self.find_reassigned_params_in_expr(expr, reassigned);
            }
            StatementKind::Compound(stmts) => {
                for s in stmts {
                    self.find_reassigned_params(s, reassigned);
                }
            }
            StatementKind::If { condition, then_stmt, else_stmt } => {
                self.find_reassigned_params_in_expr(condition, reassigned);
                self.find_reassigned_params(then_stmt, reassigned);
                if let Some(else_s) = else_stmt {
                    self.find_reassigned_params(else_s, reassigned);
                }
            }
            StatementKind::While { condition, body } |
            StatementKind::DoWhile { body, condition } => {
                self.find_reassigned_params_in_expr(condition, reassigned);
                self.find_reassigned_params(body, reassigned);
            }
            StatementKind::For { init, condition, update, body } => {
                if let Some(init_stmt) = init {
                    self.find_reassigned_params(init_stmt, reassigned);
                }
                if let Some(cond) = condition {
                    self.find_reassigned_params_in_expr(cond, reassigned);
                }
                if let Some(upd) = update {
                    self.find_reassigned_params_in_expr(upd, reassigned);
                }
                self.find_reassigned_params(body, reassigned);
            }
            _ => {}
        }
    }
    
    /// Find parameters that are reassigned in an expression
    fn find_reassigned_params_in_expr(&self, expr: &Expression, reassigned: &mut HashSet<String>) {
        match &expr.kind {
            ExpressionKind::Binary { op: BinaryOp::Assign, left, right } => {
                // Check if we're assigning to a parameter
                if let ExpressionKind::Identifier { name, .. } = &left.kind {
                    // Check if this is a parameter
                    if self.parameter_variables.contains(name) {
                        reassigned.insert(name.clone());
                    }
                }
                // Also check the right side for nested assignments
                self.find_reassigned_params_in_expr(right, reassigned);
            }
            ExpressionKind::Binary { left, right, .. } => {
                self.find_reassigned_params_in_expr(left, reassigned);
                self.find_reassigned_params_in_expr(right, reassigned);
            }
            ExpressionKind::Unary { operand, .. } => {
                self.find_reassigned_params_in_expr(operand, reassigned);
            }
            ExpressionKind::Call { arguments, .. } => {
                for arg in arguments {
                    self.find_reassigned_params_in_expr(arg, reassigned);
                }
            }
            _ => {}
        }
    }
    
    /// Generate IR for a function definition
    fn generate_function(&mut self, func_def: &FunctionDefinition) -> Result<(), CompilerError> {
        // Save global variables before clearing
        let globals: Vec<_> = self.variables.iter()
            .filter(|(_, (v, _))| matches!(v, Value::Global(_) | Value::Function(_)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // Clear per-function state
        self.variables.clear();
        self.array_variables.clear();
        self.parameter_variables.clear();
        
        // Restore globals
        for (k, v) in globals {
            self.variables.insert(k, v);
        }
        
        // Determine return type and parameters
        let return_type = convert_type(&func_def.return_type, func_def.span.start.clone())?;
        let mut param_types = Vec::new();
        for param in &func_def.parameters {
            param_types.push(convert_type(&param.param_type, func_def.span.start.clone())?);
        }
        
        // Create function
        self.builder.create_function(func_def.name.clone(), return_type);
        
        // Add parameters to the function
        for (i, param) in func_def.parameters.iter().enumerate() {
            let param_id = i as u32;
            let param_type = param_types[i].clone();
            self.builder.add_parameter(param_id, param_type.clone());
        }
        
        // Create entry block
        let entry = self.builder.new_label();
        self.builder.create_block(entry)?;
        
        // Add parameters to variables map
        // First, mark all parameters temporarily so we can scan for reassignments
        for (i, param) in func_def.parameters.iter().enumerate() {
            let param_name = param.name.clone().unwrap_or_else(|| format!("arg{}", i));
            self.parameter_variables.insert(param_name);
        }
        
        // Find which parameters are reassigned in the function
        let mut reassigned_params = HashSet::new();
        self.find_reassigned_params(&func_def.body, &mut reassigned_params);
        
        // Now process parameters based on whether they're reassigned
        for (i, param) in func_def.parameters.iter().enumerate() {
            let param_value = Value::Temp(i as u32);
            let param_type = param_types[i].clone();
            let param_name = param.name.clone().unwrap_or_else(|| format!("arg{}", i));
            
            if reassigned_params.contains(&param_name) {
                // This parameter is reassigned - create an alloca for it
                let alloca_temp = self.builder.build_alloca(param_type.clone(), None)?;
                
                // Store the original parameter value
                self.builder.build_store(param_value, Value::Temp(alloca_temp))?;
                
                // Map the parameter to the alloca
                let var_type = IrType::Ptr(Box::new(param_type));
                self.variables.insert(param_name.clone(), (Value::Temp(alloca_temp), var_type));
                
                // Remove from parameter_variables since it now uses an alloca
                self.parameter_variables.remove(&param_name);
            } else {
                // This parameter is never reassigned - use it directly
                self.variables.insert(param_name, (param_value, param_type));
                // Keep it in parameter_variables for direct use
            }
        }
        
        // Generate function body
        {
            let mut stmt_gen = StatementGenerator {
                builder: &mut self.builder,
                module: &mut self.module,
                variables: &mut self.variables,
                array_variables: &mut self.array_variables,
                parameter_variables: &mut self.parameter_variables,
                string_literals: &mut self.string_literals,
                next_string_id: &mut self.next_string_id,
                break_labels: &mut self.break_labels,
                continue_labels: &mut self.continue_labels,
            };
            stmt_gen.generate(&func_def.body)?;
        }
        
        // Ensure function has a terminator
        if !self.builder.current_block_has_terminator() {
            // Add implicit return for void functions
            self.builder.build_return(None)?;
        }
        
        // Add function to module
        if let Some(function) = self.builder.finish_function() {
            self.module.add_function(function);
        }
        
        Ok(())
    }
    
    /// Generate IR for a global declaration
    fn generate_global_declaration(&mut self, decl: &Declaration) -> Result<(), CompilerError> {
        // Handle function declarations separately (they don't generate globals)
        if matches!(decl.decl_type, Type::Function { .. }) {
            // Just add to the variables map for reference
            // For function types, we use Void as a placeholder since functions don't have an IR type
            self.variables.insert(decl.name.clone(), (Value::Function(decl.name.clone()), IrType::Void));
            return Ok(());
        }
        
        let ir_type = convert_type(&decl.decl_type, decl.span.start.clone())?;
        
        // Create global variable
        let global = GlobalVariable {
            name: decl.name.clone(),
            var_type: ir_type.clone(),
            is_constant: false,
            initializer: if let Some(init) = &decl.initializer {
                Some(self.generate_constant_initializer(init)?)
            } else {
                None
            },
            linkage: match decl.storage_class {
                crate::ast::StorageClass::Static => Linkage::Internal,
                crate::ast::StorageClass::Extern => Linkage::External,
                _ => Linkage::External,
            },
            symbol_id: decl.symbol_id,
        };
        
        // Add to module
        self.module.add_global(global);
        
        // Add to variables map for later reference
        self.variables.insert(decl.name.clone(), (Value::Global(decl.name.clone()), ir_type));
        
        Ok(())
    }
    
    /// Generate constant initializer
    fn generate_constant_initializer(&mut self, init: &crate::ast::Initializer) -> Result<Value, CompilerError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frontend;

    #[test]
    fn test_codegen_simple_main() {
        let source = r#"
int main() {
    return 42;
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let codegen = CodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "main");
    }

    #[test]
    fn test_codegen_global_variables() {
        let source = r#"
int global_x = 10;
int global_y;

int main() {
    return global_x;
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let codegen = CodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&ast);
        if let Err(e) = &result {
            eprintln!("Error in test_codegen_global_variables: {}", e);
        }
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert_eq!(module.globals.len(), 2);
        assert_eq!(module.functions.len(), 1);
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
        let codegen = CodeGenerator::new("test".to_string());
        
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
        let codegen = CodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = &module.functions[0];
        assert_eq!(function.name, "abs");
        
        // Should have multiple basic blocks for if-else
        assert!(function.blocks.len() >= 3);
    }
}