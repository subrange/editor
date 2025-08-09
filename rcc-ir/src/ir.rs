//! Full Intermediate Representation for C99
//! 
//! This module defines a comprehensive IR that can represent all C99 constructs.
//! It's designed to be lowered from AST and then compiled to assembly.

use rcc_common::{TempId, LabelId, SymbolId, SourceLocation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// IR Value - represents operands in IR instructions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Temporary variable
    Temp(TempId),
    
    /// Constant integer
    Constant(i64),
    
    /// Global symbol reference
    Global(String),
    
    /// Function reference
    Function(String),
    
    /// Undefined value (for uninitialized variables)
    Undef,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Temp(id) => write!(f, "%{}", id),
            Value::Constant(val) => write!(f, "{}", val),
            Value::Global(name) => write!(f, "@{}", name),
            Value::Function(name) => write!(f, "@{}", name),
            Value::Undef => write!(f, "undef"),
        }
    }
}

/// IR Type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrType {
    /// Void type
    Void,
    
    /// Integer types with bit width
    I1,   // Boolean
    I8,   // 8-bit integer (char)
    I16,  // 16-bit integer (short, int on Ripple)
    I32,  // 32-bit integer (long on Ripple)
    I64,  // 64-bit integer (not supported on Ripple in MVP)
    
    /// Pointer type
    Ptr(Box<IrType>),
    
    /// Array type [size x element_type]
    Array { size: u64, element_type: Box<IrType> },
    
    /// Function type
    Function {
        return_type: Box<IrType>,
        param_types: Vec<IrType>,
        is_vararg: bool,
    },
    
    /// Struct type
    Struct {
        name: Option<String>,
        fields: Vec<IrType>,
        packed: bool,
    },
    
    /// Label type (for basic block addresses)
    Label,
}

impl IrType {
    /// Get the size of this type in bytes
    pub fn size_in_bytes(&self) -> Option<u64> {
        match self {
            IrType::Void => None,
            IrType::I1 => Some(1), // Stored in full byte
            IrType::I8 => Some(1),
            IrType::I16 => Some(2),
            IrType::I32 => Some(4),
            IrType::I64 => Some(8),
            IrType::Ptr(_) => Some(2), // 16-bit pointers on Ripple
            IrType::Array { size, element_type } => {
                element_type.size_in_bytes().map(|elem_size| elem_size * size)
            }
            IrType::Function { .. } => None, // Functions don't have size
            IrType::Struct { fields, .. } => {
                let mut total = 0;
                for field in fields {
                    total += field.size_in_bytes()?;
                }
                Some(total)
            }
            IrType::Label => None,
        }
    }
    
    /// Check if this is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(self, IrType::I1 | IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64)
    }
    
    /// Check if this is a pointer type
    pub fn is_pointer(&self) -> bool {
        matches!(self, IrType::Ptr(_))
    }
    
    /// Get the element type for pointers and arrays
    pub fn element_type(&self) -> Option<&IrType> {
        match self {
            IrType::Ptr(elem) => Some(elem),
            IrType::Array { element_type, .. } => Some(element_type),
            _ => None,
        }
    }
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrType::Void => write!(f, "void"),
            IrType::I1 => write!(f, "i1"),
            IrType::I8 => write!(f, "i8"),
            IrType::I16 => write!(f, "i16"),
            IrType::I32 => write!(f, "i32"),
            IrType::I64 => write!(f, "i64"),
            IrType::Ptr(target) => write!(f, "{}*", target),
            IrType::Array { size, element_type } => write!(f, "[{} x {}]", size, element_type),
            IrType::Function { return_type, param_types, is_vararg } => {
                write!(f, "{} (", return_type)?;
                for (i, param) in param_types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                if *is_vararg { write!(f, ", ...")?; }
                write!(f, ")")
            }
            IrType::Struct { name: Some(name), .. } => write!(f, "%{}", name),
            IrType::Struct { name: None, .. } => write!(f, "%struct"),
            IrType::Label => write!(f, "label"),
        }
    }
}

/// Binary operations in IR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IrBinaryOp {
    // Arithmetic
    Add, Sub, Mul, 
    SDiv, UDiv,    // Signed/unsigned division
    SRem, URem,    // Signed/unsigned remainder
    
    // Bitwise
    And, Or, Xor,
    Shl, LShr, AShr, // Logical/arithmetic shift right
    
    // Comparison (return i1)
    Eq, Ne,
    Slt, Sle, Sgt, Sge, // Signed comparisons
    Ult, Ule, Ugt, Uge, // Unsigned comparisons
}

impl fmt::Display for IrBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            IrBinaryOp::Add => "add",
            IrBinaryOp::Sub => "sub",
            IrBinaryOp::Mul => "mul",
            IrBinaryOp::SDiv => "sdiv",
            IrBinaryOp::UDiv => "udiv",
            IrBinaryOp::SRem => "srem",
            IrBinaryOp::URem => "urem",
            IrBinaryOp::And => "and",
            IrBinaryOp::Or => "or",
            IrBinaryOp::Xor => "xor",
            IrBinaryOp::Shl => "shl",
            IrBinaryOp::LShr => "lshr",
            IrBinaryOp::AShr => "ashr",
            IrBinaryOp::Eq => "eq",
            IrBinaryOp::Ne => "ne",
            IrBinaryOp::Slt => "slt",
            IrBinaryOp::Sle => "sle",
            IrBinaryOp::Sgt => "sgt",
            IrBinaryOp::Sge => "sge",
            IrBinaryOp::Ult => "ult",
            IrBinaryOp::Ule => "ule",
            IrBinaryOp::Ugt => "ugt",
            IrBinaryOp::Uge => "uge",
        };
        write!(f, "{}", op_str)
    }
}

/// Unary operations in IR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IrUnaryOp {
    Not,     // Bitwise NOT
    Neg,     // Arithmetic negation
    ZExt,    // Zero extend
    SExt,    // Sign extend
    Trunc,   // Truncate
    PtrToInt, // Pointer to integer cast
    IntToPtr, // Integer to pointer cast
}

impl fmt::Display for IrUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            IrUnaryOp::Not => "not",
            IrUnaryOp::Neg => "neg",
            IrUnaryOp::ZExt => "zext",
            IrUnaryOp::SExt => "sext",
            IrUnaryOp::Trunc => "trunc",
            IrUnaryOp::PtrToInt => "ptrtoint",
            IrUnaryOp::IntToPtr => "inttoptr",
        };
        write!(f, "{}", op_str)
    }
}

/// IR Instruction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    /// Binary operation: result = op lhs, rhs
    Binary {
        result: TempId,
        op: IrBinaryOp,
        lhs: Value,
        rhs: Value,
        result_type: IrType,
    },
    
    /// Unary operation: result = op operand
    Unary {
        result: TempId,
        op: IrUnaryOp,
        operand: Value,
        result_type: IrType,
    },
    
    /// Load from memory: result = load ptr
    Load {
        result: TempId,
        ptr: Value,
        result_type: IrType,
    },
    
    /// Store to memory: store value, ptr
    Store {
        value: Value,
        ptr: Value,
    },
    
    /// Get element pointer: result = getelementptr ptr, index
    GetElementPtr {
        result: TempId,
        ptr: Value,
        indices: Vec<Value>,
        result_type: IrType,
    },
    
    /// Allocate stack memory: result = alloca type, count
    Alloca {
        result: TempId,
        alloc_type: IrType,
        count: Option<Value>,
        result_type: IrType, // Always pointer type
    },
    
    /// Function call: result = call func(args...)
    Call {
        result: Option<TempId>,
        function: Value,
        args: Vec<Value>,
        result_type: IrType,
    },
    
    /// Return: ret value or ret void
    Return(Option<Value>),
    
    /// Unconditional branch: br label
    Branch(LabelId),
    
    /// Conditional branch: br condition, true_label, false_label
    BranchCond {
        condition: Value,
        true_label: LabelId,
        false_label: LabelId,
    },
    
    /// Phi node: result = phi [val1, label1], [val2, label2], ...
    Phi {
        result: TempId,
        incoming: Vec<(Value, LabelId)>,
        result_type: IrType,
    },
    
    /// Type cast: result = cast value to target_type
    Cast {
        result: TempId,
        value: Value,
        target_type: IrType,
    },
    
    /// Select (ternary): result = select condition, true_value, false_value
    Select {
        result: TempId,
        condition: Value,
        true_value: Value,
        false_value: Value,
        result_type: IrType,
    },
    
    /// Intrinsic function call (for compiler builtins)
    Intrinsic {
        result: Option<TempId>,
        intrinsic: String,
        args: Vec<Value>,
        result_type: IrType,
    },
    
    /// Debug information
    DebugLoc {
        location: SourceLocation,
    },
    
    /// Inline assembly
    InlineAsm {
        assembly: String,
    },
    
    /// Comment (for debugging)
    Comment(String),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Binary { result, op, lhs, rhs, result_type } => {
                write!(f, "%{} = {} {} {}, {}", result, op, result_type, lhs, rhs)
            }
            Instruction::Unary { result, op, operand, result_type } => {
                write!(f, "%{} = {} {} {} to {}", result, op, result_type, operand, result_type)
            }
            Instruction::Load { result, ptr, result_type } => {
                write!(f, "%{} = load {}, {}* {}", result, result_type, result_type, ptr)
            }
            Instruction::Store { value, ptr } => {
                write!(f, "store {}, {}* {}", value, ptr, ptr)
            }
            Instruction::GetElementPtr { result, ptr, indices, result_type } => {
                write!(f, "%{} = getelementptr {}", result, ptr)?;
                for index in indices {
                    write!(f, ", {}", index)?;
                }
                Ok(())
            }
            Instruction::Alloca { result, alloc_type, count, .. } => {
                write!(f, "%{} = alloca {}", result, alloc_type)?;
                if let Some(count) = count {
                    write!(f, ", {}", count)?;
                }
                Ok(())
            }
            Instruction::Call { result, function, args, .. } => {
                if let Some(result) = result {
                    write!(f, "%{} = ", result)?;
                }
                write!(f, "call {}(", function)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Instruction::Return(Some(value)) => write!(f, "ret {}", value),
            Instruction::Return(None) => write!(f, "ret void"),
            Instruction::Branch(label) => write!(f, "br label %{}", label),
            Instruction::BranchCond { condition, true_label, false_label } => {
                write!(f, "br {} {}, label %{}, label %{}", condition, condition, true_label, false_label)
            }
            Instruction::Phi { result, incoming, result_type } => {
                write!(f, "%{} = phi {} ", result, result_type)?;
                for (i, (value, label)) in incoming.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "[{}, %{}]", value, label)?;
                }
                Ok(())
            }
            Instruction::Cast { result, value, target_type } => {
                write!(f, "%{} = cast {} to {}", result, value, target_type)
            }
            Instruction::Select { result, condition, true_value, false_value, result_type } => {
                write!(f, "%{} = select {} {}, {} {}, {} {}", 
                    result, result_type, condition, result_type, true_value, result_type, false_value)
            }
            Instruction::Intrinsic { result, intrinsic, args, .. } => {
                if let Some(result) = result {
                    write!(f, "%{} = ", result)?;
                }
                write!(f, "call @{}(", intrinsic)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Instruction::DebugLoc { location } => {
                write!(f, "!dbg !{}", location.line)
            }
            Instruction::InlineAsm { assembly } => write!(f, "asm \"{}\"", assembly),
            Instruction::Comment(text) => write!(f, "; {}", text),
        }
    }
}

/// Basic Block - a sequence of instructions with a single entry and exit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasicBlock {
    pub id: LabelId,
    pub instructions: Vec<Instruction>,
    pub predecessors: Vec<LabelId>,
    pub successors: Vec<LabelId>,
}

impl BasicBlock {
    pub fn new(id: LabelId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }
    
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }
    
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
    
    pub fn has_terminator(&self) -> bool {
        self.instructions.last().map_or(false, |instr| {
            matches!(instr, 
                Instruction::Return(_) | 
                Instruction::Branch(_) | 
                Instruction::BranchCond { .. }
            )
        })
    }
}

/// Function in IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub return_type: IrType,
    pub parameters: Vec<(TempId, IrType)>,
    pub blocks: Vec<BasicBlock>,
    pub is_external: bool,
    pub is_vararg: bool,
    
    // For code generation
    pub symbol_id: Option<SymbolId>,
    pub source_location: Option<SourceLocation>,
}

impl Function {
    pub fn new(name: String, return_type: IrType) -> Self {
        Self {
            name,
            return_type,
            parameters: Vec::new(),
            blocks: Vec::new(),
            is_external: false,
            is_vararg: false,
            symbol_id: None,
            source_location: None,
        }
    }
    
    pub fn add_parameter(&mut self, param_id: TempId, param_type: IrType) {
        self.parameters.push((param_id, param_type));
    }
    
    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.push(block);
    }
    
    pub fn get_block(&self, id: LabelId) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.id == id)
    }
    
    pub fn get_block_mut(&mut self, id: LabelId) -> Option<&mut BasicBlock> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }
    
    pub fn entry_block(&self) -> Option<&BasicBlock> {
        self.blocks.first()
    }
    
    pub fn entry_block_mut(&mut self) -> Option<&mut BasicBlock> {
        self.blocks.first_mut()
    }
}

/// Global variable definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalVariable {
    pub name: String,
    pub var_type: IrType,
    pub is_constant: bool,
    pub initializer: Option<Value>,
    pub linkage: Linkage,
    pub symbol_id: Option<SymbolId>,
}

/// Linkage types for global symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Linkage {
    External,  // Visible to other modules
    Internal,  // Only visible within this module (static)
    Private,   // Not visible outside this function
}

/// IR Module - represents a complete compilation unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub functions: Vec<Function>,
    pub globals: Vec<GlobalVariable>,
    pub type_definitions: HashMap<String, IrType>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: Vec::new(),
            globals: Vec::new(),
            type_definitions: HashMap::new(),
        }
    }
    
    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }
    
    pub fn add_global(&mut self, global: GlobalVariable) {
        self.globals.push(global);
    }
    
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
    
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut Function> {
        self.functions.iter_mut().find(|f| f.name == name)
    }
    
    pub fn get_global(&self, name: &str) -> Option<&GlobalVariable> {
        self.globals.iter().find(|g| g.name == name)
    }
}

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
    
    pub fn build_alloca(&mut self, alloc_type: IrType, count: Option<Value>) -> Result<TempId, String> {
        let result = self.new_temp();
        let result_type = IrType::Ptr(Box::new(alloc_type.clone()));
        let instr = Instruction::Alloca { result, alloc_type, count, result_type };
        
        self.add_instruction(instr)?;
        Ok(result)
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
        let instr = Instruction::GetElementPtr { 
            result, 
            ptr, 
            indices: vec![offset], 
            result_type: result_type.clone() 
        };
        
        self.add_instruction(instr)?;
        Ok(Value::Temp(result))
    }
    
    pub fn build_inline_asm(&mut self, assembly: String) -> Result<(), String> {
        let instr = Instruction::InlineAsm { assembly };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_types() {
        assert_eq!(IrType::I8.size_in_bytes(), Some(1));
        assert_eq!(IrType::I16.size_in_bytes(), Some(2));
        assert_eq!(IrType::I32.size_in_bytes(), Some(4));
        assert_eq!(IrType::Ptr(Box::new(IrType::I32)).size_in_bytes(), Some(2));
        
        let array_type = IrType::Array {
            size: 10,
            element_type: Box::new(IrType::I16),
        };
        assert_eq!(array_type.size_in_bytes(), Some(20));
    }

    #[test]
    fn test_ir_values() {
        let temp = Value::Temp(5);
        let constant = Value::Constant(42);
        let global = Value::Global("main".to_string());
        
        assert_eq!(format!("{}", temp), "%5");
        assert_eq!(format!("{}", constant), "42");
        assert_eq!(format!("{}", global), "@main");
    }

    #[test]
    fn test_basic_block() {
        let mut block = BasicBlock::new(0);
        assert!(block.is_empty());
        assert!(!block.has_terminator());
        
        block.add_instruction(Instruction::Comment("test".to_string()));
        assert!(!block.is_empty());
        assert!(!block.has_terminator());
        
        block.add_instruction(Instruction::Return(Some(Value::Constant(0))));
        assert!(block.has_terminator());
    }

    #[test]
    fn test_function() {
        let mut function = Function::new("test".to_string(), IrType::I32);
        function.add_parameter(0, IrType::I32);
        function.add_parameter(1, IrType::I32);
        
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.return_type, IrType::I32);
    }

    #[test]
    fn test_ir_builder() {
        let mut builder = IrBuilder::new();
        
        let func = builder.create_function("add".to_string(), IrType::I32);
        func.add_parameter(0, IrType::I32);
        func.add_parameter(1, IrType::I32);
        
        let entry_label = builder.new_label();
        builder.create_block(entry_label).unwrap();
        
        let result = builder.build_binary(
            IrBinaryOp::Add,
            Value::Temp(0),
            Value::Temp(1),
            IrType::I32,
        ).unwrap();
        
        builder.build_return(Some(Value::Temp(result))).unwrap();
        
        let function = builder.finish_function().unwrap();
        assert_eq!(function.name, "add");
        assert_eq!(function.blocks.len(), 1);
        assert!(!function.blocks[0].is_empty());
    }

    #[test]
    fn test_module() {
        let mut module = Module::new("test".to_string());
        
        let function = Function::new("main".to_string(), IrType::I32);
        module.add_function(function);
        
        let global = GlobalVariable {
            name: "global_var".to_string(),
            var_type: IrType::I32,
            is_constant: false,
            initializer: Some(Value::Constant(42)),
            linkage: Linkage::External,
            symbol_id: None,
        };
        module.add_global(global);
        
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.globals.len(), 1);
        assert!(module.get_function("main").is_some());
        assert!(module.get_global("global_var").is_some());
    }
}