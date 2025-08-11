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
            let is_loading_pointer = matches!(result_type, IrType::Ptr(_));

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
                        let bank_name = self.generate_temp_name("load_bank");
                        let bank_addr_reg = self.get_reg(bank_addr_name);
                        let bank_reg = self.get_reg(bank_name);
                        self.emit(AsmInst::LI(bank_addr_reg, (addr + 1) as i16));
                        self.emit(AsmInst::Load(bank_reg, Reg::R0, bank_addr_reg));

                        // Store bank tag for later use
                        let bank_temp_key = Self::bank_temp_key(*result);
                        self.value_locations.insert(bank_temp_key.clone(), Location::Register(bank_reg));
                        self.reg_alloc.mark_in_use(bank_reg, bank_temp_key);

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
                    
                    // Allocate destination register AFTER we have final_ptr_reg and bank
                    let result_key = Self::temp_name(*result);
                    let dest_reg = self.get_reg(result_key.clone());
                    self.value_locations.insert(result_key, Location::Register(dest_reg));

                    // Load address part
                    self.emit(AsmInst::Load(dest_reg, bank.clone(), final_ptr_reg));

                    // Load bank tag from next word
                    let next_addr_name = self.generate_temp_name("next_addr");
                    let next_addr = self.get_reg(next_addr_name);
                    self.emit(AsmInst::AddI(next_addr, final_ptr_reg, 1));
                    
                    // Get bank_reg AFTER using next_addr to avoid spilling next_addr before use
                    let bank_name = self.generate_temp_name("load_bank");
                    let bank_reg = self.get_reg(bank_name);
                    self.emit(AsmInst::Load(bank_reg, bank.clone(), next_addr));

                    // Store bank tag for later use
                    let bank_temp_key = Self::bank_temp_key(*result);
                    self.value_locations.insert(bank_temp_key.clone(), Location::Register(bank_reg));
                    self.reg_alloc.mark_in_use(bank_reg, bank_temp_key);

                    // Mark as having unknown bank since it's runtime-determined
                    
                    // Now safe to unpin the pointer value after LOAD instructions
                    self.reg_alloc.unpin_value(&ptr_value_name);
                } else {
                    // Regular load
                    // Allocate destination register AFTER we have final_ptr_reg and bank
                    let result_key = Self::temp_name(*result);
                    let dest_reg = self.get_reg(result_key.clone());
                    self.value_locations.insert(result_key, Location::Register(dest_reg));
                    
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