// Test V4 meta-programming features
import { createMacroExpanderV4 } from '../../services/macro-expander/macro-expander-v4.ts';

console.log('=== Testing V4 Meta-Programming Features ===\n');

const expander = createMacroExpanderV4();

// Test 1: Unique labels with $LABEL
console.log('Test 1: Unique labels');
const test1 = `#define loop(body) {
  $LABEL(loop):
  body
  JAL $LABEL(loop)
}

@loop({ ADD R1, R2, R3 })
@loop({ SUB R4, R5, R6 })`;

const result1 = expander.expand(test1);
console.log('Input:', test1);
console.log('Output:', result1.expanded);
console.log('Errors:', result1.errors);

// Test 2: Invocation counter
console.log('\n\nTest 2: Invocation counter');
const test2 = `#define track() "Call #$INVOC_COUNT"
@track()
@track()
@track()`;

const result2 = expander.expand(test2);
console.log('Input:', test2);
console.log('Output:', result2.expanded);

// Test 3: Complex loop with unique labels
console.log('\n\nTest 3: Complex loop');
const test3 = `#define for_loop(counter, start, end, body) {
  LI counter, start
  $LABEL(for_start):
  body
  ADDI counter, counter, 1
  BNE counter, end, $LABEL(for_start)
}

@for_loop(R3, 0, 10, {
  ADD R4, R4, R3
})

@for_loop(R5, 0, 5, {
  SUB R6, R6, R5
})`;

const result3 = expander.expand(test3);
console.log('Input:', test3);
console.log('Output:', result3.expanded);

// Test 4: Macro name and depth
console.log('\n\nTest 4: Macro context info');
const test4 = `#define outer() {
  "In $MACRO_NAME at depth $DEPTH"
  @inner()
}

#define inner() {
  "In $MACRO_NAME at depth $DEPTH, parent was $PARENT_MACRO"
}

@outer()`;

const result4 = expander.expand(test4);
console.log('Input:', test4);
console.log('Output:', result4.expanded);
