//! Simple Register Allocator
//! 
//! Based on the approach described in docs/registers-spilling-ideas.md
//! This is a minimal, spill-only register allocator with a simple free list.

use rcc_codegen::{AsmInst, Reg};
use std::collections::HashMap;

/// Simple register allocator with spill support
pub struct SimpleRegAlloc {
    /// Free list of available registers (R3-R11)
    free_list: Vec<Reg>,
    
    /// Map from register to what it contains (for spill decisions)
    /// Key is register, value is descriptive string (e.g., "t5", "const_42")
    reg_contents: HashMap<Reg, String>,
    
    /// Map from spilled values to their stack offsets
    spill_slots: HashMap<String, i16>,
    
    /// Next available spill slot offset (relative to FP)
    next_spill_offset: i16,
    
    /// Instructions to emit (for spill/reload code)
    instructions: Vec<AsmInst>,
}

impl SimpleRegAlloc {
    /// Create a new allocator with R5-R11 available (R3 is for return value, R4 for special uses)
    pub fn new() -> Self {
        Self {
            // Initialize in reverse order so R5 is preferred (popped last)
            // Following the formalized doc: POOL = [R5, R6, R7, R8, R9, R10, R11]
            free_list: vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5],
            reg_contents: HashMap::new(),
            spill_slots: HashMap::new(),
            next_spill_offset: 0,
            instructions: Vec::new(),
        }
    }
    
    /// Get a register for a value
    /// If all registers are in use, spills the least recently used one
    pub fn get_reg(&mut self, for_value: String) -> Reg {
        // Check if this value already has a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &for_value) {
            return reg;
        }
        
        // Try to get a free register
        if let Some(reg) = self.free_list.pop() {
            self.reg_contents.insert(reg, for_value);
            return reg;
        }
        
        // No free registers - need to spill
        // Pick the first register in reg_contents (simple LRU approximation)
        let victim = *self.reg_contents.keys().next().unwrap();
        let victim_value = self.reg_contents.remove(&victim).unwrap();
        
        // Spill the victim
        let spill_offset = self.get_spill_slot(&victim_value);
        self.instructions.push(AsmInst::Comment(format!("Spilling {} to FP+{}", victim_value, spill_offset)));
        self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset));
        self.instructions.push(AsmInst::Store(victim, Reg::R13, Reg::R12));
        
        // Now use this register for the new value
        self.reg_contents.insert(victim, for_value);
        victim
    }
    
    /// Allocate a register for a constant
    /// Constants are not tracked in reg_contents to allow immediate reuse
    pub fn get_const_reg(&mut self, value: i16) -> Reg {
        // Get a register without tracking it
        if let Some(reg) = self.free_list.pop() {
            self.instructions.push(AsmInst::LI(reg, value));
            return reg;
        }
        
        // Need to spill for the constant
        let victim = *self.reg_contents.keys().next().unwrap();
        let victim_value = self.reg_contents.remove(&victim).unwrap();
        
        // Spill the victim
        let spill_offset = self.get_spill_slot(&victim_value);
        self.instructions.push(AsmInst::Comment(format!("Spilling {} to FP+{}", victim_value, spill_offset)));
        self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset));
        self.instructions.push(AsmInst::Store(victim, Reg::R13, Reg::R12));
        
        // Load constant into the register
        self.instructions.push(AsmInst::LI(victim, value));
        victim
    }
    
    /// Free a register, returning it to the free list
    pub fn free_reg(&mut self, reg: Reg) {
        // Only free allocatable registers (R5-R11)
        if matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11) {
            self.reg_contents.remove(&reg);
            if !self.free_list.contains(&reg) {
                self.free_list.push(reg);
            }
        }
    }
    
    /// Free a constant register - returns it to the front of free list to avoid immediate reuse
    pub fn free_const_reg(&mut self, reg: Reg) {
        if matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11) {
            if !self.free_list.contains(&reg) {
                // Insert at front to avoid immediate reuse
                self.free_list.insert(0, reg);
            }
        }
    }
    
    /// Reload a spilled value into a register
    pub fn reload(&mut self, value: String) -> Reg {
        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &value) {
            return reg;
        }
        
        // Check if it was spilled
        if let Some(&offset) = self.spill_slots.get(&value) {
            let reg = self.get_reg(value.clone());
            self.instructions.push(AsmInst::Comment(format!("Reloading {} from FP+{}", value, offset)));
            self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, offset));
            self.instructions.push(AsmInst::Load(reg, Reg::R13, Reg::R12));
            return reg;
        }
        
        // Not spilled, just get a new register
        self.get_reg(value)
    }
    
    /// Get or allocate a spill slot for a value
    fn get_spill_slot(&mut self, value: &str) -> i16 {
        if let Some(&offset) = self.spill_slots.get(value) {
            offset
        } else {
            let offset = self.next_spill_offset;
            self.next_spill_offset += 1;
            self.spill_slots.insert(value.to_string(), offset);
            offset
        }
    }
    
    /// Free all registers (e.g., at statement boundaries)
    pub fn free_all(&mut self) {
        self.reg_contents.clear();
        // Reset to R5-R11 as per formalized doc
        self.free_list = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5];
    }
    
    /// Check if a register is currently allocated
    pub fn is_allocated(&self, reg: Reg) -> bool {
        self.reg_contents.contains_key(&reg)
    }
    
    /// Get and clear any generated instructions
    pub fn take_instructions(&mut self) -> Vec<AsmInst> {
        std::mem::take(&mut self.instructions)
    }
    
    /// Reset allocator state for a new function
    pub fn reset(&mut self) {
        // Reset to R5-R11 as per formalized doc
        self.free_list = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5];
        self.reg_contents.clear();
        self.spill_slots.clear();
        self.next_spill_offset = 0;
        self.instructions.clear();
    }
}