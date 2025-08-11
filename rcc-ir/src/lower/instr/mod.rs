use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, TempId};
use crate::{Instruction, IrBinaryOp, IrType, Value};
use crate::ir::BankTag;
use crate::lower::instr::arithmetic::emit_ne;
use crate::module_lowering::{Location, ModuleLowerer};

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
                self.emit(AsmInst::Comment(format!("Unimplemented: {:?}", instruction)));
            }
        }

        Ok(())
    }

    

    /// Get the size of a type in 16-bit words
    fn get_type_size_in_words(&self, ir_type: &IrType) -> u64 {
        match ir_type {
            IrType::Void => 0,
            IrType::I1 => 1, // Boolean takes 1 word
            IrType::I8 | IrType::I16 => 1,
            IrType::I32 | IrType::I64 => 2,
            IrType::Ptr(_) => 2, // Fat pointers: 2 words (address + bank tag)
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
                // Use the centralized allocator for constants
                let const_key = format!("const_{}_{}", n, self.label_counter);
                self.label_counter += 1;
                let reg = self.reg_alloc.get_reg(const_key);

                // Append any spill/reload instructions generated
                let instrs = self.reg_alloc.take_instructions();
                self.emit_many(instrs);

                self.emit(AsmInst::LI(reg, *n as i16));
                Ok(reg)
            }
            Value::Temp(id) => {
                // Check if this is a stack-allocated variable (from alloca)
                if let Some(&offset) = self.local_offsets.get(id) {
                    // This is a stack variable - return its address
                    // Use a unique key to avoid conflicts
                    let reg = self.get_reg(format!("addr_t{}_{}", id, self.label_counter));
                    self.label_counter += 1;
                    if offset > 0 {
                        self.emit(AsmInst::AddI(reg, Reg::R15, offset));
                    } else {
                        self.emit(AsmInst::Add(reg, Reg::R15, Reg::R0));
                    }
                    Ok(reg)
                } else {
                    // Regular temp value
                    let key =Self::temp_name(*id);

                    self.emit(AsmInst::Comment(format!("Getting register for temp {}", key)));

                    // Use SimpleRegAlloc's reload which knows about spilled values
                    let reg = self.reg_alloc.reload(key.clone());
                    let instrs = self.reg_alloc.take_instructions();
                    self.emit_many(instrs);

                    let reg_name = match reg {
                        Reg::R3 => "R3", Reg::R4 => "R4", Reg::R5 => "R5",
                        Reg::R6 => "R6", Reg::R7 => "R7", Reg::R8 => "R8",
                        Reg::R9 => "R9", Reg::R10 => "R10", Reg::R11 => "R11",
                        _ => "R?",
                    };
                    self.emit(AsmInst::Comment(format!("  {} is now in {}", key, reg_name)));

                    // Update our tracking
                    self.value_locations.insert(key, Location::Register(reg));
                    Ok(reg)
                }
            }
            Value::Function(name) => {
                // Function references not directly loadable
                Err(CompilerError::codegen_error(
                    format!("Cannot load function '{}' into register", name),
                    rcc_common::SourceLocation::new_simple(0, 0),
                ))
            }
            Value::Global(name) => {
                // Load global address into a register
                if let Some(&addr) = self.global_addresses.get(name) {
                    let reg = self.get_reg(format!("global_{}_{}", name, self.label_counter));
                    self.label_counter += 1;
                    self.emit(AsmInst::LI(reg, addr as i16));
                    Ok(reg)
                } else {
                    Err(CompilerError::codegen_error(
                        format!("Undefined global variable '{}'", name),
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
                    format!("Unsupported value type: {:?}", value),
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
                    BankTag::Stack => Ok(Reg::R13),
                }
            }
            Value::Temp(tid) => {
                // Check if this is a pointer parameter with a bank register
                let bank_temp_id = 100000 + tid;
                let bank_temp_key = Self::temp_name(bank_temp_id);
                
                // Check if we have a bank tag for this pointer
                // It might be in a register or might have been spilled
                let has_bank_tag = self.reg_alloc.is_tracked(&bank_temp_key);
                
                if has_bank_tag {
                    // Reload the bank tag (will get from register if already there, or reload from spill)
                    self.emit(AsmInst::Comment(format!("Getting bank tag for t{}", tid)));
                    let bank_reg = self.reg_alloc.reload(bank_temp_key.clone());
                    let instrs = self.reg_alloc.take_instructions();
                    self.emit_many(instrs);
                    
                    // We have the bank tag in a register - need to convert to bank register
                    // Generate runtime check to select R0 or R13 based on tag
                    let result_reg = self.get_reg(format!("bank_select_{}", self.label_counter));
                    self.label_counter += 1;

                    let stack_label = format!("bank_stack_{}", self.label_counter);
                    let done_label = format!("bank_done_{}", self.label_counter);
                    self.label_counter += 1;

                    // Check if bank_reg == 1 (stack)
                    self.emit_comment("Select bank register based on tag".to_string());
                    
                    self.emit_many(vec![
                        AsmInst::LI(result_reg, 1),
                        AsmInst::Beq(bank_reg, result_reg, stack_label.clone()),

                        // Bank is 0 (global) - use R0
                        AsmInst::Add(result_reg, Reg::R0, Reg::R0),
                        AsmInst::Beq(Reg::R0, Reg::R0, done_label.clone()),

                        // Bank is 1 (stack) - use R13
                        AsmInst::Label(stack_label),
                        AsmInst::Add(result_reg, Reg::R13, Reg::R0),

                        AsmInst::Label(done_label)
                    ]);
                   

                    // Mark this register as containing the bank to prevent reuse
                    self.reg_alloc.mark_in_use(result_reg, format!("bank_for_t{}", tid));
                    self.value_locations.insert(format!("bank_for_t{}", tid), Location::Register(result_reg));

                    return Ok(result_reg);
                }

                // Check if this temp has fat pointer components
                if let Some(components) = self.fat_ptr_components.get(tid) {
                    match components.bank_tag {
                        BankTag::Global => Ok(Reg::R0),
                        BankTag::Stack => Ok(Reg::R13),
                    }
                } else {
                    // Check if it's a direct alloca
                    if self.local_offsets.contains_key(tid) {
                        Ok(Reg::R13)
                    } else {
                        // Default to global for unknown pointers
                        self.emit(AsmInst::Comment(
                            "WARNING: Unknown pointer bank, defaulting to global".to_string()
                        ));
                        Ok(Reg::R0)
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
