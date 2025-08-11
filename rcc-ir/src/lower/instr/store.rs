use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, SourceLocation};
use crate::ir::BankTag;
use crate::module_lowering::ModuleLowerer;
use crate::Value;

impl ModuleLowerer {
    pub(crate) fn lower_store(&mut self, value: &Value , ptr: &Value ) -> Result<(), CompilerError>  {
        // Check if we're storing a pointer value (fat pointer - needs 2 stores)
        let is_pointer_value = match value {
            Value::Temp(tid) => {
                self.fat_ptr_components.contains_key(tid) ||
                    self.local_offsets.contains_key(tid)
            }
            Value::Global(_) => true,
            Value::FatPtr(_) => true,
            _ => false,
        };

        // Check if ptr is a global address
        if let Value::Global(name) = ptr {
            if let Some(&addr) = self.global_addresses.get(name) {
                // Store to global address
                self.emit(AsmInst::Comment(format!("Store {} to @{}",
                                                   self.value_to_string(value), name)));

                if is_pointer_value {
                    // Storing a fat pointer - need to store both address and bank tag
                    let value_reg = self.get_value_register(value)?;
                    let temp_name = self.generate_temp_name("store_addr");
                    let addr_reg = self.get_reg(temp_name);

                    // Store address part
                    self.emit(AsmInst::LI(addr_reg, addr as i16));
                    self.emit(AsmInst::Store(value_reg, Reg::R0, addr_reg));

                    // Get bank tag for the value
                    let bank_tag = match value {
                        Value::Temp(tid) if self.fat_ptr_components.contains_key(tid) => {
                            match self.fat_ptr_components[tid].bank_tag {
                                BankTag::Global => 0,
                                BankTag::Stack => 1,
                            }
                        }
                        Value::Temp(tid) if self.local_offsets.contains_key(tid) => 1, // Stack
                        Value::Global(_) => 0, // Global
                        _ => 0, // Default to global
                    };

                    // Store bank tag at next word
                    let temp_name = self.generate_temp_name("store_bank");
                    let bank_reg = self.get_reg(temp_name);
                    self.emit(AsmInst::LI(bank_reg, bank_tag));
                    self.emit(AsmInst::LI(addr_reg, (addr + 1) as i16));
                    self.emit(AsmInst::Store(bank_reg, Reg::R0, addr_reg));
                    
                    Ok(())
                } else {
                    // Regular value store
                    let value_reg = self.get_value_register(value)?;
                    let temp_name = self.generate_temp_name("store_addr");
                    let addr_reg = self.get_reg(temp_name);
                    self.emit(AsmInst::LI(addr_reg, addr as i16));
                    self.emit(AsmInst::Store(value_reg, Reg::R0, addr_reg));
                    
                    Ok(())
                }
                // Registers will be freed at statement boundary
            } else {
                self.emit(AsmInst::Comment(format!("Store to undefined global @{}", name)));
                Err(CompilerError::CodegenError {location: SourceLocation::dummy(), message: format!("Undefined global variable: {}", name)})
            }
        } else {
            // Get ptr register first
            let ptr_reg = self.get_value_register(ptr)?;
            
            // IMPORTANT: Pin the pointer register to prevent it from being spilled
            // when we call get_bank_for_pointer
            let ptr_pin_key = self.generate_temp_name("ptr_preserve");
            self.reg_alloc.mark_in_use(ptr_reg, ptr_pin_key.clone());
            self.reg_alloc.pin_value(ptr_pin_key.clone());

            // Get the memory bank for the pointer BEFORE getting value register
            // This ensures the bank register is allocated and preserved
            let bank = self.get_bank_for_pointer(ptr)?;

            // If bank is a dynamically selected register, we need to preserve it
            // Check if it's not R0 or R13 (static banks)
            let bank_needs_preservation = bank != Reg::R0 && bank != Reg::R13;
            let preserved_bank = if bank_needs_preservation {
                // The bank was dynamically calculated, preserve it
                bank
            } else {
                bank
            };
            
            // Unpin the pointer register
            self.reg_alloc.unpin_value(&ptr_pin_key);

            if is_pointer_value {
                // Storing a fat pointer - need to store both address and bank tag
                self.emit(AsmInst::Comment(format!("Store fat pointer {} to [{}]",
                                                   self.value_to_string(value), self.value_to_string(ptr))));

                // Pin the bank register to prevent it from being spilled when getting value
                let bank_pin_key = self.generate_temp_name("bank_preserve");
                if bank_needs_preservation {
                    self.reg_alloc.mark_in_use(preserved_bank, bank_pin_key.clone());
                    self.reg_alloc.pin_value(bank_pin_key.clone());
                }
    
                // Get address part of value
                let value_reg = self.get_value_register(value)?;
                
                // Unpin the bank register
                if bank_needs_preservation {
                    self.reg_alloc.unpin_value(&bank_pin_key);
                }

                // Store address
                self.emit(AsmInst::Store(value_reg, preserved_bank.clone(), ptr_reg));

                // Get bank tag for the value
                let value_bank_tag = self.get_bank_for_pointer(value)?;
                let bank_tag_val = if value_bank_tag == Reg::R0 {
                    0 // Global
                } else if value_bank_tag == Reg::R13 {
                    1 // Stack
                } else {
                    // It's a dynamic bank - need to extract the tag value
                    // This is complex, for now default to the tag based on static analysis
                    match value {
                        Value::Temp(tid) if self.fat_ptr_components.contains_key(tid) => {
                            match self.fat_ptr_components[tid].bank_tag {
                                BankTag::Global => 0,
                                BankTag::Stack => 1,
                            }
                        }
                        _ => 0,
                    }
                };

                // Store bank tag at next word
                // Get register for bank value first
                let bank_reg = self.get_reg(format!("store_bank_{}", self.label_counter));
                    self.emit(AsmInst::LI(bank_reg, bank_tag_val));

                // Then calculate the address for next word
                // Do this after loading bank value to avoid register conflicts
                let temp_name = self.generate_temp_name("next_addr");
                let next_addr = self.get_reg(temp_name);
                    self.emit(AsmInst::AddI(next_addr, ptr_reg, 1));

                // Store the bank tag
                self.emit(AsmInst::Store(bank_reg, preserved_bank.clone(), next_addr));
                
                Ok(())
            } else {
                // Regular value store
                self.emit(AsmInst::Comment(format!("Store {} to [{}]",
                                                   self.value_to_string(value), self.value_to_string(ptr))));

                // Pin both ptr_reg and bank to prevent them from being spilled when getting value
                let ptr_pin_key2 = self.generate_temp_name("ptr_preserve2");
                self.reg_alloc.mark_in_use(ptr_reg, ptr_pin_key2.clone());
                self.reg_alloc.pin_value(ptr_pin_key2.clone());
                
                let bank_pin_key = self.generate_temp_name("bank_preserve");
                if bank_needs_preservation {
                    self.reg_alloc.mark_in_use(preserved_bank, bank_pin_key.clone());
                    self.reg_alloc.pin_value(bank_pin_key.clone());
                }
    
                let value_reg = self.get_value_register(value)?;
                
                // Unpin registers
                self.reg_alloc.unpin_value(&ptr_pin_key2);
                if bank_needs_preservation {
                    self.reg_alloc.unpin_value(&bank_pin_key);
                }
                
                self.emit(AsmInst::Store(value_reg, preserved_bank, ptr_reg));
                
                Ok(())
            }
            // Registers will be freed at statement boundary
        }
    }
    
}