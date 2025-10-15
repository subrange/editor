// Test to understand spacing behavior
import { createMacroExpanderV3 } from '../../services/macro-expander/macro-expander.ts';

const expander = createMacroExpanderV3();

console.log('=== Testing Spacing Behavior ===\n');

// Test 1: Simple text with spaces
console.log('Test 1: Simple text with spaces');
const test1 = `#define test "ADD R1, R2, R3"
@test`;
const result1 = expander.expand(test1);
console.log('Input:', JSON.stringify(test1));
console.log('Output:', JSON.stringify(result1.expanded));
console.log('Errors:', result1.errors);

// Test 2: Text without quotes
console.log('\nTest 2: Text without quotes');
const test2 = `#define test ADD R1, R2, R3
@test`;
const result2 = expander.expand(test2);
console.log('Input:', JSON.stringify(test2));
console.log('Output:', JSON.stringify(result2.expanded));

// Test 3: Using curly braces
console.log('\nTest 3: Using curly braces');
const test3 = `#define test { ADD R1, R2, R3 }
@test`;
const result3 = expander.expand(test3);
console.log('Input:', JSON.stringify(test3));
console.log('Output:', JSON.stringify(result3.expanded));

// Test 4: Multiple lines in curly braces
console.log('\nTest 4: Multiple lines in curly braces');
const test4 = `#define test {
  ADD R1, R2, R3
  SUB R4, R5, R6
}
@test`;
const result4 = expander.expand(test4);
console.log('Input:', JSON.stringify(test4));
console.log('Output:', JSON.stringify(result4.expanded));

// Test 5: Repeat with spaces
console.log('\nTest 5: Repeat with spaces');
const test5 = `#define test {repeat(2, "ADD R1, R2, R3\\n")}
@test`;
const result5 = expander.expand(test5);
console.log('Input:', JSON.stringify(test5));
console.log('Output:', JSON.stringify(result5.expanded));

// Test 6: Text concatenation
console.log('\nTest 6: Text concatenation');
const test6 = `#define cmd(op, a, b, c) op " " a ", " b ", " c
@cmd(ADD, R1, R2, R3)`;
const result6 = expander.expand(test6);
console.log('Input:', JSON.stringify(test6));
console.log('Output:', JSON.stringify(result6.expanded));
