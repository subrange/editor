//! Tests for Load instruction lowering

use crate::ir::{Value, IrType as Type, FatPointer, BankTag};
use crate::v2::regmgmt::{RegisterPressureManager, BankInfo};
use crate::v2::naming::new_function_naming;
use crate::v2::instr::lower_load;
use rcc_codegen::{AsmInst, Reg};

#[test]
fn test_load_scalar_from_stack() {
    let mut mgr = RegisterPressureManager::new(5);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Create a stack pointer
    let ptr = Value::FatPtr(FatPointer {
        addr: Box::new(Value::Temp(10)),
        bank: BankTag::Stack,
    });
    
    // Load i16 value
    let insts = lower_load(&mut mgr, &mut naming, &ptr, &Type::I16, 20);
    
    // Should have at least one LOAD instruction
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, _, _))));
    
    // Should use R13 for stack bank
    let load_found = insts.iter().any(|i| {
        if let AsmInst::Load(_, bank, _) = i {
            *bank == Reg::Sb
        } else {
            false
        }
    });
    assert!(load_found, "Should use R13 for stack bank");
}

#[test]
fn test_load_scalar_from_global() {
    let mut mgr = RegisterPressureManager::new(5);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Create a global pointer
    let ptr = Value::FatPtr(FatPointer {
        addr: Box::new(Value::Constant(100)),
        bank: BankTag::Global,
    });
    
    // Load i16 value
    let insts = lower_load(&mut mgr, &mut naming, &ptr, &Type::I16, 30);
    
    // Should load constant address first
    assert!(insts.iter().any(|i| matches!(i, AsmInst::LI(_, 100))));
    
    // Should use R0 for global bank
    let load_found = insts.iter().any(|i| {
        if let AsmInst::Load(_, bank, _) = i {
            *bank == Reg::R0
        } else {
            false
        }
    });
    assert!(load_found, "Should use R0 for global bank");
}

#[test]
fn test_load_fat_pointer() {
    let mut mgr = RegisterPressureManager::new(5);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Create a pointer to load from
    let ptr = Value::FatPtr(FatPointer {
        addr: Box::new(Value::Temp(15)),
        bank: BankTag::Stack,
    });
    
    // Load pointer type (fat pointer)
    let ptr_type = Type::FatPtr(Box::new(Type::I16));
    let insts = lower_load(&mut mgr, &mut naming, &ptr, &ptr_type, 40);
    
    // Should have two LOAD instructions (address and bank)
    let load_count = insts.iter().filter(|i| matches!(i, AsmInst::Load(_, _, _))).count();
    assert_eq!(load_count, 2, "Should have 2 loads for fat pointer");
    
    // Should have ADDI to calculate bank address
    assert!(insts.iter().any(|i| matches!(i, AsmInst::AddI(_, _, 1))));
}

#[test]
fn test_load_from_temp_pointer() {
    let mut mgr = RegisterPressureManager::new(5);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Set up a temp that is a pointer
    let ptr_temp = 25;
    mgr.set_pointer_bank(naming.temp_name(ptr_temp), BankInfo::Stack);
    
    // Load from the temp pointer
    let ptr = Value::Temp(ptr_temp);
    let insts = lower_load(&mut mgr, &mut naming, &ptr, &Type::I16, 50);
    
    // Should have LOAD instruction
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Load(_, _, _))));
    
    // Should use R13 for stack bank (from the pointer bank info)
    let load_found = insts.iter().any(|i| {
        if let AsmInst::Load(_, bank, _) = i {
            *bank == Reg::Sb
        } else {
            false
        }
    });
    assert!(load_found, "Should use R13 for stack pointer");
}

#[test]
fn test_load_from_global_variable() {
    let mut mgr = RegisterPressureManager::new(5);
    mgr.init();
    let mut naming = new_function_naming();
    
    // Load from a global variable
    let ptr = Value::Global("global_var".to_string());
    let insts = lower_load(&mut mgr, &mut naming, &ptr, &Type::I32, 60);
    
    // Should generate label and placeholder for linker
    assert!(insts.iter().any(|i| matches!(i, AsmInst::Label(_))));
    assert!(insts.iter().any(|i| matches!(i, AsmInst::LI(_, 0)))); // Placeholder
    
    // Should have LOAD instruction with R0 (global bank)
    let load_found = insts.iter().any(|i| {
        if let AsmInst::Load(_, bank, _) = i {
            *bank == Reg::R0
        } else {
            false
        }
    });
    assert!(load_found, "Should use R0 for global bank");
}