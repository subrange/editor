# RCC-IR V2 Backend Implementation Guide

## Overview

The V2 backend is a complete rewrite of the Ripple C Compiler's code generation backend, fixing critical issues found in V1. This implementation generates assembly code for the Ripple VM, a 16-bit stack machine with memory banks.

## Critical Context

### What is Ripple VM?
- **16-bit architecture** with 18 registers (R0-R15, PC, PCB, RA, RAB)
- **Memory banks**: 4096 instructions per bank (16384 cells)
- **Stack-based calling convention**: All parameters passed on stack
- **Fat pointers**: Two-component pointers (address + bank tag)

### Why V2?
V1 had catastrophic bugs:
1. **R13 never initialized** - Stack operations used garbage bank value
2. **Wrong calling convention** - Used R3-R8 for parameters (R3-R4 are for returns!)
3. **Broken pointer returns** - Missing bank component in R4
4. **Wrong bank registers** - Memory operations used incorrect banks

V2 fixes ALL these issues and is fully conformant to specifications.

## Architecture

```
v2/
├── mod.rs                     # Module exports and constants
├── regalloc.rs               # Register allocator with spilling
├── function.rs               # Function prologue/epilogue
├── calling_convention.rs     # Stack-based parameter passing
└── tests/                    # All test files
    ├── mod.rs
    ├── regalloc_tests.rs     # 9 tests
    ├── function_tests.rs     # 6 tests
    └── calling_convention_tests.rs  # 8 tests
```

## Key Specifications

### Register Usage (CRITICAL!)

| Register | Number | Purpose | Notes |
|----------|--------|---------|-------|
| R0 | 0 | Always zero | Hardware constant |
| PC | 1 | Program counter | Instruction offset in bank |
| PCB | 2 | Program counter bank | Current bank number |
| RA | 3 | Return address | Set by JAL |
| RAB | 4 | Return address bank | Set by JAL |
| R3 | 5 | Return value / pointer addr | NEVER for parameters! |
| R4 | 6 | Second return / pointer bank | NEVER for parameters! |
| R5-R11 | 7-13 | General purpose | Allocatable pool |
| R12 | 14 | Scratch / Global bank | Temporary calculations |
| R13 | 15 | Stack bank (SB) | MUST BE INITIALIZED TO 1! |
| R14 | 16 | Stack pointer (SP) | Current stack top |
| R15 | 17 | Frame pointer (FP) | Current frame base |

### Memory Banks

| Bank | Value | Usage |
|------|-------|-------|
| 0 | R0 | Global memory (.data/.rodata) |
| 1 | R13=1 | Stack memory (MUST initialize R13!) |
| 2 | Future | Reserved for heap |

### Stack Frame Layout

```
Higher addresses
+----------------+
| Previous frame |
+================+ <- FP (Frame Pointer)
| Local vars     |
| FP+0 .. FP+L-1 |
+----------------+
| Spill slots    |
| FP+L .. FP+L+S-1|
+----------------+ <- SP (Stack Pointer)
| Arguments      |  (for next call)
+----------------+
Lower addresses
```

Parameters are at NEGATIVE offsets from FP:
- param0: FP-3
- param1: FP-4
- param2: FP-5
- etc.

## Critical Implementation Details

### 1. Function Prologue (MUST DO!)

```rust
// CRITICAL: Initialize R13 to 1 for stack bank!
LI    R13, 1        // <-- WITHOUT THIS, ALL STACK OPS FAIL!
STORE RA, R13, R14  // Save return address
ADDI  R14, R14, 1   // Increment SP
STORE R15, R13, R14 // Save old FP
ADDI  R14, R14, 1   // Increment SP
ADD   R15, R14, R0  // FP = SP
ADDI  R14, R14, L   // Allocate L locals
```

### 2. Function Epilogue

```rust
ADD   R14, R15, R0     // SP = FP
ADDI  R14, R14, -1     // Decrement SP
LOAD  R15, R13, R14    // Restore old FP
ADDI  R14, R14, -1     // Decrement SP
LOAD  RA, R13, R14     // Restore RA
ADD   PCB, RAB, R0     // Restore caller's bank!
JALR  R0, R0, RA       // Return
```

### 3. Calling Convention

**Parameters**: ALL on stack (not in registers!)
```rust
// Push arguments in reverse order
STORE arg2, R13, R14   // Push arg2
ADDI  R14, R14, 1
STORE arg1, R13, R14   // Push arg1
ADDI  R14, R14, 1
STORE arg0, R13, R14   // Push arg0
ADDI  R14, R14, 1

// Call function
JAL   func_addr        // Sets RA, RAB automatically

// Clean up stack
ADDI  R14, R14, -3     // Pop 3 arguments
```

**Returns**:
- Scalar: R3 only
- Fat pointer: R3 (address) + R4 (bank)
- NEVER use R3-R4 for parameters!

### 4. Cross-Bank Calls

JAL only jumps within current bank. For cross-bank:
```rust
LI    PCB, target_bank  // Set target bank first
JAL   target_addr       // Then jump (saves RAB)
```

### 5. Memory Operations

```rust
// Load from pointer
LOAD  rd, bank_reg, addr_reg

// Store to pointer  
STORE rs, bank_reg, addr_reg

// Where bank_reg is:
// - R0 for global memory (bank 0)
// - R13 for stack memory (bank 1, R13=1)
// - Dynamic register for heap (future)
```

## Testing

Run all tests:
```bash
cargo test --package rcc-ir --lib v2::tests::
```

23 tests cover:
- R13 initialization
- Register allocation and spilling
- Stack-based parameters
- Fat pointer returns
- Cross-bank calls
- Function prologue/epilogue

## Common Pitfalls (AVOID THESE!)

### ❌ DON'T forget to initialize R13
```rust
// BAD - R13 contains garbage!
STORE value, R13, addr  

// GOOD - R13 initialized
LI    R13, 1
STORE value, R13, addr
```

### ❌ DON'T use R3-R4 for parameters
```rust
// BAD - R3 is for returns!
ADD R3, param_value, R0  

// GOOD - Use stack
STORE param_value, R13, R14
```

### ❌ DON'T forget to restore PCB on return
```rust
// BAD - Returns to wrong bank!
JALR R0, R0, RA

// GOOD - Restore bank first
ADD  PCB, RAB, R0
JALR R0, R0, RA
```

### ❌ DON'T assume JAL can jump to any bank
```rust
// BAD - JAL only works in current bank
JAL bank_3, addr_100  

// GOOD - Set PCB first
LI   PCB, 3
JAL  0, addr_100
```

## Implementation Status

### ✅ Completed
- Register allocator with spilling
- Function prologue/epilogue with R13 init
- Stack-based parameter passing
- Fat pointer returns (R3=addr, R4=bank)
- Cross-bank function calls
- Bank register management
- 23 comprehensive tests

### 🚧 TODO
- Load/store instruction generation
- Bank-aware GEP (GetElementPtr) with overflow handling
- Integration with IR lowering
- Full function compilation tests

## Next Steps for Developers

1. **Read the specifications first!**
   - `/docs/ripple-calling-convention.md` - Calling convention details
   - `/docs/ASSEMBLY_FORMAT.md` - Instruction formats
   - `/docs/rcc-ir-conformance-report.md` - V1 issues (don't repeat!)

2. **Understand the test suite**
   - Run existing tests to see expected behavior
   - Tests demonstrate correct usage patterns

3. **When implementing new features**:
   - ALWAYS initialize R13 for stack operations
   - ALWAYS use correct bank registers (R0/R13)
   - ALWAYS follow stack-based parameter passing
   - ALWAYS return pointers as fat pointers (addr+bank)
   - ALWAYS write tests first (TDD)

4. **For GEP implementation** (next priority):
   ```rust
   // Must handle bank overflow!
   total_offset = base_addr + (index * element_size)
   new_bank = base_bank + (total_offset / BANK_SIZE)
   new_addr = total_offset % BANK_SIZE
   ```

## Questions/Issues?

- Check test files for usage examples
- Ensure R13=1 for ALL stack operations
- Verify against specification documents
- Remember: This is a 16-bit machine, not 32/64-bit!

## Quick Reference

```rust
// Typical function structure
fn compile_function() {
    // 1. Initialize R13
    emit!(LI R13, 1);
    
    // 2. Save RA/FP, setup frame
    emit_prologue(local_count);
    
    // 3. Load parameters from stack
    load_param(0);  // At FP-3
    
    // 4. Do work...
    
    // 5. Return value in R3 (R4 for pointer bank)
    emit!(ADD R3, result, R0);
    
    // 6. Restore and return
    emit_epilogue();
}
```

Remember: **R13 MUST BE 1** or everything breaks!