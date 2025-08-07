# Ripple VM Assembler

A configurable assembler for the Ripple VM that converts RISC-like assembly code into macro format suitable for the custom linker.

## Usage

```typescript
import { RippleAssembler } from './ripple-assembler';

// Create assembler with default options
const assembler = new RippleAssembler();

// Or with custom configuration
const customAssembler = new RippleAssembler({
  bankSize: 8,          // Default: 16
  maxImmediate: 255,    // Default: 65535
  caseInsensitive: true // Default: true
});

// Assemble source code
const result = assembler.assemble(source);

// Generate macro output
const macroCode = assembler.toMacroFormat(result.instructions);
```

## Configuration Options

- **bankSize**: Number of instructions per bank (default: 16)
  - Controls when the assembler switches to the next bank
  - Affects branch distance calculations
  
- **maxImmediate**: Maximum value for immediate operands (default: 65535)
  - Used for validation of immediate values in instructions
  - Should match your VM's cell width (e.g., 255 for 8-bit, 65535 for 16-bit)

- **caseInsensitive**: Whether mnemonics are case-insensitive (default: true)

- **startBank**: Initial bank number (default: 0)

## Examples

### Basic Assembly
```assembly
; Input assembly
LI R3, 5
LI R4, 3
JALR R4, R4
SUB R3, R3, R6
HALT
```

```
; Output macro format
@program_start(@OP_LI    , @R3 , 5   , 0)
@cmd(@OP_LI    , @R4 , 3   , 0)
@cmd(@OP_JALR  , @R4 , @R0 , @R4)
@cmd(@OP_SUB   , @R3 , @R3 , @R6)
@cmd(@OP_HALT  , 0   , 0   , 0)
@program_end
```

### With Data Section
```assembly
.data
.asciiz "Hello, World!"

.code
    LI R3, 0        ; Initialize pointer
    LI R4, 0        ; Data address
print_loop:
    LOAD R5, R4, 0  ; Load character
    BEQ R5, R0, end ; Check for null
    STORE R5, R0, 0 ; Output character
    ADDI R4, R4, 1  ; Next character
    JAL print_loop
end:
    HALT
```

## Data Section Directives

The assembler supports a `.data` section for defining static data:

- `.byte` / `.db` - Define bytes: `.byte 0x48, 'H', 72`
- `.word` / `.dw` - Define words: `.word 0x1234, 1000`
- `.string` / `.ascii` - Define string: `.string "Hello"`
- `.asciiz` - Define null-terminated string: `.asciiz "Hello"`
- `.space` / `.zero` - Reserve space: `.space 10`

Data defined in the `.data` section is automatically included in the output's `@lane(#L_MEM, ...)` block.

## Querying Configuration

```typescript
const assembler = new RippleAssembler({ bankSize: 8, maxImmediate: 255 });

console.log(assembler.getBankSize());      // 8
console.log(assembler.getMaxImmediate());  // 255
console.log(assembler.getCellsPerBank());  // 32 (8 * 4)
```