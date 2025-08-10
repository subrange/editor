//! Simple IR for M1 Backend Testing
//! 
//! This module defines a minimal intermediate representation that's sufficient. 
//! It will be replaced with a more
//! sophisticated IR if needed

use rcc_common::{TempId, SymbolId};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleIR {
    /// Load constant into temporary: temp = value
    Const(TempId, i16),
    
    /// Binary arithmetic: dest = src1 + src2
    Add(TempId, TempId, TempId),
    Sub(TempId, TempId, TempId),
    Mul(TempId, TempId, TempId),
    Div(TempId, TempId, TempId),
    
    /// Memory operations: store temp to memory[bank][addr]
    /// For simple testing, bank and addr are also temporaries
    Store(TempId, TempId, TempId),
    Load(TempId, TempId, TempId),
    
    /// Function operations
    Call(String),
    Return(Option<TempId>),
    
    /// Control flow
    Label(String),
    Jump(String),
    JumpIfZero(TempId, String),
    JumpIfNotZero(TempId, String),
    
    /// Comments for debugging
    Comment(String),
}

impl fmt::Display for SimpleIR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleIR::Const(temp, value) => write!(f, "t{} = {}", temp, value),
            SimpleIR::Add(dest, src1, src2) => write!(f, "t{} = t{} + t{}", dest, src1, src2),
            SimpleIR::Sub(dest, src1, src2) => write!(f, "t{} = t{} - t{}", dest, src1, src2),
            SimpleIR::Mul(dest, src1, src2) => write!(f, "t{} = t{} * t{}", dest, src1, src2),
            SimpleIR::Div(dest, src1, src2) => write!(f, "t{} = t{} / t{}", dest, src1, src2),
            SimpleIR::Store(value, bank, addr) => write!(f, "store t{} to [t{}][t{}]", value, bank, addr),
            SimpleIR::Load(dest, bank, addr) => write!(f, "t{} = load [t{}][t{}]", dest, bank, addr),
            SimpleIR::Call(name) => write!(f, "call {}", name),
            SimpleIR::Return(Some(temp)) => write!(f, "return t{}", temp),
            SimpleIR::Return(None) => write!(f, "return"),
            SimpleIR::Label(name) => write!(f, "{}:", name),
            SimpleIR::Jump(label) => write!(f, "jump {}", label),
            SimpleIR::JumpIfZero(temp, label) => write!(f, "jump {} if t{} == 0", label, temp),
            SimpleIR::JumpIfNotZero(temp, label) => write!(f, "jump {} if t{} != 0", label, temp),
            SimpleIR::Comment(text) => write!(f, "; {}", text),
        }
    }
}

/// A simple program consisting of IR instructions
#[derive(Debug, Clone)]
pub struct SimpleProgram {
    pub instructions: Vec<SimpleIR>,
    pub next_temp_id: TempId,
    pub functions: Vec<Function>,
}

impl SimpleProgram {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            next_temp_id: 0,
            functions: Vec::new(),
        }
    }
    
    /// Generate a new temporary ID
    pub fn new_temp(&mut self) -> TempId {
        let temp = self.next_temp_id;
        self.next_temp_id += 1;
        temp
    }
    
    /// Add an instruction
    pub fn push(&mut self, instr: SimpleIR) {
        self.instructions.push(instr);
    }
    
    /// Add multiple instructions
    pub fn extend(&mut self, instrs: Vec<SimpleIR>) {
        self.instructions.extend(instrs);
    }
    
    /// Display the program
    pub fn display(&self) -> String {
        self.instructions
            .iter()
            .map(|instr| format!("{}", instr))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for SimpleProgram {
    fn default() -> Self {
        Self::new()
    }
}

/// Function definition in simple IR
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<TempId>,
    pub locals: Vec<TempId>,
    pub instructions: Vec<SimpleIR>,
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name,
            params: Vec::new(),
            locals: Vec::new(),
            instructions: Vec::new(),
        }
    }
    
    pub fn with_params(mut self, params: Vec<TempId>) -> Self {
        self.params = params;
        self
    }
    
    pub fn add_instruction(&mut self, instr: SimpleIR) {
        self.instructions.push(instr);
    }
}

/// Builder for creating simple IR programs
pub struct SimpleIRBuilder {
    program: SimpleProgram,
}

impl SimpleIRBuilder {
    pub fn new() -> Self {
        Self {
            program: SimpleProgram::new(),
        }
    }
    
    /// Add a constant
    pub fn const_val(&mut self, value: i16) -> TempId {
        let temp = self.program.new_temp();
        self.program.push(SimpleIR::Const(temp, value));
        temp
    }
    
    /// Add binary operation
    pub fn add(&mut self, src1: TempId, src2: TempId) -> TempId {
        let dest = self.program.new_temp();
        self.program.push(SimpleIR::Add(dest, src1, src2));
        dest
    }
    
    pub fn sub(&mut self, src1: TempId, src2: TempId) -> TempId {
        let dest = self.program.new_temp();
        self.program.push(SimpleIR::Sub(dest, src1, src2));
        dest
    }
    
    pub fn mul(&mut self, src1: TempId, src2: TempId) -> TempId {
        let dest = self.program.new_temp();
        self.program.push(SimpleIR::Mul(dest, src1, src2));
        dest
    }
    
    /// Store to memory (for putchar: store char to [0][0])
    pub fn store(&mut self, value: TempId, bank: TempId, addr: TempId) {
        self.program.push(SimpleIR::Store(value, bank, addr));
    }
    
    /// Load from memory
    pub fn load(&mut self, bank: TempId, addr: TempId) -> TempId {
        let dest = self.program.new_temp();
        self.program.push(SimpleIR::Load(dest, bank, addr));
        dest
    }
    
    /// Add label
    pub fn label(&mut self, name: &str) {
        self.program.push(SimpleIR::Label(name.to_string()));
    }
    
    /// Add function call
    pub fn call(&mut self, name: &str) {
        self.program.push(SimpleIR::Call(name.to_string()));
    }
    
    /// Add return
    pub fn return_val(&mut self, temp: Option<TempId>) {
        self.program.push(SimpleIR::Return(temp));
    }
    
    /// Add comment
    pub fn comment(&mut self, text: &str) {
        self.program.push(SimpleIR::Comment(text.to_string()));
    }
    
    /// Build the program
    pub fn build(self) -> SimpleProgram {
        self.program
    }
}

impl Default for SimpleIRBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_ir_display() {
        let instrs = vec![
            SimpleIR::Const(0, 42),
            SimpleIR::Const(1, 10),
            SimpleIR::Add(2, 0, 1),
            SimpleIR::Label("end".to_string()),
            SimpleIR::Return(Some(2)),
        ];
        
        for instr in &instrs {
            println!("{}", instr);
        }
        
        assert_eq!(format!("{}", instrs[0]), "t0 = 42");
        assert_eq!(format!("{}", instrs[2]), "t2 = t0 + t1");
        assert_eq!(format!("{}", instrs[3]), "end:");
    }

    #[test]
    fn test_simple_program() {
        let mut program = SimpleProgram::new();
        
        let temp1 = program.new_temp();
        let temp2 = program.new_temp();
        let temp3 = program.new_temp();
        
        program.push(SimpleIR::Const(temp1, 5));
        program.push(SimpleIR::Const(temp2, 10));
        program.push(SimpleIR::Add(temp3, temp1, temp2));
        
        assert_eq!(program.instructions.len(), 3);
        assert_eq!(program.next_temp_id, 3);
        
        let display = program.display();
        assert!(display.contains("t0 = 5"));
        assert!(display.contains("t1 = 10"));
        assert!(display.contains("t2 = t0 + t1"));
    }

    #[test]
    fn test_ir_builder() {
        let mut builder = SimpleIRBuilder::new();
        
        builder.comment("Test program");
        let temp1 = builder.const_val(42);
        let temp2 = builder.const_val(8);
        let result = builder.add(temp1, temp2);
        
        builder.label("output");
        let zero = builder.const_val(0);
        builder.store(result, zero, zero);
        
        let program = builder.build();
        assert_eq!(program.instructions.len(), 7);
    }

    #[test]
    fn test_hello_world_ir() {
        let mut builder = SimpleIRBuilder::new();
        
        builder.comment("Hello World in Simple IR");
        builder.label("main");
        
        // Output 'H'
        let h_char = builder.const_val('H' as i16);
        let zero = builder.const_val(0);
        builder.store(h_char, zero, zero);
        
        // Output 'i'
        let i_char = builder.const_val('i' as i16);
        builder.store(i_char, zero, zero);
        
        builder.return_val(None);
        
        let program = builder.build();
        println!("Hello World IR:\n{}", program.display());
        
        // Verify the structure
        assert!(program.instructions.iter().any(|instr| matches!(instr, SimpleIR::Label(l) if l == "main")));
        assert!(program.instructions.iter().any(|instr| matches!(instr, SimpleIR::Const(_, 72)))); // 'H'
        assert!(program.instructions.iter().any(|instr| matches!(instr, SimpleIR::Const(_, 105)))); // 'i'
    }
}