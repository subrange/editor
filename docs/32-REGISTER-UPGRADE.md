# Ripple VM 32-Register Architecture Upgrade

## Executive Summary

Upgrade Ripple VM from 18 to 32 registers to eliminate current architectural constraints and enable efficient code generation. This is a breaking change with no backward compatibility requirements.

## Current Pain Points (18 Registers)

1. **Only 7 allocatable registers** (R5-R11) causing constant spilling
2. **R12 wasted as scratch** for spill/reload address calculations  
3. **No dedicated Global Pointer** - using R0 (zero register) limits us to single global bank
4. **No callee-saved registers** - every function call requires full spill
5. **All arguments via stack** - even simple functions have stack overhead
6. **Complex workarounds** throughout compiler for register pressure

## Proposed 32-Register Layout

### Register Naming Convention
The assembler will support BOTH numeric and symbolic names for all registers:

| Numeric | Symbolic | Purpose              | Notes            |
|---------|----------|----------------------|------------------|
| R0      | ZR, R0   | Hardware zero        | Always reads 0   |
| R1      | PC       | Program Counter      |                  |
| R2      | PCB      | Program Counter Bank |                  |
| R3      | RA       | Return Address       |                  |
| R4      | RAB      | Return Address Bank  |                  |
| R5      | RV0, V0  | Return Value 0       | Fat ptr address  |
| R6      | RV1, V1  | Return Value 1       | Fat ptr bank     |
| R7      | A0       | Argument 0           |                  |
| R8      | A1       | Argument 1           |                  |
| R9      | A2       | Argument 2           |                  |
| R10     | A3       | Argument 3           |                  |
| R11     | X0       | Reserved/Extended 0  | Future use       |
| R12     | X1       | Reserved/Extended 1  | Future use       |
| R13     | X2       | Reserved/Extended 2  | Future use       |
| R14     | X3       | Reserved/Extended 3  | Future use       |
| R15     | T0       | Temporary 0          | Caller-saved     |
| R16     | T1       | Temporary 1          | Caller-saved     |
| R17     | T2       | Temporary 2          | Caller-saved     |
| R18     | T3       | Temporary 3          | Caller-saved     |
| R19     | T4       | Temporary 4          | Caller-saved     |
| R20     | T5       | Temporary 5          | Caller-saved     |
| R21     | T6       | Temporary 6          | Caller-saved     |
| R22     | T7       | Temporary 7          | Caller-saved     |
| R23     | S0       | Saved 0              | Callee-saved     |
| R24     | S1       | Saved 1              | Callee-saved     |
| R25     | S2       | Saved 2              | Callee-saved     |
| R26     | S3       | Saved 3              | Callee-saved     |
| R27     | S4       | Saved 4              | Callee-saved     |
| R28     | S5       | Saved 5              | Callee-saved     |
| R29     | SP       | Stack Pointer        |                  |
| R30     | FP       | Frame Pointer        |                  |
| R31     | GP       | Global Pointer       | Bank for globals |

### Assembly Examples

Both forms are equivalent and can be mixed freely:

```asm
; These are identical:
ADD  R7, R15, R23
ADD  A0, T0, S0

; Mixed usage is fine:
LOAD RV0, GP, R15    ; Load return value from globals using T0 as address
STORE A0, SP, 0      ; Store first argument to stack

; Clear meaningful names for function calls:
MOVE A0, T0          ; Set first argument
MOVE A1, T1          ; Set second argument  
JAL  function
MOVE S0, RV0         ; Save return value
```

### Register Classes

- **Hardware**: R0-R4 (zero, PC, PCB, RA, RAB)
- **Return Values**: R5-R6 (RV0, RV1) - can return 32-bit/64-bit values
- **Arguments**: R7-R10 (A0-A3) - 4 function arguments
- **Reserved**: R11-R14 (X0-X3) - Reserved for future extensions
- **Temporaries**: R15-R22 (T0-T7) - caller-saved, general use
- **Saved**: R23-R28 (S0-S5) - callee-saved, general use
- **Special**: R29-R31 (SP, FP, GP)

### Allocatable Pool for Register Allocator
**14 registers**: T0-T7 (temporaries) and S0-S5 (saved)

**NOT in allocatable pool**:
- A0-A3 are ONLY for argument passing (4 args in registers)
- X0-X3 are reserved for future use (4 whole registers!)
- This gives us massive flexibility for future extensions

### Future Use of X0-X3 (R11-R14)
With 4 reserved registers, we can implement:
- **64-bit operations**: X0:X1 and X2:X3 as two 64-bit temp pairs
- **Float operations**: Single and double precision emulation
- **SIMD**: 4x16-bit vector operations
- **Special addressing**: Base+Index addressing modes
- **Crypto/Hash acceleration**: Internal state registers
- **Whatever we haven't thought of yet!**

## Benefits

1. **2x more allocatable registers** (14 vs 7) - T0-T7 and S0-S5
2. **Fast function calls** - up to 4 args in registers (A0-A3)
3. **Reduced spilling** - callee-saved registers preserve values across calls
4. **Multiple global banks** - GP register enables easy bank switching
5. **No scratch register hack** - plenty of temporaries for address calculations
6. **Better code density** - fewer spill/reload instructions
7. **Cleaner compiler** - remove complex workarounds
8. **Clear separation of concerns** - argument registers never confused with temporaries
9. **Future-proof** - X0-X3 (4 registers!) reserved for extensions

## Components Requiring Updates

### 1. Virtual Machine (rvm/)

**File: `rvm/src/vm.rs`**
```rust
// Change from:
pub struct VM {
    pub regs: [u16; 18],
}

// To:
pub struct VM {
    pub regs: [u16; 32],
}
```

**File: `rvm/src/tui_debugger.rs`**
- Update register display to show all 32 registers
- Group by category (args, temps, saved, special)
- Add switch between numeric and symbolic display modes

**File: `rvm/src/debug.rs`**
- Update simple debugger to handle 32 registers

### 2. Assembler (src/ripple-asm/)

**File: `src/ripple-asm/src/types.rs`**
- Extend `Register` enum to include R18-R31
- Add symbolic names (A0-A3, X0-X3, T0-T7, S0-S5, SP, FP, GP)

**File: `src/ripple-asm/src/parser.rs`**
- Update register parsing to accept BOTH formats:
  - Numeric: R0 through R31
  - Symbolic: PC, RA, A0-A3, X0-X3, T0-T7, S0-S5, SP, FP, GP, etc.
- Case-insensitive parsing (r0, R0, a0, A0 all valid)
- Hash map for name lookups:
```rust
fn parse_register(s: &str) -> Option<u8> {
    // Try numeric first (R0-R31)
    if let Some(num) = s.strip_prefix("R").or(s.strip_prefix("r")) {
        return num.parse().ok().filter(|&n| n < 32);
    }
    
    // Try symbolic names
    match s.to_uppercase().as_str() {
        "ZR" => Some(0),
        "PC" => Some(1),
        "PCB" => Some(2),
        "RA" => Some(3),
        "RAB" => Some(4),
        "RV0" | "V0" => Some(5),
        "RV1" | "V1" => Some(6),
        "A0" => Some(7),
        "A1" => Some(8),
        "A2" => Some(9),
        "A3" => Some(10),
        "X0" => Some(11),
        "X1" => Some(12),
        "X2" => Some(13),
        "X3" => Some(14),
        "T0" => Some(15),
        // ... etc
        "SP" => Some(29),
        "FP" => Some(30),
        "GP" => Some(31),
        _ => None
    }
}
```

### 3. Code Generation (rcc-codegen/)

**File: `rcc-codegen/src/lib.rs`**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    R0, PC, PCB, RA, RAB,
    RV0, RV1,  // Return values (R5-R6)
    A0, A1, A2, A3,  // Arguments (R7-R10)
    X0, X1, X2, X3,  // Reserved (R11-R14)
    T0, T1, T2, T3, T4, T5, T6, T7,  // Temporaries (R15-R22)
    S0, S1, S2, S3, S4, S5,  // Saved (R23-R28)
    SP, FP, GP,  // Special (R29-R31)
}
```

### 4. IR Lowering - V2 Backend (rcc-ir/src/v2/)

**File: `rcc-ir/src/v2/regmgmt/allocator.rs`**
```rust
const ALLOCATABLE_REGS: &[Reg] = &[
    // Temporaries (caller-saved)
    Reg::T0, Reg::T1, Reg::T2, Reg::T3,
    Reg::T4, Reg::T5, Reg::T6, Reg::T7,
    // Saved registers (callee-saved)
    Reg::S0, Reg::S1, Reg::S2, Reg::S3,
    Reg::S4, Reg::S5,
];
// Total: 14 allocatable registers (2x our current 7!)
// A0-A3 are NOT here - they're only for argument passing
// X0-X3 are NOT here - they're reserved for future use
```

**File: `rcc-ir/src/v2/function/calling_convention.rs`**
- Implement register-based argument passing (A0-A3)
- Handle spilling when >4 arguments  
- Implement callee-saved register preservation (S0-S5)

**File: `rcc-ir/src/v2/instr/load.rs` and `store.rs`**
```rust
// Change from:
BankInfo::Global => {
    Reg::R0  // Hardcoded to bank 0
}

// To:
BankInfo::Global => {
    Reg::GP  // Use global pointer register
}
```

### 5. Documentation Updates

- `docs/ASSEMBLY_FORMAT.md` - Update register table
- `docs/ripple-calling-convention.md` - New calling convention
- `docs/more-formalized-register-spilling.md` - Update spilling strategy

## New Calling Convention

### Extended Operations (Future)
With RV0-RV1 for return and X0-X3 reserved, we have amazing flexibility:

```asm
; 64-bit operations
MUL64 RV0, RV1, A0, A1  ; 64-bit multiply
; Uses X0:X1 for first operand expansion
; Uses X2:X3 for second operand expansion

; Double-precision float emulation
DFADD RV0, RV1, A0, A1  ; Double-precision add
; X0-X3 provide plenty of scratch space

; SIMD operations (future)
VADD4 X0, A0, A1        ; Add 4x16-bit values in parallel
; Result in X0, can use X1-X3 for complex operations
```

### Function Prologue
```asm
; First 4 arguments are in A0-A3
; Additional arguments (5+) are on stack at FP+2, FP+3, etc.
; Save callee-saved registers (if used)
ADDI  SP, SP, -6
STORE S0, SP, 0   ; Only if S0 is used
STORE S1, SP, 1   ; Only if S1 is used
; ... up to S5

; Set up frame
MOVE  FP, SP
ADDI  SP, SP, -locals_size

; Now we can use arguments directly from A0-A3
; Example: ADD T0, A0, A1  ; Use first two arguments
; For 5th+ args: LOAD T0, FP, 2  ; Load 5th argument
```

### Function Call
```asm
; Move arguments into dedicated argument registers
; Arguments MUST go in A0-A3 (4 args), rest on stack
MOVE  A0, T0       ; First arg from temporary
MOVE  A1, S0       ; Second arg from saved register
MOVE  A2, RV0      ; Third arg from previous return value
MOVE  A3, T1       ; Fourth arg
; 5th+ arguments go on stack
STORE T2, SP, 0    ; 5th argument
STORE T3, SP, 1    ; 6th argument
ADDI  SP, SP, 2

JAL   function_address

; Result in RV0/RV1 (or R5/R6)
; Stack cleanup if needed
ADDI  SP, SP, -1   ; If we pushed args
```

### Function Epilogue
```asm
; Restore stack
MOVE  SP, FP

; Restore callee-saved registers
LOAD  S0, SP, 0
LOAD  S1, SP, 1
; ...
ADDI  SP, SP, 6

; Return
JALR  R0, R0, RA
```

## Implementation Plan

### Phase 1: Core VM Changes (Week 1)
- [ ] Update VM to 32 registers
- [ ] Update instruction decoder/executor
- [ ] Update debugger display
- [ ] Test with hand-written assembly

### Phase 2: Assembler Updates (Week 1)
- [ ] Add new register names
- [ ] Update parser
- [ ] Update encoder
- [ ] Test round-trip (asm -> binary -> disasm)

### Phase 3: Code Generation (Week 2)
- [ ] Update Reg enum
- [ ] Update instruction emission
- [ ] Basic testing

### Phase 4: Compiler V2 Backend (Week 2-3)
- [ ] Update register allocator
- [ ] Implement new calling convention
- [ ] Update spilling (remove R12 hack)
- [ ] Use GP for globals

### Phase 5: Testing & Optimization (Week 3-4)
- [ ] Comprehensive test suite
- [ ] Performance benchmarks
- [ ] Optimize register allocation
- [ ] Documentation

## Testing Strategy

1. **Unit Tests**: Each component tested individually
2. **Integration Tests**: Full compilation pipeline
3. **Benchmarks**: Measure improvement in:
   - Code size (fewer spills)
   - Execution speed
   - Compilation time

## Success Metrics

- [ ] All existing tests pass with new architecture
- [ ] >50% reduction in spill/reload instructions for typical programs
- [ ] Function calls with ≤4 args require no stack operations
- [ ] Support for programs with >4KB globals without hacks

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Bugs in register allocation | Extensive testing, gradual rollout |
| Missed hardcoded assumptions | Global search for "18", "R12", etc. |
| Performance regression | Benchmark before/after |

## Future Enhancements

Once 32-register architecture is stable:
- Register windows for even faster calls
- Vector registers (if we add SIMD)
- Special-purpose registers (loop counters, predication)

## Decision Record

**Date**: 2024-12-24
**Decision**: Proceed with 32-register upgrade
**Rationale**: Current 18-register limit causes significant complexity and performance issues. No legacy code exists, so breaking change is acceptable.
**Alternatives Considered**: 
- Virtual registers: Too complex, doesn't solve fundamental issue
- Memory-mapped registers: Performance overhead
- Status quo: Unacceptable complexity and performance

---

*This is a living document. Update as implementation proceeds.*