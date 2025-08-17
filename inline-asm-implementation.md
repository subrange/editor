# Inline Assembly Implementation for Ripple C Compiler

## Overview
We're implementing extended inline assembly support for the Ripple C compiler to enable direct hardware access (MMIO) and optimized code sequences. The syntax follows GCC's extended inline assembly format:

```c
asm("assembly code" : outputs : inputs : clobbers);
```

## Current Status

### ✅ Completed Components

1. **AST Support** (DONE)
   - Added `AsmOperand` struct with constraint and expression fields
   - Updated `StatementKind::InlineAsm` to include outputs, inputs, and clobbers
   - Files: `rcc-frontend/src/ast/statements.rs`

2. **Parser** (DONE)
   - Extended parser to handle full inline assembly syntax with colons
   - Added `parse_asm_operands()` and `parse_asm_clobbers()` functions
   - Supports string concatenation for assembly code
   - Handles empty operand lists correctly
   - Files: `rcc-frontend/src/parser/statements.rs`

3. **Typed AST** (DONE)
   - Added `TypedAsmOperand` struct
   - Updated `TypedStmt::InlineAsm` with typed operands
   - Files: `rcc-frontend/src/typed_ast/statements.rs`

4. **Semantic Analysis** (DONE)
   - Updated pattern matching to handle new fields with `..` syntax
   - Type checking for operand expressions in conversion
   - Converts AST operands to typed operands with proper type checking
   - Files: `rcc-frontend/src/semantic/statements.rs`, `rcc-frontend/src/typed_ast/conversion.rs`

5. **IR Representation** (DONE)
   - Added `AsmOperandIR` struct with constraint, value, and tied_to fields
   - Updated `Instruction::InlineAsm` to include operands vectors and clobbers
   - Updated Display implementation to show operands in IR output
   - Files: `rcc-frontend/src/ir/instructions.rs`

6. **IR Builder** (DONE)
   - Updated `build_inline_asm()` to create empty operand vectors for backward compatibility
   - Added `build_inline_asm_extended()` for full operand support
   - Files: `rcc-frontend/src/ir/builder.rs`

7. **Code Generator** (DONE)
   - Created `generate_inline_asm_extended()` function
   - Handles output operands (including read-write with '+' prefix)
   - Handles input operands
   - Properly generates lvalues for outputs and rvalues for inputs
   - Manages tied operands for read-write constraints
   - Files: `rcc-frontend/src/codegen/statements/misc.rs`

### ❌ Not Started

1. **Backend - Assembly Generation** (TODO)
   - The most complex part: actual register allocation (see regmgmt module to see how we do it now)
   - Constraint parsing and validation
   - Register assignment based on constraints
   - Placeholder substitution in assembly string
   - Files: `rcc-backend/src/v2/instr/inline_asm.rs` (to be created)

## Implementation Details

### Constraint Types
- `"=r"` - Output register operand
- `"r"` - Input register operand  
- `"+r"` - Input/output register operand (read-write)
- `"m"` - Memory operand
- `"i"` - Immediate constant operand

### Register Allocation Strategy

For proper implementation, we need:

1. **Parse Constraints**
   - Extract modifiers: `=` (output), `+` (input/output)
   - Extract constraint type: `r` (register), `m` (memory), `i` (immediate)

2. **Allocate Registers**
   - Map available Ripple VM registers (T0-T7 for temporaries)
   - Avoid registers listed in clobbers
   - Handle tied operands (for `"+r"` constraints)

3. **Generate Setup Code**
   ```
   ; Load inputs into allocated registers
   LOAD T0, input1_addr
   LOAD T1, input2_addr
   ```

4. **Process Assembly String**
   - Replace %0, %1, %2 with allocated register names
   - Handle special cases like %[named] operands

5. **Generate Teardown Code**
   ```
   ; Store outputs from registers back to memory
   STORE T2, output_addr
   ```

6. **Handle Clobbers**
   - Save clobbered registers before asm
   - Restore them after

### Example Translation

C Code:
```c
int sum;
asm("ADD %0, %1, %2" : "=r"(sum) : "r"(x), "r"(y));
```

Should generate:
```asm
; Load inputs
LOAD T0, [x_address]
LOAD T1, [y_address]

; Execute inline assembly with substitution
ADD T2, T0, T1

; Store output
STORE T2, [sum_address]
```

## Test Cases

From `test_inline_asm_extended.c`:
1. Basic addition with output operand
2. Multiplication with multiple assembly instructions
3. In-place modification with `"+r"` constraint
4. Complex operations with clobbers

## Next Steps

1. **Implement IR Builder changes**
   - Update `build_inline_asm()` to accept operands
   - Generate Value objects for operand expressions

2. **Implement Code Generator changes**
   - Create `generate_inline_asm_extended()` function
   - Evaluate operand expressions to get Values
   - Build IR instruction with operands

3. **Implement Backend Register Allocator**
   - Create constraint parser
   - Implement register allocation algorithm
   - Generate setup/teardown code
   - Perform placeholder substitution

4. **Testing**
   - Start with simple cases (no operands)
   - Test input-only operands
   - Test output operands
   - Test read-write operands
   - Test clobbers

## Alternative: Simpler MMIO Solution

If full inline assembly proves too complex, we could:
1. Use offset addressing (start MMIO at address 1 instead of 0)
2. Provide compiler intrinsics like `__mmio_read()` and `__mmio_write()`
3. Special-case certain pointer dereferences in the compiler

However, proper inline assembly support provides more flexibility and follows established conventions from GCC/Clang.

## Risks and Challenges

1. **Register Allocation Complexity** - Need to handle conflicts, spilling, and tied operands
2. **Debugging Difficulty** - Inline assembly bugs are hard to diagnose
3. **Testing Coverage** - Many edge cases with different constraint combinations
4. **Documentation** - Users need clear docs on supported constraints

## Decision Point

We need to decide whether to:
1. Continue with full implementation (correct but complex)
2. Implement a minimal subset (e.g., only "r" constraints)
3. Pivot to an alternative MMIO solution

Given the risk of heisenbugs mentioned, the full implementation is recommended despite its complexity.