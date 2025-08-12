use crate::v2::function::calling_convention::{CallingConvention, CallArg};
use crate::v2::regmgmt::RegisterPressureManager;
use crate::v2::naming::new_function_naming;
use rcc_codegen::{AsmInst, Reg};

#[test]
fn test_stack_based_scalar_args() {
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    
    let args = vec![
        CallArg::Scalar(Reg::A0),
        CallArg::Scalar(Reg::A1),
    ];
    
    let mut naming = new_function_naming();
    let insts = cc.setup_call_args(&mut pm, &mut naming, args);
    
    // Should push both args to stack
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, Reg::Sp))).count();
    assert_eq!(store_count, 2);
    
    // Should increment SP after each push
    let sp_inc_count = insts.iter().filter(|i| matches!(i, AsmInst::AddI(Reg::Sp, Reg::Sp, 1))).count();
    assert_eq!(sp_inc_count, 2);
}

#[test]
fn test_stack_based_fat_pointer_arg() {
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    
    let args = vec![
        CallArg::FatPointer { addr: Reg::A0, bank: Reg::A1 },
    ];
    
    let mut naming = new_function_naming();
    let insts = cc.setup_call_args(&mut pm, &mut naming, args);
    
    // Fat pointer should push 2 values (bank then addr)
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, Reg::Sp))).count();
    assert_eq!(store_count, 2);
    
    // Should increment SP twice
    let sp_inc_count = insts.iter().filter(|i| matches!(i, AsmInst::AddI(Reg::Sp, Reg::Sp, 1))).count();
    assert_eq!(sp_inc_count, 2);
}

#[test]
fn test_stack_cleanup() {
    let cc = CallingConvention::new();
    
    // Test cleanup of 3 words
    let insts = cc.cleanup_stack(3);
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::Sp, Reg::Sp, -3))));
    
    // Test no cleanup needed
    let insts = cc.cleanup_stack(0);
    assert_eq!(insts.len(), 0);
}

#[test]
fn test_load_param_from_stack() {
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    
    // Load parameter 0 (should be at FP-3)
    let mut naming = new_function_naming();
    let (insts, _reg) = cc.load_param(0, &mut pm, &mut naming);
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::Sc, Reg::Fp, -3))));
    
    // Load parameter 2 (should be at FP-5)
    let (insts, _reg) = cc.load_param(2, &mut pm, &mut naming);
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::Sc, Reg::Fp, -5))));
}

#[test]
fn test_return_value_handling() {
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    
    // Test scalar return
    let mut naming = new_function_naming();
    let (insts, (_ret_reg, bank_reg)) = cc.handle_return_value(&mut pm, &mut naming, false);
    assert!(bank_reg.is_none());
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(_, Reg::Rv0, Reg::R0))));
    
    // Test fat pointer return
    let (insts, (_addr_reg, bank_reg)) = cc.handle_return_value(&mut pm, &mut naming, true);
    assert!(bank_reg.is_some());
    // Should copy from both R3 and R4
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(_, Reg::Rv0, Reg::R0))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(_, Reg::Rv1, Reg::R0))));
}

#[test]
fn test_cross_bank_call() {
    let cc = CallingConvention::new();
    
    // Test in-bank call (bank 0)
    let insts = cc.emit_call(100, 0);
    // Should NOT set PCB for bank 0
    assert!(!insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::Pcb, _))));
    // Should have JAL
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Jal(_, 100))));
    
    // Test cross-bank call (bank 3)
    let insts = cc.emit_call(200, 3);
    // Should set PCB to 3
    assert!(insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::Pcb, 3))));
    // Should have JAL to address 200
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Jal(_, 200))));
}

#[test]
fn test_multiple_args_pushed_in_order() {
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    
    // Create 3 scalar args
    let args = vec![
        CallArg::Scalar(Reg::A0),
        CallArg::Scalar(Reg::A1),
        CallArg::Scalar(Reg::A2),
    ];
    
    let mut naming = new_function_naming();
    let insts = cc.setup_call_args(&mut pm, &mut naming, args);
    
    // Should push all 3 args to stack
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, Reg::Sp))).count();
    assert_eq!(store_count, 3);
}

#[test]
fn test_mixed_args() {
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    
    // Mix of scalar and fat pointer args
    let args = vec![
        CallArg::Scalar(Reg::A0),
        CallArg::FatPointer { addr: Reg::A1, bank: Reg::A2 },
        CallArg::Scalar(Reg::A3),
    ];
    
    let mut naming = new_function_naming();
    let insts = cc.setup_call_args(&mut pm, &mut naming, args);
    
    // Should push 4 values total (1 + 2 + 1)
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, Reg::Sp))).count();
    assert_eq!(store_count, 4);
}