use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, TempId};
use crate::module_lowering::{FatPtrComponents, Location, ModuleLowerer};
use crate::Value;

impl ModuleLowerer {
    pub fn lower_get_element_ptr(&mut self, result: &TempId, ptr: &Value, indices: &Vec<Value>) -> Result<(), CompilerError> {
        // Get element pointer - compute address of array element
        self.emit(AsmInst::Comment(format!("GetElementPtr t{} = {} + offsets", result, self.value_to_string(ptr))));

        // Debug check: result should be different from input
        if let Value::Temp(base_tid) = ptr {
            if *base_tid == *result {
                self.emit(AsmInst::Comment(
                    format!("WARNING: GetElementPtr result t{result} overwrites input!")
                ));
            }
        }

        // Get base pointer
        let base_reg = self.get_value_register(ptr)?;
        
        // Pin the base register to prevent it from being spilled when we get the index
        let base_pin_key = format!("gep_base_{result}");
        self.reg_alloc.mark_in_use(base_reg, base_pin_key.clone());
        self.reg_alloc.pin_value(base_pin_key.clone());
        
        self.emit(AsmInst::Comment(
            format!("  Base {} in {}", self.value_to_string(ptr),
                    match base_reg {
                        Reg::Rv0 => "R3", Reg::Rv1 => "R4", Reg::A0 => "R5",
                        Reg::A1 => "R6", Reg::A2 => "R7", Reg::A3 => "R8",
                        _ => "R?"
                    })
        ));

        // For now, we only support single index (1D arrays)
        if indices.len() != 1 {
            return Err(CompilerError::codegen_error(
                "Multi-dimensional arrays not yet supported".to_string(),
                rcc_common::SourceLocation::new_simple(0, 0),
            ));
        }

        // Get index value
        let index_reg = self.get_value_register(&indices[0])?;
        
        // Keep base pinned while we allocate result to prevent it from being spilled
        
        // Pin the index register too to ensure both operands remain valid
        let index_pin_key = format!("gep_index_{result}");
        self.reg_alloc.mark_in_use(index_reg, index_pin_key.clone());
        self.reg_alloc.pin_value(index_pin_key.clone());

        // Allocate register for result - both base and index are pinned so won't be spilled
        let result_key = Self::temp_name(*result);
        let dest_reg = self.get_reg(result_key.clone());
        self.value_locations.insert(result_key, Location::Register(dest_reg));

        // Now safe to unpin both operands
        self.reg_alloc.unpin_value(&base_pin_key);
        self.reg_alloc.unpin_value(&index_pin_key);

        // Calculate address: result = base + index
        // Note: This assumes element size is 1 word. For larger types, we'd need to multiply index by element size
        self.emit(AsmInst::Add(dest_reg, base_reg, index_reg));

        // Propagate pointer provenance from base to result
        // GEP preserves the bank tag - only the address changes
        if let Value::Temp(base_tid) = ptr {
            // Check if base has fat pointer components
            if let Some(base_components) = self.fat_ptr_components.get(base_tid) {
                // Propagate fat pointer components - GEP keeps same bank
                self.fat_ptr_components.insert(*result, FatPtrComponents {
                    addr_temp: *result,  // Result temp holds the new address
                    bank_tag: base_components.bank_tag,  // Keep same bank
                });
            }
            
            // Also propagate the bank tag through the value_locations map
            // This is critical for fat pointers passed as parameters
            let base_bank_key = Self::bank_temp_key(*base_tid);
            let result_bank_key = Self::bank_temp_key(*result);
            
            // Check if the base pointer has a bank tag
            if let Some(bank_location) = self.value_locations.get(&base_bank_key).cloned() {
                self.emit(AsmInst::Comment(format!("  Propagating bank tag from {base_bank_key} to {result_bank_key}")));
                
                // Check if the bank is actually still in a register or has been spilled
                if let Location::Register(bank_reg) = bank_location {
                    // Check if the register still contains the bank value
                    if let Some(current_value) = self.reg_alloc.get_register_value(bank_reg) {
                        if current_value == base_bank_key {
                            // Register still has the bank tag - both can use it
                            self.value_locations.insert(result_bank_key.clone(), Location::Register(bank_reg));
                            self.reg_alloc.mark_in_use(bank_reg, result_bank_key.clone());
                        } else {
                            // Register has been reused! The bank must have been spilled
                            // Check if it was spilled
                            if let Some(spill_slot) = self.reg_alloc.get_spilled_slot(&base_bank_key) {
                                self.emit(AsmInst::Comment(format!("  Bank was spilled to FP+{spill_slot}")));
                                self.value_locations.insert(result_bank_key.clone(), Location::Spilled(spill_slot));
                                // CRITICAL: Also inform the register allocator that the result bank is at this spill slot
                                // This allows the reload mechanism to work correctly
                                self.reg_alloc.record_spilled_value(result_bank_key, spill_slot);
                            } else {
                                // This shouldn't happen - value lost!
                                self.emit(AsmInst::Comment(format!("  WARNING: Bank tag lost for {base_bank_key}")));
                            }
                        }
                    } else {
                        // Register is empty - check if spilled
                        if let Some(spill_slot) = self.reg_alloc.get_spilled_slot(&base_bank_key) {
                            self.value_locations.insert(result_bank_key.clone(), Location::Spilled(spill_slot));
                            // CRITICAL: Also inform the register allocator that the result bank is at this spill slot
                            self.reg_alloc.record_spilled_value(result_bank_key, spill_slot);
                        } else {
                            // Bank was never tracked or lost
                            self.emit(AsmInst::Comment(format!("  WARNING: Bank {base_bank_key} not in register or spilled")));
                        }
                    }
                } else if let Location::Spilled(spill_slot) = bank_location {
                    // Already spilled - propagate the spill location
                    self.value_locations.insert(result_bank_key.clone(), bank_location);
                    // CRITICAL: Also inform the register allocator that the result bank is at this spill slot
                    self.reg_alloc.record_spilled_value(result_bank_key, spill_slot);
                }
            }

        } else if let Value::FatPtr(fat_ptr) = ptr {
            // If the base is already a fat pointer, propagate its bank
            self.fat_ptr_components.insert(*result, FatPtrComponents {
                addr_temp: *result,
                bank_tag: fat_ptr.bank,
            });
        } else if let Value::Global(_) = ptr {
            // Global pointers are in global memory (bank 0)
            self.fat_ptr_components.insert(*result, FatPtrComponents {
                addr_temp: *result,
                bank_tag: crate::ir::BankTag::Global,
            });
            self.emit(AsmInst::Comment("  Global pointer - using global bank".to_string()));
        }
        
        Ok(())
    }
}