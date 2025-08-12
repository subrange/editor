use rcc_codegen::{AsmInst, Reg};
use rcc_common::CompilerError;
use crate::{Instruction, IrType, Value};
use crate::ir::BankTag;
use crate::module_lowering::{Location, ModuleLowerer};
use log::{debug, warn};

pub mod arithmetic;
mod alloca;
mod store;
mod load;
mod binary;
mod func;
mod control_flow;
mod get_element_ptr;
mod inline_asm;

impl ModuleLowerer {
    pub(crate) fn lower_instruction(&mut self, instruction: &Instruction) -> Result<(), CompilerError> {
        match instruction {
            Instruction::Alloca { result, alloc_type, count, .. } => self.lower_alloca(result, alloc_type, count),

            Instruction::Store { value, ptr } => self.lower_store(value, ptr)?,

            Instruction::Load { result, ptr, result_type } => self.lower_load(result, ptr, result_type)?,

            Instruction::Binary { result, op, lhs, rhs, .. } => self.lower_binary(result, op, lhs, rhs)?,
             
            Instruction::Call { result, function, args, .. } => self.lower_call(result, function, args)?,

            Instruction::Return(value) => self.lower_return(value)?,

            Instruction::Branch(target) => self.lower_branch(target)?,

            Instruction::BranchCond { condition, true_label, false_label } => self.lower_branch_cond(condition, true_label, false_label)?,

            Instruction::GetElementPtr { result, ptr, indices, .. } => self.lower_get_element_ptr(result, ptr, indices)?,

            Instruction::InlineAsm { assembly } => {
                for line in assembly.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // For now, pass through as raw assembly
                        // We'll need a way to handle this in AsmInst
                        self.emit(AsmInst::Raw(trimmed.to_string()));
                    }
                }
            }

            _ => {
                self.emit(AsmInst::Comment(format!("Unimplemented: {instruction:?}")));
            }
        }

        Ok(())
    }

    

    /// Get the size of a type in 16-bit words
    pub(crate) fn get_type_size_in_words(&self, ir_type: &IrType) -> u64 {
        match ir_type {
            IrType::Void => 0,
            IrType::I1 => 1, // Boolean takes 1 word
            IrType::I8 | IrType::I16 => 1,
            IrType::I32 | IrType::I64 => 2,
            IrType::FatPtr(_) => 2, // Fat pointers: 2 words (address + bank tag)
            IrType::Array { element_type, size } => {
                let elem_size = self.get_type_size_in_words(element_type);
                elem_size * size
            }
            IrType::Function { .. } => 0,
            IrType::Struct { .. } => 0, // TODO: Calculate struct size
            IrType::Label => 0, // Labels don't have size
        }
    }

    /// Get register for a value
    fn get_value_register(&mut self, value: &Value) -> Result<Reg, CompilerError> {
        self.get_value_register_impl(value)
    }
    
    /// Get register for a value
    fn get_value_register_impl(&mut self, value: &Value) -> Result<Reg, CompilerError> {
        match value {
            Value::Constant(n) => {
                // Use the centralized allocator for constants through the wrapper
                let const_key = self.generate_temp_name(&format!("const_{n}"));
                let reg = self.get_reg(const_key);  // Use wrapper, not direct call!

                self.emit(AsmInst::LI(reg, *n as i16));
                Ok(reg)
            }
            Value::Temp(id) => {
                // Check if this is a stack-allocated variable (from alloca)
                if let Some(&offset) = self.local_offsets.get(id) {
                    // This is a stack variable - return its address
                    // Use a unique key to avoid conflicts
                    let temp_name = self.generate_temp_name(&format!("addr_t{id}"));
                    let reg = self.get_reg(temp_name);
                    if offset > 0 {
                        self.emit(AsmInst::AddI(reg, Reg::Fp, offset));
                    } else {
                        self.emit(AsmInst::Add(reg, Reg::Fp, Reg::R0));
                    }
                    Ok(reg)
                } else {
                    // Regular temp value
                    let key =Self::temp_name(*id);

                    self.emit(AsmInst::Comment(format!("Getting register for temp {key}")));

                    // Use wrapper's reload which handles spill/reload instructions
                    let reg = self.reload(key.clone());

                    let reg_name = match reg {
                        Reg::Rv0 => "R3", Reg::Rv1 => "R4", Reg::A0 => "R5",
                        Reg::A1 => "R6", Reg::A2 => "R7", Reg::A3 => "R8",
                        Reg::X0 => "R9", Reg::X1 => "R10", Reg::X2 => "R11",
                        _ => "R?",
                    };
                    self.emit(AsmInst::Comment(format!("  {key} is now in {reg_name}")));

                    // The wrapper already updates value_locations
                    Ok(reg)
                }
            }
            Value::Function(name) => {
                // Function references not directly loadable
                Err(CompilerError::codegen_error(
                    format!("Cannot load function '{name}' into register"),
                    rcc_common::SourceLocation::new_simple(0, 0),
                ))
            }
            Value::Global(name) => {
                // Load global address into a register
                // IMPORTANT: Global addresses should NEVER be spilled - always load fresh with LI
                if let Some(&addr) = self.global_addresses.get(name) {
                    // Use a non-spillable temporary for global addresses
                    // We generate a unique name each time to ensure it's always loaded fresh
                    let temp_name = self.generate_temp_name(&format!("global_addr_{}", self.label_counter));
                    let reg = self.get_reg(temp_name);
                    self.emit(AsmInst::LI(reg, addr as i16));
                    Ok(reg)
                } else {
                    Err(CompilerError::codegen_error(
                        format!("Undefined global variable '{name}'"),
                        rcc_common::SourceLocation::new_simple(0, 0),
                    ))
                }
            }
            Value::FatPtr(ptr) => {
                // For fat pointers, we only return the address register here
                // The bank is handled separately in load/store operations
                self.get_value_register(&ptr.addr)
            }
            _ => {
                Err(CompilerError::codegen_error(
                    format!("Unsupported value type: {value:?}"),
                    rcc_common::SourceLocation::new_simple(0, 0),
                ))
            }
            
        }
    }

    /// Get the bank register for a pointer value
    fn get_bank_for_pointer(&mut self, value: &Value) -> Result<Reg, CompilerError> {
        match value {
            Value::FatPtr(ptr) => {
                // For fat pointers, return the appropriate bank register
                match ptr.bank {
                    BankTag::Global => Ok(Reg::R0),
                    BankTag::Stack => {
                        // For stack bank, we need a register containing 1
                        let temp_name = self.generate_temp_name("stack_bank");
                        let reg = self.get_reg(temp_name);
                        self.emit(AsmInst::LI(reg, 1));
                        Ok(reg)
                    }
                }
            }
            Value::Temp(tid) => {
                // Check if this is a pointer parameter with a bank register
                let bank_temp_key = Self::bank_temp_key(*tid);
                
                // Check if we have a bank tag for this pointer
                // It might be in a register or might have been spilled
                let has_bank_tag = self.reg_alloc.is_tracked(&bank_temp_key);
                debug!("Checking bank for t{tid}, bank_temp_key={bank_temp_key}, has_bank_tag={has_bank_tag}");
                
                if has_bank_tag {
                    // Reload the bank tag (will get from register if already there, or reload from spill)
                    self.emit(AsmInst::Comment(format!("Getting bank tag for t{tid}")));
                    let bank_reg = self.reload(bank_temp_key.clone());
                    
                    // Pin the bank register so it doesn't get spilled when we allocate result_reg
                    self.reg_alloc.pin_value(bank_temp_key.clone());
                    
                    // We have the bank tag in a register - need to convert to bank register
                    // Generate runtime check to select R0 or R13 based on tag
                    let temp_name = self.generate_temp_name("bank_select");
                    let result_reg = self.get_reg(temp_name);

                    let (stack_label, done_label) = self.generate_bank_labels();

                    // Check if bank_reg == 1 (stack)
                    self.emit_comment("Select bank register based on tag".to_string());
                    
                    self.emit_many(vec![
                        AsmInst::LI(result_reg, 1),
                        AsmInst::Beq(bank_reg, result_reg, stack_label.clone()),

                        // Bank is 0 (global) - use R0
                        AsmInst::Add(result_reg, Reg::R0, Reg::R0),
                        AsmInst::Beq(Reg::R0, Reg::R0, done_label.clone()),

                        // Bank is 1 (stack) - use bank value 1
                        AsmInst::Label(stack_label),
                        AsmInst::LI(result_reg, 1),

                        AsmInst::Label(done_label)
                    ]);
                   
                    // Now safe to unpin the bank tag value
                    self.reg_alloc.unpin_value(&bank_temp_key);

                    // Mark this register as containing the bank to prevent reuse
                    self.reg_alloc.mark_in_use(result_reg, format!("bank_for_t{tid}"));
                    self.value_locations.insert(format!("bank_for_t{tid}"), Location::Register(result_reg));

                    return Ok(result_reg);
                }

                // Check if this temp has fat pointer components
                if let Some(components) = self.fat_ptr_components.get(tid) {
                    match components.bank_tag {
                        BankTag::Global => Ok(Reg::R0),
                        BankTag::Stack => {
                            // For stack bank, we need a register containing 1
                            let temp_name = self.generate_temp_name("stack_bank");
                            let reg = self.get_reg(temp_name);
                            self.emit(AsmInst::LI(reg, 1));
                            Ok(reg)
                        }
                    }
                } else {
                    // Check if it's a direct alloca
                    if self.local_offsets.contains_key(tid) {
                        // Stack-allocated variable, need bank 1
                        let temp_name = self.generate_temp_name("stack_bank");
                        let reg = self.get_reg(temp_name);
                        self.emit(AsmInst::LI(reg, 1));
                        Ok(reg)
                    } else {
                        // This should never happen with properly tracked fat pointers
                        warn!("get_bank_for_pointer called for t{tid}");
                        warn!("  local_offsets: {:?}", self.local_offsets.contains_key(tid));
                        warn!("  fat_ptr_components: {:?}", self.fat_ptr_components.contains_key(tid));
                        warn!("  value_locations contains t{}: {:?}", tid, self.value_locations.contains_key(&format!("t{tid}")));
                        warn!("  value_locations contains t{}: {:?}", 100000 + tid, self.value_locations.contains_key(&format!("t{}", 100000 + tid)));
                        Err(CompilerError::codegen_error(
                            format!("Missing bank information for pointer t{tid}. This is a compiler bug - all pointers should have bank tags."),
                            rcc_common::SourceLocation::dummy(),
                        ))
                    }
                }
            }
            Value::Global(_) => {
                // Global addresses always use global bank
                Ok(Reg::R0)
            }
            _ => {
                // Other values default to global bank
                Ok(Reg::R0)
            }
        }
    }

    /// Calculate the register need for an expression tree (Sethi-Ullman number)
    /// This implements the need() function from the formalized algorithm
    fn calculate_need(&mut self, value: &Value) -> usize {
        // Check cache first
        let value_key = Self::describe_value(value);
        if let Some(&cached_need) = self.need_cache.get(&value_key) {
            return cached_need;
        }

        // Base cases: constants, loads, etc. need 1 register
        let need = match value {
            Value::Constant(_) | Value::Global(_) | Value::Temp(_) | Value::FatPtr(_) => 1,
            Value::Function(_) | Value::Undef => 1,
        };

        // Cache the result
        self.need_cache.insert(value_key, need);
        need
    }
}
