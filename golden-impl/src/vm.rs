struct Instr {
    opcode: u16,
    operands: [u16; 3],
}

pub struct VM {
    instructions: Vec<Instr>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            instructions: Vec::new(),
        }
    }

}