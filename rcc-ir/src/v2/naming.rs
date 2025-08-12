//! Centralized naming module for V2 backend
//! 
//! This module provides consistent, unique name generation for all temporary
//! values, registers, and labels used throughout the V2 backend. It ensures
//! no naming conflicts and maintains consistency between load/store operations.

use std::sync::atomic::{AtomicU32, Ordering};
use rcc_common::TempId;

/// Global operation counter for unique IDs
/// Using atomic to ensure thread safety if needed in the future
static OPERATION_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Name generator for V2 backend operations
/// 
/// This struct manages all name generation to ensure uniqueness and consistency.
/// Each instance represents a single compilation unit's naming context.
#[derive(Debug)]
pub struct NameGenerator {
    /// Operation ID for the current function being compiled
    function_id: u32,
    
    /// Counter for operations within the current function
    next_op_id: u32,
    
    /// Counter for unique labels within the current function
    next_label_id: u32,
}

impl NameGenerator {
    /// Create a new name generator for a function
    pub fn new(function_id: u32) -> Self {
        Self {
            function_id,
            next_op_id: 0,
            next_label_id: 0,
        }
    }
    
    /// Get the next operation ID and increment counter
    pub fn next_operation_id(&mut self) -> u32 {
        let id = self.next_op_id;
        self.next_op_id += 1;
        id
    }
    
    /// Get the next label ID and increment counter
    pub fn next_label_id(&mut self) -> u32 {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }
    
    // ===== Temp naming =====
    
    /// Get the standard name for a temp value
    pub fn temp_name(&self, temp_id: TempId) -> String {
        format!("t{}", temp_id)
    }
    
    // ===== Load operation naming =====
    
    /// Get name for a constant address loaded during a load operation
    pub fn load_const_addr(&mut self, result_temp: TempId) -> String {
        let op_id = self.next_operation_id();
        format!("load_f{}_op{}_t{}_addr", self.function_id, op_id, result_temp)
    }
    
    /// Get name for the bank address during pointer load
    pub fn load_bank_addr(&mut self, result_temp: TempId) -> String {
        let op_id = self.next_operation_id();
        format!("load_f{}_op{}_t{}_bank_addr", self.function_id, op_id, result_temp)
    }
    
    /// Get name for the bank value register during pointer load
    pub fn load_bank_value(&mut self, result_temp: TempId) -> String {
        let op_id = self.next_operation_id();
        format!("load_f{}_op{}_t{}_bank_val", self.function_id, op_id, result_temp)
    }
    
    /// Get name for a global address during load
    pub fn load_global_addr(&mut self, global_name: &str) -> String {
        let op_id = self.next_operation_id();
        format!("load_f{}_op{}_global_{}_addr", self.function_id, op_id, global_name)
    }
    
    // ===== Store operation naming =====
    
    /// Get name for a constant value being stored
    pub fn store_const_value(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("store_f{}_op{}_const", self.function_id, op_id)
    }
    
    /// Get name for a fat pointer address during store
    pub fn store_fatptr_addr(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("store_f{}_op{}_fp_addr", self.function_id, op_id)
    }
    
    /// Get name for a fat pointer bank during store
    pub fn store_fatptr_bank(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("store_f{}_op{}_fp_bank", self.function_id, op_id)
    }
    
    /// Get name for destination address during store
    pub fn store_dest_addr(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("store_f{}_op{}_dest_addr", self.function_id, op_id)
    }
    
    /// Get name for bank store address
    pub fn store_bank_addr(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("store_f{}_op{}_bank_addr", self.function_id, op_id)
    }
    
    /// Get name for a global address during store
    pub fn store_global_addr(&mut self, global_name: &str) -> String {
        let op_id = self.next_operation_id();
        format!("store_f{}_op{}_global_{}_addr", self.function_id, op_id, global_name)
    }
    
    // ===== Pointer bank tracking naming =====
    
    /// Get the key name for tracking a pointer's bank information
    /// This ensures consistency between load and store operations
    pub fn pointer_bank_key(&self, base_name: &str) -> String {
        // Use the base name directly for consistency
        // Don't add suffixes that might cause mismatches
        base_name.to_string()
    }
    
    /// Get the key name for a temp's pointer bank
    pub fn temp_pointer_bank_key(&self, temp_id: TempId) -> String {
        self.temp_name(temp_id)
    }
    
    // ===== Label naming =====
    
    /// Generate a label for loading a global
    pub fn load_global_label(&mut self, global_name: &str) -> String {
        let label_id = self.next_label_id();
        format!("load_global_{}_{}", global_name, label_id)
    }
    
    /// Generate a label for storing to a global
    pub fn store_global_label(&mut self, global_name: &str) -> String {
        let label_id = self.next_label_id();
        format!("store_global_{}_{}", global_name, label_id)
    }
    
    // ===== GEP operation naming (future) =====
    
    /// Get name for GEP intermediate calculations
    pub fn gep_offset_temp(&mut self, base_temp: TempId) -> String {
        let op_id = self.next_operation_id();
        format!("gep_f{}_op{}_t{}_offset", self.function_id, op_id, base_temp)
    }
    
    /// Get name for GEP bank calculation
    pub fn gep_bank_temp(&mut self, base_temp: TempId) -> String {
        let op_id = self.next_operation_id();
        format!("gep_f{}_op{}_t{}_bank", self.function_id, op_id, base_temp)
    }
    
    // ===== Calling convention naming =====
    
    /// Get name for a function parameter
    pub fn param_name(&mut self, index: usize) -> String {
        let op_id = self.next_operation_id();
        format!("param_f{}_op{}_{}", self.function_id, op_id, index)
    }
    
    /// Get name for return address register
    pub fn ret_addr_name(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("ret_addr_f{}_op{}", self.function_id, op_id)
    }
    
    /// Get name for return bank register
    pub fn ret_bank_name(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("ret_bank_f{}_op{}", self.function_id, op_id)
    }
    
    /// Get name for return value register
    pub fn ret_val_name(&mut self) -> String {
        let op_id = self.next_operation_id();
        format!("ret_val_f{}_op{}", self.function_id, op_id)
    }
    
    // ===== Function local naming =====
    
    /// Get name for a local variable at given offset
    pub fn local_name(&mut self, offset: i16) -> String {
        let op_id = self.next_operation_id();
        format!("local_f{}_op{}_off{}", self.function_id, op_id, offset)
    }
    
    /// Get name for a local variable address calculation
    pub fn local_addr_name(&mut self, offset: i16) -> String {
        let op_id = self.next_operation_id();
        format!("local_addr_f{}_op{}_off{}", self.function_id, op_id, offset)
    }
}

/// Create a new global name generator instance
/// Each function should get its own instance with a unique function ID
pub fn new_function_naming() -> NameGenerator {
    let function_id = OPERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
    NameGenerator::new(function_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unique_operation_ids() {
        let mut gen = NameGenerator::new(1);
        
        let id1 = gen.next_operation_id();
        let id2 = gen.next_operation_id();
        let id3 = gen.next_operation_id();
        
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }
    
    #[test]
    fn test_load_naming_uniqueness() {
        let mut gen = NameGenerator::new(1);
        
        // Multiple loads to the same temp should get unique names
        let name1 = gen.load_const_addr(10);
        let name2 = gen.load_const_addr(10);
        
        assert_ne!(name1, name2);
        assert!(name1.contains("op0"));
        assert!(name2.contains("op1"));
    }
    
    #[test]
    fn test_store_naming_uniqueness() {
        let mut gen = NameGenerator::new(1);
        
        // Multiple stores should get unique names
        let name1 = gen.store_const_value();
        let name2 = gen.store_const_value();
        
        assert_ne!(name1, name2);
    }
    
    #[test]
    fn test_pointer_bank_key_consistency() {
        let gen = NameGenerator::new(1);
        
        // The key should be consistent for the same temp
        let key1 = gen.temp_pointer_bank_key(10);
        let key2 = gen.temp_pointer_bank_key(10);
        
        assert_eq!(key1, key2);
        assert_eq!(key1, "t10");
    }
    
    #[test]
    fn test_function_isolation() {
        let mut gen1 = NameGenerator::new(1);
        let mut gen2 = NameGenerator::new(2);
        
        let name1 = gen1.load_const_addr(10);
        let name2 = gen2.load_const_addr(10);
        
        // Same operation in different functions should have different names
        assert_ne!(name1, name2);
        assert!(name1.contains("f1"));
        assert!(name2.contains("f2"));
    }
}