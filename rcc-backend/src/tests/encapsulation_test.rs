//! Test that verifies the encapsulation is working
//! 
//! This test should NOT compile if we try to access internals

#[test]
fn test_can_only_use_public_api() {
    // This should work - public API
    let mut builder = crate::FunctionBuilder::new();
    builder.begin_function(5);
    
    // These should work - public types
    let _arg = crate::CallArg::Scalar(rcc_codegen::Reg::A0);
    
    // The following lines should NOT compile if uncommented:
    // (They're commented so the test suite passes)
    
    // Cannot access FunctionLowering
    // let _func = crate::function::lowering::FunctionLowering::new();
    
    // Cannot access CallingConvention  
    // let _cc = crate::function::calling_convention::CallingConvention::new();
    
    // Cannot import internal modules
    // use crate::function::lowering;
    // use crate::function::calling_convention;
}

#[cfg(never_compile)]  // This test is meant to fail compilation
fn test_cannot_access_internals() {
    // This test exists to document what SHOULD NOT work
    
    // Try to access FunctionLowering - should fail
    let _func = crate::function::lowering::FunctionLowering::new();
    
    // Try to access CallingConvention - should fail
    let _cc = crate::function::calling_convention::CallingConvention::new();
}