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
                        let addr_reg = self.get_reg(format!("load_addr_{}", self.label_counter));
                        self.label_counter += 1;
                        self.emit(AsmInst::LI(addr_reg, addr as i16));
                        
                        // Now allocate the destination register
                        let result_key = Self::temp_name(*result);
                        let dest_reg = self.get_reg(result_key.clone());
                        self.value_locations.insert(result_key, Location::Register(dest_reg));
                        
                        self.emit(AsmInst::Load(dest_reg, Reg::R0, addr_reg));

                        // Load bank tag from next word
                        let bank_addr_reg = self.get_reg(format!("load_bank_addr_{}", self.label_counter));
                        let bank_reg = self.get_reg(format!("load_bank_{}", self.label_counter));
                        self.label_counter += 1;
                        self.emit(AsmInst::LI(bank_addr_reg, (addr + 1) as i16));
                        self.emit(AsmInst::Load(bank_reg, Reg::R0, bank_addr_reg));

                        // Store bank tag for later use
                        let bank_temp_id = 100000 + result;
                        self.value_locations.insert(Self::temp_name(bank_temp_id), Location::Register(bank_reg));
                        self.reg_alloc.mark_in_use(bank_reg, format!("t{}_bank", result));

                        // Set up fat pointer components based on loaded bank
                        // Mark as Unknown since it's runtime-determined
                        
                    } else {
                        // Regular load from global address
                        let addr_reg = self.get_reg(format!("load_addr_{}", self.label_counter));
                        self.label_counter += 1;
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
                let bank = self.get_bank_for_pointer(ptr)?;

                if is_loading_pointer {
                    // Loading a fat pointer - load address and bank tag
                    self.emit(AsmInst::Comment("Loading fat pointer".to_string()));

                    // Pin the ptr_reg to ensure it's not overwritten when allocating dest_reg
                    let ptr_pin_key = format!("ptr_addr_preserve_{}", self.label_counter);
                    self.reg_alloc.mark_in_use(ptr_reg, ptr_pin_key.clone());
                    self.reg_alloc.pin_value(ptr_pin_key.clone());
                    
                    // Allocate destination register AFTER we have ptr_reg and bank
                    let result_key = Self::temp_name(*result);
                    let dest_reg = self.get_reg(result_key.clone());
                    self.value_locations.insert(result_key, Location::Register(dest_reg));
                    
                    // Unpin ptr_reg now that we have dest_reg
                    self.reg_alloc.unpin_value(&ptr_pin_key);

                    // Load address part
                    self.emit(AsmInst::Load(dest_reg, bank.clone(), ptr_reg));

                    // Pin ptr_reg to prevent it from being spilled when allocating next_addr
                    let ptr_pin_key = format!("ptr_load_preserve_{}", self.label_counter);
                    self.reg_alloc.mark_in_use(ptr_reg, ptr_pin_key.clone());
                    self.reg_alloc.pin_value(ptr_pin_key.clone());

                    // Load bank tag from next word
                    let next_addr = self.get_reg(format!("next_addr_{}", self.label_counter));
                    self.emit(AsmInst::AddI(next_addr, ptr_reg, 1));
                    
                    // Unpin ptr_reg now that we're done with it
                    self.reg_alloc.unpin_value(&ptr_pin_key);
                    
                    // Get bank_reg AFTER using next_addr to avoid spilling next_addr before use
                    let bank_reg = self.get_reg(format!("load_bank_{}", self.label_counter));
                    self.label_counter += 1;
                    self.emit(AsmInst::Load(bank_reg, bank.clone(), next_addr));

                    // Store bank tag for later use
                    let bank_temp_id = 100000 + result;
                    self.value_locations.insert(Self::temp_name(bank_temp_id), Location::Register(bank_reg));
                    self.reg_alloc.mark_in_use(bank_reg, format!("t{}_bank", result));

                    // Mark as having unknown bank since it's runtime-determined
                } else {
                    // Regular load
                    // Allocate destination register AFTER we have ptr_reg and bank
                    let result_key = Self::temp_name(*result);
                    let dest_reg = self.get_reg(result_key.clone());
                    self.value_locations.insert(result_key, Location::Register(dest_reg));
                    
                    self.emit(AsmInst::Load(dest_reg, bank, ptr_reg));
                }
                // Registers will be freed at statement boundary
                
            }

            Ok(())
            
        }
    }
}