export interface MacroDefinition {
  name: string;
  parameters?: string[];
  body: string;
  sourceLocation: {
    line: number;
    column: number;
    length: number;
  };
}

export interface MacroExpansionError {
  type: 'undefined' | 'parameter_mismatch' | 'circular_dependency' | 'syntax_error';
  message: string;
  location?: {
    line: number;
    column: number;
    length: number;
  };
}

export interface MacroToken {
  type: 'macro_definition' | 'macro_invocation' | 'builtin_function';
  range: {
    start: number;
    end: number;
  };
  name: string;
}

export interface MacroExpanderOptions {
  stripComments?: boolean;
  collapseEmptyLines?: boolean;
}

export interface MacroExpanderResult {
  expanded: string;
  errors: MacroExpansionError[];
  tokens: MacroToken[];
  macros: MacroDefinition[];
}

export interface MacroExpander {
  expand(input: string, options?: MacroExpanderOptions): MacroExpanderResult;
}

// Import the new implementation
import { MacroExpanderV2 } from './macro-expander-v2.ts';

export function createMacroExpander(): MacroExpander {
  // Use the new lexer/parser-based implementation
  return new MacroExpanderV2();
}