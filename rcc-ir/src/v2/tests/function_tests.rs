use crate::v2::function::FunctionLowering;
use rcc_codegen::{AsmInst, Reg};

#[test]
fn test_prologue_initializes_r13() {
    let mut func = FunctionLowering::new();
    let insts = func.emit_prologue(5);
    
    // Should have LI R13, 1 near the beginning
    assert!(insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::R13, 1))));
    
    // Should save RA and FP
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(Reg::RA, _, _))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(Reg::R15, _, _))));
    
    // Should allocate space
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::R14, Reg::R14, 5))));
}

#[test]
fn test_epilogue_restores_state() {
    let mut func = FunctionLowering::new();
    func.pressure_manager.init(); // Initialize pressure manager
    let insts = func.emit_epilogue();
    
    // Should restore SP, FP, RA
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(Reg::R15, _, _))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(Reg::RA, _, _))));
    
    // Should restore PCB from RAB
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(Reg::PCB, Reg::RAB, Reg::R0))));
    
    // Should return with JALR
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Jalr(_, _, Reg::RA))));
}

#[test]
fn test_fat_pointer_return() {
    let mut func = FunctionLowering::new();
    func.pressure_manager.init();
    
    // Return fat pointer with addr in R5, bank in R6
    let insts = func.emit_return(Some((Reg::R5, Some(Reg::R6))));
    
    // Should move to R3 and R4
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(Reg::R3, Reg::R5, Reg::R0))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(Reg::R4, Reg::R6, Reg::R0))));
}

#[test]
fn test_scalar_return() {
    let mut func = FunctionLowering::new();
    func.pressure_manager.init();
    
    // Return scalar value in R7
    let insts = func.emit_return(Some((Reg::R7, None)));
    
    // Should move to R3 only
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(Reg::R3, Reg::R7, Reg::R0))));
    // Should NOT touch R4
    assert!(!insts.iter().any(|i| matches!(i, AsmInst::Add(Reg::R4, _, _))));
}

#[test]
fn test_local_access() {
    let mut func = FunctionLowering::new();
    func.pressure_manager.init();
    
    // Test load from local
    let load_insts = func.load_local(3, Reg::R5);
    assert!(load_insts.iter().any(|i| matches!(i, AsmInst::Load(Reg::R5, Reg::R13, _))));
    
    // Test store to local
    let store_insts = func.store_local(3, Reg::R5);
    assert!(store_insts.iter().any(|i| matches!(i, AsmInst::Store(Reg::R5, Reg::R13, _))));
}

#[test]
fn test_prologue_with_no_locals() {
    let mut func = FunctionLowering::new();
    let insts = func.emit_prologue(0);
    
    // Should still initialize R13
    assert!(insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::R13, 1))));
    
    // Should save RA and FP
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(Reg::RA, _, _))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(Reg::R15, _, _))));
    
    // Should NOT allocate space
    assert!(!insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::R14, Reg::R14, 0))));
}