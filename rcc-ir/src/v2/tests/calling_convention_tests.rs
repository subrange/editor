use crate::v2::calling_convention::{CallingConvention, CallArg};
use crate::v2::regalloc::RegAllocV2;
use rcc_codegen::{AsmInst, Reg};

#[test]
fn test_stack_based_scalar_args() {
    let cc = CallingConvention::new();
    let mut allocator = RegAllocV2::new();
    allocator.init_stack_bank();
    
    let args = vec![
        CallArg::Scalar(Reg::R5),
        CallArg::Scalar(Reg::R6),
    ];
    
    let insts = cc.setup_call_args(&mut allocator, args);
    
    // Should push both args to stack
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, Reg::R14))).count();
    assert_eq!(store_count, 2);
    
    // Should increment SP after each push
    let sp_inc_count = insts.iter().filter(|i| matches!(i, AsmInst::AddI(Reg::R14, Reg::R14, 1))).count();
    assert_eq!(sp_inc_count, 2);
}

#[test]
fn test_stack_based_fat_pointer_arg() {
    let cc = CallingConvention::new();
    let mut allocator = RegAllocV2::new();
    allocator.init_stack_bank();
    
    let args = vec![
        CallArg::FatPointer { addr: Reg::R5, bank: Reg::R6 },
    ];
    
    let insts = cc.setup_call_args(&mut allocator, args);
    
    // Fat pointer should push 2 values (bank then addr)
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, Reg::R14))).count();
    assert_eq!(store_count, 2);
    
    // Should increment SP twice
    let sp_inc_count = insts.iter().filter(|i| matches!(i, AsmInst::AddI(Reg::R14, Reg::R14, 1))).count();
    assert_eq!(sp_inc_count, 2);
}

#[test]
fn test_stack_cleanup() {
    let cc = CallingConvention::new();
    
    // Test cleanup of 3 words
    let insts = cc.cleanup_stack(3);
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::R14, Reg::R14, -3))));
    
    // Test no cleanup needed
    let insts = cc.cleanup_stack(0);
    assert_eq!(insts.len(), 0);
}

#[test]
fn test_load_param_from_stack() {
    let cc = CallingConvention::new();
    let mut allocator = RegAllocV2::new();
    allocator.init_stack_bank();
    
    // Load parameter 0 (should be at FP-3)
    let _reg = cc.load_param(0, &mut allocator);
    let insts = allocator.take_instructions();
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::R12, Reg::R15, -3))));
    
    // Load parameter 2 (should be at FP-5)
    let _reg = cc.load_param(2, &mut allocator);
    let insts = allocator.take_instructions();
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::R12, Reg::R15, -5))));
}

#[test]
fn test_return_value_handling() {
    let cc = CallingConvention::new();
    let mut allocator = RegAllocV2::new();
    allocator.init_stack_bank();
    
    // Test scalar return
    let (_ret_reg, bank_reg) = cc.handle_return_value(&mut allocator, false);
    assert!(bank_reg.is_none());
    let insts = allocator.take_instructions();
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(_, Reg::R3, Reg::R0))));
    
    // Test fat pointer return
    let (_addr_reg, bank_reg) = cc.handle_return_value(&mut allocator, true);
    assert!(bank_reg.is_some());
    let insts = allocator.take_instructions();
    // Should copy from both R3 and R4
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(_, Reg::R3, Reg::R0))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Add(_, Reg::R4, Reg::R0))));
}

#[test]
fn test_cross_bank_call() {
    let cc = CallingConvention::new();
    
    // Test in-bank call (bank 0)
    let insts = cc.emit_call(100, 0);
    // Should NOT set PCB for bank 0
    assert!(!insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::PCB, _))));
    // Should have JAL
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Jal(_, 100))));
    
    // Test cross-bank call (bank 3)
    let insts = cc.emit_call(200, 3);
    // Should set PCB to 3
    assert!(insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::PCB, 3))));
    // Should have JAL to address 200
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Jal(_, 200))));
}

#[test]
fn test_multiple_args_pushed_in_order() {
    let cc = CallingConvention::new();
    let mut allocator = RegAllocV2::new();
    allocator.init_stack_bank();
    
    // Create 3 scalar args
    let args = vec![
        CallArg::Scalar(Reg::R5),
        CallArg::Scalar(Reg::R6),
        CallArg::Scalar(Reg::R7),
    ];
    
    let insts = cc.setup_call_args(&mut allocator, args);
    
    // Should push all 3 args to stack
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, Reg::R14))).count();
    assert_eq!(store_count, 3);
}

#[test]
fn test_mixed_args() {
    let cc = CallingConvention::new();
    let mut allocator = RegAllocV2::new();
    allocator.init_stack_bank();
    
    // Mix of scalar and fat pointer args
    let args = vec![
        CallArg::Scalar(Reg::R5),
        CallArg::FatPointer { addr: Reg::R6, bank: Reg::R7 },
        CallArg::Scalar(Reg::R8),
    ];
    
    let insts = cc.setup_call_args(&mut allocator, args);
    
    // Should push 4 values total (1 + 2 + 1)
    let store_count = insts.iter().filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, Reg::R14))).count();
    assert_eq!(store_count, 4);
}