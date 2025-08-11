use rcc_codegen::AsmInst;
use rcc_common::CompilerError;
use crate::{BasicBlock, Instruction, Value};
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
            // Log the instruction being processed
            self.emit(AsmInst::Comment(format!("=== Processing instruction #{}: {:?} ===", idx, instruction)));
            
            self.lower_instruction(instruction)?;

            // Only free registers at statement boundaries
            // Statement boundaries are between different high-level statements,
            // not between every IR instruction
            match instruction {
                Instruction::Store { value, .. } => {
                    // For stores that immediately follow a value-producing instruction,
                    // we want to preserve the value being stored
                    // Check if this is storing a recently computed value
                    if idx > 0 {
                        if let Some(prev_instr) = block.instructions.get(idx - 1) {
                            match prev_instr {
                                Instruction::Call { result: Some(res), .. } |
                                Instruction::Binary { result: res, .. } |
                                Instruction::Load { result: res, .. } => {
                                    // If we're storing the result of the previous instruction,
                                    // don't free registers yet
                                    if value == &Value::Temp(*res) {
                                        // Keep registers for now
                                        self.emit(AsmInst::Comment(format!(">>> Preserving registers: storing t{} from previous instruction", res)));
                                        continue;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    // Otherwise, free all temps
                    self.emit(AsmInst::Comment(">>> Freeing all registers at Store statement boundary".to_string()));
                    self.free_all();
                }
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