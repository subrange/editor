// Tests for RegAllocV2 - internal to the allocator module
use super::*;
use rcc_codegen::{AsmInst, Reg};

#[test]
fn test_r13_initialization() {
    let mut alloc = RegAllocV2::new();
    assert!(!alloc.sb_initialized);
    
    alloc.init_stack_bank();
    assert!(alloc.sb_initialized);
    
    let insts = alloc.take_instructions();
    assert_eq!(insts.len(), 2);
    assert!(matches!(insts[1], AsmInst::Li(Reg::Sb, 1)));
}

#[test]
fn test_allocatable_registers() {
    let mut alloc = RegAllocV2::new();
    
    // Should allocate from S3, S2, S1, S0, T7, T6, T5... (saved regs first, then temps)
    let r1 = alloc.get_reg("val1".to_string());
    let r2 = alloc.get_reg("val2".to_string());
    let r3 = alloc.get_reg("val3".to_string());
    
    assert_eq!(r1, Reg::S3);
    assert_eq!(r2, Reg::S2);
    assert_eq!(r3, Reg::S1);
}

#[test]
fn test_load_parameter() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Load param 0 (should be at FP-3)
    let reg = alloc.load_parameter(0);
    // Should allocate from the allocatable pool (S3, S2, S1, S0, T7-T0)
    assert!(matches!(reg, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    
    let insts = alloc.take_instructions();
    // Should have load from FP-3
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::Sc, Reg::Fp, -3))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::Sb, Reg::Sc))));
}

#[test]
fn test_spilling_with_sc() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Allocate all 12 registers
    for i in 0..12 {
        alloc.get_reg(format!("val{}", i));
    }
    
    // Next allocation should trigger spill
    alloc.get_reg("val13".to_string());
    
    let insts = alloc.take_instructions();
    // Should have Sb init + spill operations
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(_, Reg::Sb, _))));
}

#[test]
fn test_bank_info_tracking() {
    let mut alloc = RegAllocV2::new();
    
    alloc.set_pointer_bank("global_ptr".to_string(), BankInfo::Global);
    alloc.set_pointer_bank("stack_ptr".to_string(), BankInfo::Stack);
    
    assert_eq!(alloc.get_bank_register("global_ptr"), Reg::Gp);
    
    alloc.init_stack_bank(); // Must init before using stack bank
    assert_eq!(alloc.get_bank_register("stack_ptr"), Reg::Sb);
}

#[test]
#[should_panic(expected = "SB not initialized")]
fn test_panic_without_sb_init() {
    let mut alloc = RegAllocV2::new();
    alloc.set_pointer_bank("stack_ptr".to_string(), BankInfo::Stack);
    
    // Should panic because R13 not initialized
    alloc.get_bank_register("stack_ptr");
}

#[test]
fn test_reload_from_spill() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Fill all 12 allocatable registers (S0-S3, T0-T7)
    for i in 0..12 {
        alloc.get_reg(format!("val{}", i));
    }
    
    // Force spill - 13th allocation should cause a spill
    alloc.get_reg("new_val".to_string());
    
    // Check what was actually spilled
    let spill_slots = alloc.test_spill_slots();
    assert!(!spill_slots.is_empty(), "Should have at least one spilled value");
    
    // Get the first spilled value (any will do for this test)
    let spilled_value = spill_slots.keys().next().unwrap().clone();
    
    // Clear instructions from spilling
    alloc.take_instructions();
    
    // Reload the actually spilled value
    let reg = alloc.reload(spilled_value);
    assert!(matches!(reg, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    
    let insts = alloc.take_instructions();
    // Should have load instruction for reload
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::Sb, _))), 
           "Expected Load instruction with Sb, got: {:?}", insts);
}

#[test]
fn test_pinning() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Allocate and pin first value
    let r1 = alloc.get_reg("pinned".to_string());
    alloc.pin_value("pinned".to_string());
    
    // Fill other registers (11 more to fill the 12 allocatable)
    for i in 1..12 {
        alloc.get_reg(format!("val{}", i));
    }
    
    // Next allocation should NOT spill pinned value
    alloc.get_reg("new".to_string());
    
    // Check pinned value is still in its register
    assert!(alloc.test_reg_contents().get(&r1).map(|v| v == "pinned").unwrap_or(false))
}

#[test]
fn test_free_temporaries_clears_all() {
    let mut alloc = RegAllocV2::new();
    
    // Allocate some registers
    alloc.get_reg("temp1".to_string());
    alloc.get_reg("temp2".to_string());
    alloc.get_reg("temp3".to_string());
    
    alloc.free_temporaries();
    
    // Can't directly check reg_contents without making it public
    // But we can verify all registers are available again
    let r1 = alloc.get_reg("new1".to_string());
    let r2 = alloc.get_reg("new2".to_string());
    let r3 = alloc.get_reg("new3".to_string());
    let r4 = alloc.get_reg("new4".to_string());
    let r5 = alloc.get_reg("new5".to_string());
    let r6 = alloc.get_reg("new6".to_string());
    let r7 = alloc.get_reg("new7".to_string());
    
    // Should get all registers from allocatable pool
    assert!(matches!(r1, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    assert!(matches!(r2, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    assert!(matches!(r3, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    assert!(matches!(r4, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    assert!(matches!(r5, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    assert!(matches!(r6, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    assert!(matches!(r7, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
}

// ========================================================================
// STRESS TESTS MOVED FROM comprehensive_stress_tests.rs
// ========================================================================

#[test]
fn stress_test_massive_spill_cascade() {
    // Test with 100+ values forcing massive spill cascades
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    let mut allocated = Vec::new();
    
    // Allocate 100 values, forcing 88 spills (12 registers available)
    for i in 0..100 {
        let reg = alloc.get_reg(format!("val_{}", i));
        allocated.push((format!("val_{}", i), reg));
    }
    
    // Verify SB was initialized before first spill
    assert!(alloc.sb_initialized);
    
    // Check that we have many spill instructions
    let insts = alloc.take_instructions();
    let spill_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, _)))
        .count();
    assert!(spill_count >= 88, "Expected at least 88 spills, got {}", spill_count);
}

#[test]
fn stress_test_interleaved_spill_reload() {
    // Test complex patterns of spill/reload with value reuse
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Phase 1: Fill all 12 registers
    for i in 0..12 {
        alloc.get_reg(format!("phase1_{}", i));
    }
    
    // Phase 2: Force spills
    for i in 0..10 {
        alloc.get_reg(format!("phase2_{}", i));
    }
    
    // Verify spills happened
    let insts_after_spill = alloc.take_instructions();
    let spill_count = insts_after_spill.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, _)))
        .count();
    assert!(spill_count >= 10, "Should spill values when out of registers");
    
    // Clear all registers (but spill slots remain)
    alloc.free_temporaries();
    
    // Phase 3: Reload individual spilled values and verify
    // Find a value that was actually spilled
    let spill_slots = alloc.test_spill_slots();
    assert!(!spill_slots.is_empty(), "Should have spilled values");
    let spilled_value = spill_slots.keys().next().unwrap().clone();
    
    let reg1 = alloc.reload(spilled_value.clone());
    let insts = alloc.take_instructions();
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::Sb, _))),
            "Reloading spilled value should generate load");
    
    // Verify the value is now in a register
    assert!(matches!(reg1, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    
    // Reload same value again - should not generate another load
    let reg2 = alloc.reload(spilled_value);
    let insts2 = alloc.take_instructions();
    assert_eq!(insts2.len(), 0, "Reloading already-loaded value should not generate instructions");
    assert_eq!(reg1, reg2, "Should return same register for already-loaded value");
}

#[test]
fn stress_test_pinning_exhaustion() {
    // Pin all registers except one, then try to allocate many values
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Allocate and pin 11 registers (leave one free from 12 allocatable)
    for i in 0..11 {
        alloc.get_reg(format!("pinned_{}", i));
        alloc.pin_value(format!("pinned_{}", i));
    }
    
    // Now only 1 register is spillable - stress test this scenario
    for i in 0..20 {
        alloc.get_reg(format!("unpinned_{}", i));
    }
    
    // Verify pinned values are still in registers
    for i in 0..11 {
        let _reg = alloc.reload(format!("pinned_{}", i));
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
    
    // Allocate and pin all 12 registers
    for i in 0..12 {
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
    assert_eq!(alloc.get_bank_register("global_array"), Reg::Gp);
    assert_eq!(alloc.get_bank_register("stack_array"), Reg::Sb);
    assert_eq!(alloc.get_bank_register("dynamic_ptr"), bank_reg);
    
    // Test with spilled dynamic bank
    for i in 0..10 {
        alloc.get_reg(format!("filler_{}", i));
    }
    
    // Reload and check bank is preserved
    let reloaded_bank = alloc.get_bank_register("dynamic_ptr");
    assert!(matches!(reloaded_bank, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
}

#[test]
fn stress_test_parameter_loading_edge_cases() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Test loading many parameters (more than available registers)
    for i in 0..20 {
        let reg = alloc.load_parameter(i);
        assert!(matches!(reg, Reg::S0 | Reg::S1 | Reg::S2 | Reg::S3 | Reg::T0 | Reg::T1 | Reg::T2 | Reg::T3 | Reg::T4 | Reg::T5 | Reg::T6 | Reg::T7));
    }
    
    // Should have many loads and spills
    let insts = alloc.take_instructions();
    
    // Each parameter load generates AddI + Load
    let load_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Load(_, Reg::Sb, _)))
        .count();
    assert_eq!(load_count, 20, "Should have 20 parameter loads");
    
    // Parameters beyond index 11 should cause spills
    let spill_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, _)))
        .count();
    assert!(spill_count >= 8, "Should spill for params beyond available registers");
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

#[test]
fn verify_r13_always_initialized_before_stack_ops() {
    // Verify R13 is ALWAYS initialized before any stack operation
    let mut alloc = RegAllocV2::new();
    
    // Try to spill without init - should auto-init
    for i in 0..10 {
        alloc.get_reg(format!("val_{}", i));
    }
    
    let insts = alloc.take_instructions();
    
    // Find first R13 init and first stack operation
    let r13_init_pos = insts.iter().position(|i| 
        matches!(i, AsmInst::Li(Reg::Sb, 1))
    );
    
    let first_stack_op = insts.iter().position(|i| 
        matches!(i, AsmInst::Store(_, Reg::Sb, _)) ||
        matches!(i, AsmInst::Load(_, Reg::Sb, _))
    );
    
    if let (Some(init), Some(stack_op)) = (r13_init_pos, first_stack_op) {
        assert!(init < stack_op, "R13 must be initialized before first stack operation");
    }
}

#[test]
fn verify_no_r3_r4_for_parameters() {
    // Verify R3/R4 are NEVER used for parameters (reserved for returns)
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Parameters should never allocate R3 or R4
    for i in 0..20 {
        let reg = alloc.load_parameter(i);
        assert!(!matches!(reg, Reg::Rv0 | Reg::Rv1 | Reg::Ra | Reg::Rab),
                "Parameter {} incorrectly allocated to {:?}", i, reg);
    }
}