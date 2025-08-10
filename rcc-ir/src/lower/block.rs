use rcc_codegen::AsmInst;
use rcc_common::CompilerError;
use crate::{BasicBlock, Instruction};
use crate::module_lowering::ModuleLowerer;

impl ModuleLowerer {
    /// Lower a basic block
    pub(crate) fn lower_basic_block(&mut self, block: &BasicBlock) -> Result<(), CompilerError> {
        // Add label for block if not the first one
        if block.id != 0 {
            // Generate unique label by prefixing with function name
            let func_name = self.current_function.as_ref()
                .map(|f| f.clone())
                .unwrap_or_else(|| "unknown".to_string());
            self.emit(AsmInst::Label(format!("{}_L{}", func_name, block.id)));
        }

        for (idx, instruction) in block.instructions.iter().enumerate() {
            self.lower_instruction(instruction)?;

            // Only free registers at statement boundaries
            // Statement boundaries are between different high-level statements,
            // not between every IR instruction
            // For now, we free after stores, calls, and branches
            match instruction {
                Instruction::Store { .. } |
                Instruction::Call { .. } |
                Instruction::Branch(_) |
                Instruction::BranchCond { .. } |
                Instruction::Return { .. } => {
                    // These mark the end of a statement - free all temps
                    self.free_all();
                }
                _ => {
                    // Keep registers allocated for expression evaluation
                }
            }
        }

        Ok(())
    }
}