//! Module to Assembly Lowering
//! 
//! This module implements the lowering from the full IR (Module/Function/Instruction)
//! to Ripple assembly instructions.

use crate::ir::{Module, Function, BasicBlock, Instruction, Value, IrType, IrBinaryOp, GlobalVariable};
use crate::simple_regalloc::SimpleRegAlloc;
use rcc_codegen::{AsmInst, Reg};
use rcc_common::CompilerError;
use std::collections::{HashMap, HashSet};

/// Module to assembly lowering context
pub struct ModuleLowerer {
    /// Assembly instructions being generated
    instructions: Vec<AsmInst>,
    
    /// Centralized register allocator
    reg_alloc: SimpleRegAlloc,
    
    /// Free list of available registers (R3-R11) - kept for compatibility
    free_regs: Vec<Reg>,
    
    /// Map from register to the temp it contains (for spilling decisions) - kept for compatibility  
    reg_contents: HashMap<Reg, String>,
    
    /// Mapping from IR values to either registers or spill slots
    value_locations: HashMap<String, Location>,
    
    /// Global variable addresses
    global_addresses: HashMap<String, u16>,
    
    /// Next available global memory address
    next_global_addr: u16,
    
    /// Current function being lowered
    current_function: Option<String>,
    
    /// Whether current function needs a frame
    needs_frame: bool,
    
    /// Label counter for generating unique labels
    label_counter: u32,
    
    /// Stack offset for local allocations (relative to frame pointer)
    /// This includes both user variables and spill slots
    local_stack_offset: i16,
    
    /// Map from temp IDs to stack offsets (for local variables from alloca)
    local_offsets: HashMap<u32, i16>,
    
    /// Set of temp IDs that point to stack memory (from GetElementPtr on stack arrays)
    stack_pointer_temps: HashSet<u32>,
    
    /// Next available spill slot offset
    next_spill_offset: i16,
}

/// Location of a value - either in a register or spilled to stack
#[derive(Debug, Clone, Copy)]
enum Location {
    Register(Reg),
    Spilled(i16), // Offset from FP
}

impl ModuleLowerer {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            reg_alloc: SimpleRegAlloc::new(),
            // R3-R11 are allocatable, R12 reserved for spill address calculations
            free_regs: vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5, Reg::R4, Reg::R3],
            reg_contents: HashMap::new(),
            value_locations: HashMap::new(),
            global_addresses: HashMap::new(),
            next_global_addr: 100, // Start globals at address 100
            current_function: None,
            needs_frame: false,
            label_counter: 0,
            local_stack_offset: 0,
            local_offsets: HashMap::new(),
            stack_pointer_temps: HashSet::new(),
            next_spill_offset: 0,
        }
    }
    
    /// Lower a module to assembly
    pub fn lower(&mut self, module: Module) -> Result<Vec<AsmInst>, CompilerError> {
        // Generate global initialization function
        if !module.globals.is_empty() {
            self.instructions.push(AsmInst::Label("_init_globals".to_string()));
        }
        
        // Generate globals and their initialization
        for global in &module.globals {
            self.lower_global(global)?;
        }
        
        // Return from _init_globals if we have globals
        if !module.globals.is_empty() {
            self.instructions.push(AsmInst::Ret);
        }
        
        // Generate functions
        for function in &module.functions {
            self.lower_function(function)?;
        }
        
        Ok(self.instructions.clone())
    }
    
    /// Lower a global variable
    fn lower_global(&mut self, global: &GlobalVariable) -> Result<(), CompilerError> {
        // Check if this is a string literal (name starts with __str_)
        let is_string = global.name.starts_with("__str_");
        
        // Allocate address for global
        let address = self.next_global_addr;
        self.global_addresses.insert(global.name.clone(), address);
        
        // Calculate size in words (16-bit)
        let size = match &global.var_type {
            IrType::I8 | IrType::I16 => 1,
            IrType::I32 => 2, // 32-bit takes 2 words
            IrType::Ptr(_) => 1, // Pointers are 16-bit
            IrType::Array { size, .. } if is_string => {
                // For strings, allocate space for all characters
                (*size as u16 + 1) / 2 // Round up for 16-bit words
            }
            _ => 1, // Default to 1 word
        };
        
        self.next_global_addr += size;
        
        // For string literals, decode the string from the name
        if is_string {
            // Parse the hex-encoded string from the name
            // Format: __str_ID_HEXDATA
            if let Some(hex_part) = global.name.split('_').last() {
                let mut addr = address;
                let mut chars = Vec::new();
                
                // Decode hex string
                for i in (0..hex_part.len()).step_by(2) {
                    if let Ok(byte) = u8::from_str_radix(&hex_part[i..i+2], 16) {
                        chars.push(byte);
                    }
                }
                chars.push(0); // Add null terminator
                
                // Create a safe string representation for the comment
                let safe_str: String = chars[..chars.len()-1].iter()
                    .map(|&c| match c {
                        b'\n' => "\\n".to_string(),
                        b'\t' => "\\t".to_string(),
                        b'\r' => "\\r".to_string(),
                        b'\\' => "\\\\".to_string(),
                        c if c.is_ascii_graphic() || c == b' ' => (c as char).to_string(),
                        c => format!("\\x{:02x}", c),
                    })
                    .collect();
                
                self.instructions.push(AsmInst::Comment(format!("String literal {} at address {}", 
                    safe_str, address)));
                
                // Store each character
                for byte in chars {
                    self.instructions.push(AsmInst::LI(Reg::R3, byte as i16));
                    self.instructions.push(AsmInst::LI(Reg::R4, addr as i16));
                    self.instructions.push(AsmInst::Store(Reg::R3, Reg::R0, Reg::R4));
                    addr += 1;
                }
            }
        } else {
            self.instructions.push(AsmInst::Comment(format!("Global variable: {} at address {}", 
                global.name, address)));
            
            // Generate initialization code if there's an initializer
            if let Some(init_value) = &global.initializer {
                match init_value {
                    Value::Constant(val) => {
                        // Load value into register and store at address
                        self.instructions.push(AsmInst::LI(Reg::R3, *val as i16));
                        self.instructions.push(AsmInst::LI(Reg::R4, address as i16));
                        self.instructions.push(AsmInst::Store(Reg::R3, Reg::R0, Reg::R4));
                    }
                    _ => {
                        // Other initializer types not yet supported
                        self.instructions.push(AsmInst::Comment(
                            format!("Unsupported initializer for {}", global.name)));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Lower a function to assembly
    fn lower_function(&mut self, function: &Function) -> Result<(), CompilerError> {
        self.current_function = Some(function.name.clone());
        self.value_locations.clear();
        self.reg_contents.clear();
        self.free_regs = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5, Reg::R4, Reg::R3];
        self.reg_alloc.reset(); // Reset the centralized allocator
        self.needs_frame = false;
        self.local_stack_offset = 0; // Reset local stack offset
        self.local_offsets.clear(); // Clear local variable offsets
        self.next_spill_offset = 0; // Reset spill offset
        
        // Function label
        self.instructions.push(AsmInst::Comment(format!("Function: {}", function.name)));
        self.instructions.push(AsmInst::Label(function.name.clone()));
        
        // Map function parameters to their input registers
        // Parameters arrive in R3-R8
        for (i, (param_id, _param_type)) in function.parameters.iter().enumerate() {
            if i < 6 {
                let param_reg = match i {
                    0 => Reg::R3,
                    1 => Reg::R4,
                    2 => Reg::R5,
                    3 => Reg::R6,
                    4 => Reg::R7,
                    5 => Reg::R8,
                    _ => unreachable!(),
                };
                // Map parameter temp ID to its register location
                self.value_locations.insert(format!("t{}", param_id), Location::Register(param_reg));
            } else {
                // Parameters beyond the 6th would need to be passed on the stack
                // For now, we don't support more than 6 parameters
                unimplemented!("More than 6 parameters not yet supported");
            }
        }
        
        // First pass: scan function to determine if we need a frame
        // We need a frame if:
        // 1. Function has local variables (alloca)
        // 2. Function makes calls (need to save RA)
        // 3. We might spill registers
        let has_calls = self.function_has_calls(function);
        let has_allocas = function.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| matches!(inst, Instruction::Alloca { .. }))
        });
        
        // For now, always create a frame if we have calls or allocas
        // Later we can optimize this based on actual register pressure
        if has_calls || has_allocas {
            self.needs_frame = true;
        }
        
        // Generate prologue if needed
        if self.needs_frame {
            self.generate_prologue();
        }
        
        // Lower basic blocks
        for block in &function.blocks {
            self.lower_basic_block(block)?;
        }
        
        // Note: Epilogue is generated by return instructions
        
        self.current_function = None;
        Ok(())
    }
    
    /// Check if function has any call instructions (excluding putchar which is inlined)
    fn function_has_calls(&self, function: &Function) -> bool {
        function.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| {
                match inst {
                    Instruction::Call { function: func, .. } => {
                        // putchar is special-cased as MMIO, not a real call
                        if let Value::Function(name) = func {
                            name != "putchar"
                        } else {
                            true
                        }
                    }
                    _ => false
                }
            })
        })
    }
    
    /// Lower a basic block
    fn lower_basic_block(&mut self, block: &BasicBlock) -> Result<(), CompilerError> {
        // Add label for block if not the first one
        if block.id != 0 {
            self.instructions.push(AsmInst::Label(format!("L{}", block.id)));
        }
        
        for (idx, instruction) in block.instructions.iter().enumerate() {
            self.lower_instruction(instruction)?;
            
            // Only free registers at statement boundaries
            // Statement boundaries are between different high-level statements,
            // not between every IR instruction
            // For now, we free after stores, calls, and branches
            match instruction {
                Instruction::Store { .. } | 
                Instruction::Call { .. } |
                Instruction::Branch(_) |
                Instruction::BranchCond { .. } |
                Instruction::Return { .. } => {
                    // These mark the end of a statement - free all temps
                    self.free_all_regs();
                }
                _ => {
                    // Keep registers allocated for expression evaluation
                }
            }
        }
        
        Ok(())
    }
    
    /// Lower a single instruction
    fn lower_instruction(&mut self, instruction: &Instruction) -> Result<(), CompilerError> {
        match instruction {
            Instruction::Alloca { result, alloc_type, count, .. } => {
                // Stack allocation - allocate space and compute address
                let base_size = self.get_type_size_in_words(alloc_type);
                
                // If count is provided, multiply size by count (for arrays)
                let total_size = match count {
                    Some(Value::Constant(n)) => base_size * (*n as u64),
                    _ => base_size,
                };
                
                // Allocate space on stack by incrementing offset
                // Note: For arrays, we allocate starting position, not ending
                let start_offset = self.local_stack_offset + 1; // Start after current position
                self.local_stack_offset += total_size as i16;
                let offset = start_offset;
                
                // Store the offset for this temp
                self.local_offsets.insert(*result, offset);
                
                // Allocate register for the address
                let result_key = format!("t{}", result);
                let addr_reg = self.get_reg(result_key.clone());
                self.value_locations.insert(result_key, Location::Register(addr_reg));
                
                self.instructions.push(AsmInst::Comment(format!("Alloca for t{} at FP+{}", result, offset)));
                
                // Address = R15 (FP) + offset
                if offset > 0 {
                    self.instructions.push(AsmInst::AddI(addr_reg, Reg::R15, offset));
                } else {
                    self.instructions.push(AsmInst::Add(addr_reg, Reg::R15, Reg::R0));
                }
            }
            
            Instruction::Store { value, ptr } => {
                // Check if ptr is a global address
                if let Value::Global(name) = ptr {
                    if let Some(&addr) = self.global_addresses.get(name) {
                        // Store to global address
                        self.instructions.push(AsmInst::Comment(format!("Store {} to @{}", 
                            self.value_to_string(value), name)));
                        let value_reg = self.get_value_register(value)?;
                        let addr_reg = self.get_reg(format!("store_addr_{}", self.label_counter));
                        self.label_counter += 1;
                        self.instructions.push(AsmInst::LI(addr_reg, addr as i16));
                        self.instructions.push(AsmInst::Store(value_reg, Reg::R0, addr_reg));
                        // Registers will be freed at statement boundary
                    } else {
                        self.instructions.push(AsmInst::Comment(format!("Store to undefined global @{}", name)));
                    }
                } else {
                    // For local variables, ptr should be a temp holding the address
                    // Get value first, then ptr to avoid conflicts
                    let value_reg = self.get_value_register(value)?;
                    let ptr_reg = self.get_value_register(ptr)?;
                    
                    self.instructions.push(AsmInst::Comment(format!("Store {} to [{}]", 
                        self.value_to_string(value), self.value_to_string(ptr))));
                    
                    // Check if this is a stack-allocated temp (from Alloca or GetElementPtr on stack)
                    // Stack addresses are FP-relative, globals are absolute < 1000
                    if let Value::Temp(tid) = ptr {
                        if self.local_offsets.contains_key(tid) || self.stack_pointer_temps.contains(tid) {
                            // This is a stack-allocated variable or pointer to stack
                            self.instructions.push(AsmInst::Store(value_reg, Reg::R13, ptr_reg));
                        } else {
                            // This is a pointer to global memory
                            self.instructions.push(AsmInst::Store(value_reg, Reg::R0, ptr_reg));
                        }
                    } else {
                        // Default to global memory bank
                        self.instructions.push(AsmInst::Store(value_reg, Reg::R0, ptr_reg));
                    }
                    // Registers will be freed at statement boundary
                }
            }
            
            Instruction::Load { result, ptr, .. } => {
                self.instructions.push(AsmInst::Comment(format!("Load from [{}] to t{}", 
                    self.value_to_string(ptr), result)));
                
                // Allocate register for result
                let result_key = format!("t{}", result);
                let dest_reg = self.get_reg(result_key.clone());
                self.value_locations.insert(result_key, Location::Register(dest_reg));
                
                // Check if ptr is a global address
                if let Value::Global(name) = ptr {
                    if let Some(&addr) = self.global_addresses.get(name) {
                        // Load from global address
                        let addr_reg = self.get_reg(format!("load_addr_{}", self.label_counter));
                        self.label_counter += 1;
                        self.instructions.push(AsmInst::LI(addr_reg, addr as i16));
                        self.instructions.push(AsmInst::Load(dest_reg, Reg::R0, addr_reg));
                        // Registers will be freed at statement boundary
                    } else {
                        // Uninitialized global
                        self.instructions.push(AsmInst::Comment(format!("Load from @{} (uninitialized)", name)));
                        self.instructions.push(AsmInst::LI(dest_reg, 0));
                    }
                } else {
                    // For local variables, ptr should be a temp holding the address
                    let ptr_reg = self.get_value_register(ptr)?;
                    
                    // Check if this is a stack-allocated temp (from Alloca or GetElementPtr on stack)
                    // Stack addresses are FP-relative, globals are absolute < 1000
                    if let Value::Temp(tid) = ptr {
                        if self.local_offsets.contains_key(tid) || self.stack_pointer_temps.contains(tid) {
                            // This is a stack-allocated variable or pointer to stack
                            self.instructions.push(AsmInst::Load(dest_reg, Reg::R13, ptr_reg));
                        } else {
                            // This is a pointer to global memory
                            self.instructions.push(AsmInst::Load(dest_reg, Reg::R0, ptr_reg));
                        }
                    } else {
                        // Default to global memory bank
                        self.instructions.push(AsmInst::Load(dest_reg, Reg::R0, ptr_reg));
                    }
                    // Registers will be freed at statement boundary
                }
            }
            
            Instruction::Binary { result, op, lhs, rhs, .. } => {
                // For binary operations, ensure operands get different registers
                // Use the centralized allocator for proper handling
                
                // Handle constants specially
                // For simple constants, we can use R3 and R4 temporarily
                let (left_reg, right_reg) = match (lhs, rhs) {
                    (Value::Constant(lval), Value::Constant(rval)) => {
                        // Both are constants - use R3 and R4 for simplicity
                        // These are not in the allocatable pool so won't conflict
                        self.instructions.push(AsmInst::LI(Reg::R3, *lval as i16));
                        self.instructions.push(AsmInst::LI(Reg::R4, *rval as i16));
                        (Reg::R3, Reg::R4)
                    }
                    (Value::Constant(val), _) => {
                        // Left is constant, right is not
                        // Use R3 for the constant
                        self.instructions.push(AsmInst::LI(Reg::R3, *val as i16));
                        let right = self.get_value_register(rhs)?;
                        (Reg::R3, right)
                    }
                    (_, Value::Constant(val)) => {
                        // Right is constant, left is not
                        let left = self.get_value_register(lhs)?;
                        // Use R4 for the constant
                        self.instructions.push(AsmInst::LI(Reg::R4, *val as i16));
                        (left, Reg::R4)
                    }
                    _ => {
                        // Neither is constant
                        let left = self.get_value_register(lhs)?;
                        let right = self.get_value_register(rhs)?;
                        (left, right)
                    }
                };
                
                // Allocate register for result
                let result_key = format!("t{}", result);
                let dest_reg = self.get_reg(result_key.clone());
                self.value_locations.insert(result_key.clone(), Location::Register(dest_reg));
                
                match op {
                    IrBinaryOp::Add => {
                        self.instructions.push(AsmInst::Add(dest_reg, left_reg, right_reg));
                    }
                    IrBinaryOp::Sub => {
                        self.instructions.push(AsmInst::Sub(dest_reg, left_reg, right_reg));
                    }
                    IrBinaryOp::Mul => {
                        self.instructions.push(AsmInst::Mul(dest_reg, left_reg, right_reg));
                    }
                    IrBinaryOp::SDiv | IrBinaryOp::UDiv => {
                        self.instructions.push(AsmInst::Div(dest_reg, left_reg, right_reg));
                    }
                    IrBinaryOp::SRem | IrBinaryOp::URem => {
                        self.instructions.push(AsmInst::Mod(dest_reg, left_reg, right_reg));
                    }
                    IrBinaryOp::Eq => {
                        // Set dest_reg to 1 if equal, 0 otherwise
                        // Strategy: result = !(left != right)
                        // Allocate temp registers using centralized allocator
                        let temp1 = self.reg_alloc.get_reg(format!("eq_temp1_{}", self.label_counter));
                        let temp2 = self.reg_alloc.get_reg(format!("eq_temp2_{}", self.label_counter));
                        self.label_counter += 1;
                        self.instructions.append(&mut self.reg_alloc.take_instructions());
                        
                        self.instructions.push(AsmInst::Sltu(temp1, left_reg, right_reg)); // 1 if left < right
                        self.instructions.push(AsmInst::Sltu(temp2, right_reg, left_reg)); // 1 if right < left
                        self.instructions.push(AsmInst::Or(dest_reg, temp1, temp2)); // 1 if not equal
                        
                        // Free temp registers
                        self.reg_alloc.free_reg(temp1);
                        self.reg_alloc.free_reg(temp2);
                        
                        // Now invert the result: dest = 1 - dest
                        let temp3 = self.reg_alloc.get_reg(format!("eq_temp3_{}", self.label_counter));
                        self.label_counter += 1;
                        self.instructions.append(&mut self.reg_alloc.take_instructions());
                        self.instructions.push(AsmInst::LI(temp3, 1));
                        self.instructions.push(AsmInst::Sub(dest_reg, temp3, dest_reg));
                        self.reg_alloc.free_reg(temp3);
                    }
                    IrBinaryOp::Ne => {
                        // Set dest_reg to 1 if not equal, 0 otherwise
                        // Allocate temp registers using centralized allocator
                        let temp1 = self.reg_alloc.get_reg(format!("ne_temp1_{}", self.label_counter));
                        let temp2 = self.reg_alloc.get_reg(format!("ne_temp2_{}", self.label_counter));
                        self.label_counter += 1;
                        self.instructions.append(&mut self.reg_alloc.take_instructions());
                        
                        self.instructions.push(AsmInst::Sltu(temp1, left_reg, right_reg)); // 1 if left < right
                        self.instructions.push(AsmInst::Sltu(temp2, right_reg, left_reg)); // 1 if right < left
                        self.instructions.push(AsmInst::Or(dest_reg, temp1, temp2)); // 1 if not equal
                        
                        // Free temp registers
                        self.reg_alloc.free_reg(temp1);
                        self.reg_alloc.free_reg(temp2);
                    }
                    IrBinaryOp::Slt => {
                        // Use SLTU instead of SLT since SLT might be buggy
                        self.instructions.push(AsmInst::Sltu(dest_reg, left_reg, right_reg));
                    }
                    IrBinaryOp::Sle => {
                        // a <= b is !(b < a)
                        self.instructions.push(AsmInst::Sltu(dest_reg, right_reg, left_reg));
                        let temp = self.reg_alloc.get_reg(format!("sle_temp_{}", self.label_counter));
                        self.label_counter += 1;
                        self.instructions.append(&mut self.reg_alloc.take_instructions());
                        self.instructions.push(AsmInst::LI(temp, 1));
                        self.instructions.push(AsmInst::Sub(dest_reg, temp, dest_reg)); // 1 - result
                        self.reg_alloc.free_reg(temp);
                    }
                    IrBinaryOp::Sgt => {
                        self.instructions.push(AsmInst::Sltu(dest_reg, right_reg, left_reg));
                    }
                    IrBinaryOp::Sge => {
                        // a >= b is !(a < b)
                        self.instructions.push(AsmInst::Sltu(dest_reg, left_reg, right_reg));
                        let temp = self.reg_alloc.get_reg(format!("sge_temp_{}", self.label_counter));
                        self.label_counter += 1;
                        self.instructions.append(&mut self.reg_alloc.take_instructions());
                        self.instructions.push(AsmInst::LI(temp, 1));
                        self.instructions.push(AsmInst::Sub(dest_reg, temp, dest_reg)); // 1 - result
                        self.reg_alloc.free_reg(temp);
                    }
                    IrBinaryOp::Ult | IrBinaryOp::Ule | IrBinaryOp::Ugt | IrBinaryOp::Uge => {
                        // TODO: Implement unsigned comparisons
                        self.instructions.push(AsmInst::Comment(format!("TODO: Unsigned comparison {:?}", op)));
                        self.instructions.push(AsmInst::LI(dest_reg, 0));
                    }
                    _ => {
                        self.instructions.push(AsmInst::Comment(format!("Unsupported binary op: {:?}", op)));
                    }
                }
                
                // No need to free R3/R4 as they're not in the allocatable pool
                // They're scratch registers for immediate use
            }
            
            Instruction::Call { result, function, args, .. } => {
                // Extract function name from Value
                let func_name = match function {
                    Value::Function(name) => name.clone(),
                    Value::Global(name) => name.clone(),
                    _ => "unknown".to_string(),
                };
                
                // For putchar, directly handle it as a store to MMIO
                if func_name == "putchar" && args.len() == 1 {
                    let arg_reg = self.get_value_register(&args[0])?;
                    self.instructions.push(AsmInst::Store(arg_reg, Reg::R0, Reg::R0));
                } else {
                    // General function call - set up arguments first
                    // Using calling convention: R3-R8 for arguments
                    
                    // Collect argument registers and their destinations
                    let mut arg_regs = Vec::new();
                    for (i, arg) in args.iter().enumerate() {
                        if i < 6 {  // Max 6 register arguments
                            let arg_reg = self.get_value_register(arg)?;
                            let param_reg = match i {
                                0 => Reg::R3,
                                1 => Reg::R4,
                                2 => Reg::R5,
                                3 => Reg::R6,
                                4 => Reg::R7,
                                5 => Reg::R8,
                                _ => unreachable!(),
                            };
                            arg_regs.push((arg_reg, param_reg));
                        }
                    }
                    
                    // Move arguments to parameter registers
                    // Handle potential conflicts by using R9 as temporary
                    let mut moved = vec![false; arg_regs.len()];
                    
                    // First, move any that don't conflict
                    for i in 0..arg_regs.len() {
                        let (src, dst) = arg_regs[i];
                        if src == dst {
                            moved[i] = true;
                        } else {
                            // Check if dst is needed as a source for an unmoved arg
                            let dst_needed = arg_regs.iter().enumerate()
                                .any(|(j, (s, _))| !moved[j] && j != i && *s == dst);
                            
                            if !dst_needed {
                                self.instructions.push(AsmInst::Add(dst, src, Reg::R0));
                                moved[i] = true;
                            }
                        }
                    }
                    
                    // Now handle any remaining moves (these form cycles)
                    // Simple approach: use R9 to save conflicting values
                    for i in 0..arg_regs.len() {
                        if !moved[i] {
                            let (src, dst) = arg_regs[i];
                            
                            // Check if any other unmoved arg needs our dst as src
                            let mut conflict_idx = None;
                            for j in 0..arg_regs.len() {
                                if !moved[j] && j != i {
                                    let (src2, _) = arg_regs[j];
                                    if src2 == dst {
                                        conflict_idx = Some(j);
                                        break;
                                    }
                                }
                            }
                            
                            if let Some(j) = conflict_idx {
                                // Save the conflicting source to R9 first
                                let (_, dst2) = arg_regs[j];
                                self.instructions.push(AsmInst::Add(Reg::R9, dst, Reg::R0));
                                // Now we can move src to dst
                                self.instructions.push(AsmInst::Add(dst, src, Reg::R0));
                                moved[i] = true;
                                // And move R9 to dst2
                                self.instructions.push(AsmInst::Add(dst2, Reg::R9, Reg::R0));
                                moved[j] = true;
                            } else {
                                // No conflict, just move
                                self.instructions.push(AsmInst::Add(dst, src, Reg::R0));
                                moved[i] = true;
                            }
                        }
                    }
                    
                    self.instructions.push(AsmInst::Call(func_name));
                    
                    if let Some(dest) = result {
                        // Result is in R3 by convention
                        // Allocate register for result
                        let result_key = format!("t{}", dest);
                        let dest_reg = self.get_reg(result_key.clone());
                        self.value_locations.insert(result_key, Location::Register(dest_reg));
                        
                        // Move result from R3 to allocated register if different
                        if dest_reg != Reg::R3 {
                            self.instructions.push(AsmInst::Add(dest_reg, Reg::R3, Reg::R0));
                        }
                    }
                }
            }
            
            Instruction::Return(value) => {
                if let Some(val) = value {
                    let val_reg = self.get_value_register(val)?;
                    // Move return value to return register (R3 by convention)
                    if val_reg != Reg::R3 {
                        self.instructions.push(AsmInst::Add(Reg::R3, val_reg, Reg::R0));
                    }
                }
                
                // Generate epilogue if this function has a frame
                if self.needs_frame {
                    self.generate_epilogue();
                }
                
                self.instructions.push(AsmInst::Ret);
            }
            
            Instruction::Branch(target) => {
                // Unconditional jump to label
                // Use BEQ R0, R0, label (always true) as unconditional jump
                self.instructions.push(AsmInst::Beq(Reg::R0, Reg::R0, format!("L{}", target)));
            }
            
            Instruction::BranchCond { condition, true_label, false_label } => {
                // Get the condition value in a register
                let cond_reg = self.get_value_register(condition)?;
                
                // Branch if condition is non-zero (true)
                // BNE cond_reg, R0, true_label
                self.instructions.push(AsmInst::Bne(cond_reg, Reg::R0, format!("L{}", true_label)));
                
                // If we fall through (condition was zero/false), jump to false label
                // Use BEQ R0, R0, false_label (always true) as unconditional jump
                self.instructions.push(AsmInst::Beq(Reg::R0, Reg::R0, format!("L{}", false_label)));
            }
            
            Instruction::GetElementPtr { result, ptr, indices, .. } => {
                // Get element pointer - compute address of array element
                self.instructions.push(AsmInst::Comment(format!("GetElementPtr t{} = {} + offsets", result, self.value_to_string(ptr))));
                
                // Get base pointer
                let base_reg = self.get_value_register(ptr)?;
                
                // For now, we only support single index (1D arrays)
                if indices.len() != 1 {
                    return Err(CompilerError::codegen_error(
                        format!("Multi-dimensional arrays not yet supported"),
                        rcc_common::SourceLocation::new_simple(0, 0),
                    ));
                }
                
                // Get index value
                let index_reg = self.get_value_register(&indices[0])?;
                
                // Allocate register for result
                let result_key = format!("t{}", result);
                let dest_reg = self.get_reg(result_key.clone());
                self.value_locations.insert(result_key, Location::Register(dest_reg));
                
                // Calculate address: result = base + index
                // Note: This assumes element size is 1 word. For larger types, we'd need to multiply index by element size
                self.instructions.push(AsmInst::Add(dest_reg, base_reg, index_reg));
                
                // IMPORTANT: If the base pointer was from an Alloca (stack allocation),
                // then this result also points to stack memory
                if let Value::Temp(base_tid) = ptr {
                    if self.local_offsets.contains_key(base_tid) || self.stack_pointer_temps.contains(base_tid) {
                        // Mark this result as also pointing to stack memory
                        self.stack_pointer_temps.insert(*result);
                    }
                }
            }
            
            _ => {
                self.instructions.push(AsmInst::Comment(format!("Unimplemented: {:?}", instruction)));
            }
        }
        
        Ok(())
    }
    
    /// Get register for a value
    fn get_value_register_impl(&mut self, value: &Value) -> Result<Reg, CompilerError> {
        match value {
            Value::Constant(n) => {
                // For constants, use free list first, then check what's available
                let reg = if !self.free_regs.is_empty() {
                    // Pop from free list (LIFO for better cache locality)
                    self.free_regs.pop().unwrap()
                } else {
                    // No free registers, need to spill
                    let victim = self.pick_spill_victim();
                    self.spill_register(victim);
                    self.reg_contents.remove(&victim);
                    victim
                };
                
                // Mark as containing this constant so it won't be reused until freed
                let const_key = format!("const_{}_{}", n, self.label_counter);
                self.label_counter += 1;
                self.reg_contents.insert(reg, const_key);
                
                self.instructions.push(AsmInst::LI(reg, *n as i16));
                Ok(reg)
            }
            Value::Temp(id) => {
                // Check if this is a stack-allocated variable (from alloca)
                if let Some(&offset) = self.local_offsets.get(id) {
                    // This is a stack variable - return its address
                    // Use a unique key to avoid conflicts
                    let reg = self.get_reg(format!("addr_t{}_{}", id, self.label_counter));
                    self.label_counter += 1;
                    if offset > 0 {
                        self.instructions.push(AsmInst::AddI(reg, Reg::R15, offset));
                    } else {
                        self.instructions.push(AsmInst::Add(reg, Reg::R15, Reg::R0));
                    }
                    Ok(reg)
                } else {
                    // Regular temp value
                    let key = format!("t{}", id);
                    if let Some(&loc) = self.value_locations.get(&key) {
                        // Already have a location for this temp
                        match loc {
                            Location::Register(r) => Ok(r),
                            Location::Spilled(offset) => {
                                // Reload from spill
                                let reg = self.get_reg(key.clone());
                                self.instructions.push(AsmInst::Comment(format!("Reloading {} from FP+{}", key, offset)));
                                self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, offset));
                                self.instructions.push(AsmInst::Load(reg, Reg::R13, Reg::R12));
                                // Update location
                                self.value_locations.insert(key, Location::Register(reg));
                                Ok(reg)
                            }
                        }
                    } else {
                        // This temp hasn't been seen before - shouldn't happen
                        Err(CompilerError::codegen_error(
                            format!("Undefined temp t{}", id),
                            rcc_common::SourceLocation::new_simple(0, 0),
                        ))
                    }
                }
            }
            Value::Function(name) => {
                // Function references not directly loadable
                Err(CompilerError::codegen_error(
                    format!("Cannot load function '{}' into register", name),
                    rcc_common::SourceLocation::new_simple(0, 0),
                ))
            }
            Value::Global(name) => {
                // Load global address into a register
                if let Some(&addr) = self.global_addresses.get(name) {
                    let reg = self.get_reg(format!("global_{}_{}", name, self.label_counter));
                    self.label_counter += 1;
                    self.instructions.push(AsmInst::LI(reg, addr as i16));
                    Ok(reg)
                } else {
                    Err(CompilerError::codegen_error(
                        format!("Undefined global variable '{}'", name),
                        rcc_common::SourceLocation::new_simple(0, 0),
                    ))
                }
            }
            _ => {
                Err(CompilerError::codegen_error(
                    format!("Unsupported value type: {:?}", value),
                    rcc_common::SourceLocation::new_simple(0, 0),
                ))
            }
        }
    }
    
    /// Get register for a value
    fn get_value_register(&mut self, value: &Value) -> Result<Reg, CompilerError> {
        self.get_value_register_impl(value)
    }
    
    /// Get a register, spilling if necessary - now uses centralized allocator
    fn get_reg(&mut self, for_value: String) -> Reg {
        // Use the centralized allocator
        let reg = self.reg_alloc.get_reg(for_value.clone());
        
        // Append any spill/reload instructions generated
        self.instructions.append(&mut self.reg_alloc.take_instructions());
        
        // Also track in value_locations for compatibility
        self.value_locations.insert(for_value, Location::Register(reg));
        
        reg
    }
    
    /// Spill a register to memory
    fn spill_register(&mut self, reg: Reg) {
        if let Some(old_value) = self.reg_contents.get(&reg).cloned() {
            // Don't spill temporary constants - they're single use
            if !old_value.starts_with("const_") && !old_value.starts_with("addr_") {
                if let Some(Location::Register(r)) = self.value_locations.get(&old_value) {
                    if *r == reg {
                        // Spill this value
                        let spill_offset = self.next_spill_offset;
                        self.next_spill_offset += 1;
                        self.needs_frame = true;
                        
                        // Emit spill code using R12 as scratch
                        self.instructions.push(AsmInst::Comment(format!("Spilling {} to FP+{}", old_value, spill_offset)));
                        self.instructions.push(AsmInst::AddI(Reg::R12, Reg::R15, spill_offset));
                        self.instructions.push(AsmInst::Store(reg, Reg::R13, Reg::R12));
                        
                        // Update location
                        self.value_locations.insert(old_value, Location::Spilled(spill_offset));
                    }
                }
            }
        }
    }
    
    /// Pick a register to spill - try to pick one that's not actively needed
    fn pick_spill_victim(&self) -> Reg {
        // Try to pick a register containing a constant or temporary value first
        for reg in [Reg::R3, Reg::R4, Reg::R5, Reg::R6, Reg::R7, Reg::R8, Reg::R9, Reg::R10, Reg::R11] {
            if let Some(content) = self.reg_contents.get(&reg) {
                if content.starts_with("const_") || content.starts_with("addr_") {
                    // This is a temporary value, prefer to spill this
                    return reg;
                }
            }
        }
        
        // Otherwise pick the first allocatable register
        // Could implement LRU or other heuristics here
        Reg::R3
    }
    
    /// Free a register - now uses centralized allocator
    fn free_reg(&mut self, reg: Reg) {
        // Use the centralized allocator
        self.reg_alloc.free_reg(reg);
    }
    
    /// Free all registers (at statement boundaries)
    fn free_all_regs(&mut self) {
        // Use centralized allocator's free_all method
        self.reg_alloc.free_all();
        // Also clear the compatibility tracking
        self.reg_contents.clear();
        self.free_regs = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5, Reg::R4, Reg::R3];
    }
    
    /// Get a scratch register for temporary use
    /// This register should be used immediately and not stored
    /// The `hint` parameter helps avoid conflicts when loading multiple values
    fn get_scratch_register(&mut self, hint: u8) -> Reg {
        // Use R3-R4 as scratch registers
        // The hint helps us pick different registers for different operands
        match hint % 2 {
            0 => Reg::R3,
            1 => Reg::R4,
            _ => Reg::R3,
        }
    }
    
    /// Load a value from a location into a register
    /// Returns the register containing the value
    /// The hint parameter helps avoid conflicts when loading multiple values
    fn load_from_location_with_hint(&mut self, loc: Location, hint: u8) -> Reg {
        match loc {
            Location::Register(reg) => reg,
            Location::Spilled(offset) => {
                // Load from spill slot into scratch register using hint
                let scratch = self.get_scratch_register(hint);
                let addr_reg = if hint == 0 { Reg::R5 } else { Reg::R6 }; // Use different reg for address calculation
                self.instructions.push(AsmInst::AddI(addr_reg, Reg::R15, offset));
                self.instructions.push(AsmInst::Load(scratch, Reg::R13, addr_reg));
                scratch
            }
        }
    }
    
    /// Load a value from a location into a register (defaults to hint 0)
    fn load_from_location(&mut self, loc: Location) -> Reg {
        self.load_from_location_with_hint(loc, 0)
    }
    
    /// Store a register value to a location
    fn store_to_location(&mut self, reg: Reg, loc: Location) {
        match loc {
            Location::Register(dest_reg) => {
                if reg != dest_reg {
                    self.instructions.push(AsmInst::Add(dest_reg, reg, Reg::R0));
                }
            }
            Location::Spilled(offset) => {
                // Store to spill slot
                let addr_reg = Reg::R4; // Use R4 as temp for address calculation
                self.instructions.push(AsmInst::AddI(addr_reg, Reg::R15, offset));
                self.instructions.push(AsmInst::Store(reg, Reg::R13, addr_reg));
            }
        }
    }
    
    /// Generate function prologue
    fn generate_prologue(&mut self) {
        // Save RA
        self.instructions.push(AsmInst::Store(Reg::RA, Reg::R13, Reg::R14));
        self.instructions.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
        
        // Save old FP
        self.instructions.push(AsmInst::Store(Reg::R15, Reg::R13, Reg::R14));
        self.instructions.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
        
        // Set new FP = SP
        self.instructions.push(AsmInst::Add(Reg::R15, Reg::R14, Reg::R0));
        
        // We'll reserve space for locals and spills when we know how much we need
    }
    
    /// Generate function epilogue
    fn generate_epilogue(&mut self) {
        // Restore SP to FP (deallocate locals)
        self.instructions.push(AsmInst::Add(Reg::R14, Reg::R15, Reg::R0));
        
        // Pop old FP
        self.instructions.push(AsmInst::AddI(Reg::R14, Reg::R14, -1));
        self.instructions.push(AsmInst::Load(Reg::R15, Reg::R13, Reg::R14));
        
        // Pop RA
        self.instructions.push(AsmInst::AddI(Reg::R14, Reg::R14, -1));
        self.instructions.push(AsmInst::Load(Reg::RA, Reg::R13, Reg::R14));
    }
    
    /// Generate a unique label
    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }
    
    /// Get the size of a type in 16-bit words
    fn get_type_size_in_words(&self, ir_type: &IrType) -> u64 {
        match ir_type {
            IrType::Void => 0,
            IrType::I1 => 1, // Boolean takes 1 word
            IrType::I8 | IrType::I16 => 1,
            IrType::I32 | IrType::I64 => 2,
            IrType::Ptr(_) => 1, // 16-bit pointers
            IrType::Array { element_type, size } => {
                let elem_size = self.get_type_size_in_words(element_type);
                elem_size * size
            }
            IrType::Function { .. } => 0,
            IrType::Struct { .. } => 0, // TODO: Calculate struct size
            IrType::Label => 0, // Labels don't have size
        }
    }
    
    /// Convert a value to string for debug output
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Constant(n) => format!("{}", n),
            Value::Temp(id) => format!("t{}", id),
            Value::Function(name) => name.clone(),
            Value::Global(name) => format!("@{}", name),
            _ => "?".to_string(),
        }
    }
}

/// Lower a Module to assembly instructions
pub fn lower_module_to_assembly(module: Module) -> Result<Vec<AsmInst>, CompilerError> {
    let mut lowerer = ModuleLowerer::new();
    lowerer.lower(module)
}