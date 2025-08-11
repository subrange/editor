use rcc_codegen::{AsmInst, Reg};

pub fn emit_ne(dst: Reg, a: Reg, b: Reg, temp1: Reg, temp2: Reg) -> Vec<AsmInst> {
    // eq = !(a<b || b<a)
    vec![
        AsmInst::Sltu(temp1, a, b),
        AsmInst::Sltu(temp2, b, a),
        AsmInst::Or(dst, temp1, temp2),
    ]
}
