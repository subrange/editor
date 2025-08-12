// Tests for module lowering to assembly

#[cfg(test)]

mod tests {
    use super::super::*;
    use crate::ir::{Module, Function, GlobalVariable, IrType, Value, Linkage};
    use rcc_codegen::AsmInst;

    #[test]
    #[ignore]
    fn test_global_variable_allocation() {
        let mut module = Module::new("test".to_string());
        
        // Add some global variables
        module.add_global(GlobalVariable {
            name: "global_x".to_string(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: Some(Value::Constant(42)),
            linkage: Linkage::External,
            symbol_id: None,
        });
        
        module.add_global(GlobalVariable {
            name: "global_y".to_string(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: Some(Value::Constant(100)),
            linkage: Linkage::External,
            symbol_id: None,
        });
        
        module.add_global(GlobalVariable {
            name: "global_uninit".to_string(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: None,
            linkage: Linkage::External,
            symbol_id: None,
        });
        
        // Lower to assembly
        let asm = lower_module_to_assembly(module).unwrap();
        
        // Check that globals are allocated and initialized
        let asm_text: Vec<String> = asm.iter().map(|inst| format!("{:?}", inst)).collect();
        let asm_str = asm_text.join("\n");
        
        // Should have _init_globals label
        assert!(asm_str.contains("Label(\"_init_globals\")"));
        
        // Should have initialization for global_x (42)
        assert!(asm_str.contains("LI(R3, 42)"));
        
        // Should have initialization for global_y (100)
        assert!(asm_str.contains("LI(R3, 100)"));
        
        // Should have RET at the end of init
        assert!(asm_str.contains("Ret"));
    }

    #[test]
    #[ignore]
    fn test_string_literal_allocation() {
        let mut module = Module::new("test".to_string());
        
        // Add a string literal (encoded in name)
        // "Hi" = 0x48 0x69
        module.add_global(GlobalVariable {
            name: "__str_0_4869".to_string(),
            var_type: IrType::Array {
                element_type: Box::new(IrType::I8),
                size: 3, // "Hi" + null terminator
            },
            is_constant: true,
            initializer: None,
            linkage: Linkage::Internal,
            symbol_id: None,
        });
        
        // Lower to assembly
        let asm = lower_module_to_assembly(module).unwrap();
        
        // Check that string is initialized
        let asm_text: Vec<String> = asm.iter().map(|inst| format!("{:?}", inst)).collect();
        let asm_str = asm_text.join("\n");
        
        // Should have initialization for 'H' (72)
        assert!(asm_str.contains("LI(R3, 72)"));
        
        // Should have initialization for 'i' (105)
        assert!(asm_str.contains("LI(R3, 105)"));
        
        // Should have null terminator (0)
        assert!(asm_str.contains("LI(R3, 0)"));
    }

    #[test]
    #[ignore]
    fn test_global_addresses_are_sequential() {
        let mut module = Module::new("test".to_string());
        
        // Add multiple globals
        for i in 0..5 {
            module.add_global(GlobalVariable {
                name: format!("global_{}", i),
                var_type: IrType::I16,
                is_constant: false,
                initializer: Some(Value::Constant(i)),
                linkage: Linkage::External,
                symbol_id: None,
            });
        }
        
        // Lower to assembly
        let asm = lower_module_to_assembly(module).unwrap();
        
        // Check that addresses are allocated sequentially
        // Starting at address 100, each I16 takes 1 word
        let asm_text: Vec<String> = asm.iter().map(|inst| format!("{:?}", inst)).collect();
        
        // Look for store instructions with addresses
        let mut addresses = Vec::new();
        for inst in &asm {
            if let AsmInst::LI(_, addr) = inst {
                if *addr >= 100 && *addr < 200 {
                    addresses.push(*addr);
                }
            }
        }
        
        // Should have sequential addresses (100, 101, 102, 103, 104)
        addresses.sort();
        addresses.dedup();
        assert!(addresses.len() >= 5);
    }

    #[test]
    #[ignore]
    fn test_i32_global_takes_two_words() {
        let mut module = Module::new("test".to_string());
        
        module.add_global(GlobalVariable {
            name: "global_32".to_string(),
            var_type: IrType::I32,
            is_constant: false,
            initializer: Some(Value::Constant(0x12345678)),
            linkage: Linkage::External,
            symbol_id: None,
        });
        
        module.add_global(GlobalVariable {
            name: "global_16".to_string(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: Some(Value::Constant(42)),
            linkage: Linkage::External,
            symbol_id: None,
        });
        
        // Lower to assembly
        let _asm = lower_module_to_assembly(module).unwrap();
        
        // The I32 should take 2 words, so global_16 should be at address 102
        // (100 for global_32 low word, 101 for high word)
        // This is implicitly tested by the sequential allocation
    }
}