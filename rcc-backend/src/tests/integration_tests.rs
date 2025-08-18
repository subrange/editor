//! Integration tests using only the public V2 API
//! 
//! These tests verify the V2 backend works correctly using only
//! the safe, public interfaces.

use crate::{FunctionBuilder, CallArg, lower_module_v2, lower_function_v2};
use rcc_codegen::{AsmInst, Reg};

#[test]
fn test_complete_function_with_calls() {
    // Test building a complete function with calls using only public API
    let mut builder = FunctionBuilder::new();
    
    builder.begin_function(10);  // 10 locals
    
    // Load parameters
    let param0 = builder.load_parameter(0);
    let param1 = builder.load_parameter(1);
    
    // Store to locals
    builder
        .store_local(0, param0)
        .store_local(1, param1);
    
    // Make a function call
    let (result, _) = builder.call_function(
        0x100, 
        2,
        vec![
            CallArg::Scalar(param0),
            CallArg::FatPointer { addr: param1, bank: Reg::A2 }
        ],
        false
    );
    
    // Return the result
    builder.end_function(Some((result, None)));
    
    let instructions = builder.build();
    
    // Verify key invariants
    assert!(!instructions.is_empty());
    // Stack bank is initialized in crt0.asm, not in function prologue
    // So we don't check for Li(Reg::Sb, 1) anymore
    assert!(instructions.iter().any(|i| matches!(i, AsmInst::Jalr(_, _, Reg::Ra))),
            "Missing Jalr instruction");
}

#[test]
fn test_recursive_function_pattern() {
    let mut builder = FunctionBuilder::new();
    
    builder.begin_function(5);
    
    // Load parameters for recursion
    let n = builder.load_parameter(0);
    
    // Recursive call
    let (result, _) = builder.call_function(
        0x0,  // Call self
        0,    // Same bank
        vec![CallArg::Scalar(n)],
        false
    );
    
    builder.end_function(Some((result, None)));
    
    let instructions = builder.build();
    assert!(!instructions.is_empty());
}

#[test]
fn test_multiple_nested_calls() {
    let mut builder = FunctionBuilder::new();
    
    builder.begin_function(0);
    
    // First call
    let (result1, _) = builder.call_function(
        0x100, 0,
        vec![CallArg::Scalar(Reg::A0)],
        false
    );
    
    // Second call using result of first
    let (result2, _) = builder.call_function(
        0x200, 1,
        vec![CallArg::Scalar(result1)],
        false
    );
    
    // Third call
    let (final_result, _) = builder.call_function(
        0x300, 2,
        vec![CallArg::Scalar(result2)],
        false
    );
    
    builder.end_function(Some((final_result, None)));
    
    let instructions = builder.build();
    
    // Should have 3 JAL instructions
    let jal_count = instructions.iter()
        .filter(|i| matches!(i, AsmInst::Jal(_, _)))
        .count();
    assert_eq!(jal_count, 3);
}

#[test]
fn test_fat_pointer_operations() {
    let mut builder = FunctionBuilder::new();
    
    builder.begin_function(5);
    
    // Load fat pointer parameters
    let addr = builder.load_parameter(0);
    let bank = builder.load_parameter(1);
    
    // Call with fat pointer
    let (ret_addr, ret_bank) = builder.call_function(
        0x400, 3,
        vec![CallArg::FatPointer { addr, bank }],
        true  // Returns fat pointer
    );
    
    // Return the fat pointer
    builder.end_function(Some((ret_addr, ret_bank)));
    
    let instructions = builder.build();
    
    // Should handle fat pointers correctly
    assert!(!instructions.is_empty());
}

#[test]
fn test_large_stack_frame() {
    let mut builder = FunctionBuilder::new();
    
    // Test with many locals
    builder.begin_function(1000);
    
    // Access some locals
    builder
        .store_local(0, Reg::A0)
        .store_local(999, Reg::A1)
        .load_local(500, Reg::A2);
    
    builder.end_function(None);
    
    let instructions = builder.build();
    
    // Should allocate large frame
    assert!(instructions.iter().any(|i| 
        matches!(i, AsmInst::AddI(Reg::Sp, Reg::Sp, 1000))
    ));
}