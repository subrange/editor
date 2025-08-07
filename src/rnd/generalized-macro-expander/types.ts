// Core types for the generalized macro expander

import type { ASTNode, ContentNode, BodyNode } from '../../services/macro-expander/macro-parser.ts';

export interface MacroBackend {
  name: string;
  fileExtension: string;
  
  // Transform expanded content to target language
  generateOutput(expandedNodes: ContentNode[]): string;
  
  // Backend-specific builtin functions
  builtins?: {
    [name: string]: (args: any[], context: any) => ContentNode[];
  };
  
  // Validation rules
  validateNode?(node: ASTNode): ValidationError[];
}

export interface BackendResult {
  output: string;
  errors: ValidationError[];
}

export interface ValidationError {
  type: 'error' | 'warning';
  message: string;
  location?: {
    line: number;
    column: number;
  };
}

export interface GeneralizedExpanderOptions {
  stripComments?: boolean;
  collapseEmptyLines?: boolean;
  preserveWhitespace?: boolean;
}

// Extension to handle backend-specific content
export interface BackendContentNode extends ContentNode {
  backendHint?: string; // Hints for backend-specific handling
}