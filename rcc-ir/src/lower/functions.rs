use log::trace;
use rcc_codegen::{AsmInst, Reg};
use rcc_common::CompilerError;
use crate::{Function, Instruction, IrType};
use crate::module_lowering::ModuleLowerer;

impl ModuleLowerer {
    pub(crate) fn lower_function(&mut self, function: &Function) -> Result<(), CompilerError> {
        trace!("=== Lowering function: {} ===", function.name);
        self.current_function = Some(function.name.clone());
        self.value_locations.clear();
        // Old system - no longer used, using reg_alloc instead
        // self.reg_contents.clear();
        // self.free_regs = vec![Reg::R11, Reg::R10, Reg::R9, Reg::R8, Reg::R7, Reg::R6, Reg::R5, Reg::R4, Reg::R3];
        self.reg_alloc.reset(); // Reset the centralized allocator
        self.needs_frame = false;
        self.local_stack_offset = 0; // Reset local stack offset
        self.local_offsets.clear(); // Clear local variable offsets
        self.fat_ptr_components.clear(); // Clear fat pointer components
        self.need_cache.clear(); // Clear need() cache for new function

        // Function label
        self.emit(AsmInst::Comment(format!("Function: {}", function.name)));
        self.emit(AsmInst::Label(function.name.clone()));

        // Map function parameters to their input registers
        // Parameters arrive in R3-R8, extras on stack
        // For fat pointers, address is in one register and bank tag in the next
        // Stack parameters are at positive offsets from FP (after saved registers)
        let mut next_param_reg_idx = 0;
        let mut stack_param_offset = 2; // After saved RA and old FP

        for (param_id, param_type) in function.parameters.iter() {
            if matches!(param_type, IrType::FatPtr(_)) {
                // Pointer parameter - receives as fat pointer (addr, bank)
                if next_param_reg_idx < 5 {
                    // Can fit both parts in registers

                    // Address is in current register
                    let addr_reg = match next_param_reg_idx {
                        0 => Reg::R3,
                        1 => Reg::R4,
                        2 => Reg::R5,
                        3 => Reg::R6,
                        4 => Reg::R7,
                        _ => Reg::R8,
                    };

                    // Map parameter temp ID to its register location (address)
                    self.value_locations.insert(Self::temp_name(*param_id), crate::module_lowering::Location::Register(addr_reg));
                    next_param_reg_idx += 1;

                    // Bank tag is in next register
                    let bank_reg = match next_param_reg_idx {
                        1 => Reg::R4,
                        2 => Reg::R5,
                        3 => Reg::R6,
                        4 => Reg::R7,
                        5 => Reg::R8,
                        _ => unreachable!(),
                    };

                    trace!("DEBUG: Setting up pointer parameter {} - address in {:?}, bank in {:?}", param_id, addr_reg, bank_reg);
                    
                    // Inform the centralized allocator that the address register is in use
                    self.reg_alloc.mark_in_use(addr_reg, Self::temp_name(*param_id));
                    self.reg_alloc.mark_as_parameter(addr_reg);

                    // Store the bank tag in a temporary for later use
                    // We'll need to check the bank value and set up fat pointer components
                    // For fat pointers, we need to use the bank register at runtime
                    // Store it in a temp that we can track
                    let bank_temp_key = Self::bank_temp_key(*param_id);
                    trace!("DEBUG: Marking bank register {:?} with key {}", bank_reg, bank_temp_key);
                    self.value_locations.insert(bank_temp_key.clone(), crate::module_lowering::Location::Register(bank_reg));
                    self.reg_alloc.mark_in_use(bank_reg, bank_temp_key);
                    self.reg_alloc.mark_as_parameter(bank_reg);

                    // The bank is in a register, tracked above
                    // Note: We don't add to fat_ptr_components here because pointer parameters
                    // have their bank tracked separately via bank_temp_key
                    next_param_reg_idx += 1;
                } else if next_param_reg_idx == 5 {
                    // Address in R8, bank on stack
                    self.value_locations.insert(Self::temp_name(*param_id), crate::module_lowering::Location::Register(Reg::R8));
                    self.reg_alloc.mark_in_use(Reg::R8, Self::temp_name(*param_id));
                    self.reg_alloc.mark_as_parameter(Reg::R8);
                    next_param_reg_idx += 1;

                    // Bank tag is on stack - will be loaded by caller
                    let bank_temp_key = Self::bank_temp_key(*param_id);
                    self.value_locations.insert(bank_temp_key, crate::module_lowering::Location::Spilled(stack_param_offset));
                    stack_param_offset += 1;
                    self.needs_frame = true;
                } else {
                    // Both parts on stack
                    self.value_locations.insert(Self::temp_name(*param_id), crate::module_lowering::Location::Spilled(stack_param_offset));
                    stack_param_offset += 1;

                    let bank_temp_key = Self::bank_temp_key(*param_id);
                    self.value_locations.insert(bank_temp_key, crate::module_lowering::Location::Spilled(stack_param_offset));
                    stack_param_offset += 1;
                    self.needs_frame = true;
                }
            } else {
                // Non-pointer parameter
                if next_param_reg_idx < 6 {
                    // Fits in register
                    let param_reg = match next_param_reg_idx {
                        0 => Reg::R3,
                        1 => Reg::R4,
                        2 => Reg::R5,
                        3 => Reg::R6,
                        4 => Reg::R7,
                        5 => Reg::R8,
                        _ => unreachable!(),
                    };

                    self.value_locations.insert(Self::temp_name(*param_id), crate::module_lowering::Location::Register(param_reg));
                    self.reg_alloc.mark_in_use(param_reg, Self::temp_name(*param_id));
                    self.reg_alloc.mark_as_parameter(param_reg);
                    next_param_reg_idx += 1;
                } else {
                    // Goes on stack
                    self.value_locations.insert(Self::temp_name(* param_id), crate::module_lowering::Location::Spilled(stack_param_offset));
                    stack_param_offset += 1;
                    self.needs_frame = true;
                }
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

        // First, process all allocas to determine stack space needed
        // This is necessary to set the spill base offset correctly
        let mut max_stack_offset = 0i16;
        for block in &function.blocks {
            for instruction in &block.instructions {
                if let Instruction::Alloca { result, alloc_type, count, .. } = instruction {
                    // Calculate size for this alloca
                    let base_size = self.get_type_size_in_words(alloc_type);
                    let total_size = match count {
                        Some(crate::Value::Constant(n)) => base_size * (*n as u64),
                        _ => base_size,
                    };
                    
                    // Update the maximum offset needed
                    let start_offset = max_stack_offset + 1;
                    max_stack_offset = start_offset + (total_size as i16) - 1;
                }
            }
        }
        
        // Set the spill base in the register allocator to start after user variables
        self.reg_alloc.set_spill_base(max_stack_offset + 1);
        log::debug!("Stack layout: user variables use FP+1 to FP+{}, spills start at FP+{}", 
                   max_stack_offset, max_stack_offset + 1);
        
        // Store the max stack offset so we can reserve space in prologue
        self.max_stack_offset = max_stack_offset;

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

    fn generate_prologue(&mut self) {
        // The STORE instruction format is: STORE src_reg, bank_reg, addr_reg
        // where addr_reg contains the address to store to
        // R14 is the stack pointer, R13 is the stack bank
        
        // Save RA at current stack pointer location
        self.emit(AsmInst::Store(Reg::RA, Reg::R13, Reg::R14));
        self.emit(AsmInst::AddI(Reg::R14, Reg::R14, 1));

        // Save old FP at new stack pointer location
        self.emit(AsmInst::Store(Reg::R15, Reg::R13, Reg::R14));
        self.emit(AsmInst::AddI(Reg::R14, Reg::R14, 1));

        // Set new FP = SP
        self.emit(AsmInst::Add(Reg::R15, Reg::R14, Reg::R0));
        
        // Reserve space for local variables and potential spills
        // We need to advance the stack pointer to reserve space
        // The register allocator might need additional space for spills
        // We'll be conservative and reserve extra space
        let total_stack_space = self.max_stack_offset + 20; // Extra space for spills
        if total_stack_space > 0 {
            self.emit(AsmInst::AddI(Reg::R14, Reg::R14, total_stack_space));
        }
    }

    /// Generate function epilogue
    pub(crate) fn generate_epilogue(&mut self) {
        // Restore SP to FP (deallocate locals)
        self.emit(AsmInst::Add(Reg::R14, Reg::R15, Reg::R0));

        // Pop old FP
        self.emit(AsmInst::AddI(Reg::R14, Reg::R14, -1));
        self.emit(AsmInst::Load(Reg::R15, Reg::R13, Reg::R14));

        // Pop RA
        self.emit(AsmInst::AddI(Reg::R14, Reg::R14, -1));
        self.emit(AsmInst::Load(Reg::RA, Reg::R13, Reg::R14));
    }

    /// Check if function has any call instructions
    fn function_has_calls(&self, function: &Function) -> bool {
        function.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| {
                matches!(inst, Instruction::Call { .. })
            })
        })
    }
}