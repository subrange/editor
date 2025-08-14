//! Register Pressure Manager for V2 Backend
//! 
//! Based on the formalized register spilling algorithm with Sethi-Ullman
//! expression evaluation order to minimize register pressure.

use std::collections::{BTreeMap, VecDeque};
use rcc_codegen::{AsmInst, Reg};
use rcc_common::TempId;
use rcc_frontend::ir::{BasicBlock, Instruction, Value, IrBinaryOp};
use super::allocator::{RegAllocV2, ALLOCATABLE_REGISTERS};
use super::bank::BankInfo;
use log::{debug, trace};

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
    /// Base allocator for low-level operations (internal only)
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
    
    /// Map from alloca temp names to their FP offsets
    /// This allows recomputing alloca addresses when they're not in registers
    alloca_offsets: BTreeMap<String, i16>,
}

impl RegisterPressureManager {
    pub fn new(local_count: i16) -> Self {
        // Initialize free_list with allocatable registers from centralized constant
        let mut free_list = VecDeque::new();
        for &reg in ALLOCATABLE_REGISTERS.iter() {
            free_list.push_back(reg);
        }
        
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
            alloca_offsets: BTreeMap::new(),
        }
    }
    
    /// Initialize the allocator (must be called before use)
    pub fn init(&mut self) {
        debug!("Initializing RegisterPressureManager with {} locals", self.local_count);
        self.allocator.init_stack_bank();
        self.allocator.set_spill_base(self.local_count);
        // Take any initialization instructions from the allocator
        self.instructions.extend(self.allocator.take_instructions());
        trace!("  R13 initialized, spill base set to FP+{}", self.local_count);
    }
    
    /// Get the number of local slots this manager was initialized with
    pub fn local_count(&self) -> i16 {
        self.local_count
    }

    /// Highest FP-relative cell index touched by any spill: FP + local_count + (next_spill_slot-1).
    /// None when nothing has been spilled yet.
    pub fn max_spill_fp_offset(&self) -> Option<i16> {
        if self.next_spill_slot > 0 {
            Some(self.local_count + self.next_spill_slot - 1)
        } else {
            None
        }
    }

    /// Top of caller-owned frame (locals + all spill slots).
    /// Use FP + this value + 1 as the minimum SP before any CALL.
    pub fn frame_high_water(&self) -> i16 {
        self.local_count + self.next_spill_slot
    }
    
    /// Initialize SB for stack operations (called automatically when needed)
    fn ensure_sb_initialized(&mut self) {
        if !self.allocator.sb_initialized {
            self.allocator.init_stack_bank();
            // The init_stack_bank generates instructions, take them
            self.instructions.extend(self.allocator.take_instructions());
        }
    }
    
    /// Set pointer bank information
    pub fn set_pointer_bank(&mut self, ptr_value: String, bank: BankInfo) {
        trace!("Setting bank info for '{}': {:?}", ptr_value, bank);
        self.allocator.set_pointer_bank(ptr_value, bank);
    }
    
    /// Get pointer bank information
    pub fn get_pointer_bank(&self, ptr_value: &str) -> Option<BankInfo> {
        self.allocator.pointer_banks.get(ptr_value).cloned()
    }
    
    /// Get bank register for a pointer (internal use)
    pub fn get_bank_register(&mut self, ptr_value: &str) -> Reg {
        self.ensure_sb_initialized();
        self.allocator.get_bank_register(ptr_value)
    }
    
    /// Load parameter from stack
    pub fn load_parameter(&mut self, param_idx: usize) -> Reg {
        self.ensure_sb_initialized();
        let reg = self.allocator.load_parameter(param_idx);
        self.instructions.extend(self.allocator.take_instructions());
        reg
    }
    
    /// Check if SB is initialized (internal use)
    pub(super) fn is_sb_initialized(&self) -> bool {
        self.allocator.sb_initialized
    }
    
    /// Get the list of callee-saved registers that have been used
    /// These need to be saved in prologue and restored in epilogue
    pub(super) fn get_used_callee_saved(&self) -> Vec<Reg> {
        self.allocator.get_used_callee_saved()
    }
    
    /// Calculate register need for an expression (Sethi-Ullman)
    pub fn calculate_need(&self, value: &Value) -> RegisterNeed {
        let need = match value {
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
            Value::ConstantArray(_) => {
                // Constant arrays are initialized separately, not loaded into registers
                RegisterNeed { count: 0, is_leaf: true }
            }
            Value::Undef => {
                RegisterNeed { count: 0, is_leaf: true }
            }
        };
        trace!("calculate_need({:?}) = {:?}", value, need);
        need
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
        
        trace!("calculate_binary_need: left={:?}, right={:?}, total={:?}, swap={}", 
               left_need, right_need, total_need, swap);
        
        (total_need, swap)
    }
    
    /// Get a register using LRU spilling
    pub fn get_register(&mut self, for_value: String) -> Reg {
        trace!("get_register('{}'), LRU queue: {:?}", for_value, self.lru_queue);
        
        // Check if already in a register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &for_value) {
            // Move to back of LRU (most recently used)
            if let Some(pos) = self.lru_queue.iter().position(|&r| r == reg) {
                self.lru_queue.remove(pos);
                self.lru_queue.push_back(reg);
                trace!("  '{}' already in {:?}, moved to MRU position", for_value, reg);
            }
            return reg;
        }
        
        // Check if this is an alloca that needs to be recomputed
        if let Some(&fp_offset) = self.alloca_offsets.get(&for_value) {
            debug!("  '{}' is an alloca at FP+{}, recomputing address", for_value, fp_offset);
            
            // Get a register for the recomputed address
            let reg = if let Some(free_reg) = self.free_list.pop_front() {
                free_reg
            } else {
                // Need to spill to make room
                let victim = self.lru_queue.pop_front().expect("No registers to spill!");
                debug!("  Spilling {:?} to make room for alloca recomputation", victim);
                self.spill_register(victim);
                victim
            };
            
            // Generate instructions to recompute FP + offset
            self.instructions.push(AsmInst::Comment(format!("Recompute alloca {} at FP+{}", for_value, fp_offset)));
            self.instructions.push(AsmInst::Add(reg, Reg::Fp, Reg::R0));
            if fp_offset != 0 {
                self.instructions.push(AsmInst::AddI(reg, reg, fp_offset));
            }
            
            // Update tracking
            self.reg_contents.insert(reg, for_value.clone());
            self.lru_queue.push_back(reg);
            
            debug!("  Recomputed alloca '{}' into {:?}", for_value, reg);
            return reg;
        }
        
        // Check if this value was previously spilled and needs reloading
        if let Some(&slot) = self.value_to_slot.get(&for_value) {
            debug!("  '{}' was spilled to slot {}, reloading", for_value, slot);
            
            // Get a register for the reload (might trigger another spill)
            let reg = if let Some(free_reg) = self.free_list.pop_front() {
                free_reg
            } else {
                // Need to spill to make room
                let victim = self.lru_queue.pop_front().expect("No registers to spill!");
                debug!("  Spilling {:?} to make room for reload", victim);
                self.spill_register(victim);
                victim
            };
            
            // Ensure R13 is initialized before any stack operation
            self.ensure_sb_initialized();
            
            // Generate reload instructions
            self.instructions.push(AsmInst::Comment(format!("Reload {} from slot {}", for_value, slot)));
            self.instructions.push(AsmInst::Add(Reg::Sc, Reg::Fp, Reg::R0));
            self.instructions.push(AsmInst::AddI(Reg::Sc, Reg::Sc, self.local_count + slot));
            self.instructions.push(AsmInst::Load(reg, Reg::Sb, Reg::Sc));
            
            // Update tracking
            self.reg_contents.insert(reg, for_value.clone());
            self.lru_queue.push_back(reg);
            
            debug!("  Reloaded '{}' into {:?} from slot {}", for_value, reg, slot);
            return reg;
        }
        
        // Try to get a free register
        if let Some(reg) = self.free_list.pop_front() {
            self.reg_contents.insert(reg, for_value.clone());
            self.lru_queue.push_back(reg);
            debug!("Allocated {:?} for '{}' (was free)", reg, for_value);
            trace!("  Free list now: {:?}", self.free_list);
            return reg;
        }
        
        // Need to spill - pick LRU victim
        debug!("No free registers, need to spill for '{}'", for_value);
        let victim = self.lru_queue.pop_front()
            .expect("No registers to spill!");
        
        debug!("Spilling LRU victim {:?} to make room", victim);
        self.spill_register(victim);
        
        // Now victim is free
        self.reg_contents.insert(victim, for_value.clone());
        self.lru_queue.push_back(victim);
        debug!("Reused {:?} for '{}' after spilling", victim, for_value);
        victim
    }
    
    /// Spill a register to stack
    fn spill_register(&mut self, reg: Reg) {
        if let Some(value) = self.reg_contents.get(&reg).cloned() {
            trace!("spill_register({:?}) containing '{}'", reg, value);
            
            self.ensure_sb_initialized();
            
            // Get or allocate spill slot
            // IMPORTANT: Check if the VALUE already has a slot, not the register
            let slot = self.value_to_slot.get(&value).copied()
                .unwrap_or_else(|| {
                    let s = self.next_spill_slot;
                    self.next_spill_slot += 1;
                    self.reg_to_slot.insert(reg, s);
                    self.value_to_slot.insert(value.clone(), s);
                    trace!("  Allocated new spill slot {} for '{}'", s, value);
                    s
                });
            
            // Generate spill instructions using Sc scratch
            self.instructions.push(AsmInst::Comment(format!("Spill {value} to slot {slot}")));
            self.instructions.push(AsmInst::Add(Reg::Sc, Reg::Fp, Reg::R0));
            self.instructions.push(AsmInst::AddI(Reg::Sc, Reg::Sc, self.local_count + slot));
            self.instructions.push(AsmInst::Store(reg, Reg::Sb, Reg::Sc));
            
            debug!("Spilled '{}' from {:?} to slot {} (FP+{})", value, reg, slot, self.local_count + slot);
        } else {
            trace!("spill_register({:?}) - register was empty", reg);
        }
        
        self.reg_contents.remove(&reg);
    }
    
    /// Reload a value from spill slot
    pub fn reload_value(&mut self, value: String) -> Reg {
        trace!("reload_value('{}'), value_to_slot: {:?}", value, self.value_to_slot);
        
        // Check if already in register
        if let Some((&reg, _)) = self.reg_contents.iter().find(|(_, v)| *v == &value) {
            trace!("  '{}' already in {:?}, no reload needed", value, reg);
            return reg;
        }
        
        // Check if spilled
        if let Some(&slot) = self.value_to_slot.get(&value) {
            debug!("Reloading '{}' from spill slot {}", value, slot);
            let reg = self.get_register(value.clone());
            
            // Ensure R13 is initialized before any stack operation
            self.ensure_sb_initialized();
            
            // Generate reload instructions
            self.instructions.push(AsmInst::Comment(format!("Reload {value} from slot {slot}")));
            self.instructions.push(AsmInst::Add(Reg::Sc, Reg::Fp, Reg::R0));
            self.instructions.push(AsmInst::AddI(Reg::Sc, Reg::Sc, self.local_count + slot));
            self.instructions.push(AsmInst::Load(reg, Reg::Sb, Reg::Sc));
            
            debug!("Reloaded '{}' into {:?} from slot {} (FP+{})", value, reg, slot, self.local_count + slot);
            return reg;
        }
        
        // Not spilled, allocate new
        trace!("  '{}' not spilled, allocating new register", value);
        self.get_register(value)
    }
    
    /// Free a register
    pub fn free_register(&mut self, reg: Reg) {
        trace!("free_register({:?})", reg);
        if let Some(value) = self.reg_contents.get(&reg) {
            debug!("Freeing {:?} containing '{}'" , reg, value);
        }
        if let Some(pos) = self.lru_queue.iter().position(|&r| r == reg) {
            self.lru_queue.remove(pos);
        }
        self.reg_contents.remove(&reg);
        if !self.free_list.contains(&reg) {
            self.free_list.push_back(reg);
            trace!("  Added {:?} back to free list", reg);
        }
    }
    
    /// Bind a value to a specific register (e.g., after a function call)
    /// This is used when we know a value is in a specific register (like Rv0 after a call)
    pub fn bind_value_to_register(&mut self, value: String, reg: Reg) {
        debug!("Binding '{}' to {:?}", value, reg);
        
        // First, if the register contains something else, we need to handle that
        if let Some(old_value) = self.reg_contents.get(&reg).cloned() {
            if old_value != value {
                trace!("  {:?} previously contained '{}', marking as free", reg, old_value);
                // The old value is no longer in a register
            }
        }
        
        // Remove register from free list if it's there
        if let Some(pos) = self.free_list.iter().position(|&r| r == reg) {
            self.free_list.remove(pos);
            trace!("  Removed {:?} from free list", reg);
        }
        
        // Update LRU queue
        if let Some(pos) = self.lru_queue.iter().position(|&r| r == reg) {
            self.lru_queue.remove(pos);
        }
        self.lru_queue.push_back(reg); // Mark as most recently used
        
        // Update contents
        self.reg_contents.insert(reg, value);
        trace!("  {:?} now contains bound value", reg);
    }
    
    /// Spill all registers (e.g., before a call)
    pub fn spill_all(&mut self) {
        debug!("Spilling all registers (e.g., for function call)");
        trace!("  Current LRU queue: {:?}", self.lru_queue);
        trace!("  Current contents: {:?}", self.reg_contents);
        let regs_to_spill: Vec<Reg> = self.lru_queue.iter().copied().collect();
        for reg in regs_to_spill {
            self.spill_register(reg);
            self.free_register(reg);
        }
        debug!("All registers spilled, {} slots used", self.next_spill_slot);
    }
    
    /// Register an alloca temp with its FP offset
    pub fn register_alloca(&mut self, temp_name: String, fp_offset: i16) {
        debug!("Registering alloca '{}' at FP+{}", temp_name, fp_offset);
        self.alloca_offsets.insert(temp_name, fp_offset);
    }
    
    /// Get register for a Value
    pub fn get_value_register(&mut self, value: &Value) -> Reg {
        match value {
            Value::Temp(id) => {
                self.get_register(format!("t{id}"))
            }
            Value::Constant(val) => {
                let reg = self.get_register(format!("const_{val}"));
                self.instructions.push(AsmInst::Li(reg, *val as i16));
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
        debug!("emit_binary_op({:?}, lhs={:?}, rhs={:?}) -> t{}", op, lhs, rhs, result_temp);
        let mut insts = Vec::new();
        
        // Calculate needs and determine evaluation order
        let (_, should_swap) = self.calculate_binary_need(lhs, rhs);
        
        let (first, second) = if should_swap {
            trace!("  Swapping operands for better register usage");
            (rhs, lhs)
        } else {
            (lhs, rhs)
        };
        
        // Evaluate in optimal order
        let first_reg = self.get_value_register(first);
        let second_reg = self.get_value_register(second);
        trace!("  Operands in {:?} and {:?}", first_reg, second_reg);
        
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