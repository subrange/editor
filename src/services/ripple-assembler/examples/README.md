# Ripple Assembly Examples

This directory contains example programs written in Ripple assembly language, demonstrating various features and programming techniques.

## Examples

### 1. hello-world.asm
A simple "Hello, World!" program that demonstrates:
- Basic string output
- Loading addresses with `LI`
- Character-by-character string processing
- Conditional branching with `BEQ`

### 2. 99-bottles.asm
The classic "99 Bottles of Beer" song that demonstrates:
- Loop constructs
- Arithmetic operations (`SUBI`)
- Subroutine calls with `JALR`
- Number-to-ASCII conversion
- Conditional text selection (singular vs plural)

### 3. fibonacci.asm
Calculates and displays the first 20 Fibonacci numbers:
- Iterative algorithm implementation
- Array storage and manipulation
- Multi-digit number printing
- Formatted output with line breaks

### 4. fizzbuzz.asm
The FizzBuzz problem (1-100) demonstrating:
- Modulo operation via repeated subtraction
- Multiple conditional branches
- Subroutine-based code organization
- String and number output mixing

### 5. linked-list.asm
A complete linked list implementation featuring:
- Dynamic memory allocation (simple heap)
- Pointer manipulation
- Data structure operations (push, pop, append)
- List traversal and length calculation
- Complex subroutine interactions

## Running the Examples

To assemble and run these examples:

```bash
# Assemble to macro format
ripple-asm examples/hello-world-old.asm -o hello-world.bfm

# Or use the assembler programmatically
npx tsx your-script.ts
```

## Memory Layout

All examples assume:
- Output device at address `0xFFFF` (character output)
- 16-bit word size
- Bank size of 16 instructions (configurable)

## Programming Techniques

These examples demonstrate several important assembly programming techniques:

1. **Subroutine Conventions**: Using registers for parameter passing and return values
2. **Stack-less Design**: Managing return addresses manually in registers
3. **Division by Subtraction**: Since there's no DIV instruction
4. **String Handling**: Null-terminated strings and character processing
5. **Memory Management**: Simple heap allocation for dynamic structures

## Notes

- Register usage conventions vary by example but generally:
  - `R3-R6`: General purpose, often used for parameters
  - `R7-R11`: Temporary/working registers
  - `R12-R15`: Often used to save return addresses
  - `RA` (R3): Return address register for JAL/JALR
  
- All examples use position-independent code where possible
- Error handling is minimal for clarity