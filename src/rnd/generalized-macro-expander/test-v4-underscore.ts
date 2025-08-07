// Test V4 with double-underscore meta-variables
import { createMacroExpanderV4 } from '../../services/macro-expander/macro-expander-v4.ts';

console.log('=== Testing V4 with __META__ Variables ===\n');

const expander = createMacroExpanderV4();

// Test 1: Unique labels
console.log('Test 1: Unique labels');
const test1 = `#define loop(body) {
  __LABEL__loop:
  body
  JAL __LABEL__loop
}

@loop({ ADD R1, R2, R3 })
@loop({ SUB R4, R5, R6 })`;

const result1 = expander.expand(test1);
console.log('Output:', result1.expanded);
console.log();

// Test 2: Invocation counter
console.log('Test 2: Invocation counter');
const test2 = `#define track() "Call #__INVOC_COUNT__"
@track()
@track()
@track()`;

const result2 = expander.expand(test2);
console.log('Output:', result2.expanded);
console.log();

// Test 3: Complex loop with unique labels
console.log('Test 3: Complex loop');
const test3 = `#define for_loop(counter, start, end, body) {
  LI counter, start
  __LABEL__for_start:
  body
  ADDI counter, counter, 1
  BNE counter, end, __LABEL__for_start
}

@for_loop(R3, 0, 10, {
  ADD R4, R4, R3
})

@for_loop(R5, 0, 5, {
  SUB R6, R6, R5
})`;

const result3 = expander.expand(test3);
console.log('Output:', result3.expanded);
console.log();

// Test 4: Simpler label syntax
console.log('Test 4: Simpler label syntax');
const test4 = `#define block(name, code) {
  __LABEL__name:
  code
  JAL __LABEL__name
}

@block(init, { LI R1, 0 })
@block(main, { ADD R1, R1, 1 })`;

const result4 = expander.expand(test4);
console.log('Output:', result4.expanded);
console.log();

// Test 5: Counter
console.log('Test 5: Counter');
const test5 = `#define item() "Item __COUNTER__, again __COUNTER__"
@item()
@item()`;

const result5 = expander.expand(test5);
console.log('Output:', result5.expanded);