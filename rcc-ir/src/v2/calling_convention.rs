//! Correct Calling Convention Implementation
//! 
//! Key fixes from v1:
//! - Parameters passed on STACK (as per spec, not in registers!)
//! - Return values: R3 for scalar/addr, R4 for bank
//! - Proper fat pointer handling
//! - Stack parameters pushed before call, accessed via FP in callee

use rcc_codegen::{AsmInst, Reg};
use crate::v2::regmgmt::RegisterPressureManager;
use log::debug;

pub struct CallingConvention {}

impl Default for CallingConvention {
    fn default() -> Self {
        Self::new()
    }
}

impl CallingConvention {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Setup parameters for a function call
    /// All parameters are passed on the stack according to the calling convention
    pub fn setup_call_args(&self, 
                           pressure_manager: &mut RegisterPressureManager,
                           args: Vec<CallArg>) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        let mut stack_offset = 0i16;
        
        insts.push(AsmInst::Comment(format!("Pushing {} arguments to stack", args.len())));
        
        // Push arguments onto stack in reverse order (rightmost first)
        for (idx, arg) in args.into_iter().enumerate().rev() {
            match arg {
                CallArg::Scalar(src_reg) => {
                    insts.push(AsmInst::Comment(format!("Push arg {idx} (scalar)")));
                    // Push scalar value
                    insts.push(AsmInst::Store(src_reg, Reg::R13, Reg::R14));
                    insts.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
                    stack_offset += 1;
                }
                
                CallArg::FatPointer { addr, bank } => {
                    insts.push(AsmInst::Comment(format!("Push arg {idx} (fat ptr)")));
                    // Push bank first (higher address)
                    insts.push(AsmInst::Store(bank, Reg::R13, Reg::R14));
                    insts.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
                    // Then push address
                    insts.push(AsmInst::Store(addr, Reg::R13, Reg::R14));
                    insts.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
                    stack_offset += 2;
                }
            }
        }
        
        // Spill all registers before call
        pressure_manager.spill_all();
        insts.extend(pressure_manager.take_instructions());
        
        // Add comment about stack adjustment
        insts.push(AsmInst::Comment(format!("Pushed {stack_offset} words to stack")));
        
        insts
    }
    
    /// Generate call instruction
    /// For cross-bank calls, sets PCB first then uses JAL
    pub fn emit_call(&self, func_addr: u16, func_bank: u16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment(format!("Call function at bank:{func_bank}, addr:{func_addr}")));
        
        // JAL only jumps within current bank, saving RA/RAB
        // For cross-bank calls, we need to set PCB first
        if func_bank != 0 {
            insts.push(AsmInst::Comment("Set PCB for cross-bank call".to_string()));
            insts.push(AsmInst::LI(Reg::PCB, func_bank as i16));
        }
        
        // JAL addr - sets RA←PC+1, RAB←PCB, PC←addr
        // The actual instruction is: JAL RA, R0, addr (RA is implicit)
        // Our AsmInst::Jal(bank, addr) abstraction will be lowered to proper format
        // First param is traditionally bank but for in-bank jumps it's 0
        insts.push(AsmInst::Jal(0, func_addr as i16));
        
        insts
    }
    
    /// Handle return value after call
    pub fn handle_return_value(&self, 
                              pressure_manager: &mut RegisterPressureManager,
                              is_pointer: bool) -> (Vec<AsmInst>, (Reg, Option<Reg>)) {
        let mut insts = Vec::new();
        
        if is_pointer {
            // Fat pointer return in R3 (addr) and R4 (bank)
            debug!("Handling fat pointer return");
            
            // Allocate registers for the return value
            let addr_reg = pressure_manager.get_register("ret_addr".to_string());
            let bank_reg = pressure_manager.get_register("ret_bank".to_string());
            insts.extend(pressure_manager.take_instructions());
            
            // Copy from R3/R4
            insts.push(AsmInst::Comment("Get fat pointer return value".to_string()));
            insts.push(AsmInst::Add(addr_reg, Reg::R3, Reg::R0));
            insts.push(AsmInst::Add(bank_reg, Reg::R4, Reg::R0));
            
            (insts, (addr_reg, Some(bank_reg)))
        } else {
            // Scalar return in R3
            debug!("Handling scalar return");
            
            let ret_reg = pressure_manager.get_register("ret_val".to_string());
            insts.extend(pressure_manager.take_instructions());
            
            insts.push(AsmInst::Comment("Get scalar return value".to_string()));
            insts.push(AsmInst::Add(ret_reg, Reg::R3, Reg::R0));
            
            (insts, (ret_reg, None))
        }
    }
    
    /// Clean up stack after call
    pub fn cleanup_stack(&self, num_args_words: i16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        if num_args_words > 0 {
            insts.push(AsmInst::Comment(format!("Clean up {num_args_words} words from stack")));
            insts.push(AsmInst::AddI(Reg::R14, Reg::R14, -num_args_words));
        }
        insts
    }
    
    /// Load parameter from stack in callee
    /// Parameters are at negative offsets from FP (before the frame)
    pub fn load_param(&self, index: usize, 
                     pressure_manager: &mut RegisterPressureManager) -> (Vec<AsmInst>, Reg) {
        let mut insts = Vec::new();
        
        // Parameters are before the frame (negative offsets from FP)
        // They are pushed in reverse order, so param 0 is closest to FP
        let param_offset = -(index as i16 + 3); // -3 because: -1 for FP, -1 for RA, -1 for first param
        
        let dest = pressure_manager.get_register(format!("param{index}"));
        insts.extend(pressure_manager.take_instructions());
        
        insts.push(AsmInst::Comment(format!("Load param {index} from FP{param_offset}")));
        insts.push(AsmInst::AddI(Reg::R12, Reg::R15, param_offset));
        insts.push(AsmInst::Load(dest, Reg::R13, Reg::R12));
        
        (insts, dest)
    }
}

/// Argument types for function calls
#[derive(Debug, Clone)]
pub enum CallArg {
    Scalar(Reg),
    FatPointer { addr: Reg, bank: Reg },
}

// Tests moved to tests/calling_convention_tests.rs