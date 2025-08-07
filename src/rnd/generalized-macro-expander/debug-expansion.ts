// Debug script to see what the macro expander is producing
import { GeneralizedMacroExpander } from './generalized-expander.ts';
import { AssemblyBackend } from './assembly-backend.ts';
import { createMacroExpanderV3 } from '../../services/macro-expander/macro-expander.ts';

console.log('=== Debug Macro Expansion ===\n');

// Test 1: Simple macro
const simpleInput = `
#define inc(reg) ADDI reg, reg, 1
@inc(R3)
`;

console.log('Input:', simpleInput);

// First, see what the base expander produces
const baseExpander = createMacroExpanderV3();
const baseResult = baseExpander.expand(simpleInput);
console.log('Base expander output:', JSON.stringify(baseResult.expanded));
console.log('Base expander errors:', baseResult.errors);

// Now try with our generalized expander
const backend = new AssemblyBackend();
const expander = GeneralizedMacroExpander.createWithBackend(backend);
const result = expander.expandWithBackend(simpleInput);
console.log('Generalized expander output:', JSON.stringify(result.output));
console.log('Generalized expander errors:', result.errors);

// Test 2: Macro with spaces
console.log('\n--- Test 2: Spaces ---');
const spacesInput = `#define test ADD R1, R2, R3
@test`;

const spacesBase = baseExpander.expand(spacesInput);
console.log('Base output:', JSON.stringify(spacesBase.expanded));

const spacesResult = expander.expandWithBackend(spacesInput);
console.log('Generalized output:', JSON.stringify(spacesResult.output));

// Test 3: Repeat builtin
console.log('\n--- Test 3: Repeat ---');
const repeatInput = `#define move_right(n) {repeat(n, "ADDI R3, R3, 1")}
@move_right(3)`;

const repeatBase = baseExpander.expand(repeatInput);
console.log('Base output:', JSON.stringify(repeatBase.expanded));
console.log('Base errors:', repeatBase.errors);

// Test with newlines
console.log('\n--- Test 4: Newlines ---');
const newlineInput = `#define test {
  "Line 1"
  "Line 2"
}
@test`;

const newlineBase = baseExpander.expand(newlineInput);
console.log('Base output:', JSON.stringify(newlineBase.expanded));

// Test multiline assembly
console.log('\n--- Test 5: Multiline Assembly ---');
const multilineInput = `#define swap(a, b) {
  "ADD R8, " a ", R0"
  "ADD " a ", " b ", R0"
  "ADD " b ", R8, R0"
}
@swap("R3", "R4")`;

const multilineBase = baseExpander.expand(multilineInput);
console.log('Base output:', JSON.stringify(multilineBase.expanded));