//! Simple Register Allocator
//! 
//! Based on the approach described in docs/registers-spilling-ideas.md
//! This is a minimal, spill-only register allocator with a simple free list.

use rcc_codegen::{AsmInst, Reg};
use std::collections::BTreeMap;

/// Simple register allocator with spill support
pub struct SimpleRegAlloc {
    /// Free list of available registers (R3-R11)
    free_list: Vec<Reg>,
    
    /// Map from register to what it contains (for spill decisions)
    /// Key is register, value is descriptive string (e.g., "t5", "const_42")
    reg_contents: BTreeMap<Reg, String>,
    
    /// Map from spilled values to their stack offsets
    spill_slots: BTreeMap<String, i16>,
    
    /// Next available spill slot offset (relative to FP)
    next_spill_offset: i16,
    
    /// Instructions to emit (for spill/reload code)
    instructions: Vec<AsmInst>,
    
    /// Set of values that are temporarily pinned and cannot be spilled
    /// Used during binary operations to prevent spilling operands
    pinned_values: std::collections::HashSet<String>,
    
    /// Set of registers that hold function parameters and should not be freed at statement boundaries
    /// These persist throughout the function
    parameter_registers: std::collections::HashSet<Reg>,
}

impl SimpleRegAlloc {
    /// Create a new allocator with R5-R11 available (R3 is for return value, R4 for special uses)
    pub fn new() -> Self {
        Self {
            // Initialize in reverse order so R5 is preferred (popped last)
            // Following the formalized doc: POOL = [R5, R6, R7, R8, R9, R10, R11]
            free_list: vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5],
            reg_contents: BTreeMap::new(),
            spill_slots: BTreeMap::new(),
            next_spill_offset: 0,
            instructions: Vec::new(),
            pinned_values: std::collections::HashSet::new(),
            parameter_registers: std::collections::HashSet::new(),
        }
    }
    
    /// Mark a register as holding a parameter that should persist across statement boundaries
    pub fn mark_as_parameter(&mut self, reg: Reg) {
        self.parameter_registers.insert(reg);
    }
    
    /// Get a register for a value
    /// If all registers are in use, spills the least recently used one
    pub fn get_reg(&mut self, for_value: String) -> Reg {
        eprintln!("DEBUG get_reg called for '{}', reg_contents: {:?}", for_value, self.reg_contents);
        self.instructions.push(AsmInst::Comment(format!("get_reg for '{}'", for_value)));
        
        // Check if this value already has a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &for_value) {
            self.instructions.push(AsmInst::Comment(format!("  {} already in register", for_value)));
            return reg;
        }
        
        // Try to get a free register
        if let Some(reg) = self.free_list.pop() {
            self.instructions.push(AsmInst::Comment(format!("  Allocated free register for {}", for_value)));
            self.reg_contents.insert(reg, for_value);
            return reg;
        }
        
        // No free registers - need to spill
        self.instructions.push(AsmInst::Comment(format!("  No free registers, need to spill for {}", for_value)));
        
        // Log current register contents
        for (reg, val) in &self.reg_contents {
            let reg_name = match reg {
                Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
                Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
                Reg::R9 => "R9", Reg::R10 => "R10", Reg::R11 => "R11",
                _ => "R?",
            };
            self.instructions.push(AsmInst::Comment(format!("    {} contains '{}'", reg_name, val)));
        }
        
        // Pick the first non-pinned register in reg_contents
        let victim = self.reg_contents.iter()
            .find(|(_, val)| !self.pinned_values.contains(*val))
            .map(|(reg, _)| *reg)
            .expect("No spillable registers available!");
        let victim_value = self.reg_contents.remove(&victim).unwrap();
        
        let victim_name = match victim {
            Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
            Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
            Reg::R9 => "R9", Reg::R10 => "R10", Reg::R11 => "R11",
            _ => "R?",
        };
        self.instructions.push(AsmInst::Comment(format!("  Chose to spill {} from {}", victim_value, victim_name)));
        
        // Spill the victim
        let spill_offset = self.get_spill_slot(&victim_value);
        self.instructions.push(AsmInst::Comment(format!("Spilling {} to FP+{}", victim_value, spill_offset)));
        self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset));
        self.instructions.push(AsmInst::Store(victim, Reg::R13, Reg::R12));
        
        // Now use this register for the new value
        self.instructions.push(AsmInst::Comment(format!("  Now {} will contain {}", victim_name, for_value)));
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
    
    /// Get a temporary register that won't be tracked for spilling
    /// This is useful for short-lived temporaries in computations
    pub fn get_temp_reg(&mut self) -> Option<Reg> {
        // Try to get a free register without tracking it
        self.free_list.pop()
    }
    
    /// Check if a register is free
    pub fn is_free(&self, reg: Reg) -> bool {
        // A register is free if it's not tracking any value
        !self.reg_contents.contains_key(&reg)
    }
    
    /// Check if a value is tracked (either in register or spilled)
    pub fn is_tracked(&self, value: &str) -> bool {
        eprintln!("DEBUG is_tracked: Looking for '{}'", value);
        eprintln!("  reg_contents: {:?}", self.reg_contents);
        eprintln!("  spill_slots: {:?}", self.spill_slots.keys().collect::<Vec<_>>());
        // Check if it's in a register
        if self.reg_contents.values().any(|v| v == value) {
            return true;
        }
        // Check if it's been spilled
        self.spill_slots.contains_key(value)
    }
    
    /// Mark a register as containing a value without spilling
    /// This prevents the register from being chosen for spilling
    pub fn mark_in_use(&mut self, reg: Reg, value: String) {
        eprintln!("DEBUG mark_in_use: {:?} = {}", reg, value);
        self.reg_contents.insert(reg, value);
        // Remove from free list if it's there
        self.free_list.retain(|&r| r != reg);
    }
    
    /// Pin a value so it cannot be spilled
    pub fn pin_value(&mut self, value: String) {
        self.pinned_values.insert(value);
    }
    
    /// Unpin a value, allowing it to be spilled again
    pub fn unpin_value(&mut self, value: &str) {
        self.pinned_values.remove(value);
    }
    
    /// Clear all pinned values
    pub fn clear_pins(&mut self) {
        self.pinned_values.clear();
    }
    
    /// Get two different registers for operations that need them
    /// This ensures they are different and handles spilling
    pub fn get_two_regs(&mut self, value1: String, value2: String) -> ((Reg, Reg), Vec<AsmInst>) {
        let mut insts = Vec::new();
        
        // Try to get both from free list
        if self.free_list.len() >= 2 {
            let reg2 = self.free_list.pop().unwrap();
            let reg1 = self.free_list.pop().unwrap();
            self.reg_contents.insert(reg1, value1.clone());
            self.reg_contents.insert(reg2, value2.clone());
            return ((reg1, reg2), insts);
        }
        
        // If we have one free, use it for reg1
        if let Some(reg1) = self.free_list.pop() {
            self.reg_contents.insert(reg1, value1.clone());
            
            // Need to spill for reg2, but don't spill reg1 or pinned values
            let victims: Vec<_> = self.reg_contents.iter()
                .filter(|(reg, val)| **reg != reg1 && !self.pinned_values.contains(*val))
                .map(|(reg, _)| *reg)
                .collect();
            
            if let Some(&victim) = victims.first() {
                let victim_value = self.reg_contents.remove(&victim).unwrap();
                let spill_offset = self.get_spill_slot(&victim_value);
                insts.push(AsmInst::Comment(format!("Spilling {} to FP+{}", victim_value, spill_offset)));
                insts.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset));
                insts.push(AsmInst::Store(victim, Reg::R13, Reg::R12));
                self.reg_contents.insert(victim, value2.clone());
                return ((reg1, victim), insts);
            }
        }
        
        // Need to spill for both - but don't spill pinned values
        let victims: Vec<_> = self.reg_contents.iter()
            .filter(|(_, val)| !self.pinned_values.contains(*val))
            .map(|(reg, _)| *reg)
            .collect();
        if victims.len() >= 2 {
            let reg1 = victims[0];
            let reg2 = victims[1];
            
            // Spill reg1's current value
            let victim1_value = self.reg_contents.remove(&reg1).unwrap();
            let spill_offset1 = self.get_spill_slot(&victim1_value);
            insts.push(AsmInst::Comment(format!("Spilling {} to FP+{}", victim1_value, spill_offset1)));
            insts.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset1));
            insts.push(AsmInst::Store(reg1, Reg::R13, Reg::R12));
            
            // Spill reg2's current value
            let victim2_value = self.reg_contents.remove(&reg2).unwrap();
            let spill_offset2 = self.get_spill_slot(&victim2_value);
            insts.push(AsmInst::Comment(format!("Spilling {} to FP+{}", victim2_value, spill_offset2)));
            insts.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset2));
            insts.push(AsmInst::Store(reg2, Reg::R13, Reg::R12));
            
            self.reg_contents.insert(reg1, value1);
            self.reg_contents.insert(reg2, value2);
            return ((reg1, reg2), insts);
        }
        
        // Fallback - should never happen if we have at least 2 allocatable registers
        panic!("Cannot allocate two registers");
    }
    
    /// Clear a register - mark it as not containing any value
    /// This is used after function calls where registers may be clobbered
    /// Parameter registers are preserved and not cleared
    pub fn clear_register(&mut self, reg: Reg) {
        // Don't clear parameter registers
        if self.parameter_registers.contains(&reg) {
            eprintln!("DEBUG clear_register: Skipping parameter register {:?}", reg);
            return;
        }
        
        if let Some(val) = self.reg_contents.remove(&reg) {
            eprintln!("DEBUG clear_register: {:?} (contained {})", reg, val);
            self.instructions.push(AsmInst::Comment(format!("Clearing {:?} which contained {}", reg, val)));
        }
        // Add to free list if it's an allocatable register
        if matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11) {
            if !self.free_list.contains(&reg) {
                self.free_list.push(reg);
            }
        }
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
        // Debug: show what we're looking for and what's in registers
        self.instructions.push(AsmInst::Comment(format!("Looking for {} in registers", value)));
        for (reg, val) in &self.reg_contents {
            let reg_name = match reg {
                Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
                Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
                _ => "R?",
            };
            self.instructions.push(AsmInst::Comment(format!("  {} contains {}", reg_name, val)));
        }
        
        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &value) {
            let reg_name = match reg {
                Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
                Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
                _ => "R?",
            };
            self.instructions.push(AsmInst::Comment(format!("{} found in {}", value, reg_name)));
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
        self.instructions.push(AsmInst::Comment(format!("{} not found, allocating new register", value)));
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
    
    /// Free all temporaries (e.g., at statement boundaries)
    /// Parameters are preserved across statement boundaries
    pub fn free_all(&mut self) {
        eprintln!("DEBUG free_all called! Clearing temporaries (preserving parameters)");
        
        // Preserve parameter registers, free everything else
        let mut preserved = BTreeMap::new();
        for (reg, val) in &self.reg_contents {
            if self.parameter_registers.contains(reg) {
                preserved.insert(*reg, val.clone());
            }
        }
        
        self.reg_contents = preserved;
        
        // Reset free list but exclude parameter registers
        self.free_list = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5]
            .into_iter()
            .filter(|r| !self.parameter_registers.contains(r))
            .collect();
    }
    
    /// Check if a register is currently allocated
    pub fn is_allocated(&self, reg: Reg) -> bool {
        self.reg_contents.contains_key(&reg)
    }
    
    /// Get the value name stored in a register, if any
    pub fn get_register_value(&self, reg: Reg) -> Option<String> {
        self.reg_contents.get(&reg).cloned()
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
        self.pinned_values.clear();
        self.parameter_registers.clear();
    }
}