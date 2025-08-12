//! V2 Register Allocator with Proper R13 Initialization and Correct ABI
//! 
//! Key improvements over v1:
//! - R13 (stack bank) is properly initialized to 1
//! - Correct parameter passing in R5-R11 (not R3-R8)
//! - Proper fat pointer handling (R3=addr, R4=bank)
//! - Correct bank register usage for memory operations

use rcc_codegen::{AsmInst, Reg};
use std::collections::{BTreeMap, BTreeSet};
use log::{debug, trace};

/// V2 register allocator with correct ABI and bank handling
pub struct RegAllocV2 {
    /// Free list of available registers (R5-R11 only, R3-R4 reserved for returns)
    free_list: Vec<Reg>,
    
    /// Map from register to what it contains
    reg_contents: BTreeMap<Reg, String>,
    
    /// Map from spilled values to their stack offsets
    spill_slots: BTreeMap<String, i16>,
    
    /// Next available spill slot offset (relative to FP)
    next_spill_offset: i16,
    
    /// Instructions to emit (for spill/reload code)
    pub instructions: Vec<AsmInst>,
    
    /// Track the last spilled value and its offset
    last_spilled: Option<(String, i16)>,
    
    /// Set of values that are temporarily pinned (BTreeSet for determinism)
    pinned_values: BTreeSet<String>,
    
    /// Stack offset for parameters (obsolete - params are on stack)
    /// Kept for compatibility but not used
    _parameter_placeholder: Option<()>,
    
    /// Track if R13 has been initialized for this function
    pub r13_initialized: bool,
    
    /// Track fat pointer bank components
    /// Maps value name to its bank register or tag
    pointer_banks: BTreeMap<String, BankInfo>,
}

#[derive(Debug, Clone)]
pub enum BankInfo {
    Global,     // Bank 0 - use R0
    Stack,      // Bank 1 - use R13 (must be initialized!)
    Register(Reg), // Dynamic bank in a register
}

impl Default for RegAllocV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl RegAllocV2 {
    /// Create a new allocator with R5-R11 available
    /// R3-R4 are reserved for return values/fat pointers
    pub fn new() -> Self {
        Self {
            // R5-R11 are allocatable (7 registers total)
            free_list: vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5],
            reg_contents: BTreeMap::new(),
            spill_slots: BTreeMap::new(),
            next_spill_offset: 0,
            instructions: Vec::new(),
            last_spilled: None,
            pinned_values: BTreeSet::new(),
            _parameter_placeholder: None,
            r13_initialized: false,
            pointer_banks: BTreeMap::new(),
        }
    }
    
    /// Initialize R13 as stack bank register (MUST be called at function start!)
    pub fn init_stack_bank(&mut self) {
        if !self.r13_initialized {
            self.instructions.push(AsmInst::Comment("Initialize R13 as stack bank (1)".to_string()));
            self.instructions.push(AsmInst::LI(Reg::R13, 1));
            self.r13_initialized = true;
            debug!("Initialized R13 to 1 for stack bank");
        }
    }
    
    /// Get the bank register for a pointer
    pub fn get_bank_register(&mut self, ptr_value: &str) -> Reg {
        match self.pointer_banks.get(ptr_value) {
            Some(BankInfo::Global) => {
                trace!("Using R0 for global bank");
                Reg::R0  // Global bank is always 0
            }
            Some(BankInfo::Stack) => {
                if !self.r13_initialized {
                    panic!("R13 not initialized! Call init_stack_bank() first");
                }
                trace!("Using R13 for stack bank");
                Reg::R13  // Stack bank register
            }
            Some(BankInfo::Register(reg)) => {
                trace!("Using {reg:?} for dynamic bank");
                *reg
            }
            None => {
                // Default to stack if unknown
                debug!("No bank info for '{ptr_value}', defaulting to stack");
                if !self.r13_initialized {
                    panic!("R13 not initialized! Call init_stack_bank() first");
                }
                Reg::R13
            }
        }
    }
    
    /// Set bank info for a pointer value
    pub fn set_pointer_bank(&mut self, ptr_value: String, bank: BankInfo) {
        debug!("Setting bank info for '{ptr_value}': {bank:?}");
        self.pointer_banks.insert(ptr_value, bank);
    }
    
    /// Load parameter from stack (params are stack-based per spec)
    /// Parameters are at negative offsets from FP
    pub fn load_parameter(&mut self, param_idx: usize) -> Reg {
        // Parameters are pushed before call, accessed via FP
        // param0 at FP-3, param1 at FP-4, etc.
        let offset = -(param_idx as i16 + 3);
        let dest = self.get_reg(format!("param{param_idx}"));
        
        self.instructions.push(AsmInst::Comment(format!("Load param {param_idx} from FP{offset}")));
        self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, offset));
        self.instructions.push(AsmInst::Load(dest, Reg::R13, Reg::R12));
        
        dest
    }
    
    /// Get a register for a value
    pub fn get_reg(&mut self, for_value: String) -> Reg {
        trace!("get_reg for '{}', reg_contents: {:?}", for_value, self.reg_contents);
        
        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &for_value) {
            trace!("  {for_value} already in {reg:?}");
            return reg;
        }
        
        // Try to get a free register
        if let Some(reg) = self.free_list.pop() {
            trace!("  Allocated free {reg:?} for {for_value}");
            self.reg_contents.insert(reg, for_value);
            return reg;
        }
        
        // Need to spill - find non-pinned victim
        debug!("  Need to spill for {}, pinned: {:?}", for_value, self.pinned_values);
        
        let victim = self.reg_contents.iter()
            .find(|(_, val)| !self.pinned_values.contains(*val))
            .map(|(reg, _)| *reg)
            .expect("No spillable registers!");
        
        let victim_value = self.reg_contents.remove(&victim).unwrap();
        trace!("  Spilling {victim_value} from {victim:?}");
        
        // Spill the victim
        let spill_offset = self.get_spill_slot(&victim_value);
        
        // Ensure R13 is initialized before any stack access
        if !self.r13_initialized {
            self.init_stack_bank();
        }
        
        self.instructions.push(AsmInst::Comment(format!("Spilling {victim_value} to FP+{spill_offset}")));
        self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset));
        self.instructions.push(AsmInst::Store(victim, Reg::R13, Reg::R12));
        
        // Preserve bank info if this was a pointer
        if let Some(_bank) = self.pointer_banks.get(&victim_value).cloned() {
            // We'll need to track this for reload
            debug!("Preserved bank info for spilled pointer {victim_value}");
        }
        
        self.last_spilled = Some((victim_value, spill_offset));
        
        // Use the register for new value
        self.reg_contents.insert(victim, for_value);
        victim
    }
    
    /// Reload a spilled value
    pub fn reload(&mut self, value: String) -> Reg {
        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &value) {
            return reg;
        }
        
        // Check if spilled
        if let Some(&offset) = self.spill_slots.get(&value) {
            let reg = self.get_reg(value.clone());
            
            // Ensure R13 is initialized
            if !self.r13_initialized {
                self.init_stack_bank();
            }
            
            self.instructions.push(AsmInst::Comment(format!("Reloading {value} from FP+{offset}")));
            self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, offset));
            self.instructions.push(AsmInst::Load(reg, Reg::R13, Reg::R12));
            return reg;
        }
        
        // Not spilled, allocate new
        self.get_reg(value)
    }
    
    /// Pin a value to prevent spilling
    pub fn pin_value(&mut self, value: String) {
        self.pinned_values.insert(value);
    }
    
    /// Unpin a value
    pub fn unpin_value(&mut self, value: &str) {
        self.pinned_values.remove(value);
    }
    
    /// Clear all pins
    pub fn clear_pins(&mut self) {
        self.pinned_values.clear();
    }
    
    /// Free all temporaries at statement boundaries
    /// Since params are on stack, we can free all registers
    pub fn free_temporaries(&mut self) {
        debug!("Freeing all temporaries");
        
        self.reg_contents.clear();
        
        // Reset free list to all allocatable registers
        self.free_list = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5];
    }
    
    /// Mark a register as in use
    pub fn mark_in_use(&mut self, reg: Reg, value: String) {
        self.reg_contents.insert(reg, value);
        self.free_list.retain(|&r| r != reg);
    }
    
    /// Free a specific register
    pub fn free_reg(&mut self, reg: Reg) {
        // Only free R5-R11
        if matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11) {
            self.reg_contents.remove(&reg);
            if !self.free_list.contains(&reg) {
                // Maintain deterministic order - insert in the right position
                // Order should be R11, R10, R9, R8, R7, R6, R5
                let position = match reg {
                    Reg::R11 => 0,
                    Reg::R10 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::R9 | Reg::R8 | Reg::R7 | Reg::R6 | Reg::R5)).unwrap_or(self.free_list.len()),
                    Reg::R9 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::R8 | Reg::R7 | Reg::R6 | Reg::R5)).unwrap_or(self.free_list.len()),
                    Reg::R8 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::R7 | Reg::R6 | Reg::R5)).unwrap_or(self.free_list.len()),
                    Reg::R7 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::R6 | Reg::R5)).unwrap_or(self.free_list.len()),
                    Reg::R6 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::R5)).unwrap_or(self.free_list.len()),
                    Reg::R5 => self.free_list.len(),
                    _ => unreachable!(),
                };
                self.free_list.insert(position, reg);
            }
        }
    }
    
    /// Get a spill slot for a value
    fn get_spill_slot(&mut self, value: &str) -> i16 {
        if let Some(&offset) = self.spill_slots.get(value) {
            offset
        } else {
            let offset = self.next_spill_offset;
            self.next_spill_offset += 1;
            self.spill_slots.insert(value.to_string(), offset);
            debug!("Allocated spill slot for '{value}' at FP+{offset}");
            offset
        }
    }
    
    /// Set the base offset for spill slots
    pub fn set_spill_base(&mut self, offset: i16) {
        self.next_spill_offset = offset;
    }
    
    /// Take accumulated instructions
    pub fn take_instructions(&mut self) -> Vec<AsmInst> {
        std::mem::take(&mut self.instructions)
    }
    
    /// Get last spilled info
    pub fn take_last_spilled(&mut self) -> Option<(String, i16)> {
        self.last_spilled.take()
    }
    
    /// Check if a value is tracked
    pub fn is_tracked(&self, value: &str) -> bool {
        self.reg_contents.values().any(|v| v == value) || 
        self.spill_slots.contains_key(value)
    }
    
    /// Reset for new function
    pub fn reset(&mut self) {
        self.free_list = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5];
        self.reg_contents.clear();
        self.spill_slots.clear();
        self.next_spill_offset = 0;
        self.instructions.clear();
        self.last_spilled = None;
        self.pinned_values.clear();
        self._parameter_placeholder = None;
        self.r13_initialized = false;
        self.pointer_banks.clear();
    }
}

// Tests moved to tests/regalloc_tests.rs

// Make some fields accessible for testing
#[cfg(test)]
impl RegAllocV2 {
    pub fn test_reg_contents(&self) -> &BTreeMap<Reg, String> {
        &self.reg_contents
    }
    
    pub fn test_free_list_len(&self) -> usize {
        self.free_list.len()
    }
    
    pub fn test_next_spill_offset(&self) -> i16 {
        self.next_spill_offset
    }
}