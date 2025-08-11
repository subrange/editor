use crate::module_lowering::{Location, ModuleLowerer};
use crate::Value;
use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, TempId};

impl ModuleLowerer {
    pub(crate) fn lower_call(
        &mut self,
        result: &Option<TempId>,
        function: &Value,
        args: &Vec<Value>,
    ) -> Result<(), CompilerError> {
        // Extract function name from Value
        let func_name = match function {
            Value::Function(name) => name.clone(),
            Value::Global(name) => name.clone(),
            _ => "unknown".to_string(),
        };

        // General function call - set up arguments first
        // Using calling convention: R3-R8 for arguments
        // For fat pointers, we pass address in one register and bank in the next

        // Collect argument registers and their destinations
        let mut arg_regs = Vec::new();
        let mut stack_args = Vec::new(); // Arguments that go on stack
        let mut next_param_reg_idx = 0;

        for arg in args.iter() {
            // Check if we still have registers available
            let use_stack = next_param_reg_idx >= 6;

            // Check if this argument is a pointer that needs fat pointer handling
            let is_pointer = match arg {
                Value::Temp(tid) => {
                    // Check if this temp has fat pointer components
                    self.fat_ptr_components.contains_key(tid)
                        || self.local_offsets.contains_key(tid)
                }
                Value::Global(_) => true, // Globals are always pointers
                Value::FatPtr(_) => true,
                _ => false,
            };

            if is_pointer && !use_stack && next_param_reg_idx < 5 {
                // Need 2 registers for fat pointer
                // Get address register
                let addr_reg = self.get_value_register(arg)?;
                let addr_param_reg = match next_param_reg_idx {
                    0 => Reg::R3,
                    1 => Reg::R4,
                    2 => Reg::R5,
                    3 => Reg::R6,
                    4 => Reg::R7,
                    _ => Reg::R8,
                };
                arg_regs.push((addr_reg, addr_param_reg));
                next_param_reg_idx += 1;

                // Get bank tag and pass it in the next register
                let bank_param_reg = match next_param_reg_idx {
                    1 => Reg::R4,
                    2 => Reg::R5,
                    3 => Reg::R6,
                    4 => Reg::R7,
                    5 => Reg::R8,
                    _ => unreachable!(),
                };

                // Check if we have a loaded pointer with bank tag stored
                let bank_val_reg = if let Value::Temp(tid) = arg {
                    let bank_temp_key = Self::bank_temp_key(*tid);
                    if let Some(&Location::Register(stored_bank_reg)) =
                        self.value_locations.get(&bank_temp_key)
                    {
                        // We have the bank tag value in a register already
                        stored_bank_reg
                    } else {
                        // Fall back to get_bank_for_pointer
                        let bank_reg = self.get_bank_for_pointer(arg)?;
                        if bank_reg == Reg::R0 {
                            // Global bank = 0
                            let temp_name = self.generate_temp_name("bank_global");
                            let temp_reg = self.get_reg(temp_name);
                            self.emit(AsmInst::LI(temp_reg, 0));
                            temp_reg
                        } else if bank_reg == Reg::R13 {
                            // Stack bank = 1
                            let temp_name = self.generate_temp_name("bank_tag_stack");
                            let temp_reg = self.get_reg(temp_name);
                            self.emit(AsmInst::LI(temp_reg, 1));
                            temp_reg
                        } else {
                            bank_reg // Already a value register
                        }
                    }
                } else {
                    // Not a temp, use get_bank_for_pointer
                    let bank_reg = self.get_bank_for_pointer(arg)?;
                    if bank_reg == Reg::R0 {
                        // Global bank = 0
                        let temp_name = self.generate_temp_name("bank_global");
                        let temp_reg = self.get_reg(temp_name);
                        self.emit(AsmInst::LI(temp_reg, 0));
                        temp_reg
                    } else if bank_reg == Reg::R13 {
                        // Stack bank = 1
                        let temp_name = self.generate_temp_name("bank_tag_stack");
                        let temp_reg = self.get_reg(temp_name);
                        self.emit(AsmInst::LI(temp_reg, 1));
                        temp_reg
                    } else {
                        bank_reg // Already a value register
                    }
                };

                arg_regs.push((bank_val_reg, bank_param_reg));
                next_param_reg_idx += 1;
            } else if use_stack {
                // Argument goes on stack
                if is_pointer {
                    // Push fat pointer (2 values) on stack
                    let addr_reg = self.get_value_register(arg)?;
                    stack_args.push(addr_reg);

                    // Get bank tag value
                    let bank_val = if let Value::Temp(tid) = arg {
                        let bank_temp_key = Self::bank_temp_key(*tid);
                        if let Some(&Location::Register(stored_bank_reg)) =
                            self.value_locations.get(&bank_temp_key)
                        {
                            stored_bank_reg
                        } else {
                            // Get bank and convert to value
                            let bank_reg = self.get_bank_for_pointer(arg)?;
                            let temp_name = self.generate_temp_name("bank_val");
                            let temp_reg = self.get_reg(temp_name);
                            if bank_reg == Reg::R0 {
                                self.emit(AsmInst::LI(temp_reg, 0));
                            } else if bank_reg == Reg::R13 {
                                self.emit(AsmInst::LI(temp_reg, 1));
                            } else {
                                self.emit(AsmInst::Add(temp_reg, bank_reg, Reg::R0));
                            }
                            temp_reg
                        }
                    } else {
                        let bank_reg = self.get_bank_for_pointer(arg)?;
                        let temp_name = self.generate_temp_name("bank_val");
                        let temp_reg = self.get_reg(temp_name);
                        if bank_reg == Reg::R0 {
                            self.emit(AsmInst::LI(temp_reg, 0));
                        } else if bank_reg == Reg::R13 {
                            self.emit(AsmInst::LI(temp_reg, 1));
                        } else {
                            self.emit(AsmInst::Add(temp_reg, bank_reg, Reg::R0));
                        }
                        temp_reg
                    };
                    stack_args.push(bank_val);
                } else {
                    // Regular value on stack
                    let arg_reg = self.get_value_register(arg)?;
                    stack_args.push(arg_reg);
                }
            } else {
                // Non-pointer argument in register
                let arg_reg = self.get_value_register(arg)?;
                let param_reg = match next_param_reg_idx {
                    0 => Reg::R3,
                    1 => Reg::R4,
                    2 => Reg::R5,
                    3 => Reg::R6,
                    4 => Reg::R7,
                    5 => Reg::R8,
                    _ => unreachable!(),
                };
                arg_regs.push((arg_reg, param_reg));
                next_param_reg_idx += 1;
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
                let dst_needed = arg_regs
                    .iter()
                    .enumerate()
                    .any(|(j, (s, _))| !moved[j] && j != i && *s == dst);

                if !dst_needed {
                    self.emit(AsmInst::Add(dst, src, Reg::R0));
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
                    self.emit(AsmInst::Add(Reg::R9, dst, Reg::R0));
                    // Now we can move src to dst
                    self.emit(AsmInst::Add(dst, src, Reg::R0));
                    moved[i] = true;
                    // And move R9 to dst2
                    self.emit(AsmInst::Add(dst2, Reg::R9, Reg::R0));
                    moved[j] = true;
                } else {
                    // No conflict, just move
                    self.emit(AsmInst::Add(dst, src, Reg::R0));
                    moved[i] = true;
                }
            }
        }

        // Push stack arguments in reverse order (rightmost first)
        if !stack_args.is_empty() {
            // Adjust stack pointer for arguments
            let stack_space = stack_args.len() as i16;
            self.emit(AsmInst::AddI(Reg::R14, Reg::R14, -stack_space));

            // Push each argument onto the stack
            for (i, arg_reg) in stack_args.iter().enumerate() {
                let offset = i as i16;
                let addr_reg = self.get_reg(format!("stack_arg_addr_{}", i));
                self.emit(AsmInst::AddI(addr_reg, Reg::R14, offset));
                self.emit(AsmInst::Store(*arg_reg, Reg::R13, addr_reg));
            }
        }

        self.emit(AsmInst::Call(func_name));

        // Clean up stack after call
        if !stack_args.is_empty() {
            let stack_space = stack_args.len() as i16;
            self.emit(AsmInst::AddI(Reg::R14, Reg::R14, stack_space));
        }
        
        // After the call, argument registers R3-R8 can be clobbered
        // Clear them from the allocator's tracking
        // Note: R3 will contain the return value if any
        self.reg_alloc.clear_register(Reg::R3);
        self.reg_alloc.clear_register(Reg::R4);
        self.reg_alloc.clear_register(Reg::R5);
        self.reg_alloc.clear_register(Reg::R6);
        self.reg_alloc.clear_register(Reg::R7);
        self.reg_alloc.clear_register(Reg::R8);

        if let Some(dest) = result {
            // Result is in R3 by convention
            let result_key = Self::temp_name(*dest);
            
            // Always allocate a register for the result and move R3 to it
            // This ensures the value is properly tracked in the allocator
            let dest_reg = self.get_reg(result_key.clone());
            
            // Move result from R3 to the allocated register (may be R3 itself if free)
            if dest_reg != Reg::R3 {
                self.emit(AsmInst::Add(dest_reg, Reg::R3, Reg::R0));
            }
            
            // Update our local tracking
            self.value_locations
                .insert(result_key, Location::Register(dest_reg));
        }

        Ok(())
    }

    pub(crate) fn lower_return(&mut self, value: &Option<Value>) -> Result<(), CompilerError> {
        if let Some(val) = value {
            let val_reg = self.get_value_register(val)?;
            // Move return value to return register (R3 by convention)
            if val_reg != Reg::R3 {
                self.emit(AsmInst::Add(Reg::R3, val_reg, Reg::R0));
            }
        }

        // Generate epilogue if this function has a frame
        if self.needs_frame {
            self.generate_epilogue();
        }

        self.emit(AsmInst::Ret);
        
        Ok(())
    }
}
