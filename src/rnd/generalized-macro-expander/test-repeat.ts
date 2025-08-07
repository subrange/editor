// Test repeat function specifically
import { createMacroExpanderV3 } from '../../services/macro-expander/macro-expander.ts';

const expander = createMacroExpanderV3();

console.log('=== Testing Repeat Function ===\n');

// Test 1: Simple repeat with BF command
console.log('Test 1: Repeat with BF command');
const test1 = `{repeat(3, +)}`;
const result1 = expander.expand(test1);
console.log('Input:', test1);
console.log('Output:', result1.expanded);
console.log('Errors:', result1.errors);

// Test 2: Repeat with text in quotes
console.log('\nTest 2: Repeat with quoted text');
const test2 = `{repeat(3, "hello")}`;
const result2 = expander.expand(test2);
console.log('Input:', test2);
console.log('Output:', result2.expanded);
console.log('Errors:', result2.errors);

// Test 3: Repeat with assembly command (no quotes)
console.log('\nTest 3: Repeat with assembly (no quotes)');
const test3 = `{repeat(3, ADDI R1 R2 1)}`;
const result3 = expander.expand(test3);
console.log('Input:', test3);
console.log('Output:', result3.expanded);
console.log('Errors:', result3.errors);

// Test 4: Repeat in a macro
console.log('\nTest 4: Repeat in macro');
const test4 = `#define test(n) {repeat(n, +)}
@test(5)`;
const result4 = expander.expand(test4);
console.log('Input:', test4);
console.log('Output:', result4.expanded);
console.log('Errors:', result4.errors);

// Test 5: Let's try using expression list
console.log('\nTest 5: Repeat with expression list');
const test5 = `{repeat(3, {+>})}`;
const result5 = expander.expand(test5);
console.log('Input:', test5);
console.log('Output:', result5.expanded);
console.log('Errors:', result5.errors);

// Test 6: Assembly in expression list
console.log('\nTest 6: Assembly in expression list');
const test6 = `{repeat(2, {ADD R1, R2, R3})}`;
const result6 = expander.expand(test6);
console.log('Input:', test6);
console.log('Output:', result6.expanded);
console.log('Errors:', result6.errors);