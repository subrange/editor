//! Stress tests for function internals
//! 
//! These tests have access to internal modules for thorough testing

use super::super::calling_convention::{CallingConvention, CallArg};
use super::super::lowering::FunctionLowering;
use crate::v2::regmgmt::RegisterPressureManager;
use crate::v2::naming::new_function_naming;
use rcc_codegen::{AsmInst, Reg};

// ========================================================================
// CALLING CONVENTION STRESS TESTS
// ========================================================================

#[test]
fn stress_test_massive_argument_list() {
    // Test with 50+ arguments of mixed types
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    
    let mut args = Vec::new();
    let mut expected_stack_words = 0;
    
    for i in 0..50 {
        if i % 3 == 0 {
            // Every third arg is a fat pointer
            args.push(CallArg::FatPointer { 
                addr: Reg::R5, 
                bank: Reg::R6 
            });
            expected_stack_words += 2;
        } else {
            args.push(CallArg::Scalar(Reg::R5));
            expected_stack_words += 1;
        }
    }
    let mut naming = new_function_naming();
    let insts = cc.setup_call_args(&mut pm, &mut naming, args);
    
    // Count stores to verify all args pushed
    let store_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, Reg::R14)))
        .count();
    assert_eq!(store_count, expected_stack_words as usize);
    
    // Verify cleanup
    let cleanup = cc.cleanup_stack(expected_stack_words);
    assert!(cleanup.iter().any(|i| 
        matches!(i, AsmInst::AddI(Reg::R14, Reg::R14, n) if *n == -expected_stack_words)
    ));
}

#[test]
fn stress_test_nested_calls() {
    // Simulate deeply nested function calls
    let cc = CallingConvention::new();
    
    // Track stack depth
    let mut stack_adjustments = Vec::new();
    
    for depth in 0..10 {
        // Setup args for this call level
        let args = vec![
            CallArg::Scalar(Reg::R5),
            CallArg::FatPointer { addr: Reg::R6, bank: Reg::R7 },
        ];
        
        let mut pm = RegisterPressureManager::new(0);
        pm.init();
        let mut naming = new_function_naming();
        let setup = cc.setup_call_args(&mut pm, &mut naming, args);
        
        // Make the call
        let call = cc.emit_call(100 + depth as u16, depth as u16 % 4);
        
        // Handle return
        let (insts, (_ret, _)) = cc.handle_return_value(&mut pm, &mut naming, depth % 2 == 0);
        
        // Cleanup
        let cleanup = cc.cleanup_stack(3); // 1 scalar + 2 for fat pointer
        
        stack_adjustments.push((setup, call, cleanup));
    }
    
    // Verify each level has proper setup/cleanup
    for (setup, call, cleanup) in stack_adjustments {
        assert!(!setup.is_empty());
        assert!(!call.is_empty());
        assert!(!cleanup.is_empty());
    }
}

// ========================================================================
// FUNCTION LOWERING STRESS TESTS
// ========================================================================

#[test]
fn stress_test_huge_stack_frame() {
    // Test with massive local allocation
    let mut func = FunctionLowering::new();
    let insts = func.emit_prologue(1000);
    
    // Should initialize R13
    assert!(insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::R13, 1))));
    
    // Should allocate 1000 slots
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::R14, Reg::R14, 1000))));
}

#[test]
fn stress_test_many_local_accesses() {
    // Test accessing many different local variables
    let mut func = FunctionLowering::new();
    func.pressure_manager.init();
    
    // Access 100 different locals
    for i in 0..100 {
        let addr = func.get_local_addr(i);
        
        // Load from local
        let load_insts = func.load_local(i, Reg::R5);
        assert!(load_insts.iter().any(|inst| 
            matches!(inst, AsmInst::Load(Reg::R5, Reg::R13, _))
        ));
        
        // Store to local
        let store_insts = func.store_local(i, Reg::R6);
        assert!(store_insts.iter().any(|inst| 
            matches!(inst, AsmInst::Store(Reg::R6, Reg::R13, _))
        ));
    }
}

#[test]
fn stress_test_complex_return_scenarios() {
    let mut func = FunctionLowering::new();
    func.pressure_manager.init();
    
    // Test 1: Return with value already in R3
    let insts1 = func.emit_return(Some((Reg::R3, None)));
    // Should not generate move to R3
    let moves_to_r3 = insts1.iter()
        .filter(|i| matches!(i, AsmInst::Add(Reg::R3, _, Reg::R0)))
        .count();
    assert_eq!(moves_to_r3, 0, "Should not move R3 to R3");
    
    // Test 2: Fat pointer already in R3/R4
    let mut func2 = FunctionLowering::new();
    func2.pressure_manager.init();
    let insts2 = func2.emit_return(Some((Reg::R3, Some(Reg::R4))));
    let moves = insts2.iter()
        .filter(|i| matches!(i, AsmInst::Add(Reg::R3, _, Reg::R0)) || 
                    matches!(i, AsmInst::Add(Reg::R4, _, Reg::R0)))
        .count();
    assert_eq!(moves, 0, "Should not move R3/R4 to themselves");
    
    // Test 3: Values in high registers
    let mut func3 = FunctionLowering::new();
    func3.pressure_manager.init();
    let insts3 = func3.emit_return(Some((Reg::R11, Some(Reg::R10))));
    assert!(insts3.iter().any(|i| matches!(i, AsmInst::Add(Reg::R3, Reg::R11, Reg::R0))));
    assert!(insts3.iter().any(|i| matches!(i, AsmInst::Add(Reg::R4, Reg::R10, Reg::R0))));
}