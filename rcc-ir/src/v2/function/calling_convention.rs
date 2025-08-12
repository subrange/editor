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
    
    /// Core logic for analyzing parameter/argument placement
    /// Takes a closure that determines if an item is a fat pointer
    /// Returns (register_items_with_slots, first_stack_item_index)
    fn analyze_placement<F>(&self, count: usize, is_fat_ptr: F) -> (Vec<(usize, usize)>, usize)
    where
        F: Fn(usize) -> bool,
    {
        let mut register_items = Vec::new();
        let mut reg_slots_used = 0;
        let mut first_stack_item = count;
        
        for i in 0..count {
            if is_fat_ptr(i) {
                // Fat pointer needs 2 slots
                if reg_slots_used + 1 < 4 {  // Can fit both parts in registers
                    register_items.push((i, reg_slots_used));
                    reg_slots_used += 2;
                } else {
                    // Fat pointer doesn't fit, it and all subsequent go to stack
                    first_stack_item = i;
                    break;
                }
            } else {
                // Scalar needs 1 slot
                if reg_slots_used < 4 {
                    register_items.push((i, reg_slots_used));
                    reg_slots_used += 1;
                } else {
                    // No more register slots
                    first_stack_item = i;
                    break;
                }
            }
        }
        
        (register_items, first_stack_item)
    }
    
    /// Analyze which arguments go in registers vs stack based on CallArg types
    fn analyze_arg_placement(&self, args: &[CallArg]) -> (Vec<(usize, usize)>, usize) {
        self.analyze_placement(args.len(), |i| matches!(args[i], CallArg::FatPointer { .. }))
    }
    
    /// Analyze parameter placement for load_param based on IrType
    fn analyze_param_placement(&self, param_types: &[(rcc_common::TempId, crate::ir::IrType)]) 
        -> (Vec<(usize, usize)>, usize) {
        self.analyze_placement(param_types.len(), |i| param_types[i].1.is_pointer())
    }
    
    /// Setup parameters for a function call
    /// First 4 scalar arguments or 2 fat pointers go in A0-A3
    /// Remaining arguments are passed on the stack
    /// Returns (instructions, stack_words_used)
    pub(super) fn setup_call_args(&self, 
                           pressure_manager: &mut RegisterPressureManager,
                           _naming: &mut NameGenerator,
                           args: Vec<CallArg>) -> Vec<AsmInst> {
        info!("Setting up {} call arguments", args.len());
        let mut insts = Vec::new();
        let mut stack_offset = 0i16;
        let arg_regs = [Reg::A0, Reg::A1, Reg::A2, Reg::A3];
        
        // Use common logic to determine placement
        let (register_arg_slots, first_stack_arg) = self.analyze_arg_placement(&args);
        
        // Separate args into register args and stack args
        let mut reg_args = Vec::new();
        let mut stack_args = Vec::new();
        
        for (idx, arg) in args.into_iter().enumerate() {
            if let Some((_, reg_slot)) = register_arg_slots.iter().find(|(i, _)| *i == idx) {
                // This arg goes in register(s)
                match &arg {
                    CallArg::Scalar(_) => {
                        trace!("  Arg {} (scalar) goes in {:?}", idx, arg_regs[*reg_slot]);
                        reg_args.push((idx, arg_regs[*reg_slot], arg));
                    }
                    CallArg::FatPointer { .. } => {
                        trace!("  Arg {} (fat ptr) goes in {:?} and {:?}", idx, 
                               arg_regs[*reg_slot], arg_regs[*reg_slot + 1]);
                        reg_args.push((idx, arg_regs[*reg_slot], arg));
                    }
                }
            } else {
                // This arg goes on stack
                match &arg {
                    CallArg::Scalar(_) => trace!("  Arg {} (scalar) goes on stack", idx),
                    CallArg::FatPointer { .. } => trace!("  Arg {} (fat ptr) goes on stack", idx),
                }
                stack_args.push((idx, arg));
            }
        }
        
        // First, push stack arguments in reverse order
        if !stack_args.is_empty() {
            insts.push(AsmInst::Comment(format!("Pushing {} arguments to stack", stack_args.len())));
            debug!("  Pushing {} stack arguments in reverse order", stack_args.len());
            
            for (idx, arg) in stack_args.into_iter().rev() {
                match arg {
                    CallArg::Scalar(src_reg) => {
                        insts.push(AsmInst::Comment(format!("Push arg {idx} (scalar) to stack")));
                        insts.push(AsmInst::Store(src_reg, Reg::Sb, Reg::Sp));
                        insts.push(AsmInst::AddI(Reg::Sp, Reg::Sp, 1));
                        stack_offset += 1;
                    }
                    CallArg::FatPointer { addr, bank } => {
                        insts.push(AsmInst::Comment(format!("Push arg {idx} (fat ptr) to stack")));
                        // Push bank first (higher address)
                        insts.push(AsmInst::Store(bank, Reg::Sb, Reg::Sp));
                        insts.push(AsmInst::AddI(Reg::Sp, Reg::Sp, 1));
                        // Then push address
                        insts.push(AsmInst::Store(addr, Reg::Sb, Reg::Sp));
                        insts.push(AsmInst::AddI(Reg::Sp, Reg::Sp, 1));
                        stack_offset += 2;
                    }
                }
            }
        }
        
        // Then, move register arguments to A0-A3
        if !reg_args.is_empty() {
            insts.push(AsmInst::Comment(format!("Setting up {} register arguments", reg_args.len())));
            debug!("  Setting up {} register arguments", reg_args.len());
            
            for (idx, dest_reg, arg) in reg_args {
                match arg {
                    CallArg::Scalar(src_reg) => {
                        insts.push(AsmInst::Comment(format!("Arg {idx} (scalar) to {:?}", dest_reg)));
                        if src_reg != dest_reg {
                            insts.push(AsmInst::Add(dest_reg, src_reg, Reg::R0));
                        }
                    }
                    CallArg::FatPointer { addr, bank } => {
                        // Fat pointer uses two consecutive registers
                        let bank_reg = match dest_reg {
                            Reg::A0 => Reg::A1,
                            Reg::A1 => Reg::A2,
                            Reg::A2 => Reg::A3,
                            _ => panic!("Invalid fat pointer register assignment"),
                        };
                        insts.push(AsmInst::Comment(format!("Arg {idx} (fat ptr) to {:?},{:?}", dest_reg, bank_reg)));
                        if addr != dest_reg {
                            insts.push(AsmInst::Add(dest_reg, addr, Reg::R0));
                        }
                        if bank != bank_reg {
                            insts.push(AsmInst::Add(bank_reg, bank, Reg::R0));
                        }
                    }
                }
            }
        }
        
        // Spill all registers before call (except A0-A3 which hold arguments)
        debug!("  Spilling all registers before call");
        pressure_manager.spill_all();
        let spill_insts = pressure_manager.take_instructions();
        if !spill_insts.is_empty() {
            trace!("  Generated {} spill instructions", spill_insts.len());
        }
        insts.extend(spill_insts);
        
        // Add comment about stack adjustment
        if stack_offset > 0 {
            insts.push(AsmInst::Comment(format!("Pushed {stack_offset} words to stack")));
        }
        debug!("Call args setup complete: {} in registers, {} words on stack, {} total instructions", 
               register_arg_slots.len(), stack_offset, insts.len());
        
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
            insts.push(AsmInst::LI(Reg::Pcb, func_bank as i16));
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
            insts.push(AsmInst::Add(addr_reg, Reg::Rv0, Reg::R0));
            insts.push(AsmInst::Add(bank_reg, Reg::Rv1, Reg::R0));
            
            (insts, (addr_reg, Some(bank_reg)))
        } else {
            // Scalar return in R3
            debug!("Handling scalar return");
            
            let ret_reg = pressure_manager.get_register(naming.ret_val_name());
            insts.extend(pressure_manager.take_instructions());
            
            insts.push(AsmInst::Comment("Get scalar return value".to_string()));
            insts.push(AsmInst::Add(ret_reg, Reg::Rv0, Reg::R0));
            
            (insts, (ret_reg, None))
        }
    }
    
    /// Clean up stack after call
    pub(super) fn cleanup_stack(&self, num_args_words: i16) -> Vec<AsmInst> {
        let mut insts = Vec::new();
        if num_args_words > 0 {
            debug!("Cleaning up {} words from stack after call", num_args_words);
            insts.push(AsmInst::Comment(format!("Clean up {num_args_words} words from stack")));
            insts.push(AsmInst::AddI(Reg::Sp, Reg::Sp, -num_args_words));
            trace!("  Adjusted SP by -{}", num_args_words);
        } else {
            trace!("No stack cleanup needed (0 arguments)");
        }
        insts
    }
    
    /// Load parameter in callee
    /// First 4 scalar parameters are in A0-A3
    /// Additional parameters are on the stack at negative offsets from FP
    /// param_types: The types of all parameters to calculate correct stack offsets
    pub(super) fn load_param(&self, index: usize, 
                     param_types: &[(rcc_common::TempId, crate::ir::IrType)],
                     pressure_manager: &mut RegisterPressureManager,
                     naming: &mut NameGenerator) -> (Vec<AsmInst>, Reg) {
        info!("Loading parameter {}", index);
        let mut insts = Vec::new();
        
        let param_name = naming.param_name(index);
        trace!("  Allocating register for parameter '{}'", param_name);
        let dest = pressure_manager.get_register(param_name);
        
        let spill_insts = pressure_manager.take_instructions();
        if !spill_insts.is_empty() {
            trace!("  Generated {} spill/reload instructions", spill_insts.len());
        }
        insts.extend(spill_insts);
        
        // Use common logic to determine placement
        let (register_params, first_stack_param) = self.analyze_param_placement(param_types);
        
        // Check if this parameter is in a register
        let param_reg = if let Some((_, reg_slot)) = register_params.iter().find(|(i, _)| *i == index) {
            Some(match *reg_slot {
                0 => Reg::A0,
                1 => Reg::A1,
                2 => Reg::A2,
                3 => Reg::A3,
                _ => unreachable!(),
            })
        } else {
            None
        };
        
        if let Some(arg_reg) = param_reg {
            // Parameter is in a register
            debug!("  Parameter {} is in register {:?}", index, arg_reg);
            insts.push(AsmInst::Comment(format!("Load param {index} from {:?}", arg_reg)));
            if dest != arg_reg {
                insts.push(AsmInst::Add(dest, arg_reg, Reg::R0));
            }
        } else {
            // Parameter is on the stack
            // Stack parameters start after the 4 register parameters
            // They are before the frame (negative offsets from FP)
            // Stack layout: ... param6, param5, param4, RA, FP, S0, S1, S2, S3, locals...
            
            // Calculate the actual offset based on parameter types
            // We need to account for fat pointers taking 2 cells
            let mut param_offset = -6i16; // Start at -6 for FP, RA, and S0-S3
            
            // Count stack words before our parameter
            for i in first_stack_param..index {
                if i >= param_types.len() {
                    break;
                }
                if param_types[i].1.is_pointer() {
                    param_offset -= 2; // Fat pointer takes 2 words
                } else {
                    param_offset -= 1; // Scalar takes 1 word
                }
            }
            
            // Account for this parameter itself
            param_offset -= 1;
            
            debug!("  Parameter {} (stack param) is at FP{}", index, param_offset);
            
            insts.push(AsmInst::Comment(format!("Load param {index} from FP{param_offset}")));
            trace!("  Computing address: FP + {}", param_offset);
            insts.push(AsmInst::AddI(Reg::Sc, Reg::Fp, param_offset));
            trace!("  Loading from stack (bank SB) at computed address into {:?}", dest);
            insts.push(AsmInst::Load(dest, Reg::Sb, Reg::Sc));
        }
        
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