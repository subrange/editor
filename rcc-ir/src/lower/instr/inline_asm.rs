use rcc_codegen::AsmInst;
use rcc_common::CompilerError;
use crate::module_lowering::ModuleLowerer;

impl ModuleLowerer {
    pub(crate) fn lower_inline_asm(&mut self, assembly: &String) -> Result<(), CompilerError> {
        for line in assembly.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                // For now, pass through as raw assembly
                // We'll need a way to handle this in AsmInst
                self.emit(AsmInst::Raw(trimmed.to_string()));
            }
        }
        
        Ok(())
    }
}
