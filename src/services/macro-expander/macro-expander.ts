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
  generateSourceMap?: boolean;
  enableCircularDependencyDetection?: boolean;
}

export interface MacroExpanderResult {
  expanded: string;
  errors: MacroExpansionError[];
  tokens: MacroToken[];
  macros: MacroDefinition[];
  sourceMap?: import('./source-map.ts').SourceMap;
}

export interface MacroExpander {
  expand(input: string, options?: MacroExpanderOptions): MacroExpanderResult;
}

// Import the implementations
import { MacroExpanderV3 } from './macro-expander-v3.ts';
import { createMacroExpanderWasm, initializeWasm } from './macro-expander-wasm.ts';

export function createMacroExpander(): MacroExpander {
  return createMacroExpanderV3();
}

export function createMacroExpanderV3(): MacroExpander {
  return new MacroExpanderV3();
}

export function createMacroExpanderRust(): MacroExpander {
  return createMacroExpanderWasm();
}

export { initializeWasm };
