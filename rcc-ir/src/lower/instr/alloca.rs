use rcc_codegen::{AsmInst, Reg};
use rcc_common::TempId;
use crate::{IrType, Value};
use crate::ir::BankTag;
use crate::module_lowering::{FatPtrComponents, Location, ModuleLowerer};

impl ModuleLowerer {
    pub fn lower_alloca(&mut self, result: &TempId, alloc_type: &IrType, count: &Option<Value>) {
        // Stack allocation - allocate space and compute address
        let base_size = self.get_type_size_in_words(alloc_type);

        // If count is provided, multiply size by count (for arrays)
        let total_size = match count {
            Some(Value::Constant(n)) => base_size * (*n as u64),
            _ => base_size,
        };

        // Allocate space on stack by incrementing offset
        // Note: For arrays, we allocate starting position, not ending
        let start_offset = self.local_stack_offset + 1; // Start after current position
        self.local_stack_offset += total_size as i16;
        let offset = start_offset;

        // Store the offset for this temp (frame slot)
        self.local_offsets.insert(*result, offset);


        // Store fat pointer components - alloca produces stack pointers
        self.fat_ptr_components.insert(*result, FatPtrComponents {
            addr_temp: *result,  // The result temp holds the address
            bank_tag: BankTag::Stack,
        });

        // Allocate register for the address
        let result_key = Self::temp_name(*result);
        let addr_reg = self.get_reg(result_key.clone());
        self.value_locations.insert(result_key, Location::Register(addr_reg));

        self.emit(AsmInst::Comment(format!("Alloca for t{} at FP+{} (fat ptr: stack bank)", result, offset)));

        // Address = R15 (FP) + offset
        if offset > 0 {
            self.emit(AsmInst::AddI(addr_reg, Reg::R15, offset));
        } else {
            self.emit(AsmInst::Add(addr_reg, Reg::R15, Reg::R0));
        }
    }
}