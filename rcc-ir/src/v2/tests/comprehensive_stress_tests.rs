//! Comprehensive stress tests for V2 backend components
//! 
//! This module contains tricky, edge-case, and stress-testing scenarios
//! to ensure the V2 backend is robust and handles all corner cases correctly.

use crate::v2::calling_convention::{CallingConvention, CallArg};
use crate::v2::regmgmt::RegisterPressureManager;
use crate::v2::function::FunctionLowering;
use rcc_codegen::{AsmInst, Reg};

// ========================================================================
// REGISTER ALLOCATOR STRESS TESTS
// ========================================================================
// Note: The original RegAllocV2 stress tests have been moved to
// regmgmt/tests.rs as they test internal implementation details.

/*
#[test]
fn stress_test_massive_spill_cascade() {
    // Test with 100+ values forcing massive spill cascades
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    let mut allocated = Vec::new();
    
    // Allocate 100 values, forcing 93 spills (only 7 registers available)
    for i in 0..100 {
        let reg = alloc.get_reg(format!("val_{}", i));
        allocated.push((format!("val_{}", i), reg));
    }
    
    // Verify R13 was initialized before first spill
    assert!(alloc.r13_initialized);
    
    // Check that we have many spill instructions
    let insts = alloc.take_instructions();
    let spill_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, _)))
        .count();
    assert!(spill_count >= 93, "Expected at least 93 spills, got {}", spill_count);
}

#[test]
fn stress_test_interleaved_spill_reload() {
    // Test complex patterns of spill/reload with value reuse
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Phase 1: Fill all registers
    for i in 0..7 {
        alloc.get_reg(format!("phase1_{}", i));
    }
    
    // Phase 2: Force spills
    for i in 0..10 {
        alloc.get_reg(format!("phase2_{}", i));
    }
    
    // Verify spills happened
    let insts_after_spill = alloc.take_instructions();
    let spill_count = insts_after_spill.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, _)))
        .count();
    assert!(spill_count >= 10, "Should spill values when out of registers");
    
    // Clear all registers (but spill slots remain)
    alloc.free_temporaries();
    
    // Phase 3: Reload individual spilled values and verify
    // Test that reload generates a load for a spilled value not in registers
    let reg1 = alloc.reload(format!("phase1_0"));
    let insts = alloc.take_instructions();
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::R13, _))),
            "Reloading spilled value should generate load");
    
    // Verify the value is now in a register
    assert!(matches!(reg1, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    
    // Reload same value again - should not generate another load
    let reg2 = alloc.reload(format!("phase1_0"));
    let insts2 = alloc.take_instructions();
    assert_eq!(insts2.len(), 0, "Reloading already-loaded value should not generate instructions");
    assert_eq!(reg1, reg2, "Should return same register for already-loaded value");
}

#[test]
fn stress_test_pinning_exhaustion() {
    // Pin all registers except one, then try to allocate many values
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Allocate and pin 6 registers (leave one free)
    for i in 0..6 {
        alloc.get_reg(format!("pinned_{}", i));
        alloc.pin_value(format!("pinned_{}", i));
    }
    
    // Now only 1 register is spillable - stress test this scenario
    for i in 0..20 {
        alloc.get_reg(format!("unpinned_{}", i));
    }
    
    // Verify pinned values are still in registers
    for i in 0..6 {
        let reg = alloc.reload(format!("pinned_{}", i));
        // Should not generate load instructions for pinned values
        let insts = alloc.take_instructions();
        let has_load = insts.iter().any(|inst| matches!(inst, AsmInst::Load(_, _, _)));
        assert!(!has_load, "Pinned value should still be in register");
    }
}

#[test]
#[should_panic(expected = "No spillable registers")]
fn stress_test_pin_all_registers_panic() {
    // Pin ALL registers and try to allocate - should panic
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Allocate and pin all 7 registers
    for i in 0..7 {
        alloc.get_reg(format!("pinned_{}", i));
        alloc.pin_value(format!("pinned_{}", i));
    }
    
    // This should panic - no spillable registers
    alloc.get_reg("impossible".to_string());
}

#[test]
fn stress_test_bank_tracking_complex() {
    // Test complex bank tracking scenarios
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Create pointers with different bank types
    alloc.set_pointer_bank("global_array".to_string(), BankInfo::Global);
    alloc.set_pointer_bank("stack_array".to_string(), BankInfo::Stack);
    
    // Dynamic bank in register
    let bank_reg = alloc.get_reg("dynamic_bank".to_string());
    alloc.set_pointer_bank("dynamic_ptr".to_string(), BankInfo::Register(bank_reg));
    
    // Verify correct bank registers are returned
    assert_eq!(alloc.get_bank_register("global_array"), Reg::R0);
    assert_eq!(alloc.get_bank_register("stack_array"), Reg::R13);
    assert_eq!(alloc.get_bank_register("dynamic_ptr"), bank_reg);
    
    // Test with spilled dynamic bank
    for i in 0..10 {
        alloc.get_reg(format!("filler_{}", i));
    }
    
    // Reload and check bank is preserved
    let reloaded_bank = alloc.get_bank_register("dynamic_ptr");
    assert!(matches!(reloaded_bank, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
}

#[test]
fn stress_test_parameter_loading_edge_cases() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Test loading many parameters (more than available registers)
    for i in 0..20 {
        let reg = alloc.load_parameter(i);
        assert!(matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    }
    
    // Should have many loads and spills
    let insts = alloc.take_instructions();
    
    // Each parameter load generates AddI + Load
    let load_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Load(_, Reg::R13, _)))
        .count();
    assert_eq!(load_count, 20, "Should have 20 parameter loads");
    
    // Parameters beyond index 6 should cause spills
    let spill_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::R13, _)))
        .count();
    assert!(spill_count >= 13, "Should spill for params beyond available registers");
}

#[test]
fn stress_test_spill_slot_reuse() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    alloc.set_spill_base(10); // Start spill slots at FP+10
    
    // Allocate values and force spills
    for i in 0..20 {
        alloc.get_reg(format!("temp_{}", i));
    }
    
    // Free all temporaries
    alloc.free_temporaries();
    
    // Allocate new set - should reuse registers but not spill slots
    for i in 0..20 {
        alloc.get_reg(format!("new_{}", i));
    }
    
    // Spill slots should continue from where they left off
    let last_spilled = alloc.take_last_spilled();
    if let Some((_, offset)) = last_spilled {
        assert!(offset >= 10, "Spill slots should not be reused");
    }
}
*/

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
    let insts = cc.setup_call_args(&mut pm, args);
    
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
        let setup = cc.setup_call_args(&mut pm, args);
        
        // Make the call
        let call = cc.emit_call(100 + depth as u16, depth as u16 % 4);
        
        // Handle return
        let (insts, (_ret, _)) = cc.handle_return_value(&mut pm, depth % 2 == 0);
        
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

#[test]
fn stress_test_cross_bank_call_patterns() {
    let cc = CallingConvention::new();
    
    // Test all possible bank combinations (0-15 for realistic range)
    for bank in 0u16..16 {
        let insts = cc.emit_call(0x1234, bank);
        
        if bank == 0 {
            // Bank 0 should not set PCB
            assert!(!insts.iter().any(|i| matches!(i, AsmInst::LI(Reg::PCB, _))));
        } else {
            // Other banks should set PCB
            assert!(insts.iter().any(|i| 
                matches!(i, AsmInst::LI(Reg::PCB, b) if *b == bank as i16)
            ));
        }
        
        // All should have JAL
        assert!(insts.iter().any(|i| matches!(i, AsmInst::Jal(_, 0x1234))));
    }
}

#[test]
fn stress_test_parameter_loading_order() {
    // Verify parameters are loaded in correct order from stack
    let cc = CallingConvention::new();
    
    // Load 20 parameters and verify offsets
    for i in 0..20 {
        let mut pm = RegisterPressureManager::new(0);
        pm.init();
        let (insts, _reg) = cc.load_param(i, &mut pm);
        // Instructions are returned directly now
        
        // Should have AddI with correct offset
        let expected_offset = -(i as i16 + 3);
        assert!(insts.iter().any(|inst| 
            matches!(inst, AsmInst::AddI(Reg::R12, Reg::R15, offset) if *offset == expected_offset)
        ), "Parameter {} should be at FP{}", i, expected_offset);
    }
}

#[test]
fn stress_test_mixed_return_patterns() {
    // Test alternating scalar/pointer returns in sequence
    let cc = CallingConvention::new();
    
    for i in 0..20 {
        let is_pointer = i % 2 == 0;
        let mut pm = RegisterPressureManager::new(0);
        pm.init();
        let (insts, (addr_reg, bank_reg)) = cc.handle_return_value(&mut pm, is_pointer);
        
        if is_pointer {
            assert!(bank_reg.is_some(), "Iteration {}: pointer should have bank", i);
        } else {
            assert!(bank_reg.is_none(), "Iteration {}: scalar should not have bank", i);
        }
        
        // Verify registers are allocated
        assert!(matches!(addr_reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
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
    
    // Spill base should be set correctly in pressure manager
    // Note: We can't directly test internal state anymore
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

#[test]
fn stress_test_epilogue_correctness() {
    let mut func = FunctionLowering::new();
    func.pressure_manager.init();
    
    let epilogue = func.emit_epilogue();
    
    // Verify correct order of operations
    let mut found_sp_restore = false;
    let mut found_fp_restore = false;
    let mut found_ra_restore = false;
    let mut found_pcb_restore = false;
    let mut found_jalr = false;
    
    for inst in &epilogue {
        match inst {
            AsmInst::Add(Reg::R14, Reg::R15, Reg::R0) if !found_sp_restore => {
                found_sp_restore = true;
            }
            AsmInst::Load(Reg::R15, Reg::R13, Reg::R14) if found_sp_restore && !found_fp_restore => {
                found_fp_restore = true;
            }
            AsmInst::Load(Reg::RA, Reg::R13, Reg::R14) if found_fp_restore && !found_ra_restore => {
                found_ra_restore = true;
            }
            AsmInst::Add(Reg::PCB, Reg::RAB, Reg::R0) if found_ra_restore && !found_pcb_restore => {
                found_pcb_restore = true;
            }
            AsmInst::Jalr(Reg::R0, Reg::R0, Reg::RA) if found_pcb_restore && !found_jalr => {
                found_jalr = true;
            }
            _ => {}
        }
    }
    
    assert!(found_sp_restore && found_fp_restore && found_ra_restore && 
            found_pcb_restore && found_jalr, 
            "Epilogue must restore state in correct order");
}

// ========================================================================
// INTEGRATION STRESS TESTS
// ========================================================================

#[test]
fn integration_stress_test_full_function() {
    // Test a complete function with prologue, body, calls, and epilogue
    let mut func = FunctionLowering::new();
    let cc = CallingConvention::new();
    
    // Prologue with locals
    let prologue = func.emit_prologue(10);
    assert!(prologue.iter().any(|i| matches!(i, AsmInst::LI(Reg::R13, 1))));
    
    // Load some parameters
    for i in 0..5 {
        func.pressure_manager.load_parameter(i);
    }
    
    // Do some local variable work
    for i in 0..10 {
        let addr = func.get_local_addr(i);
        func.store_local(i, addr);
    }
    
    // Make a function call
    let args = vec![
        CallArg::Scalar(Reg::R5),
        CallArg::FatPointer { addr: Reg::R6, bank: Reg::R7 },
        CallArg::Scalar(Reg::R8),
    ];
    let setup = cc.setup_call_args(&mut func.pressure_manager, args);
    let call = cc.emit_call(0x100, 2);
    let (insts, (_ret_addr, _ret_bank)) = cc.handle_return_value(&mut func.pressure_manager, true);
    func.instructions.extend(insts);
    let cleanup = cc.cleanup_stack(4); // 2 scalars + 1 fat pointer (2 words)
    
    // Return with fat pointer
    let ret = func.emit_return(Some((Reg::R9, Some(Reg::R10))));
    
    // Verify we have a complete function
    assert!(!prologue.is_empty());
    assert!(!setup.is_empty());
    assert!(!call.is_empty());
    assert!(!cleanup.is_empty());
    assert!(!ret.is_empty());
}

#[test]
fn integration_stress_test_recursive_pattern() {
    // Simulate a recursive function pattern
    let mut func = FunctionLowering::new();
    let cc = CallingConvention::new();
    
    // Prologue
    func.emit_prologue(5);
    
    // Load parameters that would be used in recursion
    let param0 = func.pressure_manager.load_parameter(0);
    let param1 = func.pressure_manager.load_parameter(1);
    
    // Simulate recursive call with modified parameters
    let args = vec![
        CallArg::Scalar(param0),
        CallArg::Scalar(param1),
    ];
    
    cc.setup_call_args(&mut func.pressure_manager, args);
    cc.emit_call(0x0, 0); // Call self (same bank)
    let (insts, (ret_val, _)) = cc.handle_return_value(&mut func.pressure_manager, false);
    func.instructions.extend(insts);
    cc.cleanup_stack(2);
    
    // Return the recursive result
    func.emit_return(Some((ret_val, None)));
}

#[test]
fn integration_stress_test_bank_boundaries() {
    // Test operations near bank boundaries
    let mut func = FunctionLowering::new();
    
    // Test with local at maximum offset
    func.emit_prologue(4095); // Near bank boundary
    
    // This should still work correctly
    let addr = func.get_local_addr(4094);
    let store = func.store_local(4094, Reg::R5);
    let load = func.load_local(4094, Reg::R6);
    
    assert!(!store.is_empty());
    assert!(!load.is_empty());
}

// ========================================================================
// CONFORMANCE VERIFICATION TESTS
// ========================================================================
// Note: The tests that directly verify RegAllocV2 behavior have been moved
// to regmgmt/tests.rs. These tests verify behavior through the public API.

#[test]
fn verify_correct_bank_registers() {
    // Verify correct bank registers are used for memory operations
    let mut func = FunctionLowering::new();
    func.emit_prologue(10);
    
    // Local variable access should use R13
    let local_load = func.load_local(5, Reg::R5);
    assert!(local_load.iter().any(|i| 
        matches!(i, AsmInst::Load(_, Reg::R13, _))
    ), "Local loads must use R13 for stack bank");
    
    let local_store = func.store_local(5, Reg::R5);
    assert!(local_store.iter().any(|i| 
        matches!(i, AsmInst::Store(_, Reg::R13, _))
    ), "Local stores must use R13 for stack bank");
}

#[test]
fn verify_fat_pointer_return_convention() {
    // Verify handle_return_value correctly copies FROM R3/R4 after a call
    let cc = CallingConvention::new();
    let mut pm = RegisterPressureManager::new(0);
    pm.init();
    let (insts, (_addr_reg, bank_reg)) = cc.handle_return_value(&mut pm, true);
    
    // Instructions are now returned directly from handle_return_value
    
    // Should copy FROM R3 and R4 to new registers
    assert!(insts.iter().any(|i| 
        matches!(i, AsmInst::Add(_, Reg::R3, Reg::R0))
    ), "Must copy address from R3");
    
    assert!(insts.iter().any(|i| 
        matches!(i, AsmInst::Add(_, Reg::R4, Reg::R0))
    ), "Must copy bank from R4");
    
    // Verify we got both components
    assert!(bank_reg.is_some(), "Fat pointer must have bank component");
}

#[test]
fn verify_pcb_restoration_on_return() {
    // Verify PCB is always restored from RAB before return
    let mut func = FunctionLowering::new();
    func.pressure_manager.init();
    
    let epilogue = func.emit_epilogue();
    
    // Must restore PCB from RAB before JALR
    let pcb_restore_pos = epilogue.iter().position(|i| 
        matches!(i, AsmInst::Add(Reg::PCB, Reg::RAB, Reg::R0))
    );
    
    let jalr_pos = epilogue.iter().position(|i| 
        matches!(i, AsmInst::Jalr(_, _, Reg::RA))
    );
    
    assert!(pcb_restore_pos.is_some() && jalr_pos.is_some(), 
            "Epilogue must restore PCB and have JALR");
    
    assert!(pcb_restore_pos.unwrap() < jalr_pos.unwrap(), 
            "PCB must be restored before JALR");
}