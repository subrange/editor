import { describe, it, expect } from 'vitest';
import { createMacroExpander } from './macro-expander.ts';
import { parseMacro } from './macro-parser.ts';

describe('Debug for loops', () => {
  it('should debug simple for loop', () => {
    const input = `{for(x in {1, 2, 3}, x )}`;
    
    const expander = createMacroExpander();
    const result = expander.expand(input);
    
    console.log('Input:', input);
    console.log('Errors:', result.errors);
    console.log('Expanded:', JSON.stringify(result.expanded));
    console.log('Expected: "1 2 3 "');
  });

  it('should debug for loop parsing', () => {
    const input = `{for(x in {1, 2, 3}, x )}`;
    
    const parseResult = parseMacro(input);
    console.log('Parse errors:', parseResult.errors);
    console.log('AST:', JSON.stringify(parseResult.ast, null, 2));
    
    const forNode = parseResult.ast.statements[0].content[0];
    if (forNode && forNode.type === 'BuiltinFunction') {
      console.log('Body argument:', forNode.arguments[2]);
      console.log('Body value:', JSON.stringify(forNode.arguments[2].value));
    }
  });

  it('should debug nested for loops', () => {
    const input = `{for(i in {1, 2}, {for(j in {a, b}, i j)})}`;
    
    const expander = createMacroExpander();
    const result = expander.expand(input);
    
    console.log('Input:', input);
    console.log('Errors:', result.errors);
    console.log('Expanded:', JSON.stringify(result.expanded));
    console.log('Expected: "1 a1 b2 a2 b"');
  });
});