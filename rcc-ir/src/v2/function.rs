//! Function Lowering with Correct Prologue/Epilogue
//! 
//! Key fixes:
//! - R13 initialized to 1 at function start
//! - Proper stack frame management
//! - Correct RA/FP save/restore

use rcc_codegen::{AsmInst, Reg};
use crate::v2::regmgmt::{RegisterPressureManager, BankInfo};
use log::debug;

pub struct FunctionLowering {
    pub pressure_manager: RegisterPressureManager,
    pub local_count: i16,
    pub spill_count: i16,
    pub instructions: Vec<AsmInst>,
}

impl Default for FunctionLowering {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionLowering {
    pub fn new() -> Self {
        Self {
            pressure_manager: RegisterPressureManager::new(0),
            local_count: 0,
            spill_count: 0,
            instructions: Vec::new(),
        }
    }
    
    /// Generate function prologue
    /// Parameters are already on stack (pushed by caller)
    pub fn emit_prologue(&mut self, local_slots: i16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment("=== Function Prologue ===".to_string()));
        
        // Initialize pressure manager with local count
        // This will handle R13 initialization automatically
        self.pressure_manager = RegisterPressureManager::new(local_slots);
        self.pressure_manager.init();
        
        // Take any initialization instructions (R13 setup)
        insts.extend(self.pressure_manager.take_instructions());
        
        // Save RA at current SP
        insts.push(AsmInst::Comment("Save RA at SP".to_string()));
        insts.push(AsmInst::Store(Reg::RA, Reg::R13, Reg::R14));
        insts.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
        
        // Save old FP
        insts.push(AsmInst::Comment("Save old FP".to_string()));
        insts.push(AsmInst::Store(Reg::R15, Reg::R13, Reg::R14));
        insts.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
        
        // Set new FP = SP
        insts.push(AsmInst::Comment("Set FP = SP".to_string()));
        insts.push(AsmInst::Add(Reg::R15, Reg::R14, Reg::R0));
        
        // Allocate space for locals
        if local_slots > 0 {
            insts.push(AsmInst::Comment(format!("Allocate {local_slots} slots for locals")));
            insts.push(AsmInst::AddI(Reg::R14, Reg::R14, local_slots));
        }
        
        self.local_count = local_slots;
        
        debug!("Generated prologue: locals={local_slots}");
        insts
    }
    
    /// Generate function epilogue
    pub fn emit_epilogue(&mut self) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment("=== Function Epilogue ===".to_string()));
        
        // Restore SP = FP
        insts.push(AsmInst::Comment("Restore SP = FP".to_string()));
        insts.push(AsmInst::Add(Reg::R14, Reg::R15, Reg::R0));
        
        // Restore old FP
        insts.push(AsmInst::Comment("Restore old FP".to_string()));
        insts.push(AsmInst::AddI(Reg::R14, Reg::R14, -1));
        insts.push(AsmInst::Load(Reg::R15, Reg::R13, Reg::R14));
        
        // Restore RA
        insts.push(AsmInst::Comment("Restore RA".to_string()));
        insts.push(AsmInst::AddI(Reg::R14, Reg::R14, -1));
        insts.push(AsmInst::Load(Reg::RA, Reg::R13, Reg::R14));
        
        // Return to caller
        // Need to restore PCB from RAB for cross-bank returns
        // Then JALR to RA
        insts.push(AsmInst::Comment("Return to caller".to_string()));
        
        // Restore the caller's bank (RAB was saved by JAL)
        // Note: RAB is automatically saved by JAL instruction
        // but we need to explicitly restore PCB before jumping back
        insts.push(AsmInst::Add(Reg::PCB, Reg::RAB, Reg::R0));
        
        // JALR rd, 0, rs format: rd←PC+1 (unused), PC←rs
        // This jumps to the address in RA within the bank we just restored
        insts.push(AsmInst::Jalr(Reg::R0, Reg::R0, Reg::RA));
        
        debug!("Generated epilogue");
        insts
    }
    
    /// Handle return statement
    pub fn emit_return(&mut self, value: Option<(Reg, Option<Reg>)>) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        if let Some((addr_reg, bank_reg)) = value {
            // Return value or fat pointer
            if let Some(bank) = bank_reg {
                // Fat pointer return: R3=addr, R4=bank
                insts.push(AsmInst::Comment("Return fat pointer".to_string()));
                if addr_reg != Reg::R3 {
                    insts.push(AsmInst::Add(Reg::R3, addr_reg, Reg::R0));
                }
                if bank != Reg::R4 {
                    insts.push(AsmInst::Add(Reg::R4, bank, Reg::R0));
                }
            } else {
                // Scalar return: R3=value
                insts.push(AsmInst::Comment("Return scalar value".to_string()));
                if addr_reg != Reg::R3 {
                    insts.push(AsmInst::Add(Reg::R3, addr_reg, Reg::R0));
                }
            }
        }
        
        // Add epilogue
        insts.extend(self.emit_epilogue());
        insts
    }
    
    /// Get local variable address
    pub fn get_local_addr(&mut self, offset: i16) -> Reg {
        // Use pressure manager for better register allocation
        let reg = self.pressure_manager.get_register(format!("local_{offset}"));
        
        self.instructions.push(AsmInst::Comment(format!("Get address of local at FP+{offset}")));
        self.instructions.push(AsmInst::Add(reg, Reg::R15, Reg::R0));
        self.instructions.push(AsmInst::AddI(reg, reg, offset));
        
        // Take any spill/reload instructions generated
        self.instructions.extend(self.pressure_manager.take_instructions());
        
        // Mark this as a stack pointer
        self.pressure_manager.set_pointer_bank(format!("local_{offset}"), 
                                        BankInfo::Stack);
        reg
    }
    
    /// Load from local variable
    pub fn load_local(&mut self, offset: i16, dest: Reg) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment(format!("Load from local at FP+{offset}")));
        
        // Calculate address using pressure manager
        let addr_reg = self.pressure_manager.get_register(format!("local_addr_{offset}"));
        insts.extend(self.pressure_manager.take_instructions());
        
        insts.push(AsmInst::Add(addr_reg, Reg::R15, Reg::R0));
        insts.push(AsmInst::AddI(addr_reg, addr_reg, offset));
        
        // Load using R13 as bank
        insts.push(AsmInst::Load(dest, Reg::R13, addr_reg));
        
        self.pressure_manager.free_register(addr_reg);
        insts
    }
    
    /// Store to local variable
    pub fn store_local(&mut self, offset: i16, src: Reg) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment(format!("Store to local at FP+{offset}")));
        
        // Calculate address using pressure manager
        let addr_reg = self.pressure_manager.get_register(format!("local_addr_{offset}"));
        insts.extend(self.pressure_manager.take_instructions());
        
        insts.push(AsmInst::Add(addr_reg, Reg::R15, Reg::R0));
        insts.push(AsmInst::AddI(addr_reg, addr_reg, offset));
        
        // Store using R13 as bank
        insts.push(AsmInst::Store(src, Reg::R13, addr_reg));
        
        self.pressure_manager.free_register(addr_reg);
        insts
    }
}

// Tests moved to tests/function_tests.rs