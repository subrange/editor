use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, TempId};
use crate::module_lowering::{Location, ModuleLowerer};
use crate::{IrType, Value};

impl ModuleLowerer {
    pub(crate) fn lower_load(&mut self, result: &TempId, ptr: &Value, result_type: &IrType) -> Result<(), CompilerError> {
        {
            self.emit(AsmInst::Comment(format!("Load from [{}] to t{}",
                                               self.value_to_string(ptr), result)));

            // Check if we're loading a pointer type
            let is_loading_pointer = matches!(result_type, IrType::FatPtr(_));

            // Check if ptr is a global address
            if let Value::Global(name) = ptr {
                if let Some(&addr) = self.global_addresses.get(name) {
                    if is_loading_pointer {
                        // Loading a fat pointer from global - load address and bank tag
                        self.emit(AsmInst::Comment("Loading fat pointer from global".to_string()));

                        // Load address part
                        let temp_name = self.generate_temp_name("load_addr");
                        let addr_reg = self.get_reg(temp_name);
                        self.emit(AsmInst::LI(addr_reg, addr as i16));
                        
                        // Now allocate the destination register
                        let result_key = Self::temp_name(*result);
                        let dest_reg = self.get_reg(result_key.clone());
                        self.value_locations.insert(result_key, Location::Register(dest_reg));
                        
                        self.emit(AsmInst::Load(dest_reg, Reg::R0, addr_reg));

                        // Load bank tag from next word
                        let bank_addr_name = self.generate_temp_name("load_bank_addr");
                        let bank_temp_key = Self::bank_temp_key(*result);
                        let bank_addr_reg = self.get_reg(bank_addr_name);
                        let bank_reg = self.get_reg(bank_temp_key.clone());
                        self.emit(AsmInst::LI(bank_addr_reg, (addr + 1) as i16));
                        self.emit(AsmInst::Load(bank_reg, Reg::R0, bank_addr_reg));

                        // Store bank tag location for later use
                        // Note: bank_reg is already marked as in use with bank_temp_key from get_reg above
                        self.value_locations.insert(bank_temp_key.clone(), Location::Register(bank_reg));

                        // Set up fat pointer components based on loaded bank
                        // Mark as Unknown since it's runtime-determined
                        
                    } else {
                        // Regular load from global address
                        let temp_name = self.generate_temp_name("load_addr");
                        let addr_reg = self.get_reg(temp_name);
                        self.emit(AsmInst::LI(addr_reg, addr as i16));
                        
                        // Now allocate the destination register
                        let result_key = Self::temp_name(*result);
                        let dest_reg = self.get_reg(result_key.clone());
                        self.value_locations.insert(result_key, Location::Register(dest_reg));
                        
                        self.emit(AsmInst::Load(dest_reg, Reg::R0, addr_reg));
                    }
                    // Registers will be freed at statement boundary
                } else {
                    // Uninitialized global
                    self.emit(AsmInst::Comment(format!("Load from @{} (uninitialized)", name)));
                    
                    // Allocate the destination register
                    let result_key = Self::temp_name(*result);
                    let dest_reg = self.get_reg(result_key.clone());
                    self.value_locations.insert(result_key, Location::Register(dest_reg));
                    
                    self.emit(AsmInst::LI(dest_reg, 0));
                    
                }
                
            } else {
                // Get pointer address and bank
                let ptr_reg = self.get_value_register(ptr)?;
                
                // Pin ptr_reg to prevent it from being spilled during get_bank_for_pointer
                // Get the actual value name that's in the register
                let ptr_value_name = if let Some(name) = self.reg_alloc.get_register_value(ptr_reg) {
                    name
                } else {
                    // This shouldn't happen, but handle it gracefully
                    format!("ptr_reg_{}", self.label_counter)
                };
                
                self.emit(AsmInst::Comment(format!("Pinning {} in register to prevent spilling", ptr_value_name)));
                self.reg_alloc.pin_value(ptr_value_name.clone());
                let bank = self.get_bank_for_pointer(ptr)?;
                // Don't unpin yet - we still need ptr_reg for the LOAD instruction!
                
                // ptr_reg should still be valid now since we pinned it
                let final_ptr_reg = ptr_reg;

                if is_loading_pointer {
                    // Loading a fat pointer - load address and bank tag
                    self.emit(AsmInst::Comment("Loading fat pointer".to_string()));

                    // Note: ptr_value_name is already pinned from above
                    
                    // Pin the bank register before allocating dest_reg
                    let bank_value = self.reg_alloc.get_register_value(bank);
                    if let Some(bank_name) = bank_value {
                        self.reg_alloc.pin_value(bank_name.clone());
                    }
                    
                    // Allocate destination register AFTER we have final_ptr_reg and bank
                    let result_key = Self::temp_name(*result);
                    let dest_reg = self.get_reg(result_key.clone());
                    self.value_locations.insert(result_key, Location::Register(dest_reg));

                    // Unpin the bank register
                    if let Some(bank_name) = self.reg_alloc.get_register_value(bank) {
                        self.reg_alloc.unpin_value(&bank_name);
                    }

                    // Load address part
                    self.emit(AsmInst::Load(dest_reg, bank.clone(), final_ptr_reg));

                    // Capture the original bank value BEFORE any allocations
                    let original_bank_value = self.reg_alloc.get_register_value(bank);
                    
                    // Load bank tag from next word
                    let next_addr_name = self.generate_temp_name("next_addr");
                    let next_addr = self.get_reg(next_addr_name.clone());
                    self.emit(AsmInst::AddI(next_addr, final_ptr_reg, 1));
                    
                    // Pin next_addr to prevent it from being spilled
                    self.reg_alloc.pin_value(next_addr_name.clone());
                    
                    // FIRST: Get the bank register for the load operation
                    // The bank register tells us which memory bank the fat pointer is stored in
                    // We need to do this BEFORE allocating bank_reg to avoid register conflicts
                    self.emit(AsmInst::Comment(format!("Checking bank register status for loading bank tag")));
                    let load_bank = if let Some(ref bname) = original_bank_value.clone() {
                        self.emit(AsmInst::Comment(format!("Bank value was: {}", bname)));
                        // Check if the bank register is still valid
                        if self.reg_alloc.get_register_value(bank) == Some(bname.clone()) {
                            // Still valid, use it
                            self.emit(AsmInst::Comment(format!("Bank register still valid in {:?}", bank)));
                            bank
                        } else {
                            // Was spilled, try to reload it  
                            // Note: bname contains a bank value (0 or 1), not a bank register
                            self.emit(AsmInst::Comment(format!("Bank register was spilled, reloading {}", bname)));
                            
                            // Pin the bname so it doesn't get spilled again when we allocate bank_reg
                            self.reg_alloc.pin_value(bname.clone());
                            
                            let reloaded = self.reg_alloc.reload(bname.clone());
                            let instructions = self.reg_alloc.take_instructions();
                            self.emit_many(instructions);
                            self.emit(AsmInst::Comment(format!("Reloaded bank value to {:?}", reloaded)));
                            
                            // Keep it pinned until after the LOAD
                            // It will be unpinned after line 172
                            
                            // The reloaded value is the bank tag (0 or 1), use it directly
                            reloaded
                        }
                    } else {
                        // No bank value tracked, we need to regenerate it
                        // Since we're loading from a pointer on the stack, it's bank 1
                        self.emit(AsmInst::Comment(format!("No bank value tracked, generating stack bank")));
                        let temp_bank_name = self.generate_temp_name("stack_bank_load");
                        let temp_bank = self.get_reg(temp_bank_name);
                        self.emit(AsmInst::LI(temp_bank, 1));
                        temp_bank
                    };
                    
                    // NOW allocate bank_reg for the result, after we have load_bank secured
                    // IMPORTANT: Use the bank_temp_key directly so the register is properly tracked
                    let bank_temp_key = Self::bank_temp_key(*result);
                    let bank_reg = self.get_reg(bank_temp_key.clone());
                    
                    // Unpin next_addr
                    self.reg_alloc.unpin_value(&next_addr_name);
                    
                    self.emit(AsmInst::Load(bank_reg, load_bank, next_addr));
                    
                    // Unpin the bank value if we had to reload it
                    if let Some(ref bname) = original_bank_value {
                        if self.reg_alloc.get_register_value(load_bank) == Some(bname.clone()) {
                            self.reg_alloc.unpin_value(bname);
                        }
                    }

                    // Store bank tag location for later use
                    // Note: bank_reg is already marked as in use with bank_temp_key from get_reg above
                    self.value_locations.insert(bank_temp_key.clone(), Location::Register(bank_reg));

                    // Now safe to unpin the pointer value after LOAD instructions
                    self.reg_alloc.unpin_value(&ptr_value_name);
                } else {
                    // Regular load
                    // Pin the bank register before allocating dest_reg
                    let bank_value = self.reg_alloc.get_register_value(bank);
                    if let Some(bank_name) = bank_value {
                        self.reg_alloc.pin_value(bank_name.clone());
                    }
                    
                    // Allocate destination register AFTER we have final_ptr_reg and bank
                    let result_key = Self::temp_name(*result);
                    let dest_reg = self.get_reg(result_key.clone());
                    self.value_locations.insert(result_key, Location::Register(dest_reg));
                    
                    // Unpin the bank register
                    if let Some(bank_name) = self.reg_alloc.get_register_value(bank) {
                        self.reg_alloc.unpin_value(&bank_name);
                    }
                    
                    self.emit(AsmInst::Load(dest_reg, bank, final_ptr_reg));
                    
                    // Now safe to unpin the pointer value after LOAD instruction
                    self.reg_alloc.unpin_value(&ptr_value_name);
                }
                // Registers will be freed at statement boundary
                
            }

            Ok(())
            
        }
    }
}