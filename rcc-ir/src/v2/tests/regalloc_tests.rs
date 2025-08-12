use crate::v2::regalloc::{RegAllocV2, BankInfo};
use rcc_codegen::{AsmInst, Reg};

#[test]
fn test_r13_initialization() {
    let mut alloc = RegAllocV2::new();
    assert!(!alloc.r13_initialized);
    
    alloc.init_stack_bank();
    assert!(alloc.r13_initialized);
    
    let insts = alloc.take_instructions();
    assert_eq!(insts.len(), 2);
    assert!(matches!(insts[1], AsmInst::LI(Reg::R13, 1)));
}

#[test]
fn test_allocatable_registers() {
    let mut alloc = RegAllocV2::new();
    
    // Should allocate R5-R11 in order
    let r1 = alloc.get_reg("val1".to_string());
    let r2 = alloc.get_reg("val2".to_string());
    let r3 = alloc.get_reg("val3".to_string());
    
    assert_eq!(r1, Reg::R5);
    assert_eq!(r2, Reg::R6);
    assert_eq!(r3, Reg::R7);
}

#[test]
fn test_load_parameter() {
    let mut alloc = RegAllocV2::new();
    alloc.init_stack_bank();
    
    // Load param 0 (should be at FP-3)
    let reg = alloc.load_parameter(0);
    assert!(matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    
    let insts = alloc.take_instructions();
    // Should have load from FP-3
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(Reg::R12, Reg::R15, -3))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::R13, Reg::R12))));
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
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Store(_, Reg::R13, _))));
}

#[test]
fn test_bank_info_tracking() {
    let mut alloc = RegAllocV2::new();
    
    alloc.set_pointer_bank("global_ptr".to_string(), BankInfo::Global);
    alloc.set_pointer_bank("stack_ptr".to_string(), BankInfo::Stack);
    
    assert_eq!(alloc.get_bank_register("global_ptr"), Reg::R0);
    
    alloc.init_stack_bank(); // Must init before using stack bank
    assert_eq!(alloc.get_bank_register("stack_ptr"), Reg::R13);
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
    assert!(matches!(reg, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    
    let insts = alloc.take_instructions();
    // Should have load instruction for reload
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, Reg::R13, _))));
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
    assert!(matches!(r1, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    assert!(matches!(r2, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    assert!(matches!(r3, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    assert!(matches!(r4, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    assert!(matches!(r5, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    assert!(matches!(r6, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
    assert!(matches!(r7, Reg::R5 | Reg::R6 | Reg::R7 | Reg::R8 | Reg::R9 | Reg::R10 | Reg::R11));
}