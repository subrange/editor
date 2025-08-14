//! Global variable handling for V2 backend
//! 
//! This module manages global variable allocation, tracking, and initialization
//! code generation for the V2 backend.

use rcc_frontend::ir::{GlobalVariable, IrType, Value};
use rcc_codegen::{AsmInst, Reg};
use std::collections::HashMap;
use log::{debug, info, trace};

/// Information about a global variable's allocation
#[derive(Debug, Clone)]
pub struct GlobalInfo {
    /// Address in global memory (bank 0)
    pub address: u16,
    /// Size in 16-bit words
    pub size: u16,
}

/// Global variable manager for tracking and lowering globals
pub struct GlobalManager {
    /// Map from global name to allocation info
    allocations: HashMap<String, GlobalInfo>,
    /// Next available global address
    next_address: u16,
}

impl GlobalManager {
    /// Create a new global manager
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            next_address: 0,
        }
    }
    
    /// Allocate space for a global variable
    pub fn allocate_global(&mut self, global: &GlobalVariable) -> GlobalInfo {
        // Calculate size in words (16-bit)
        let size = global.var_type.size_in_words().unwrap_or(1) as u16;
        
        let info = GlobalInfo {
            address: self.next_address,
            size,
        };
        
        self.next_address += size;
        self.allocations.insert(global.name.clone(), info.clone());
        
        debug!("Allocated global '{}' at address {} (size: {} words)", 
               global.name, info.address, size);
        
        info
    }
    
    /// Get allocation info for a global variable
    pub fn get_global_info(&self, name: &str) -> Option<&GlobalInfo> {
        self.allocations.get(name)
    }
    
    /// Get all global allocations
    pub fn get_all_allocations(&self) -> &HashMap<String, GlobalInfo> {
        &self.allocations
    }
    
    /// Generate initialization code for a global variable
    pub fn lower_global_init(global: &GlobalVariable, info: &GlobalInfo) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        // Generate initialization based on the initializer
        match &global.initializer {
            Some(Value::ConstantArray(values)) => {
                // Initialize array with provided values
                insts.extend(Self::lower_array_init(global, values, info.address));
            }
            Some(init_value) => {
                // Initialize with single value
                insts.extend(Self::lower_single_value_init(global, init_value, info.address));
            }
            None => {
                // No initializer - leave uninitialized
                insts.push(AsmInst::Comment(format!("Uninitialized global {}", global.name)));
            }
        }
        
        insts
    }
    
    /// Lower array initialization
    fn lower_array_init(global: &GlobalVariable, values: &[i64], address: u16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        // Add comment - if it looks like string data, format it nicely
        let is_likely_string = values.last() == Some(&0) && 
            values[..values.len().saturating_sub(1)].iter()
                .all(|&v| v >= 0 && v <= 127);
        
        if is_likely_string {
            // Create a safe string representation for the comment
            let safe_str: String = values[..values.len().saturating_sub(1)].iter()
                .map(|&c| match c as u8 {
                    b'\n' => "\\n".to_string(),
                    b'\t' => "\\t".to_string(),
                    b'\r' => "\\r".to_string(),
                    b'\\' => "\\\\".to_string(),
                    c if c.is_ascii_graphic() || c == b' ' => (c as char).to_string(),
                    c => format!("\\x{:02x}", c),
                })
                .collect();
            insts.push(AsmInst::Comment(format!("String data \"{}\" at address {}", safe_str, address)));
        } else {
            insts.push(AsmInst::Comment(format!("Array {} at address {}", global.name, address)));
        }
        
        // Store each value
        let mut addr = address;
        for &value in values {
            insts.push(AsmInst::Li(Reg::T0, value as i16));
            insts.push(AsmInst::Li(Reg::T1, addr as i16));
            insts.push(AsmInst::Store(Reg::T0, Reg::Gp, Reg::T1)); // Store to global memory (bank GP = R0)
            addr += 1;
        }
        
        insts
    }
    
    /// Lower single value initialization
    fn lower_single_value_init(global: &GlobalVariable, init_value: &Value, address: u16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment(format!("Global variable: {} at address {}", 
                                            global.name, address)));
        
        match init_value {
            Value::Constant(val) => {
                // Handle different sizes
                match &global.var_type {
                    IrType::I32 => {
                        // 32-bit values need two stores
                        let low = (*val & 0xFFFF) as i16;
                        let high = ((*val >> 16) & 0xFFFF) as i16;
                        
                        // Store low word
                        insts.push(AsmInst::Li(Reg::T0, low));
                        insts.push(AsmInst::Li(Reg::T1, address as i16));
                        insts.push(AsmInst::Store(Reg::T0, Reg::Gp, Reg::T1));
                        
                        // Store high word
                        insts.push(AsmInst::Li(Reg::T0, high));
                        insts.push(AsmInst::Li(Reg::T1, (address + 1) as i16));
                        insts.push(AsmInst::Store(Reg::T0, Reg::Gp, Reg::T1));
                    }
                    _ => {
                        // Default: single word store
                        insts.push(AsmInst::Li(Reg::T0, *val as i16));
                        insts.push(AsmInst::Li(Reg::T1, address as i16));
                        insts.push(AsmInst::Store(Reg::T0, Reg::Gp, Reg::T1));
                    }
                }
            }
            _ => {
                // Other initializer types not yet supported
                insts.push(AsmInst::Comment(format!("Unsupported initializer for {}", global.name)));
            }
        }
        
        insts
    }
}

impl Default for GlobalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcc_frontend::ir::Linkage;
    
    #[test]
    fn test_global_allocation() {
        let mut manager = GlobalManager::new();
        
        let global1 = GlobalVariable {
            name: "global_x".to_string(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: Some(Value::Constant(42)),
            linkage: Linkage::External,
            symbol_id: None,
        };
        
        let global2 = GlobalVariable {
            name: "global_y".to_string(),
            var_type: IrType::I32,
            is_constant: false,
            initializer: Some(Value::Constant(0x12345678)),
            linkage: Linkage::External,
            symbol_id: None,
        };
        
        let info1 = manager.allocate_global(&global1);
        let info2 = manager.allocate_global(&global2);
        
        assert_eq!(info1.address, 0);
        assert_eq!(info1.size, 1);
        assert_eq!(info2.address, 1);
        assert_eq!(info2.size, 2);
    }
    
    #[test]
    fn test_string_literal_allocation() {
        let mut manager = GlobalManager::new();
        
        // "Hi" = 0x48 0x69, but we'll have 3 words (H, i, \0)
        let global = GlobalVariable {
            name: "__str_0_4869".to_string(),
            var_type: IrType::Array {
                element_type: Box::new(IrType::I8),
                size: 3,
            },
            is_constant: true,
            initializer: None,
            linkage: Linkage::Internal,
            symbol_id: None,
        };
        
        let info = manager.allocate_global(&global);
        assert_eq!(info.address, 0);
        assert_eq!(info.size, 3); // One word per character including null
    }
    
    #[test]
    fn test_global_initialization_code() {
        let global = GlobalVariable {
            name: "test_var".to_string(),
            var_type: IrType::I16,
            is_constant: false,
            initializer: Some(Value::Constant(100)),
            linkage: Linkage::External,
            symbol_id: None,
        };
        
        let info = GlobalInfo { address: 10, size: 1 };
        let insts = GlobalManager::lower_global_init(&global, &info);
        
        // Should have: comment, Li(T0, 100), Li(T1, 10), Store
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Comment(_))));
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Li(Reg::T0, 100))));
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Li(Reg::T1, 10))));
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(Reg::T0, Reg::Gp, Reg::T1))));
    }
}