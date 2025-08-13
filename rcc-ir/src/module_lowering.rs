//! Module to Assembly Lowering
//! 
//! This module implements the lowering from the full IR (Module/Function/Instruction)
//! to Ripple assembly instructions.

use crate::ir::{Module, Value, BankTag};
use crate::simple_regalloc::SimpleRegAlloc;
use rcc_codegen::{AsmInst, Reg};
use rcc_common::CompilerError;
use std::collections::HashMap;
use log::{debug, trace};


/// Fat pointer components tracking
#[derive(Debug, Clone)]
pub struct FatPtrComponents {
    pub(crate) addr_temp: u32,  // Temp ID holding the address
    pub(crate) bank_tag: BankTag,  // Bank tag for this pointer
}

/// Module to assembly lowering context
pub struct ModuleLowerer {
    /// Assembly instructions being generated
    instructions: Vec<AsmInst>,
    
    /// Centralized register allocator
    pub(crate) reg_alloc: SimpleRegAlloc,
    
    // Old register allocation system - REMOVED
    // Now using SimpleRegAlloc for all register management
    
    /// Mapping from IR values to either registers or spill slots
    pub(crate) value_locations: HashMap<String, Location>,
    
    /// Global variable addresses
    pub(crate) global_addresses: HashMap<String, u16>,
    
    /// Next available global memory address
    pub(crate) next_global_addr: u16,
    
    /// Current function being lowered
    pub(crate) current_function: Option<String>,
    
    /// Whether current function needs a frame
    pub(crate) needs_frame: bool,
    
    /// Label counter for generating unique labels
    pub(crate) label_counter: u32,
    
    /// Stack offset for local allocations (relative to frame pointer)
    /// This includes both user variables and spill slots
    pub(crate) local_stack_offset: i16,
    
    /// Map from temp IDs to stack offsets (ONLY for direct Alloca results)
    pub(crate) local_offsets: HashMap<u32, i16>,
    
    
    /// Fat pointer components for pointer-typed temporaries
    pub(crate) fat_ptr_components: HashMap<u32, FatPtrComponents>,
    
    /// Cache for need() calculations to avoid recomputation
    pub(crate) need_cache: HashMap<String, usize>,
    
    /// Maximum stack offset needed for this function (for allocating stack space)
    pub(crate) max_stack_offset: i16,
}

/// Location of a value - either in a register or spilled to stack
#[derive(Debug, Clone, Copy)]
pub(crate) enum Location {
    Register(Reg),
    Spilled(i16), // Offset from FP
}

impl Default for ModuleLowerer {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleLowerer {

    pub(crate) fn emit(&mut self, i: AsmInst) { self.instructions.push(i) }
    pub(crate) fn emit_comment(&mut self, comment: String) {
        self.instructions.push(AsmInst::Comment(comment));
    }
    pub(crate) fn emit_many(&mut self, instructions: Vec<AsmInst>) {
        self.instructions.extend(instructions);
    }
    pub(crate) fn temp(&mut self, name: impl Into<String>) -> Reg {
        let r = self.reg_alloc.get_reg(name.into());
        self.instructions.append(&mut self.reg_alloc.take_instructions());
        r
    }
    pub(crate) fn free(&mut self, r: Reg) { self.reg_alloc.free_reg(r) }
    pub(crate) fn free_all(&mut self) {
        // Use centralized allocator's free_all method
        self.reg_alloc.free_all();
    }
    /// Generate a unique label with the given prefix
    /// Automatically increments the label counter and includes function context
    pub(crate) fn generate_label(&mut self, prefix: &str) -> String {
        let func_prefix = self.current_function.as_ref()
            .map(|f| format!("{f}_"))
            .unwrap_or_default();
        let label = format!("{}{}_{}", func_prefix, prefix, self.label_counter);
        self.label_counter += 1;
        label
    }
    
    /// Generate a unique temporary name for register allocation
    /// This is different from labels - it's for tracking values in registers
    pub(crate) fn generate_temp_name(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        name
    }
    
    /// Generate a pair of labels for branching (e.g., for if/else)
    pub(crate) fn generate_branch_labels(&mut self, prefix: &str) -> (String, String) {
        let true_label = self.generate_label(&format!("{prefix}_true"));
        let false_label = self.generate_label(&format!("{prefix}_false"));
        (true_label, false_label)
    }
    
    /// Generate labels for bank selection
    pub(crate) fn generate_bank_labels(&mut self) -> (String, String) {
        let stack_label = self.generate_label("bank_stack");
        let done_label = self.generate_label("bank_done");
        (stack_label, done_label)
    }
    
    // Keep these as static methods since they don't need the counter
    /// Format a temp ID as a string (e.g., 42 -> "t42")
    pub(crate) fn temp_name(id: u32) -> String {
        format!("t{id}")
    }
    
    /// Get the bank temp key for a pointer temp ID
    /// This is used to track the bank tag for fat pointers
    pub(crate) fn bank_temp_key(tid: u32) -> String {
        Self::temp_name(100000 + tid)
    }
    
    /// Format a constant as a string for tracking
    fn const_name(value: i64) -> String {
        format!("const_{value}")
    }
    
    /// Format a global name for tracking
    fn global_name(name: &str) -> String {
        format!("global_{name}")
    }
    
    // Deprecated - use generate_label instead
    pub(crate) fn label(&mut self, prefix: &str) -> String {
        self.generate_label(prefix)
    }
    
    /// Get a descriptive string for a value (used for register tracking)
    pub(crate) fn describe_value(value: &Value) -> String {
        match value {
            Value::Temp(id) => Self::temp_name(*id),
            Value::Constant(n) => Self::const_name(*n),
            Value::Global(name) => Self::global_name(name),
            Value::Function(name) => format!("func_{name}"),
            Value::FatPtr(components) => format!("fatptr_{components:?}"),
            Value::Undef => "undef".to_string(),
        }
    }
    

    
    pub fn new() -> Self {
        debug!("Creating new ModuleLowerer");
        Self {
            instructions: Vec::new(),
            reg_alloc: SimpleRegAlloc::new(),
            value_locations: HashMap::new(),
            global_addresses: HashMap::new(),
            next_global_addr: 100, // Start globals at address 100
            current_function: None,
            needs_frame: false,
            label_counter: 0,
            max_stack_offset: 0,
            local_stack_offset: 0,
            local_offsets: HashMap::new(),
            fat_ptr_components: HashMap::new(),
            need_cache: HashMap::new(),
        }
    }
    
    /// Lower a module to assembly
    pub fn lower(&mut self, module: Module) -> Result<Vec<AsmInst>, CompilerError> {
        // Check if this module has a main function (indicating it's the main module)
        let has_main = module.functions.iter().any(|f| f.name == "main");
        
        // Only generate _init_globals for the main module
        // This avoids duplicate labels when linking multiple object files
        if has_main {
            self.instructions.push(AsmInst::Label("_init_globals".to_string()));
            
            // Generate globals and their initialization
            for global in &module.globals {
                self.lower_global(global)?;
            }
            
            self.instructions.push(AsmInst::Ret);
        } else {
            // For library modules, just generate globals without _init_globals
            for global in &module.globals {
                self.lower_global(global)?;
            }
        }
        
        // Generate functions
        for function in &module.functions {
            self.lower_function(function)?;
        }
        
        Ok(self.instructions.clone())
    }
    
    pub(crate) fn get_reg(&mut self, for_value: String) -> Reg {
        trace!("ModuleLowerer::get_reg for '{for_value}'");
        self.instructions.push(AsmInst::Comment(format!("=== ModuleLowerer::get_reg for '{for_value}' ===")));
        
        // Use the centralized allocator
        let reg = self.reg_alloc.get_reg(for_value.clone());
        
        // Check if the allocator spilled anything
        if let Some((spilled_value, spill_offset)) = self.reg_alloc.take_last_spilled() {
            debug!("Register allocator spilled '{spilled_value}' to FP+{spill_offset}");
            // Update value_locations to show this value is now spilled
            self.value_locations.insert(spilled_value, Location::Spilled(spill_offset));
        }
        
        // Append any spill/reload instructions generated
        let spill_instructions = self.reg_alloc.take_instructions();
        self.instructions.extend(spill_instructions);
        
        // Track the new value in its register
        self.value_locations.insert(for_value, Location::Register(reg));
        
        reg
    }
    
    /// Wrapper for reload to ensure spill/reload instructions are emitted
    pub(crate) fn reload(&mut self, value: String) -> Reg {
        let reg = self.reg_alloc.reload(value.clone());
        
        // Append any spill/reload instructions generated
        self.instructions.append(&mut self.reg_alloc.take_instructions());
        
        // Also track in value_locations for compatibility
        self.value_locations.insert(value, Location::Register(reg));
        
        reg
    }
    
    /// Get the value name stored in a register, if any
    pub(crate) fn get_register_value_name(&self, reg: Reg) -> Option<String> {
        // Check the reg_alloc's reg_contents to see what's in this register
        // We need to make reg_contents accessible
        self.reg_alloc.get_register_value(reg)
    }
    
    /// Convert a value to string for debug output
    pub(crate) fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Constant(n) => format!("{n}"),
            Value::Temp(id) => format!("t{id}"),
            Value::Function(name) => name.clone(),
            Value::Global(name) => format!("@{name}"),
            Value::FatPtr(ptr) => format!("{{addr: {}, bank: {:?}}}", self.value_to_string(&ptr.addr), ptr.bank),
            _ => "?".to_string(),
        }
    }
}

/// Lower a Module to assembly instructions
pub fn lower_module_to_assembly(module: Module) -> Result<Vec<AsmInst>, CompilerError> {
    lower_module_to_assembly_with_options(module, 4096)
}

/// Lower a Module to assembly instructions with configuration options
pub fn lower_module_to_assembly_with_options(module: Module, bank_size: u16) -> Result<Vec<AsmInst>, CompilerError> {
    // Check if V2 backend should be used (can be controlled by env var or feature flag)
    let use_v2 = std::env::var("USE_V2_BACKEND").unwrap_or_else(|_| "true".to_string()) == "true";
    
    if use_v2 {
        // Use the new V2 backend
        crate::v2::lower_module_v2(&module, bank_size)
            .map_err(|e| CompilerError::codegen_error(e, rcc_common::SourceLocation::dummy()))
    } else {
        // Use the old backend (ignores bank_size for now)
        let mut lowerer = ModuleLowerer::new();
        lowerer.lower(module)
    }
}