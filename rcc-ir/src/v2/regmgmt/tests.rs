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
    assert!(matches!(insts[1], AsmInst::LI(Reg::Sb, 1)));
}

#[test]
fn test_allocatable_registers() {
    let mut alloc = RegAllocV2::new();
    
    // Should allocate R5-R11 in order
    let r1 = alloc.get_reg("val1".to_string());
    let r2 = alloc.get_reg("val2".to_string());
    let r3 = alloc.get_reg("val3".to_string());
    
    assert_eq!(r1, Reg::A0);
    assert_eq!(r2, Reg::A1);
    assert_eq!(r3, Reg::A2);
}

#[test]
fn test_load_parameter() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Load param 0 (should be at FP-3)
    let reg = alloc.load_parameter(0);
    assert!(matches!(reg, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    
    let insts = alloc.take_instructions();
    // Should have load from FP-3
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::Sc, Reg::Fp, -3))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::Sb, Reg::Sc))));
}

#[test]
fn test_spilling_with_r13() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Allocate all 7 registers
    for i in 0..7 {
        alloc.get_reg(format!("val{}", i));
    }
    
    // Next allocation should trigger spill
    alloc.get_reg("val7".to_string());
    
    let insts = alloc.take_instructions();
    // Should have R13 init + spill operations
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(_, Reg::Sb, _))));
}

#[test]
fn test_bank_info_tracking() {
    let mut alloc = RegAllocV2::new();
    
    alloc.set_pointer_bank("global_ptr".to_string(), BankInfo::Global);
    alloc.set_pointer_bank("stack_ptr".to_string(), BankInfo::Stack);
    
    assert_eq!(alloc.get_bank_register("global_ptr"), Reg::R0);
    
    alloc.init_stack_bank(); // Must init before using stack bank
    assert_eq!(alloc.get_bank_register("stack_ptr"), Reg::Sb);
}

#[test]
#[should_panic(expected = "R13 not initialized")]
fn test_panic_without_r13_init() {
    let mut alloc = RegAllocV2::new();
    alloc.set_pointer_bank("stack_ptr".to_string(), BankInfo::Stack);
    
    // Should panic because R13 not initialized
    alloc.get_bank_register("stack_ptr");
}

#[test]
fn test_reload_from_spill() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Fill all registers
    for i in 0..7 {
        alloc.get_reg(format!("val{}", i));
    }
    
    // Force spill
    alloc.get_reg("new_val".to_string());
    
    // Reload spilled value
    let reg = alloc.reload("val0".to_string());
    assert!(matches!(reg, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    
    let insts = alloc.take_instructions();
    // Should have load instruction for reload
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::Sb, _))));
}

#[test]
fn test_pinning() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Allocate and pin first value
    let r1 = alloc.get_reg("pinned".to_string());
    alloc.pin_value("pinned".to_string());
    
    // Fill other registers
    for i in 1..7 {
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
    
    // Should get all 7 registers
    assert!(matches!(r1, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    assert!(matches!(r2, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    assert!(matches!(r3, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    assert!(matches!(r4, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    assert!(matches!(r5, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    assert!(matches!(r6, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    assert!(matches!(r7, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
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
    
    // Allocate 100 values, forcing 93 spills (only 7 registers available)
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
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, _)))
        .count();
    assert!(spill_count >= 10, "Should spill values when out of registers");
    
    // Clear all registers (but spill slots remain)
    alloc.free_temporaries();
    
    // Phase 3: Reload individual spilled values and verify
    let reg1 = alloc.reload(format!("phase1_0"));
    let insts = alloc.take_instructions();
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::Sb, _))),
            "Reloading spilled value should generate load");
    
    // Verify the value is now in a register
    assert!(matches!(reg1, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    
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
    assert_eq!(alloc.get_bank_register("stack_array"), Reg::Sb);
    assert_eq!(alloc.get_bank_register("dynamic_ptr"), bank_reg);
    
    // Test with spilled dynamic bank
    for i in 0..10 {
        alloc.get_reg(format!("filler_{}", i));
    }
    
    // Reload and check bank is preserved
    let reloaded_bank = alloc.get_bank_register("dynamic_ptr");
    assert!(matches!(reloaded_bank, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
}

#[test]
fn stress_test_parameter_loading_edge_cases() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Test loading many parameters (more than available registers)
    for i in 0..20 {
        let reg = alloc.load_parameter(i);
        assert!(matches!(reg, Reg::A0 | Reg::A1 | Reg::A2 | Reg::A3 | Reg::X0 | Reg::X1 | Reg::X2));
    }
    
    // Should have many loads and spills
    let insts = alloc.take_instructions();
    
    // Each parameter load generates AddI + Load
    let load_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Load(_, Reg::Sb, _)))
        .count();
    assert_eq!(load_count, 20, "Should have 20 parameter loads");
    
    // Parameters beyond index 6 should cause spills
    let spill_count = insts.iter()
        .filter(|i| matches!(i, AsmInst::Store(_, Reg::Sb, _)))
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
        matches!(i, AsmInst::LI(Reg::Sb, 1))
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