// Assembly backend for MacroExpanderV4
import type { MacroBackend, BackendOptions, ExpandedNode, BuiltinHandler } from '../../services/macro-expander/macro-expander-v4.ts';

export class AssemblyBackendV4 implements MacroBackend {
  name = 'ripple-asm-v4';
  
  generate(nodes: ExpandedNode[], options: BackendOptions): string {
    // Debug logging
    // console.log('Input nodes:', nodes.map(n => ({ type: n.type, value: JSON.stringify(n.value) })));
    
    // First pass: filter and process nodes
    const processedNodes: ExpandedNode[] = [];
    
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      const prev = i > 0 ? nodes[i - 1] : null;
      const next = i < nodes.length - 1 ? nodes[i + 1] : null;
      
      // Skip whitespace nodes around commas
      if (node.type === 'whitespace' && 
          ((prev?.type === 'text' && prev.value.endsWith(',')) || 
           (next?.type === 'text' && next.value === ','))) {
        continue;
      }
      
      processedNodes.push(node);
    }
    
    // Second pass: generate output
    let result = '';
    let lastWasText = false;
    let lastNode: ExpandedNode | null = null;
    let inComment = false;
    
    for (const node of processedNodes) {
      switch (node.type) {
        case 'text':
          // Check if we're starting a comment
          if (node.value === ';') {
            inComment = true;
            result += node.value;
            // Always add a space after semicolon for consistency
            result += ' ';
            lastWasText = true;
            break;
          }
          
          
          // If we were in a comment and now we see an instruction keyword, add newline
          if (inComment && this.isAssemblyInstruction(node.value)) {
            result += '\n';
            inComment = false;
          }
          
          // Add space between text nodes that need it
          if (lastWasText && 
              lastNode?.type === 'text' &&
              !this.endsWithSpecialChar(lastNode.value) && 
              !this.startsWithSpecialChar(node.value) &&
              !lastNode.value.endsWith(',') &&
              node.value !== ',') {
            result += ' ';
          }
          result += node.value;
          // Add space after comma
          if (node.value === ',') {
            result += ' ';
          }
          lastWasText = true;
          break;
          
        case 'command':
          // Brainfuck commands become comments in assembly
          result += `; BF: ${node.value}`;
          lastWasText = false;
          break;
          
        case 'whitespace': {
          // Add a single space for remaining whitespace nodes
          // But skip if the previous node was a newline (redundant whitespace)
          // Also skip if we're in a comment and about to see an instruction
          const nextNode = processedNodes[processedNodes.indexOf(node) + 1];
          const skipSpace = inComment && nextNode?.type === 'text' && this.isAssemblyInstruction(nextNode.value);
          
          // Skip whitespace immediately after semicolon (we already added a space)
          if (lastNode?.type === 'text' && lastNode.value === ';') {
            lastWasText = false;
            break;
          }
          
          if (lastWasText && lastNode?.type !== 'newline' && !skipSpace) {
            result += ' ';
          }
          lastWasText = false;
          break;
        }
          
        case 'newline':
          result += '\n';
          lastWasText = false;
          inComment = false;  // Reset comment flag on newline
          break;
          
        case 'comment':
          if (options.preserveComments) {
            result += node.value;
          }
          break;
      }
      lastNode = node;
    }
    
    // Clean up the output
    if (options.collapseEmptyLines) {
      const lines = result.split('\n');
      const nonEmpty = lines.filter(line => line.trim().length > 0);
      result = nonEmpty.join('\n');
    }
    
    return result.trim();
  }
  
  private isSpecialChar(char: string): boolean {
    return /[;:[\]{}()#]/.test(char);  // Added # as special char
  }
  
  private startsWithSpecialChar(text: string): boolean {
    return text.length > 0 && this.isSpecialChar(text[0]);
  }
  
  private endsWithSpecialChar(text: string): boolean {
    return text.length > 0 && this.isSpecialChar(text[text.length - 1]);
  }
  
  private isAssemblyInstruction(text: string): boolean {
    // Common assembly instruction mnemonics
    const instructions = [
      'ADD', 'SUB', 'MUL', 'DIV', 'AND', 'OR', 'XOR', 'NOT',
      'LI', 'LOAD', 'STORE', 'MOVE', 'MOV',
      'ADDI', 'SUBI', 'ANDI', 'ORI', 'XORI',
      'BEQ', 'BNE', 'BLT', 'BGT', 'BLE', 'BGE',
      'JMP', 'JAL', 'JALR', 'RET',
      'PUSH', 'POP', 'CALL', 'NOP'
    ];
    return instructions.includes(text.toUpperCase());
  }
  
  
  // Assembly-specific builtins
  builtins = new Map<string, BuiltinHandler>([
    ['align', {
      name: 'align',
      expand: (args) => {
        const alignment = args[0] ? args[0].toString() : '4';
        return [{
          type: 'text',
          value: `.align ${alignment}`
        }];
      }
    }],
    
    ['db', {
      name: 'db',
      expand: (args) => {
        const values = args.map(arg => arg.toString()).join(', ');
        return [{
          type: 'text',
          value: `.byte ${values}`
        }];
      }
    }],
    
    ['dw', {
      name: 'dw', 
      expand: (args) => {
        const values = args.map(arg => arg.toString()).join(', ');
        return [{
          type: 'text',
          value: `.word ${values}`
        }];
      }
    }]
  ]);
}