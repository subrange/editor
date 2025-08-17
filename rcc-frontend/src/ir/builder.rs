//! IR Builder
//! 
//! Provides utilities for constructing IR programmatically.

use rcc_common::{TempId, LabelId};
use crate::BankTag;
use crate::ir::{
    Value, IrType, FatPointer,
    IrBinaryOp, Instruction,
    BasicBlock, Function
};

/// Builder for constructing IR
pub struct IrBuilder {
    current_function: Option<Function>,
    current_block: Option<LabelId>,
    next_temp_id: TempId,
    next_label_id: LabelId,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            current_function: None,
            current_block: None,
            next_temp_id: 0,
            next_label_id: 0,
        }
    }
    
    pub fn new_temp(&mut self) -> TempId {
        let temp = self.next_temp_id;
        self.next_temp_id += 1;
        temp
    }
    
    pub fn new_label(&mut self) -> LabelId {
        let label = self.next_label_id;
        self.next_label_id += 1;
        label
    }
    
    pub fn create_function(&mut self, name: String, return_type: IrType) -> &mut Function {
        let function = Function::new(name, return_type);
        self.current_function = Some(function);
        // Reset temp counter for new function
        self.next_temp_id = 0;
        self.current_function.as_mut().unwrap()
    }
    
    pub fn add_parameter(&mut self, param_id: TempId, param_type: IrType) {
        if let Some(ref mut function) = self.current_function {
            function.add_parameter(param_id, param_type);
            // Update next_temp_id to avoid conflicts with parameter IDs
            if param_id >= self.next_temp_id {
                self.next_temp_id = param_id + 1;
            }
        }
    }
    
    pub fn create_block(&mut self, label_id: LabelId) -> Result<&mut BasicBlock, String> {
        let block = BasicBlock::new(label_id);
        
        if let Some(ref mut function) = self.current_function {
            function.add_block(block);
            self.current_block = Some(label_id);
            Ok(function.get_block_mut(label_id).unwrap())
        } else {
            Err("No current function".to_string())
        }
    }
    
    pub fn build_binary(&mut self, op: IrBinaryOp, lhs: Value, rhs: Value, result_type: IrType) -> Result<TempId, String> {
        let result = self.new_temp();
        let instr = Instruction::Binary { result, op, lhs, rhs, result_type };
        
        self.add_instruction(instr)?;
        Ok(result)
    }
    
    pub fn build_load(&mut self, ptr: Value, result_type: IrType) -> Result<TempId, String> {
        let result = self.new_temp();
        let instr = Instruction::Load { result, ptr, result_type };
        
        self.add_instruction(instr)?;
        Ok(result)
    }
    
    pub fn build_store(&mut self, value: Value, ptr: Value) -> Result<(), String> {
        let instr = Instruction::Store { value, ptr };
        self.add_instruction(instr)
    }
    
    pub fn build_alloca(&mut self, alloc_type: IrType, count: Option<Value>) -> Result<Value, String> {
        let result = self.new_temp();
        let result_type = IrType::FatPtr(Box::new(alloc_type.clone()));
        let instr = Instruction::Alloca { result, alloc_type, count, result_type };
        
        self.add_instruction(instr)?;
        // Alloca always returns a stack pointer
        Ok(Value::FatPtr(FatPointer {
            addr: Box::new(Value::Temp(result)),
            bank: BankTag::Stack,
        }))
    }
    
    pub fn build_call(&mut self, function: Value, args: Vec<Value>, result_type: IrType) -> Result<Option<TempId>, String> {
        let result = if matches!(result_type, IrType::Void) {
            None
        } else {
            Some(self.new_temp())
        };
        
        let instr = Instruction::Call { result, function, args, result_type };
        
        self.add_instruction(instr)?;
        Ok(result)
    }
    
    pub fn build_return(&mut self, value: Option<Value>) -> Result<(), String> {
        let instr = Instruction::Return(value);
        self.add_instruction(instr)
    }
    
    pub fn build_branch(&mut self, label: LabelId) -> Result<(), String> {
        let instr = Instruction::Branch(label);
        self.add_instruction(instr)
    }
    
    pub fn build_branch_cond(&mut self, condition: Value, true_label: LabelId, false_label: LabelId) -> Result<(), String> {
        let instr = Instruction::BranchCond { condition, true_label, false_label };
        self.add_instruction(instr)
    }
    
    pub fn build_pointer_offset(&mut self, ptr: Value, offset: Value, result_type: IrType) -> Result<Value, String> {
        let result = self.new_temp();
        
        // Extract bank from input pointer if it's a fat pointer
        let bank = if let Value::FatPtr(ref fat_ptr) = ptr {
            fat_ptr.bank
        } else {
            let func_name = self.current_function.as_ref().map(|f| f.name.clone()).unwrap_or_else(|| "unknown".to_string());
            return Err(format!("Pointer must be a fat pointer in function '{func_name}', got: {ptr:?}"));
        };
        
        let instr = Instruction::GetElementPtr { 
            result, 
            ptr: ptr.clone(), 
            indices: vec![offset], 
            result_type: result_type.clone(),
        };
        
        self.add_instruction(instr)?;
        
        // Return a fat pointer
        Ok(Value::FatPtr(FatPointer {
            addr: Box::new(Value::Temp(result)),
            bank,
        }))
    }
    
    pub fn build_pointer_offset_with_bank(&mut self, ptr: Value, offset: Value, result_type: IrType, bank: Option<BankTag>) -> Result<Value, String> {
        let result = self.new_temp();
        
        // Use provided bank or extract from input pointer
        let actual_bank = bank.unwrap_or({
            if let Value::FatPtr(ref fat_ptr) = ptr {
                fat_ptr.bank
            } else {
                BankTag::Stack // Default to stack
            }
        });
        
        let instr = Instruction::GetElementPtr { 
            result, 
            ptr: ptr.clone(), 
            indices: vec![offset], 
            result_type: result_type.clone(),
        };
        
        self.add_instruction(instr)?;
        
        // Return a fat pointer
        Ok(Value::FatPtr(FatPointer {
            addr: Box::new(Value::Temp(result)),
            bank: actual_bank,
        }))
    }
    
    pub fn build_inline_asm(&mut self, assembly: String) -> Result<(), String> {
        // Simple version - no operands
        let instr = Instruction::InlineAsm { 
            assembly,
            outputs: Vec::new(),
            inputs: Vec::new(),
            clobbers: Vec::new(),
        };
        self.add_instruction(instr)
    }
    
    pub fn build_inline_asm_extended(
        &mut self, 
        assembly: String,
        outputs: Vec<crate::ir::instructions::AsmOperandIR>,
        inputs: Vec<crate::ir::instructions::AsmOperandIR>,
        clobbers: Vec<String>,
    ) -> Result<(), String> {
        let instr = Instruction::InlineAsm {
            assembly,
            outputs,
            inputs,
            clobbers,
        };
        self.add_instruction(instr)
    }
    
    fn add_instruction(&mut self, instr: Instruction) -> Result<(), String> {
        if let Some(ref mut function) = self.current_function {
            if let Some(block_id) = self.current_block {
                if let Some(block) = function.get_block_mut(block_id) {
                    block.add_instruction(instr);
                    Ok(())
                } else {
                    Err("Current block not found".to_string())
                }
            } else {
                Err("No current block".to_string())
            }
        } else {
            Err("No current function".to_string())
        }
    }
    
    pub fn current_block_has_terminator(&self) -> bool {
        if let Some(ref function) = self.current_function {
            if let Some(block_id) = self.current_block {
                if let Some(block) = function.get_block(block_id) {
                    return block.has_terminator();
                }
            }
        }
        false
    }
    
    pub fn finish_function(&mut self) -> Option<Function> {
        self.current_function.take()
    }
}

impl Default for IrBuilder {
    fn default() -> Self {
        Self::new()
    }
}