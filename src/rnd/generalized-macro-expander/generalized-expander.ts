// Generalized macro expander that can target different backends

import type { MacroBackend, BackendResult, GeneralizedExpanderOptions } from './types.ts';
import type { ContentNode, BodyNode, MacroInvocationNode, BuiltinFunctionNode, ExpressionNode } from '../../services/macro-expander/macro-parser.ts';
import { MacroExpanderV3 } from '../../services/macro-expander/macro-expander-v3.ts';

export class GeneralizedMacroExpander extends MacroExpanderV3 {
  private backend: MacroBackend;
  
  constructor(backend: MacroBackend) {
    super();
    this.backend = backend;
  }
  
  expandWithBackend(input: string, options?: GeneralizedExpanderOptions): BackendResult {
    // First, use the parent class to parse and expand macros
    const result = this.expand(input, {
      stripComments: options?.stripComments ?? false,
      collapseEmptyLines: false, // We'll handle this in backend
      generateSourceMap: false
    });
    
    if (result.errors.length > 0) {
      return {
        output: '',
        errors: result.errors.map(e => ({
          type: 'error' as const,
          message: e.message,
          location: e.location ? {
            line: e.location.line,
            column: e.location.column
          } : undefined
        }))
      };
    }
    
    // Convert the expanded string back to nodes for backend processing
    // This is a simplified approach - in a real implementation, we'd modify
    // the parent class to expose the expanded AST directly
    const expandedNodes = this.parseExpandedContent(result.expanded);
    
    // Let the backend generate the final output
    const output = this.backend.generateOutput(expandedNodes);
    
    // Run backend validation if available
    const validationErrors = [];
    if (this.backend.validateNode) {
      for (const node of expandedNodes) {
        validationErrors.push(...this.backend.validateNode(node));
      }
    }
    
    return {
      output: options?.collapseEmptyLines ? this.collapseEmpty(output) : output,
      errors: validationErrors
    };
  }
  
  private parseExpandedContent(content: string): ContentNode[] {
    // This is a simplified parser for the expanded content
    // In a real implementation, we'd modify the expander to return AST nodes
    const nodes: ContentNode[] = [];
    
    if (!content) return nodes;
    
    // Split by lines but preserve the line structure
    const lines = content.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.trim()) {
        nodes.push({
          type: 'Text',
          value: line + (i < lines.length - 1 ? '\n' : ''),
          position: { line: i + 1, column: 1, start: 0, end: line.length }
        } as any);
      } else if (i < lines.length - 1) {
        // Preserve empty lines as newlines
        nodes.push({
          type: 'Text',
          value: '\n',
          position: { line: i + 1, column: 1, start: 0, end: 0 }
        } as any);
      }
    }
    
    return nodes;
  }
  
  private collapseEmpty(output: string): string {
    return output
      .split('\n')
      .filter(line => line.trim().length > 0)
      .join('\n');
  }
  
  // Override to inject backend-specific builtins
  protected expandBuiltinFunction(
    node: BuiltinFunctionNode, 
    context: any, 
    generateSourceMap: boolean,
    sourceRange: any
  ): void {
    // Check if backend has this builtin
    if (this.backend.builtins && this.backend.builtins[node.name]) {
      const args = node.arguments.map(arg => this.expandExpressionToString(arg, context));
      const resultNodes = this.backend.builtins[node.name](args, context);
      
      // Convert nodes to string and append
      for (const resultNode of resultNodes) {
        const text = (resultNode as any).value || '';
        this.appendToExpanded(text, context, generateSourceMap, sourceRange);
      }
      return;
    }
    
    // Otherwise use parent implementation
    super.expandBuiltinFunction(node, context, generateSourceMap, sourceRange);
  }
  
  // Expose a clean API method
  static createWithBackend(backend: MacroBackend): GeneralizedMacroExpander {
    return new GeneralizedMacroExpander(backend);
  }
}