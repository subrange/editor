# Fat Pointer Register Allocation Issues

## Problem Summary

The Ripple C compiler uses "fat pointers" - 2-word pointers consisting of:
1. **Address** (16-bit): The memory address
2. **Bank tag** (16-bit): Indicates memory bank (0=global, 1=stack)

The current register allocator treats these as separate values, leading to correctness issues when registers are spilled and reloaded.

## Current Issues

### 1. Separate Tracking
- Fat pointer address stored as temp `tN`
- Bank tag stored as temp `t10000N` (N + 100000)
- Register allocator doesn't understand these are related
- Can spill one without the other

### 2. Spilling Problems
When register pressure is high:
- Address (e.g., `t14`) might be spilled to `FP+17`
- Bank tag (e.g., `t100014`) might be spilled to `FP+19`
- When reloaded, the connection between them is lost
- GEP operations that propagate bank tags fail

### 3. Bank Tag Propagation in GEP
- GetElementPtr should preserve bank tags when computing new addresses
- Current implementation updates metadata but doesn't ensure register values are preserved
- When bank tag register is reused for other purposes, the value is lost

## Symptoms

Test failures showing:
- String literals printing null bytes instead of characters
- Example: "OK!" prints as "O\0!" because bank tag (0) is loaded as character value

## Attempted Workarounds

1. **Pinning registers during GEP** - Partially successful but doesn't solve spilling issue
2. **Propagating bank tags in metadata** - Doesn't preserve actual register values
3. **Reusing bank registers** - Conflicts with register allocator's ownership model

## Proper Fix Design

### 1. Multi-Register Value Support

Extend the register allocator to understand composite values:

```rust
enum RegisterValue {
    Single(String),           // Normal 1-register value
    FatPointer {             // 2-register value
        addr_reg: Reg,
        bank_reg: Reg,
        name: String,
    }
}
```

### 2. Atomic Spilling

When spilling fat pointers, spill both components together:

```rust
fn spill_fat_pointer(addr_reg: Reg, bank_reg: Reg, base_offset: i16) {
    // Spill address to base_offset
    emit(Store(addr_reg, R13, FP + base_offset));
    // Spill bank to base_offset + 1
    emit(Store(bank_reg, R13, FP + base_offset + 1));
}

fn reload_fat_pointer(base_offset: i16) -> (Reg, Reg) {
    let addr_reg = get_reg();
    let bank_reg = get_reg();
    emit(Load(addr_reg, R13, FP + base_offset));
    emit(Load(bank_reg, R13, FP + base_offset + 1));
    (addr_reg, bank_reg)
}
```

### 3. Bank Tag Immutability

Recognize that bank tags are immutable for a pointer's lifetime:
- Multiple pointers can share the same bank tag value
- Bank tag registers could be marked as "shared" and read-only
- Only need one register for all pointers with same bank

### 4. Alternative: Tagged Pointer Representation

Instead of 2 separate registers, use a different approach:
- Store pointers as single values with bank encoded in high bits
- Extract bank when needed for memory operations
- Reduces register pressure but requires more instructions

### 5. Type-Aware Register Allocation

Make the allocator aware of value types:

```rust
struct TypedValue {
    name: String,
    ty: ValueType,
}

enum ValueType {
    Integer,
    FatPointer,
    // ...
}

impl RegisterAllocator {
    fn get_registers_for_value(&mut self, value: TypedValue) -> RegisterAllocation {
        match value.ty {
            ValueType::FatPointer => {
                // Allocate 2 registers atomically
                let (addr_reg, bank_reg) = self.allocate_pair();
                RegisterAllocation::Pair(addr_reg, bank_reg)
            }
            ValueType::Integer => {
                RegisterAllocation::Single(self.allocate_one())
            }
        }
    }
}
```

## Implementation Priority

1. **Short term**: Document the limitation, avoid high register pressure scenarios
2. **Medium term**: Implement atomic spilling for fat pointers
3. **Long term**: Full type-aware register allocation

## Related Files

- `rcc-ir/src/simple_regalloc.rs` - Current register allocator
- `rcc-ir/src/lower/instr/load.rs` - Fat pointer loading
- `rcc-ir/src/lower/instr/store.rs` - Fat pointer storing
- `rcc-ir/src/lower/instr/get_element_ptr.rs` - GEP with bank propagation
- `docs/more-formalized-register-spilling.md` - Formal spilling algorithm (doesn't handle fat pointers)

## Test Cases Affected

- `test_m3_comprehensive.c` - String literal "OK!" prints incorrectly
- `test_puts_debug.c` - String "ABC" issues
- `test_puts_simple.c` - Basic string output
- `test_puts_string.c` - Timeout due to incorrect addressing
- `test_puts_string_literal.c` - "XYZ" not printing