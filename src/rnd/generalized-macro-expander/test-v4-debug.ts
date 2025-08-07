// Debug V4 meta-variable expansion
import { createMacroExpanderV4 } from '../../services/macro-expander/macro-expander-v4.ts';

const expander = createMacroExpanderV4();

// Simple test
const input = `#define test() loop_$INVOC_COUNT:
@test()
@test()`;

console.log('Input:', input);
const result = expander.expand(input);
console.log('Output:', JSON.stringify(result.expanded));
console.log('Errors:', result.errors);

// Test with quoted text
const input2 = `#define test2() "Label is $INVOC_COUNT"
@test2()`;

console.log('\n\nInput2:', input2);
const result2 = expander.expand(input2);
console.log('Output2:', JSON.stringify(result2.expanded));

// Direct text node
const input3 = `This is invocation $INVOC_COUNT`;
console.log('\n\nInput3:', input3);
const result3 = expander.expand(input3);
console.log('Output3:', JSON.stringify(result3.expanded));