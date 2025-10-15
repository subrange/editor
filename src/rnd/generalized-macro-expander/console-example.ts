// Console example demonstrating generalized macro expansion to assembly
// Run with: npx tsx src/rnd/generalized-macro-expander/console-example.ts

import { createMacroExpanderV3 } from '../../services/macro-expander/macro-expander.ts';

// For now, we'll demonstrate the concept by showing how the current
// macro expander could be adapted to generate assembly

console.log('=== Generalized Macro Expander Concept Demo ===\n');

// Example 1: Simple macro that could work for both BF and ASM
console.log('1. Backend-agnostic macro:');
console.log('Input:');
const simpleInput = `
#define increment(n) {repeat(n, +)}
@increment(5)
`;
console.log(simpleInput);

const expander = createMacroExpanderV3();
const bfResult = expander.expand(simpleInput);
console.log('Current BF output:', bfResult.expanded.trim());
console.log('Hypothetical ASM output: ADDI R3, R3, 5\n');

// Example 2: Conditional compilation
console.log('2. Conditional compilation:');
const conditionalInput = `
#define DEBUG 1
#define assert(cond) {if(@DEBUG, "CHECK: " cond, "")}
@assert("R3 != 0")
`;
console.log('Input:', conditionalInput);
const condResult = expander.expand(conditionalInput);
console.log('Expanded:', condResult.expanded.trim());
console.log('In ASM backend: BEQ R3, R0, error_handler\n');

// Example 3: Complex pattern generation
console.log('3. Pattern generation with for loops:');
const patternInput = `
#define registers {R3, R4, R5, R6}
#define clear_all {for(r in @registers, "CLEAR " r " ")}
@clear_all
`;
console.log('Input:', patternInput);
const patternResult = expander.expand(patternInput);
console.log('Expanded:', patternResult.expanded.trim());
console.log('Would generate assembly:');
console.log('LI R3, 0');
console.log('LI R4, 0');
console.log('LI R5, 0');
console.log('LI R6, 0\n');

// Example 4: Macro that generates different output based on backend
console.log('4. Backend-aware macro expansion:');
console.log(`
Conceptual macro definition:
#define zero(location) {
  #if BACKEND == "brainfuck"
    [-]
  #elif BACKEND == "assembly"  
    LI location, 0
  #endif
}
`);

// Example 5: Demonstrating how current expander could be extended
console.log('\n5. How to extend current expander:');
console.log(`
Current MacroExpander flow:
1. Parse input -> AST
2. Expand macros -> Expanded AST
3. Convert to string -> Brainfuck code

Generalized flow:
1. Parse input -> AST
2. Expand macros -> Expanded AST  
3. Backend.generate(AST) -> Target code

Backend interface:
interface Backend {
  generate(ast: ASTNode[]): string;
  builtins?: Map<string, BuiltinHandler>;
}
`);

// Example 6: Real macro that could work for assembly
console.log('\n6. Assembly-compatible macro using current expander:');
const asmCompatible = `
#define MOV(dst, src) "MOV " dst ", " src
#define ADD(dst, a, b) "ADD " dst ", " a ", " b
#define LOOP(n, body) {for(i in {1,2,3,4,5}, body)}

; Clear accumulator
@MOV("R7", "R0")

; Sum first 5 numbers
@LOOP(5, {
  @ADD("R7", "R7", "i")
})
`;
console.log('Input:', asmCompatible);
const asmResult = expander.expand(asmCompatible);
console.log('Output:', asmResult.expanded.trim());

console.log('\n=== Key Insights ===');
console.log('1. The macro language itself is already backend-agnostic');
console.log('2. Only the final output generation needs to change');
console.log('3. Builtins like repeat, if, for work for any imperative target');
console.log('4. Backend-specific builtins can be added as needed');
console.log('5. Same source -> multiple targets is achievable');

// Demonstrate a more complex example
console.log('\n=== Complex Example: Function Generation ===');
const functionGen = `
#define MAKE_ADDER(name, increment) \\
  name ": ; Function: Add " increment " to R3\\n" \\
  "  ADDI R3, R3, " increment "\\n" \\
  "  JALR RA, RA\\n"

@MAKE_ADDER("add5", "5")
@MAKE_ADDER("add10", "10")
`;
console.log('Macro input:', functionGen);
const funcResult = expander.expand(functionGen);
console.log('Generated assembly:');
console.log(funcResult.expanded.trim());
