// Trace V4 expansion
import { MacroExpanderV4 } from '../../services/macro-expander/macro-expander-v4.ts';

// Extend the class to add logging
class TracingMacroExpanderV4 extends MacroExpanderV4 {
  protected expandBodyWithSubstitutions(
    body: any[], 
    substitutions: Record<string, string>,
    context: any
  ): void {
    console.log('expandBodyWithSubstitutions called');
    console.log('Body nodes:', body.map(n => ({ type: n.type, value: n.value || n.commands })));
    console.log('Context:', { 
      invocationCounter: context.invocationCounter,
      macroName: context.currentMacroName,
      depth: context.expansionDepth
    });
    super.expandBodyWithSubstitutions(body, substitutions, context);
  }
  
  protected expandMetaVariables(text: string, context: any): string {
    console.log('expandMetaVariables called');
    console.log('Input text:', JSON.stringify(text));
    console.log('Context:', {
      invocationCounter: context.invocationCounter,
      currentMacroName: context.currentMacroName
    });
    const result = super.expandMetaVariables(text, context);
    console.log('Output text:', JSON.stringify(result));
    return result;
  }
}

const expander = new TracingMacroExpanderV4();

const input = `#define test() "Label is $INVOC_COUNT"
@test()`;

console.log('=== Starting expansion ===');
const result = expander.expand(input);
console.log('\nFinal output:', JSON.stringify(result.expanded));