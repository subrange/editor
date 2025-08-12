//! Correct Calling Convention Implementation
//! 
//! Key fixes from v1:
//! - Parameters passed on STACK (as per spec, not in registers!)
//! - Return values: R3 for scalar/addr, R4 for bank
//! - Proper fat pointer handling
//! - Stack parameters pushed before call, accessed via FP in callee

use rcc_codegen::{AsmInst, Reg};
use crate::v2::regmgmt::RegisterPressureManager;
use crate::v2::naming::NameGenerator;
use log::{debug, trace, info};

pub(super) struct CallingConvention {}

impl Default for CallingConvention {
    fn default() -> Self {
        Self::new()
    }
}

impl CallingConvention {
    pub(super) fn new() -> Self {
        Self {}
    }
    
    /// Setup parameters for a function call
    /// All parameters are passed on the stack according to the calling convention
    pub(super) fn setup_call_args(&self, 
                           pressure_manager: &mut RegisterPressureManager,
                           _naming: &mut NameGenerator,
                           args: Vec<CallArg>) -> Vec<AsmInst> {
        info!("Setting up {} call arguments", args.len());
        let mut insts = Vec::new();
        let mut stack_offset = 0i16;
        
        insts.push(AsmInst::Comment(format!("Pushing {} arguments to stack", args.len())));
        
        // Push arguments onto stack in reverse order (rightmost first)
        debug!("  Pushing arguments in reverse order (rightmost first)");
        for (idx, arg) in args.into_iter().enumerate().rev() {
            match arg {
                CallArg::Scalar(src_reg) => {
                    trace!("  Pushing arg {} (scalar) from {:?}", idx, src_reg);
                    insts.push(AsmInst::Comment(format!("Push arg {idx} (scalar)")));
                    // Push scalar value
                    insts.push(AsmInst::Store(src_reg, Reg::R13, Reg::R14));
                    insts.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
                    stack_offset += 1;
                }
                
                CallArg::FatPointer { addr, bank } => {
                    trace!("  Pushing arg {} (fat ptr) - addr: {:?}, bank: {:?}", idx, addr, bank);
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
        debug!("  Spilling all registers before call");
        pressure_manager.spill_all();
        let spill_insts = pressure_manager.take_instructions();
        if !spill_insts.is_empty() {
            trace!("  Generated {} spill instructions", spill_insts.len());
        }
        insts.extend(spill_insts);
        
        // Add comment about stack adjustment
        insts.push(AsmInst::Comment(format!("Pushed {stack_offset} words to stack")));
        debug!("Call args setup complete: pushed {} words, generated {} instructions", 
               stack_offset, insts.len());
        
        insts
    }
    
    /// Generate call instruction
    /// For cross-bank calls, sets PCB first then uses JAL
    pub(super) fn emit_call(&self, func_addr: u16, func_bank: u16) -> Vec<AsmInst> {
        info!("Emitting call to function at bank:{}, addr:{}", func_bank, func_addr);
        let mut insts = Vec::new();
        
        insts.push(AsmInst::Comment(format!("Call function at bank:{func_bank}, addr:{func_addr}")));
        
        // JAL only jumps within current bank, saving RA/RAB
        // For cross-bank calls, we need to set PCB first
        if func_bank != 0 {
            debug!("  Cross-bank call: setting PCB to {}", func_bank);
            insts.push(AsmInst::Comment("Set PCB for cross-bank call".to_string()));
            insts.push(AsmInst::LI(Reg::PCB, func_bank as i16));
        } else {
            trace!("  In-bank call (bank 0)");
        }
        
        // JAL addr - sets RA←PC+1, RAB←PCB, PC←addr
        // The actual instruction is: JAL RA, R0, addr (RA is implicit)
        // Our AsmInst::Jal(bank, addr) abstraction will be lowered to proper format
        // First param is traditionally bank but for in-bank jumps it's 0
        trace!("  JAL instruction: saves RA←PC+1, RAB←PCB, jumps to addr {}", func_addr);
        insts.push(AsmInst::Jal(0, func_addr as i16));
        
        debug!("Call emission complete: generated {} instructions", insts.len());
        insts
    }
    
    /// Handle return value after call
    pub(super) fn handle_return_value(&self, 
                              pressure_manager: &mut RegisterPressureManager,
                              naming: &mut NameGenerator,
                              is_pointer: bool) -> (Vec<AsmInst>, (Reg, Option<Reg>)) {
        let mut insts = Vec::new();
        
        if is_pointer {
            // Fat pointer return in R3 (addr) and R4 (bank)
            debug!("Handling fat pointer return");
            
            // Allocate registers for the return value
            let addr_reg = pressure_manager.get_register(naming.ret_addr_name());
            let bank_reg = pressure_manager.get_register(naming.ret_bank_name());
            insts.extend(pressure_manager.take_instructions());
            
            // Copy from R3/R4
            insts.push(AsmInst::Comment("Get fat pointer return value".to_string()));
            insts.push(AsmInst::Add(addr_reg, Reg::R3, Reg::R0));
            insts.push(AsmInst::Add(bank_reg, Reg::R4, Reg::R0));
            
            (insts, (addr_reg, Some(bank_reg)))
        } else {
            // Scalar return in R3
            debug!("Handling scalar return");
            
            let ret_reg = pressure_manager.get_register(naming.ret_val_name());
            insts.extend(pressure_manager.take_instructions());
            
            insts.push(AsmInst::Comment("Get scalar return value".to_string()));
            insts.push(AsmInst::Add(ret_reg, Reg::R3, Reg::R0));
            
            (insts, (ret_reg, None))
        }
    }
    
    /// Clean up stack after call
    pub(super) fn cleanup_stack(&self, num_args_words: i16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        if num_args_words > 0 {
            debug!("Cleaning up {} words from stack after call", num_args_words);
            insts.push(AsmInst::Comment(format!("Clean up {num_args_words} words from stack")));
            insts.push(AsmInst::AddI(Reg::R14, Reg::R14, -num_args_words));
            trace!("  Adjusted SP by -{}", num_args_words);
        } else {
            trace!("No stack cleanup needed (0 arguments)");
        }
        insts
    }
    
    /// Load parameter from stack in callee
    /// Parameters are at negative offsets from FP (before the frame)
    pub(super) fn load_param(&self, index: usize, 
                     pressure_manager: &mut RegisterPressureManager,
                     naming: &mut NameGenerator) -> (Vec<AsmInst>, Reg) {
        info!("Loading parameter {} from stack", index);
        let mut insts = Vec::new();
        
        // Parameters are before the frame (negative offsets from FP)
        // They are pushed in reverse order, so param 0 is closest to FP
        let param_offset = -(index as i16 + 3); // -3 because: -1 for FP, -1 for RA, -1 for first param
        debug!("  Parameter {} is at FP{} (offset calculation: -({}+3))", 
               index, param_offset, index);
        
        let param_name = naming.param_name(index);
        trace!("  Allocating register for parameter '{}'", param_name);
        let dest = pressure_manager.get_register(param_name);
        
        let spill_insts = pressure_manager.take_instructions();
        if !spill_insts.is_empty() {
            trace!("  Generated {} spill/reload instructions", spill_insts.len());
        }
        insts.extend(spill_insts);
        
        insts.push(AsmInst::Comment(format!("Load param {index} from FP{param_offset}")));
        trace!("  Computing address: FP + {}", param_offset);
        insts.push(AsmInst::AddI(Reg::R12, Reg::R15, param_offset));
        trace!("  Loading from stack (bank R13) at computed address into {:?}", dest);
        insts.push(AsmInst::Load(dest, Reg::R13, Reg::R12));
        
        debug!("Parameter load complete: generated {} instructions, result in {:?}", 
               insts.len(), dest);
        (insts, dest)
    }
}

/// Argument types for function calls
#[derive(Debug, Clone)]
pub enum CallArg {  // This needs to stay pub for the re-export
    Scalar(Reg),
    FatPointer { addr: Reg, bank: Reg },
}

// Tests moved to tests/calling_convention_tests.rs