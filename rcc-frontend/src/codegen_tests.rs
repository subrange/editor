// Tests for code generation

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::Frontend;
    use crate::ir::{Value, IrType};
    
    #[test]
    fn test_global_variable_codegen() {
        let source = r#"
            int global_x = 42;
            int global_y = 100;
            int global_uninit;
            
            int main() {
                return global_x;
            }
        "#;
        
        let ast = Frontend::parse_source(source).unwrap();
        
        let mut codegen = CodeGenerator::new("test".to_string());
        let module = codegen.generate(&ast).unwrap();
        
        // Check that globals are in the module
        assert_eq!(module.globals.len(), 3);
        assert_eq!(module.globals[0].name, "global_x");
        assert_eq!(module.globals[1].name, "global_y");
        assert_eq!(module.globals[2].name, "global_uninit");
        
        // Check initializers
        assert!(matches!(module.globals[0].initializer, Some(Value::Constant(42))));
        assert!(matches!(module.globals[1].initializer, Some(Value::Constant(100))));
        assert!(module.globals[2].initializer.is_none());
    }
    
    #[test]
    fn test_string_literal_codegen() {
        let source = r#"
            int main() {
                char *msg = "Hello";
                return 0;
            }
        "#;
        
        let ast = Frontend::parse_source(source).unwrap();
        
        let mut codegen = CodeGenerator::new("test".to_string());
        let module = codegen.generate(&ast).unwrap();
        
        // Check that a string literal global was created
        let string_globals: Vec<_> = module.globals.iter()
            .filter(|g| g.name.starts_with("__str_"))
            .collect();
        
        assert_eq!(string_globals.len(), 1);
        
        // Check that it's an array type
        if let IrType::Array { element_type, size } = &string_globals[0].var_type {
            assert!(matches!(**element_type, IrType::I8));
            assert_eq!(*size, 6); // "Hello" + null terminator
        } else {
            panic!("String literal should be an array type");
        }
        
        // Check that the string data is encoded in the name
        assert!(string_globals[0].name.contains("48656c6c6f")); // "Hello" in hex
    }
    
    #[test]
    fn test_multiple_string_literals() {
        let source = r#"
            int main() {
                char *msg1 = "Hi";
                char *msg2 = "Bye";
                return 0;
            }
        "#;
        
        let ast = Frontend::parse_source(source).unwrap();
        
        let mut codegen = CodeGenerator::new("test".to_string());
        let module = codegen.generate(&ast).unwrap();
        
        // Check that two string literal globals were created
        let string_globals: Vec<_> = module.globals.iter()
            .filter(|g| g.name.starts_with("__str_"))
            .collect();
        
        assert_eq!(string_globals.len(), 2);
        
        // Check that they have unique IDs
        let names: Vec<_> = string_globals.iter().map(|g| &g.name).collect();
        assert_ne!(names[0], names[1]);
    }
    
    #[test]
    fn test_global_in_expression() {
        let source = r#"
            int global_x = 10;
            
            int main() {
                int local = global_x + 5;
                global_x = 20;
                return global_x;
            }
        "#;
        
        let ast = Frontend::parse_source(source).unwrap();
        
        let mut codegen = CodeGenerator::new("test".to_string());
        let module = codegen.generate(&ast).unwrap();
        
        // Should compile without errors
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.globals.len(), 1);
    }
    
    #[test]
    fn test_string_literal_special_chars() {
        let source = r#"
            int main() {
                char *msg = "Hello\nWorld\t!";
                return 0;
            }
        "#;
        
        let ast = Frontend::parse_source(source).unwrap();
        
        let mut codegen = CodeGenerator::new("test".to_string());
        let module = codegen.generate(&ast).unwrap();
        
        // Check that string with special chars is handled
        let string_globals: Vec<_> = module.globals.iter()
            .filter(|g| g.name.starts_with("__str_"))
            .collect();
        
        assert_eq!(string_globals.len(), 1);
        
        // Check size includes special chars
        if let IrType::Array { size, .. } = &string_globals[0].var_type {
            assert_eq!(*size, 14); // "Hello\nWorld\t!" + null = 13 + 1
        }
        
        // Check hex encoding includes newline (0x0a) and tab (0x09)
        assert!(string_globals[0].name.contains("0a")); // \n
        assert!(string_globals[0].name.contains("09")); // \t
    }
}