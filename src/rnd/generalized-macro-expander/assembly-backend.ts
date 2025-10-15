// Assembly backend implementation for the generalized macro expander

import type { MacroBackend, ValidationError } from './types.ts';
import type {
  ContentNode,
  TextNode,
  BrainfuckCommandNode,
} from '../../services/macro-expander/macro-parser.ts';

export class AssemblyBackend implements MacroBackend {
  name = 'ripple-asm';
  fileExtension = '.asm';

  private labelCounter = 0;

  generateOutput(expandedNodes: ContentNode[]): string {
    // Simply concatenate all content, preserving spaces and newlines
    let output = '';

    for (const node of expandedNodes) {
      output += this.nodeToString(node);
    }

    // Clean up the output while preserving necessary spaces
    const lines = output.split('\n');
    const processedLines: string[] = [];

    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed) {
        // For assembly, preserve spaces between tokens
        // but normalize multiple spaces to single space
        const normalized = trimmed.replace(/\s+/g, ' ');
        processedLines.push(normalized);
      }
    }

    return processedLines.join('\n');
  }

  private nodeToString(node: ContentNode): string {
    switch (node.type) {
      case 'Text':
        return (node as TextNode).value;

      case 'BrainfuckCommand':
        // For assembly backend, we might want to ignore or translate BF commands
        // For now, let's treat them as comments
        const commands = (node as BrainfuckCommandNode).commands;
        return `; BF: ${commands}`;

      default:
        return '';
    }
  }

  validateNode(node: any): ValidationError[] {
    // Basic validation - could be extended
    const errors: ValidationError[] = [];

    if (node.type === 'BrainfuckCommand') {
      errors.push({
        type: 'warning',
        message:
          'Brainfuck commands in assembly context will be treated as comments',
        location: node.position,
      });
    }

    return errors;
  }

  // Assembly-specific builtins could be added here
  builtins = {
    // Example: align builtin
    align: (args: any[]) => {
      const alignment = args[0] || 4;
      return [
        {
          type: 'Text' as const,
          value: `.align ${alignment}`,
          position: { line: 1, column: 1, start: 0, end: 0 },
        },
      ];
    },

    // Generate unique labels
    genlabel: (args: any[]) => {
      const prefix = args[0] || 'L';
      const label = `${prefix}_${this.labelCounter++}`;
      return [
        {
          type: 'Text' as const,
          value: label,
          position: { line: 1, column: 1, start: 0, end: 0 },
        },
      ];
    },
  };
}
