//! Module to Assembly Lowering
//! 
//! This module implements the lowering from the full IR (Module/Function/Instruction)
//! to Ripple assembly instructions.

use crate::ir::{Module, Instruction, Value, IrType, IrBinaryOp, BankTag};
use crate::simple_regalloc::SimpleRegAlloc;
use rcc_codegen::{AsmInst, Reg};
use rcc_common::CompilerError;
use std::collections::HashMap;
use crate::lower::instr::arithmetic::emit_ne;


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
}

/// Location of a value - either in a register or spilled to stack
#[derive(Debug, Clone, Copy)]
pub(crate) enum Location {
    Register(Reg),
    Spilled(i16), // Offset from FP
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
    pub(crate) fn label(&mut self, prefix: &str) -> String {
        let s = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        s
    }
    
    
    /// Format a temp ID as a string (e.g., 42 -> "t42")
    pub(crate) fn temp_name(id: u32) -> String {
        format!("t{}", id)
    }
    
    /// Format a constant as a string for tracking
    fn const_name(value: i64) -> String {
        format!("const_{}", value)
    }
    
    /// Format a global name for tracking
    fn global_name(name: &str) -> String {
        format!("global_{}", name)
    }
    
    fn get_label_num(&mut self) -> u32 {
        let c = self.label_counter;
        self.label_counter += 1;
        c
    }
    
    /// Get a descriptive string for a value (used for register tracking)
    pub(crate) fn describe_value(value: &Value) -> String {
        match value {
            Value::Temp(id) => Self::temp_name(*id),
            Value::Constant(n) => Self::const_name(*n),
            Value::Global(name) => Self::global_name(name),
            Value::Function(name) => format!("func_{}", name),
            Value::FatPtr(components) => format!("fatptr_{:?}", components),
            Value::Undef => "undef".to_string(),
        }
    }
    

    
    pub fn new() -> Self {
        eprintln!("Creating new ModuleLowerer");
        Self {
            instructions: Vec::new(),
            reg_alloc: SimpleRegAlloc::new(),
            value_locations: HashMap::new(),
            global_addresses: HashMap::new(),
            next_global_addr: 100, // Start globals at address 100
            current_function: None,
            needs_frame: false,
            label_counter: 0,
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
        eprintln!("=== ModuleLowerer::get_reg for '{}' ===", for_value);
        self.instructions.push(AsmInst::Comment(format!("=== ModuleLowerer::get_reg for '{}' ===", for_value)));
        
        // Use the centralized allocator
        let reg = self.reg_alloc.get_reg(for_value.clone());
        
        // Append any spill/reload instructions generated
        self.instructions.append(&mut self.reg_alloc.take_instructions());
        
        // Also track in value_locations for compatibility
        self.value_locations.insert(for_value, Location::Register(reg));
        
        reg
    }
    
    /// Convert a value to string for debug output
    pub(crate) fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Constant(n) => format!("{}", n),
            Value::Temp(id) => format!("t{}", id),
            Value::Function(name) => name.clone(),
            Value::Global(name) => format!("@{}", name),
            Value::FatPtr(ptr) => format!("{{addr: {}, bank: {:?}}}", self.value_to_string(&ptr.addr), ptr.bank),
            _ => "?".to_string(),
        }
    }
}

/// Lower a Module to assembly instructions
pub fn lower_module_to_assembly(module: Module) -> Result<Vec<AsmInst>, CompilerError> {
    let mut lowerer = ModuleLowerer::new();
    lowerer.lower(module)
}