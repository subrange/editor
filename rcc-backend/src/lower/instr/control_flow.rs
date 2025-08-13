use crate::module_lowering::ModuleLowerer;
use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, LabelId};
use crate::Value;

impl ModuleLowerer {
    pub(crate) fn lower_branch(&mut self, target: &LabelId) -> Result<(), CompilerError> {
        // Unconditional jump to label
        // Use BEQ R0, R0, label (always true) as unconditional jump
        let func_name = self
            .current_function.clone()
            .unwrap_or_else(|| "unknown".to_string());

        self.emit(AsmInst::Beq(
            Reg::R0,
            Reg::R0,
            format!("{func_name}_L{target}"),
        ));

        Ok(())
    }

    pub(crate) fn lower_branch_cond(
        &mut self,
        condition: &Value,
        true_label: &LabelId,
        false_label: &LabelId,
    ) -> Result<(), CompilerError> {
        // Get the condition value in a register
        let cond_reg = self.get_value_register(condition)?;

        let func_name = self
            .current_function.clone()
            .unwrap_or_else(|| "unknown".to_string());

        // Branch if condition is non-zero (true)
        // BNE cond_reg, R0, true_label
        self.emit(AsmInst::Bne(
            cond_reg,
            Reg::R0,
            format!("{func_name}_L{true_label}"),
        ));

        // If we fall through (condition was zero/false), jump to false label
        // Use BEQ R0, R0, false_label (always true) as unconditional jump
        self.emit(AsmInst::Beq(
            Reg::R0,
            Reg::R0,
            format!("{func_name}_L{false_label}"),
        ));

        Ok(())
    }
}
