use rcc_codegen::{AsmInst, Reg};

pub fn emit_ne(dst: Reg, a: Reg, b: Reg, temp1: Reg, temp2: Reg) -> Vec<AsmInst> {
    // ne = (a<b || b<a)
    // IMPORTANT: temp1 and temp2 must be different from dst, a, and b
    // to avoid overwriting source values before use
    vec![
        AsmInst::Sltu(temp1, a, b),
        AsmInst::Sltu(temp2, b, a),
        AsmInst::Or(dst, temp1, temp2),
    ]
}
