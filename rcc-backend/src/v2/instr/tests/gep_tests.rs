//! Comprehensive tests for GEP (GetElementPtr) instruction lowering

use rcc_frontend::ir::{Value, FatPointer};
use crate::v2::instr::lower_gep;
use crate::v2::regmgmt::{RegisterPressureManager, BankInfo};
use crate::v2::naming::new_function_naming;
use rcc_codegen::{AsmInst, Reg};
use rcc_frontend::BankTag;

#[test]
fn test_gep_simple_array_access() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // int arr[10]; arr[3]
    let base_ptr = Value::Temp(1);
    mgr.set_pointer_bank("t1".to_string(), BankInfo::Stack);
    
    let indices = vec![Value::Constant(3)];
    let element_size = 1; // 16-bit integers take 1 cell each
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 10);
    
    // Should calculate offset = 3 * 1 = 3
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::AddI(_, _, 3))
    }));
    
    // Result should maintain stack bank
    assert!(matches!(
        mgr.get_pointer_bank("t10"),
        Some(BankInfo::Stack)
    ));
}

#[test]
fn test_gep_zero_offset() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // arr[0] - should just copy the base pointer
    let base_ptr = Value::Temp(5);
    mgr.set_pointer_bank("t5".to_string(), BankInfo::Global);
    
    let indices = vec![Value::Constant(0)];
    let element_size = 2; // Fat pointer takes 2 cells
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 20);
    
    // Should just copy base address (add with R0)
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Add(_, _, r) if *r == Reg::R0)
    }));
    
    // Bank should remain global (uses GP register)
    assert!(matches!(
        mgr.get_pointer_bank("t20"),
        Some(BankInfo::Global)
    ));
}

#[test]
fn test_gep_dynamic_index() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // arr[i] where i is in a temp
    let base_ptr = Value::FatPtr(FatPointer {
        addr: Box::new(Value::Temp(10)),
        bank: BankTag::Stack,
    });
    
    let indices = vec![Value::Temp(11)]; // Dynamic index
    let element_size = 2; // Fat pointer takes 2 cells
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 30);
    
    // Should generate shift instruction (element_size = 2 = 2^1)
    // First load shift amount (1), then shift
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Li(_, 1))
    }), "Should load shift amount 1 for element_size=2");
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Sll(_, _, _))
    }), "Should generate shift instruction");
    
    // Should add offset to base
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Add(_, _, _))
    }));
    
    // Should have runtime overflow calculation comment
    assert!(insts.iter().any(|inst| {
        if let AsmInst::Comment(s) = inst {
            s.contains("Runtime bank overflow calculation")
        } else {
            false
        }
    }));
    
    // Should generate DIV and MOD instructions for bank calculation
    assert!(insts.iter().any(|inst| matches!(inst, AsmInst::Div(_, _, _))));
    assert!(insts.iter().any(|inst| matches!(inst, AsmInst::Mod(_, _, _))));
}

#[test]
fn test_gep_power_of_two_optimization() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Test various power-of-2 element sizes
    let test_cases = vec![
        (1, None),           // No shift needed
        (2, Some(1)),        // Shift by 1
        (4, Some(2)),        // Shift by 2
        (8, Some(3)),        // Shift by 3
        (16, Some(4)),       // Shift by 4
    ];
    
    for (element_size, expected_shift) in test_cases {
        let base_ptr = Value::Temp(100);
        mgr.set_pointer_bank("t100".to_string(), BankInfo::Stack);
        
        let indices = vec![Value::Temp(101)]; // Dynamic index
        
        let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 200);
        
        if let Some(shift) = expected_shift {
            // Check that we load the shift amount and then perform the shift
            assert!(
                insts.iter().any(|inst| matches!(inst, AsmInst::Li(_, s) if *s == shift)),
                "Expected to load shift amount {} for element size {}", shift, element_size
            );
            assert!(
                insts.iter().any(|inst| matches!(inst, AsmInst::Sll(_, _, _))),
                "Expected shift instruction for element size {}", element_size
            );
        }
    }
}

#[test]
fn test_gep_non_power_of_two_size() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Element size that's not a power of 2
    let base_ptr = Value::Temp(50);
    mgr.set_pointer_bank("t50".to_string(), BankInfo::Stack);
    
    let indices = vec![Value::Temp(51)];
    let element_size = 3; // Not a power of 2
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 60);
    
    // Should use multiplication
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Mul(_, _, _))
    }));
    
    // Should load the size constant
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Li(_, 3))
    }));
}

#[test]
fn test_gep_large_static_offset() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Large offset that crosses bank boundary
    let base_ptr = Value::Temp(70);
    mgr.set_pointer_bank("t70".to_string(), BankInfo::Stack);
    
    let indices = vec![Value::Constant(2000)]; // Large index
    let element_size = 8; // Total offset = 16000 (crosses bank at 4096)
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 80);
    
    // Should generate large offset
    let expected_offset = 2000 * 8;
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::AddI(_, _, off) if *off == expected_offset)
    }));
    
    // Should have bank overflow warning
    assert!(insts.iter().any(|inst| {
        if let AsmInst::Comment(s) = inst {
            s.contains("bank overflow")
        } else {
            false
        }
    }), "Expected bank overflow warning for large offset");
}

#[test]
fn test_gep_global_pointer() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // GEP on a global array - simulate how lower.rs would resolve it
    let base_ptr = Value::FatPtr(FatPointer {
        addr: Box::new(Value::Constant(1000)), // Simulated global address
        bank: BankTag::Global,
    });
    
    let indices = vec![Value::Constant(5)];
    let element_size = 2;
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 90);
    
    // Should load the constant global address
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Li(_, 1000))
    }), "Should load global address 1000");
    
    // Should calculate offset
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::AddI(_, _, 10))
    }));
    
    // Result should have global bank (uses GP register) 
    assert!(matches!(
        mgr.get_pointer_bank("t90"),
        Some(BankInfo::Global)
    ));
}

#[test]
fn test_gep_fat_pointer_with_constant_addr() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Fat pointer with constant address
    let base_ptr = Value::FatPtr(FatPointer {
        addr: Box::new(Value::Constant(0x1000)),
        bank: BankTag::Global,
    });
    
    let indices = vec![Value::Constant(10)];
    let element_size = 4;
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 100);
    
    // Should load constant address
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::Li(_, 0x1000))
    }));
    
    // Should add offset
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::AddI(_, _, 40))
    }));
}

#[test]
fn test_gep_preserves_bank_info() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Test that bank info is preserved through GEP
    let test_cases = vec![
        (BankTag::Global, BankInfo::Global),
        (BankTag::Stack, BankInfo::Stack),
    ];
    
    for (input_bank, expected_bank) in test_cases {
        let base_ptr = Value::FatPtr(FatPointer {
            addr: Box::new(Value::Temp(200)),
            bank: input_bank,
        });
        
        let indices = vec![Value::Constant(5)];
        let element_size = 2;
        let result_temp = 210;
        
        let _insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, result_temp);
        
        // Check that bank info is preserved
        let result_name = format!("t{}", result_temp);
        assert!(matches!(
            mgr.get_pointer_bank(&result_name),
            Some(ref bank) if std::mem::discriminant(bank) == std::mem::discriminant(&expected_bank)
        ), "Bank info should be preserved for {:?}", input_bank);
    }
}

#[test]
fn test_gep_negative_offset() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Negative array index (going backwards)
    let base_ptr = Value::Temp(300);
    mgr.set_pointer_bank("t300".to_string(), BankInfo::Stack);
    
    let indices = vec![Value::Constant(-5)];
    let element_size = 4;
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 310);
    
    // Should generate negative offset
    assert!(insts.iter().any(|inst| {
        matches!(inst, AsmInst::AddI(_, _, -20))
    }));
}

#[test]
fn test_gep_register_pressure() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Fill up some registers to test spilling behavior
    for i in 0..10 {
        let name = format!("dummy_{}", i);
        mgr.get_register(name);
    }
    
    // Now do GEP with dynamic offset (needs multiple registers)
    let base_ptr = Value::Temp(400);
    mgr.set_pointer_bank("t400".to_string(), BankInfo::Stack);
    
    let indices = vec![Value::Temp(401)];
    let element_size = 7; // Non-power-of-2 to force multiplication
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 410);
    
    // Should have spill/reload comments if register pressure is high
    let has_spills = insts.iter().any(|inst| {
        if let AsmInst::Comment(s) = inst {
            s.contains("Spill") || s.contains("Reload")
        } else {
            false
        }
    });
    
    // This test just ensures the code doesn't panic under register pressure
    assert!(!insts.is_empty());
    
    // If spills occurred, that's fine
    if has_spills {
        println!("Register spills occurred as expected under pressure");
    }
}

#[test]
fn test_gep_runtime_bank_overflow() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Test that runtime bank overflow correctly updates the bank register
    let base_ptr = Value::Temp(500);
    mgr.set_pointer_bank("t500".to_string(), BankInfo::Stack);
    
    // Dynamic index that will cause bank overflow
    let indices = vec![Value::Temp(501)];
    let element_size = 1; // Simple case
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 510);
    
    // Should calculate bank delta
    assert!(insts.iter().any(|inst| matches!(inst, AsmInst::Div(_, _, _))), 
            "Should calculate bank delta with DIV");
    
    // Should calculate new address within bank
    assert!(insts.iter().any(|inst| matches!(inst, AsmInst::Mod(_, _, _))), 
            "Should calculate address within bank with MOD");
    
    // Should update bank register (adding to SB since base is stack)
    assert!(insts.iter().any(|inst| {
        if let AsmInst::Add(_, sb, _) = inst {
            *sb == Reg::Sb
        } else {
            false
        }
    }), "Should add bank delta to SB for stack-based pointer");
    
    // Result should have dynamic bank
    assert!(matches!(
        mgr.get_pointer_bank("t510"),
        Some(BankInfo::Register(_))
    ), "Result should have dynamic bank in a register");
}

#[test] 
fn test_gep_dynamic_with_existing_dynamic_bank() {
    let mut mgr = RegisterPressureManager::new(0);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Start with a pointer that already has a dynamic bank
    let base_ptr = Value::Temp(600);
    mgr.set_pointer_bank("t600".to_string(), BankInfo::Register(Reg::T5));
    
    let indices = vec![Value::Temp(601)];
    let element_size = 2;
    
    let insts = lower_gep(&mut mgr, &mut naming, &base_ptr, &indices, element_size, 610);
    
    // Should update the existing dynamic bank register
    assert!(insts.iter().any(|inst| {
        if let AsmInst::Add(t5, t5_2, _) = inst {
            *t5 == Reg::T5 && *t5_2 == Reg::T5
        } else {
            false
        }
    }), "Should update existing dynamic bank register in place");
}