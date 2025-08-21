//! V2 Register Allocator - 32 Register Architecture

#![allow(dead_code)]
//! 
//! Updated for 32-register architecture:
//! - Allocatable registers: T0-T7 (temporaries) and S0-S3 (saved)
//! - Arguments passed in A0-A3
//! - Return values in RV0-RV1
//! - SC/SB/SP/FP/GP for scratch, stack bank, stack, frame, and global pointers

use rcc_codegen::{AsmInst, Reg};
use std::collections::{HashMap, BTreeMap, BTreeSet};
use log::{debug, trace};
use super::bank::BankInfo;

/// The allocatable registers for the 32-register architecture
/// Order matters: we prefer saved registers (S0-S3) first as they're callee-saved
/// Then temporaries (T0-T7) which are caller-saved
pub(crate) const ALLOCATABLE_REGISTERS: [Reg; 12] = [
    // Saved registers first (callee-saved, less likely to need spilling across calls)
    Reg::S3, Reg::S2, Reg::S1, Reg::S0,
    // Then temporaries (caller-saved)
    Reg::T7, Reg::T6, Reg::T5, Reg::T4, Reg::T3, Reg::T2, Reg::T1, Reg::T0,
];

/// V2 register allocator with 32-register architecture
pub(super) struct RegAllocV2 {
    /// Free list of available registers (T0-T7, S0-S3)
    free_list: Vec<Reg>,
    
    /// Map from register to what it contains
    reg_contents: HashMap<Reg, String>,
    
    /// Map from spilled values to their stack offsets
    spill_slots: BTreeMap<String, i16>,
    
    /// Next available spill slot offset (relative to FP)
    next_spill_offset: i16,
    
    /// Instructions to emit (for spill/reload code)
    pub(super) instructions: Vec<AsmInst>,
    
    /// Track the last spilled value and its offset
    last_spilled: Option<(String, i16)>,
    
    /// Set of values that are temporarily pinned (BTreeSet for determinism)
    pinned_values: BTreeSet<String>,
    
    /// Track if SB has been initialized for this function
    pub(super) sb_initialized: bool,
    
    /// Track fat pointer bank components
    /// Maps value name to its bank register or tag
    pub(super) pointer_banks: BTreeMap<String, BankInfo>,
    
    /// Track which callee-saved registers (S0-S3) have been used
    /// These need to be saved in prologue and restored in epilogue
    pub(super) used_callee_saved: BTreeSet<Reg>,
}


impl Default for RegAllocV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl RegAllocV2 {
    /// Create a new allocator with T0-T7 and S0-S3 available
    /// RV0-RV1 are reserved for return values
    /// A0-A3 are reserved for arguments
    pub(super) fn new() -> Self {
        Self {
            // Use the centralized list, reversed for pop() which takes from the end
            free_list: ALLOCATABLE_REGISTERS.iter().rev().copied().collect(),
            reg_contents: HashMap::new(),
            spill_slots: BTreeMap::new(),
            next_spill_offset: 0,
            instructions: Vec::new(),
            last_spilled: None,
            pinned_values: BTreeSet::new(),
            sb_initialized: false,
            pointer_banks: BTreeMap::new(),
            used_callee_saved: BTreeSet::new(),
        }
    }
    
    /// Mark SB as initialized (it's already set by function prologue or crt0)
    pub(super) fn init_stack_bank(&mut self) {
        if !self.sb_initialized {
            // SB is already initialized by the function prologue (for functions)
            // or by crt0.asm (at program start). We just mark it as initialized
            // so the allocator knows it can use it for spilling.
            self.sb_initialized = true;
            debug!("SB already initialized by prologue/crt0, marking as ready");
        }
    }

    /// Get the bank register for a pointer
    pub(super) fn get_bank_register(&mut self, ptr_value: &str) -> Reg {
        match self.pointer_banks.get(ptr_value) {
            Some(BankInfo::Global) => {
                trace!("Using GP for global bank");
                Reg::Gp  // Global pointer register for globals
            }
            Some(BankInfo::Stack) => {
                if !self.sb_initialized {
                    panic!("SB not initialized! Call init_stack_bank() first");
                }
                trace!("Using SB for stack bank");
                Reg::Sb  // Stack bank register
            }
            Some(BankInfo::Heap(_)) => {
                panic!("Cannot get bank register for Heap bank in allocator - heap banks require explicit register allocation");
            }
            Some(BankInfo::Register(reg)) => {
                trace!("Using {reg:?} for dynamic bank");
                *reg
            }
            Some(BankInfo::Dynamic(name)) => {
                panic!("Cannot get bank register for Dynamic('{name}') in allocator - should use RegisterPressureManager");
            }
            None => {
                // Default to stack if unknown
                debug!("No bank info for '{ptr_value}', defaulting to stack");
                if !self.sb_initialized {
                    panic!("SB not initialized! Call init_stack_bank() first");
                }
                Reg::Sb
            }
        }
    }

    /// Set bank info for a pointer value
    pub(super) fn set_pointer_bank(&mut self, ptr_value: String, bank: BankInfo) {
        debug!("Setting bank info for '{ptr_value}': {bank:?}");
        self.pointer_banks.insert(ptr_value, bank);
    }

    /// Load parameter from stack (params are stack-based per spec)
    /// Parameters are at negative offsets from FP
    pub(super) fn load_parameter(&mut self, param_idx: usize) -> Reg {
        // Parameters are pushed before call, accessed via FP
        // param0 at FP-3, param1 at FP-4, etc.
        let offset = -(param_idx as i16 + 3);
        debug!("Loading parameter {param_idx} from stack at FP{offset}");
        let dest = self.get_reg(format!("param{param_idx}"));

        self.instructions.push(AsmInst::Comment(format!("Load param {param_idx} from FP{offset}")));
        self.instructions.push(AsmInst::AddI(Reg::Sc, Reg::Fp, offset));
        self.instructions.push(AsmInst::Load(dest, Reg::Sb, Reg::Sc));
        trace!("  Loaded param{param_idx} into {dest:?}");

        dest
    }

    /// Get a register for a value
    pub(super) fn get_reg(&mut self, for_value: String) -> Reg {
        trace!("get_reg for '{}', reg_contents: {:?}", for_value, self.reg_contents);
        trace!("  free_list: {:?}, pinned: {:?}", self.free_list, self.pinned_values);

        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &for_value) {
            trace!("  {for_value} already in {reg:?}");
            return reg;
        }

        // Try to get a free register
        if let Some(reg) = self.free_list.pop() {
            trace!("  Allocated free {reg:?} for {for_value}");
            debug!("Allocated {reg:?} for '{for_value}' (was free)");
            
            // Track if we're using a callee-saved register
            if matches!(reg, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3) {
                self.used_callee_saved.insert(reg);
                trace!("  Marked callee-saved register {reg:?} as used");
            }
            
            self.reg_contents.insert(reg, for_value);
            return reg;
        }

        // Need to spill - find non-pinned victim
        debug!("Need to spill for '{}', pinned: {:?}", for_value, self.pinned_values);
        trace!("  Current register contents: {:?}", self.reg_contents);

        let victim = self.reg_contents.iter()
            .find(|(_, val)| !self.pinned_values.contains(*val))
            .map(|(reg, _)| *reg)
            .expect("No spillable registers!");

        let victim_value = self.reg_contents.remove(&victim).unwrap();
        debug!("Spilling '{victim_value}' from {victim:?} to make room for '{for_value}'");
        trace!("  Spilling {victim_value} from {victim:?}");

        // Spill the victim
        let spill_offset = self.get_spill_slot(&victim_value);

        // Ensure SB is initialized before any stack access
        if !self.sb_initialized {
            self.init_stack_bank();
        }

        self.instructions.push(AsmInst::Comment(format!("Spilling {victim_value} to FP+{spill_offset}")));
        self.instructions.push(AsmInst::AddI(Reg::Sc, Reg::Fp, spill_offset));
        self.instructions.push(AsmInst::Store(victim, Reg::Sb, Reg::Sc));

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
    pub(super) fn reload(&mut self, value: String) -> Reg {
        trace!("reload('{}'), spill_slots: {:?}", value, self.spill_slots);

        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &value) {
            trace!("  '{value}' already in {reg:?}, no reload needed");
            return reg;
        }

        // Check if spilled
        if let Some(&offset) = self.spill_slots.get(&value) {
            debug!("Reloading '{value}' from spill slot FP+{offset}");
            let reg = self.get_reg(value.clone());

            // Ensure SB is initialized
            if !self.sb_initialized {
                trace!("  Initializing SB before reload");
                self.init_stack_bank();
            }

            self.instructions.push(AsmInst::Comment(format!("Reloading {value} from FP+{offset}")));
            self.instructions.push(AsmInst::AddI(Reg::Sc, Reg::Fp, offset));
            self.instructions.push(AsmInst::Load(reg, Reg::Sb, Reg::Sc));
            debug!("Reloaded '{value}' into {reg:?} from FP+{offset}");
            return reg;
        }
        
        // Not spilled, allocate new
        trace!("  '{value}' not spilled, allocating new register");
        self.get_reg(value)
    }
    
    /// Pin a value to prevent spilling
    pub(super) fn pin_value(&mut self, value: String) {
        trace!("Pinning value '{value}'");
        self.pinned_values.insert(value);
    }
    
    /// Unpin a value
    pub(super) fn unpin_value(&mut self, value: &str) {
        trace!("Unpinning value '{value}'");
        self.pinned_values.remove(value);
    }
    
    /// Clear all pins
    pub(super) fn clear_pins(&mut self) {
        trace!("Clearing all pinned values (was: {:?})", self.pinned_values);
        self.pinned_values.clear();
    }
    
    /// Free all temporaries at statement boundaries
    /// Since params are on stack, we can free all registers
    pub(super) fn free_temporaries(&mut self) {
        debug!("Freeing all temporaries, was: {:?}", self.reg_contents);
        trace!("  Resetting free_list to all allocatable registers");
        
        self.reg_contents.clear();
        
        // Reset free list using the centralized constant, reversed for pop()
        self.free_list = ALLOCATABLE_REGISTERS.iter().rev().copied().collect();
    }
    
    /// Mark a register as in use
    pub(super) fn mark_in_use(&mut self, reg: Reg, value: String) {
        trace!("Marking {reg:?} as in use for '{value}'");
        self.reg_contents.insert(reg, value);
        self.free_list.retain(|&r| r != reg);
    }
    
    /// Free a specific register
    pub(super) fn free_reg(&mut self, reg: Reg) {
        // Only free allocatable registers (T0-T7, S0-S3)
        if matches!(reg, 
            Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7 |
            Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3
        ) {
            self.reg_contents.remove(&reg);
            if !self.free_list.contains(&reg) {
                // Maintain deterministic order - insert in the right position
                // Order: S3, S2, S1, S0, T7, T6, T5, T4, T3, T2, T1, T0
                let position = match reg {
                    Reg::S3 => 0,
                    Reg::S2 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::S1 | Reg::S0 | Reg::T7 | Reg::T6 | Reg::T5 | Reg::T4 | Reg::T3 | Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::S1 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::S0 | Reg::T7 | Reg::T6 | Reg::T5 | Reg::T4 | Reg::T3 | Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::S0 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T7 | Reg::T6 | Reg::T5 | Reg::T4 | Reg::T3 | Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T7 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T6 | Reg::T5 | Reg::T4 | Reg::T3 | Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T6 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T5 | Reg::T4 | Reg::T3 | Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T5 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T4 | Reg::T3 | Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T4 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T3 | Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T3 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T2 | Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T2 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T1 | Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T1 => self.free_list.iter().position(|&r| 
                        matches!(r, Reg::T0)).unwrap_or(self.free_list.len()),
                    Reg::T0 => self.free_list.len(),
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
    pub(super) fn set_spill_base(&mut self, offset: i16) {
        self.next_spill_offset = offset;
    }
    
    /// Take accumulated instructions
    pub(super) fn take_instructions(&mut self) -> Vec<AsmInst> {
        std::mem::take(&mut self.instructions)
    }
    
    /// Get last spilled info
    pub(super) fn take_last_spilled(&mut self) -> Option<(String, i16)> {
        self.last_spilled.take()
    }
    
    /// Check if a value is tracked
    pub(super) fn is_tracked(&self, value: &str) -> bool {
        self.reg_contents.values().any(|v| v == value) || 
        self.spill_slots.contains_key(value)
    }
    
    /// Reset for new function
    pub(super) fn reset(&mut self) {
        debug!("Resetting allocator for new function");
        trace!("  Clearing spill_slots: {:?}, reg_contents: {:?}", self.spill_slots, self.reg_contents);
        self.free_list = ALLOCATABLE_REGISTERS.iter().rev().copied().collect();
        self.reg_contents.clear();
        self.spill_slots.clear();
        self.next_spill_offset = 0;
        self.instructions.clear();
        self.last_spilled = None;
        self.pinned_values.clear();
        self.sb_initialized = false;
        self.pointer_banks.clear();
        self.used_callee_saved.clear();
    }
    
    /// Get the list of callee-saved registers that have been used
    /// These need to be saved in prologue and restored in epilogue
    pub(super) fn get_used_callee_saved(&self) -> Vec<Reg> {
        let mut regs: Vec<Reg> = self.used_callee_saved.iter().copied().collect();
        // Sort for deterministic order (S0, S1, S2, S3)
        regs.sort_by_key(|r| match r {
            Reg::S0 => 0,
            Reg::S1 => 1,
            Reg::S2 => 2,
            Reg::S3 => 3,
            _ => unreachable!("Only S0-S3 should be in used_callee_saved, found {:?}", r),
        });
        regs
    }
}

// Unit tests for the allocator
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

// Make some fields accessible for testing
#[cfg(test)]
impl RegAllocV2 {
    pub fn test_reg_contents(&self) -> &HashMap<Reg, String> {
        &self.reg_contents
    }
    
    pub fn test_free_list_len(&self) -> usize {
        self.free_list.len()
    }
    
    pub fn test_next_spill_offset(&self) -> i16 {
        self.next_spill_offset
    }
    
    pub fn test_spill_slots(&self) -> &BTreeMap<String, i16> {
        &self.spill_slots
    }
}