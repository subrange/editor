//! Register Pressure Manager for V2 Backend
//! 
//! Based on the formalized register spilling algorithm with Sethi-Ullman
//! expression evaluation order to minimize register pressure.

use std::collections::{BTreeMap, VecDeque};
use rcc_codegen::{AsmInst, Reg};
use rcc_common::TempId;
use crate::ir::{BasicBlock, Instruction, Value, IrBinaryOp};
use crate::v2::regalloc::RegAllocV2;
use log::debug;

/// Register need calculation result
#[derive(Debug, Clone)]
pub struct RegisterNeed {
    /// Number of registers needed to evaluate this expression
    pub count: usize,
    
    /// Is this a leaf node (constant, load, etc)
    pub is_leaf: bool,
}

/// Value lifetime information
#[derive(Debug, Clone)]
pub struct ValueLifetime {
    /// Instruction index where value is defined
    pub def_point: usize,
    
    /// Last instruction index where value is used
    pub last_use: Option<usize>,
    
    /// All instruction indices where value is used
    pub use_points: Vec<usize>,
    
    /// Whether this value crosses a function call
    pub crosses_call: bool,
}

/// LRU-based register allocator with Sethi-Ullman ordering
pub struct RegisterPressureManager {
    /// Base allocator for low-level operations
    allocator: RegAllocV2,
    
    /// Free list of available registers
    free_list: VecDeque<Reg>,
    
    /// LRU queue of in-use registers (front = LRU, back = MRU)
    lru_queue: VecDeque<Reg>,
    
    /// Map from register to what it contains
    reg_contents: BTreeMap<Reg, String>,
    
    /// Map from register to spill slot (if spilled)
    reg_to_slot: BTreeMap<Reg, i16>,
    
    /// Map from value to spill slot (for reloading)
    value_to_slot: BTreeMap<String, i16>,
    
    /// Next available spill slot
    next_spill_slot: i16,
    
    /// Lifetime information for values
    lifetimes: BTreeMap<TempId, ValueLifetime>,
    
    /// Instructions to emit
    instructions: Vec<AsmInst>,
    
    /// Number of local variables (for calculating spill addresses)
    local_count: i16,
}

impl RegisterPressureManager {
    pub fn new(local_count: i16) -> Self {
        // Initialize with R5-R11 available
        let mut free_list = VecDeque::new();
        free_list.push_back(Reg::R5);
        free_list.push_back(Reg::R6);
        free_list.push_back(Reg::R7);
        free_list.push_back(Reg::R8);
        free_list.push_back(Reg::R9);
        free_list.push_back(Reg::R10);
        free_list.push_back(Reg::R11);
        
        Self {
            allocator: RegAllocV2::new(),
            free_list,
            lru_queue: VecDeque::new(),
            reg_contents: BTreeMap::new(),
            reg_to_slot: BTreeMap::new(),
            value_to_slot: BTreeMap::new(),
            next_spill_slot: 0,
            lifetimes: BTreeMap::new(),
            instructions: Vec::new(),
            local_count,
        }
    }
    
    /// Initialize the allocator (must be called before use)
    pub fn init(&mut self) {
        self.allocator.init_stack_bank();
        self.allocator.set_spill_base(self.local_count);
    }
    
    /// Calculate register need for an expression (Sethi-Ullman)
    pub fn calculate_need(&self, value: &Value) -> RegisterNeed {
        match value {
            Value::Constant(_) | Value::Global(_) | Value::Function(_) => {
                RegisterNeed { count: 1, is_leaf: true }
            }
            Value::Temp(id) => {
                // If already in register, need 0, else need 1
                if self.reg_contents.values().any(|v| v == &format!("t{id}")) {
                    RegisterNeed { count: 0, is_leaf: false }
                } else {
                    RegisterNeed { count: 1, is_leaf: true }
                }
            }
            Value::FatPtr(_) => {
                // Fat pointers need 2 registers
                RegisterNeed { count: 2, is_leaf: false }
            }
            Value::Undef => {
                RegisterNeed { count: 0, is_leaf: true }
            }
        }
    }
    
    /// Calculate register need for a binary operation
    pub fn calculate_binary_need(&self, lhs: &Value, rhs: &Value) -> (RegisterNeed, bool) {
        let left_need = self.calculate_need(lhs);
        let right_need = self.calculate_need(rhs);
        
        // Should evaluate right first if it needs more registers
        let swap = right_need.count > left_need.count;
        
        let total_need = if left_need.count == right_need.count {
            // If both need same, we need one extra register
            RegisterNeed { 
                count: left_need.count + 1, 
                is_leaf: false 
            }
        } else {
            // We can reuse registers, need max of both
            RegisterNeed { 
                count: left_need.count.max(right_need.count), 
                is_leaf: false 
            }
        };
        
        (total_need, swap)
    }
    
    /// Get a register using LRU spilling
    pub fn get_register(&mut self, for_value: String) -> Reg {
        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &for_value) {
            // Move to back of LRU (most recently used)
            if let Some(pos) = self.lru_queue.iter().position(|&r| r == reg) {
                self.lru_queue.remove(pos);
                self.lru_queue.push_back(reg);
            }
            return reg;
        }
        
        // Try to get a free register
        if let Some(reg) = self.free_list.pop_front() {
            self.reg_contents.insert(reg, for_value.clone());
            self.lru_queue.push_back(reg);
            debug!("Allocated {reg:?} for {for_value}");
            return reg;
        }
        
        // Need to spill - pick LRU victim
        let victim = self.lru_queue.pop_front()
            .expect("No registers to spill!");
        
        self.spill_register(victim);
        
        // Now victim is free
        self.reg_contents.insert(victim, for_value);
        self.lru_queue.push_back(victim);
        victim
    }
    
    /// Spill a register to stack
    fn spill_register(&mut self, reg: Reg) {
        if let Some(value) = self.reg_contents.get(&reg).cloned() {
            // Get or allocate spill slot
            let slot = self.reg_to_slot.get(&reg).copied()
                .unwrap_or_else(|| {
                    let s = self.next_spill_slot;
                    self.next_spill_slot += 1;
                    self.reg_to_slot.insert(reg, s);
                    self.value_to_slot.insert(value.clone(), s);
                    s
                });
            
            // Generate spill instructions using R12 as scratch
            self.instructions.push(AsmInst::Comment(format!("Spill {value} to slot {slot}")));
            self.instructions.push(AsmInst::Add(Reg::R12, Reg::R15, Reg::R0));
            self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R12, self.local_count + slot));
            self.instructions.push(AsmInst::Store(reg, Reg::R13, Reg::R12));
            
            debug!("Spilled {value} from {reg:?} to slot {slot}");
        }
        
        self.reg_contents.remove(&reg);
    }
    
    /// Reload a value from spill slot
    pub fn reload_value(&mut self, value: String) -> Reg {
        // Check if already in register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &value) {
            return reg;
        }
        
        // Check if spilled
        if let Some(&slot) = self.value_to_slot.get(&value) {
            let reg = self.get_register(value.clone());
            
            // Generate reload instructions
            self.instructions.push(AsmInst::Comment(format!("Reload {value} from slot {slot}")));
            self.instructions.push(AsmInst::Add(Reg::R12, Reg::R15, Reg::R0));
            self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R12, self.local_count + slot));
            self.instructions.push(AsmInst::Load(reg, Reg::R13, Reg::R12));
            
            debug!("Reloaded {value} into {reg:?} from slot {slot}");
            return reg;
        }
        
        // Not spilled, allocate new
        self.get_register(value)
    }
    
    /// Free a register
    pub fn free_register(&mut self, reg: Reg) {
        if let Some(pos) = self.lru_queue.iter().position(|&r| r == reg) {
            self.lru_queue.remove(pos);
        }
        self.reg_contents.remove(&reg);
        if !self.free_list.contains(&reg) {
            self.free_list.push_back(reg);
        }
    }
    
    /// Spill all registers (e.g., before a call)
    pub fn spill_all(&mut self) {
        let regs_to_spill: Vec<Reg> = self.lru_queue.iter().copied().collect();
        for reg in regs_to_spill {
            self.spill_register(reg);
            self.free_register(reg);
        }
    }
    
    /// Get register for a Value
    pub fn get_value_register(&mut self, value: &Value) -> Reg {
        match value {
            Value::Temp(id) => {
                self.get_register(format!("t{id}"))
            }
            Value::Constant(val) => {
                let reg = self.get_register(format!("const_{val}"));
                self.instructions.push(AsmInst::LI(reg, *val as i16));
                reg
            }
            Value::Global(name) => {
                // For globals, we need to load the address
                let reg = self.get_register(format!("global_{name}"));
                // This would need proper global offset calculation
                self.instructions.push(AsmInst::Comment(format!("Load global {name}")));
                reg
            }
            Value::FatPtr(ptr) => {
                // Handle fat pointer - needs special handling
                
                // Bank would need separate handling
                self.get_value_register(&ptr.addr)
            }
            _ => {
                panic!("Unsupported value type for register allocation");
            }
        }
    }
    
    /// Emit a binary operation with Sethi-Ullman ordering
    pub fn emit_binary_op(&mut self, 
                          op: IrBinaryOp, 
                          lhs: &Value, 
                          rhs: &Value,
                          result_temp: TempId) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        // Calculate needs and determine evaluation order
        let (_, should_swap) = self.calculate_binary_need(lhs, rhs);
        
        let (first, second) = if should_swap {
            (rhs, lhs)
        } else {
            (lhs, rhs)
        };
        
        // Evaluate in optimal order
        let first_reg = self.get_value_register(first);
        let second_reg = self.get_value_register(second);
        
        // Emit the operation (reusing first register for result)
        let result_reg = first_reg;
        
        match op {
            IrBinaryOp::Add => {
                insts.push(AsmInst::Add(result_reg, first_reg, second_reg));
            }
            IrBinaryOp::Sub => {
                if should_swap {
                    // If we swapped, need to be careful with non-commutative ops
                    insts.push(AsmInst::Sub(result_reg, second_reg, first_reg));
                } else {
                    insts.push(AsmInst::Sub(result_reg, first_reg, second_reg));
                }
            }
            IrBinaryOp::Mul => {
                insts.push(AsmInst::Mul(result_reg, first_reg, second_reg));
            }
            IrBinaryOp::And => {
                insts.push(AsmInst::And(result_reg, first_reg, second_reg));
            }
            IrBinaryOp::Or => {
                insts.push(AsmInst::Or(result_reg, first_reg, second_reg));
            }
            IrBinaryOp::Xor => {
                insts.push(AsmInst::Xor(result_reg, first_reg, second_reg));
            }
            IrBinaryOp::Slt => {
                insts.push(AsmInst::Slt(result_reg, first_reg, second_reg));
            }
            // ... other operations
            _ => {
                insts.push(AsmInst::Comment(format!("TODO: Binary op {op:?}")));
            }
        }
        
        // Free the second register (first is reused for result)
        self.free_register(second_reg);
        
        // Update register contents to track the result
        self.reg_contents.insert(result_reg, format!("t{result_temp}"));
        
        insts
    }
    
    /// Analyze a basic block to build lifetime information
    pub fn analyze_block(&mut self, block: &BasicBlock) {
        for (idx, inst) in block.instructions.iter().enumerate() {
            match inst {
                Instruction::Binary { result, lhs, rhs, .. } => {
                    // Define result
                    self.lifetimes.entry(*result).or_insert(ValueLifetime {
                        def_point: idx,
                        last_use: None,
                        use_points: Vec::new(),
                        crosses_call: false,
                    });
                    
                    // Use operands
                    self.record_use(lhs, idx);
                    self.record_use(rhs, idx);
                }
                Instruction::Load { result, ptr, .. } => {
                    self.lifetimes.entry(*result).or_insert(ValueLifetime {
                        def_point: idx,
                        last_use: None,
                        use_points: Vec::new(),
                        crosses_call: false,
                    });
                    self.record_use(ptr, idx);
                }
                Instruction::Store { value, ptr } => {
                    self.record_use(value, idx);
                    self.record_use(ptr, idx);
                }
                Instruction::Call { result, args, .. } => {
                    if let Some(res) = result {
                        self.lifetimes.entry(*res).or_insert(ValueLifetime {
                            def_point: idx,
                            last_use: None,
                            use_points: Vec::new(),
                            crosses_call: false,
                        });
                    }
                    
                    // Mark all live values as crossing a call
                    for lifetime in self.lifetimes.values_mut() {
                        if lifetime.def_point < idx {
                            if let Some(last) = lifetime.last_use {
                                if last > idx {
                                    lifetime.crosses_call = true;
                                }
                            }
                        }
                    }
                    
                    for arg in args {
                        self.record_use(arg, idx);
                    }
                }
                Instruction::Return(val) => {
                    if let Some(v) = val {
                        self.record_use(v, idx);
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Record a use of a value
    fn record_use(&mut self, value: &Value, at_index: usize) {
        if let Value::Temp(id) = value {
            if let Some(lifetime) = self.lifetimes.get_mut(id) {
                lifetime.use_points.push(at_index);
                lifetime.last_use = Some(at_index);
            }
        }
    }
    
    /// Take accumulated instructions
    pub fn take_instructions(&mut self) -> Vec<AsmInst> {
        std::mem::take(&mut self.instructions)
    }
    
    /// Get spill count for metrics
    pub fn get_spill_count(&self) -> usize {
        self.value_to_slot.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sethi_ullman_ordering() {
        let rpm = RegisterPressureManager::new(0);
        
        // Test that complex expressions get proper ordering
        let const1 = Value::Constant(10);
        let const2 = Value::Constant(20);
        
        let need1 = rpm.calculate_need(&const1);
        let need2 = rpm.calculate_need(&const2);
        
        assert_eq!(need1.count, 1);
        assert_eq!(need2.count, 1);
        
        let (binary_need, swap) = rpm.calculate_binary_need(&const1, &const2);
        assert_eq!(binary_need.count, 2); // Both need 1, so total is 1+1
        assert!(!swap); // No need to swap equal needs
    }
    
    #[test]
    fn test_lru_spilling() {
        let mut rpm = RegisterPressureManager::new(5);
        rpm.init();
        
        // Allocate all 7 registers
        for i in 0..7 {
            let reg = rpm.get_register(format!("val{}", i));
            assert!(matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
        }
        
        // Next allocation should spill LRU (val0)
        let reg = rpm.get_register("val7".to_string());
        assert!(matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
        
        // Check that spill happened
        let insts = rpm.take_instructions();
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(_, Reg::R13, _))));
    }
    
    #[test]
    fn test_reload() {
        let mut rpm = RegisterPressureManager::new(0);
        rpm.init();
        
        // Allocate and force spill
        for i in 0..8 {
            rpm.get_register(format!("val{}", i));
        }
        
        // Reload a spilled value
        let reg = rpm.reload_value("val0".to_string());
        assert!(matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
        
        // Check that reload happened
        let insts = rpm.take_instructions();
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::R13, _))));
    }
}