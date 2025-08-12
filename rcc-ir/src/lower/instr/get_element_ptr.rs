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
                    format!("WARNING: GetElementPtr result t{} overwrites input!", result)
                ));
            }
        }

        // Get base pointer
        let base_reg = self.get_value_register(ptr)?;
        
        // Pin the base register to prevent it from being spilled when we get the index
        let base_pin_key = format!("gep_base_{}", result);
        self.reg_alloc.mark_in_use(base_reg, base_pin_key.clone());
        self.reg_alloc.pin_value(base_pin_key.clone());
        
        self.emit(AsmInst::Comment(
            format!("  Base {} in {}", self.value_to_string(ptr),
                    match base_reg {
                        Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
                        Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
                        _ => "R?"
                    })
        ));

        // For now, we only support single index (1D arrays)
        if indices.len() != 1 {
            return Err(CompilerError::codegen_error(
                format!("Multi-dimensional arrays not yet supported"),
                rcc_common::SourceLocation::new_simple(0, 0),
            ));
        }

        // Get index value
        let index_reg = self.get_value_register(&indices[0])?;
        
        // Unpin the base register now that we have both
        self.reg_alloc.unpin_value(&base_pin_key);

        // Allocate register for result
        let result_key = Self::temp_name(*result);
        let dest_reg = self.get_reg(result_key.clone());
        self.value_locations.insert(result_key, Location::Register(dest_reg));

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
            
            // Check if the base pointer has a bank tag in value_locations
            if let Some(bank_location) = self.value_locations.get(&base_bank_key).cloned() {
                self.emit(AsmInst::Comment(format!("  Propagating bank tag from {} to {}", base_bank_key, result_bank_key)));
                self.value_locations.insert(result_bank_key.clone(), bank_location.clone());
                
                // If the bank is in a register, mark it as in use for the result too
                if let Location::Register(bank_reg) = bank_location {
                    self.reg_alloc.mark_in_use(bank_reg, result_bank_key);
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
            self.emit(AsmInst::Comment(format!("  Global pointer - using global bank")));
        }
        
        Ok(())
    }
}