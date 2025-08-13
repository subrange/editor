//! Global variable handling for V2 backend
//! 
//! This module manages global variable allocation, tracking, and initialization
//! code generation for the V2 backend.

use crate::ir::{GlobalVariable, IrType, Value};
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
        let is_string = global.name.starts_with("__str_"); // TODO: Extend Ir properly for string literals
        
        // Calculate size in words (16-bit)
        let size = match &global.var_type {
            IrType::I8 | IrType::I16 => 1,
            IrType::I32 => 2, // 32-bit takes 2 words
            IrType::FatPtr(_) => 2, // Fat pointers are 2 words (address + bank)
            IrType::Array { size, .. } if is_string => {
                // For strings, allocate one word per character (including null terminator)
                *size as u16
            }
            IrType::Array { size, element_type } => {
                // For arrays, calculate total size
                let elem_size = element_type.size_in_bytes().unwrap_or(1) as u16;
                (*size as u16) * elem_size
            }
            _ => 1, // Default to 1 word
        };
        
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
        
        // Check if this is a string literal (name starts with __str_)
        let is_string = global.name.starts_with("__str_");
        
        if is_string {
            insts.extend(Self::lower_string_literal(global, info.address));
        } else {
            insts.extend(Self::lower_regular_global(global, info.address));
        }
        
        insts
    }
    
    /// Lower a string literal global
    fn lower_string_literal(global: &GlobalVariable, address: u16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        // For string literals, decode the string from the name
        // Format: __str_ID_HEXDATA
        if let Some(hex_part) = global.name.split('_').last() {
            let mut addr = address;
            let mut chars = Vec::new();
            
            // Decode hex string
            for i in (0..hex_part.len()).step_by(2) {
                let end = (i + 2).min(hex_part.len());
                if let Ok(byte) = u8::from_str_radix(&hex_part[i..end], 16) {
                    chars.push(byte);
                }
            }
            chars.push(0); // Add null terminator
            
            // Create a safe string representation for the comment
            let safe_str: String = chars[..chars.len().saturating_sub(1)].iter()
                .map(|&c| match c {
                    b'\n' => "\\n".to_string(),
                    b'\t' => "\\t".to_string(),
                    b'\r' => "\\r".to_string(),
                    b'\\' => "\\\\".to_string(),
                    c if c.is_ascii_graphic() || c == b' ' => (c as char).to_string(),
                    c => format!("\\x{:02x}", c),
                })
                .collect();
            
            insts.push(AsmInst::Comment(format!("String literal \"{}\" at address {}", safe_str, address)));
            
            // Store each character
            for byte in chars {
                insts.push(AsmInst::Li(Reg::T0, byte as i16));
                insts.push(AsmInst::Li(Reg::T1, addr as i16));
                insts.push(AsmInst::Store(Reg::T0, Reg::Gp, Reg::T1)); // Store to global memory (bank GP = R0)
                addr += 1;
            }
        }
        
        insts
    }
    
    /// Lower a regular (non-string) global variable
    fn lower_regular_global(global: &GlobalVariable, address: u16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment(format!("Global variable: {} at address {}", 
                                            global.name, address)));
        
        // Generate initialization code if there's an initializer
        if let Some(init_value) = &global.initializer {
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
        } else {
            // No initializer - leave uninitialized (could zero-initialize if needed)
            insts.push(AsmInst::Comment(format!("Uninitialized global {}", global.name)));
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
    use crate::ir::Linkage;
    
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