# Frontend-Backend Compatibility Report

## Executive Summary

This report analyzes the RCC frontend implementation to identify issues that would prevent implementing a correct backend according to the Ripple VM Calling Convention. The frontend has **fundamental design issues** that make it impossible to implement fat pointers and proper bank management in the backend.

## Critical Frontend Issues

### 1. ❌ No Fat Pointer Support in Type System

**Problem**: The frontend's type system only supports simple pointers:
```rust
// ast.rs:36
Pointer(Box<Type>)  // Only stores target type, no bank information
```

**Impact**: 
- Cannot represent fat pointers (address + bank) at the AST level
- No way to track pointer provenance through the frontend
- Backend receives incomplete pointer information

**Required Changes**:
```rust
// Should be:
enum BankTag {
    Global = 0,  // .rodata/.data
    Stack = 1,   // frame/alloca  
    Heap = 2,    // Reserved for future heap
    Unknown,     // Parameter or loaded pointer
    Mixed,       // Can be different banks on different paths
}

Pointer {
    target: Box<Type>,
    bank: Option<BankTag>,
}
```

### 2. ❌ IR Type System Lacks Fat Pointer Support

**Problem**: The IR pointer type (IrType::Ptr) only contains the target type:
```rust
// Frontend generates:
IrType::Ptr(Box::new(target_type))  // No bank information
```

**Impact**:
- Backend must guess or infer bank information
- Cannot properly handle pointer parameters from external functions
- Mixed provenance pointers are impossible to track

### 3. ❌ No Bank Tracking in Codegen

**Problem**: The frontend codegen module has virtually no awareness of memory banks:
- Only one mention of "bank" in entire frontend (expressions.rs:240)
- No bank tag propagation through expressions
- No distinction between stack and global pointers

**Impact**:
- Backend receives pointers without knowing their memory region
- Must retroactively infer banks, leading to the broken implementations we saw

### 4. ⚠️ Pointer Size Hardcoded as 2 Bytes

**Problem**: Pointers are assumed to be 16-bit throughout:
```rust
// ast.rs:85, types.rs:80
Type::Pointer(_) => Some(2), // 16-bit pointers
```

**Impact**:
- If fat pointers need 32 bits (16 for address, 16 for bank), this breaks
- May need to adjust memory layout calculations
- Could affect struct field offsets

### 5. ❌ Function Parameters/Returns Don't Support Fat Pointers

**Problem**: Function signatures in AST and IR don't account for fat pointers:
- Parameters are single values, not pairs
- Return types are single types, not pairs
- No way to represent "pointer returns in R3+R4"

**Example Issue**:
```rust
// Current: function returns Type::Pointer(Int)
// Needed: function returns (address: I16, bank: I16)
```

### 6. ⚠️ Array to Pointer Decay Loses Information

**Problem**: Arrays decay to simple pointers without preserving bank information:
```rust
// statements.rs:197
(alloca_temp, IrType::Ptr(element_type.clone()), true)
// Lost: This is a stack allocation!
```

**Impact**:
- Backend doesn't know array pointers are stack-based
- Could incorrectly use global bank for local arrays

### 7. ❌ GEP Operations Don't Consider Banks

**Problem**: Pointer arithmetic (GEP) in frontend doesn't track bank boundaries:
```rust
// expressions.rs:239-241
self.builder.build_gep(
    base_ptr,
    index,
    IrType::Ptr(Box::new(elem_type))  // No bank overflow handling
)?;
```

**Impact**:
- No way to detect arrays spanning banks
- Backend can't implement safe bank-aware GEP

### 8. ⚠️ Global Variable Handling

**Problem**: Global variables are referenced by name without explicit bank tagging:
```rust
// codegen/mod.rs
Value::Global(name)  // No bank information attached
```

**Impact**:
- Backend must assume all globals are in bank 0
- No flexibility for different memory layouts

## Blocking Issues for Backend Implementation

The frontend issues create these backend problems:

1. **Cannot implement fat pointers** - No type system support
2. **Cannot track pointer provenance** - No bank information flow
3. **Cannot handle mixed provenance** - No Unknown/Mixed bank states
4. **Cannot implement safe GEP** - No bank boundary information
5. **Cannot properly pass/return pointers** - ABI mismatch with calling convention

## Required Frontend Changes

1. **Add fat pointer support to AST**:
   - Extend Type::Pointer to include bank information
   - Add BankTag enum (Global, Stack, Unknown, Mixed)

2. **Extend IR type system**:
   - Change IrType::Ptr to include bank tag
   - Or add IrType::FatPtr variant

3. **Track pointer provenance in codegen**:
   - Propagate bank tags through all pointer operations
   - Mark allocas as Stack, globals as Global
   - Track Unknown for parameters

4. **Fix function ABI**:
   - Support multi-value returns for pointers
   - Handle fat pointer parameters as pairs

5. **Implement bank-aware GEP**:
   - Check for bank overflow in array indexing
   - Add bank crossing warnings/errors

6. **Update type sizes**:
   - Consider fat pointers as 4 bytes (2 for addr, 2 for bank)
   - Update struct layout calculations

7. **Add explicit bank annotations**:
   - Allow source-level bank hints
   - Support mixed-bank data structures

## Conclusion

The frontend has **fundamental architectural issues** that prevent implementing a correct backend. The lack of fat pointer support throughout the type system and IR generation makes it impossible to track memory banks properly.

**Recommendation**: The frontend needs significant refactoring to support fat pointers before any backend fixes can be meaningful. Without frontend changes, the backend will always be broken, regardless of how it's implemented.

The current architecture is fundamentally incompatible with the Ripple VM's banked memory model.