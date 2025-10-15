// Test V4 with Assembly Backend
import { createMacroExpanderV4 } from '../../services/macro-expander/macro-expander-v4.ts';
import { AssemblyBackendV4 } from './assembly-backend-v4.ts';

console.log('=== Testing V4 with Assembly Backend ===\n');

const backend = new AssemblyBackendV4();
const expander = createMacroExpanderV4(backend);

// Test 1: Simple macro
console.log('Test 1: Simple assembly macro');
const test1 = `#define inc(reg) ADDI reg, reg, 1
@inc(R3)`;

const result1 = expander.expand(test1);
console.log('Output:', result1.expanded);
console.log();

// Test 2: Loop with unique labels
console.log('Test 2: Loop with unique labels');
const test2 = `#define loop(counter, start, end, body) {
  LI counter, start
  __LABEL__loop_start:
  body
  ADDI counter, counter, 1
  BNE counter, end, __LABEL__loop_start
}

@loop(R3, 0, 10, {
  ADD R4, R4, R3
})

@loop(R5, 0, 5, {
  SUB R6, R6, R5
})`;

const result2 = expander.expand(test2);
console.log('Output:', result2.expanded);
console.log();

// Test 3: Using backend-specific builtins
console.log('Test 3: Backend-specific builtins');
const test3 = `#define data_section {
  {align(4)}
  {db(0x48, 0x65, 0x6C, 0x6C, 0x6F)}
  {dw(1234, 5678)}
}

@data_section`;

const result3 = expander.expand(test3);
console.log('Output:', result3.expanded);
console.log();

// Test 4: Complex macro with meta-variables
console.log('Test 4: Complex function macro');
const test4 = `#define function(name, body) {
  name:
  ; Function __MACRO_NAME__ at depth __DEPTH__
  ; Invocation #__INVOC_COUNT__
  body
  JALR RA, RA
}

@function(init, {
  LI R1, 0
  LI R2, 0
})

@function(main, {
  ADD R3, R1, R2
})`;

const result4 = expander.expand(test4);
console.log('Output:', result4.expanded);
console.log();

// Test 5: Nested macros
console.log('Test 5: Nested macros');
const test5 = `#define outer(x) {
  ; In __MACRO_NAME__
  @inner(x)
}

#define inner(y) {
  ; In __MACRO_NAME__, parent was __PARENT_MACRO__
  ADD R1, y, y
}

@outer(R5)`;

const result5 = expander.expand(test5);
console.log('Output:', result5.expanded);
