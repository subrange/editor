// Test file for preserve syntax highlighting

// Basic preserve function
{preserve(MOV R1, R2;)}

// Shorthand syntax
{: ADD R3, R4; }

// In macro definitions
#define asm_mov(src, dst) {preserve(MOV dst, src;)}
#define asm_add(a, b) {: ADD a, b }

// Macro invocations
@asm_mov(R5, R6);
@asm_add(R7, R8);

// Nested in other constructs
{for(i in {1, 2, 3}) {
    {: MOV R[i], #i }
}}

// Mixed with regular brainfuck
+++[->+<]
{preserve(JMP label;)}
>.<
{: NOP }