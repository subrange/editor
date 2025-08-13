//! Code generation from TypedAST to IR
//!
//! This module transforms the type-checked TypedAST into the intermediate representation (IR)
//! that can be lowered to assembly code.

mod errors;
mod types;
pub mod typed_expressions;
pub mod typed_statements;
pub mod typed_codegen;

pub use errors::CodegenError;
pub use typed_codegen::TypedCodeGenerator;
pub use types::{convert_type, get_ast_type_size};

use crate::ir::FatPointer;
use crate::ir::Value;
use crate::BankTag;

/// Variable information including bank tag
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub value: Value,
    pub ir_type: crate::ir::IrType,
    pub bank: Option<BankTag>,
}

impl VarInfo {
    /// Get the value as a fat pointer if it's a pointer with bank info
    pub fn as_fat_ptr(&self) -> Value {
        // If the value is already a FatPtr, return it as-is
        if matches!(self.value, Value::FatPtr(_)) {
            return self.value.clone();
        }
        
        // Otherwise, wrap it if we have bank info
        if let Some(bank) = self.bank {
            Value::FatPtr(FatPointer {
                addr: Box::new(self.value.clone()),
                bank,
            })
        } else {
            self.value.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frontend;
    use crate::typed_ast::type_translation_unit;

    #[test]
    fn test_codegen_simple_main() {
        let source = r#"
int main() {
    return 42;
}
"#;
        
        let ast = Frontend::analyze_source(source).unwrap();
        let typed_ast = type_translation_unit(&ast).unwrap();
        let codegen = TypedCodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&typed_ast);
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
        let typed_ast = type_translation_unit(&ast).unwrap();
        let codegen = TypedCodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&typed_ast);
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
        let typed_ast = type_translation_unit(&ast).unwrap();
        let codegen = TypedCodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&typed_ast);
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
        let typed_ast = type_translation_unit(&ast).unwrap();
        let codegen = TypedCodeGenerator::new("test".to_string());
        
        let result = codegen.generate(&typed_ast);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        let function = &module.functions[0];
        assert_eq!(function.name, "abs");
        
        // Should have multiple basic blocks for if-else
        assert!(function.blocks.len() >= 3);
    }
}