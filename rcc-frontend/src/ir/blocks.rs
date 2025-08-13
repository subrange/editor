//! Basic Block Management
//! 
//! Defines basic blocks - sequences of instructions with single entry/exit points.

use rcc_common::LabelId;
use serde::{Deserialize, Serialize};
use crate::ir::Instruction;

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
        self.instructions.last().is_some_and(|instr| {
            matches!(instr, 
                Instruction::Return(_) | 
                Instruction::Branch(_) | 
                Instruction::BranchCond { .. }
            )
        })
    }
}