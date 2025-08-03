import type { 
  MacroDefinition, 
  MacroExpansionError, 
  MacroToken, 
  MacroExpanderOptions, 
  MacroExpanderResult,
  MacroExpander
} from './macro-expander.ts';
import { parseMacro } from './macro-parser.ts';
import type { ASTNode, ContentNode, MacroInvocationNode, BuiltinFunctionNode, ExpressionNode, BodyNode, ProgramNode, MacroDefinitionNode, CodeLineNode, BrainfuckCommandNode, TextNode, NumberNode, IdentifierNode, ExpressionListNode, ArrayLiteralNode } from './macro-parser.ts';

export class MacroExpanderV2 implements MacroExpander {
  private macros: Map<string, MacroDefinitionNode> = new Map();
  private errors: MacroExpansionError[] = [];
  private tokens: MacroToken[] = [];
  private expansionDepth = 0;
  private maxExpansionDepth = 100;
  private expansionChain: Set<string> = new Set();

  expand(input: string, options?: MacroExpanderOptions): MacroExpanderResult {
    const opts = {
      stripComments: true,
      collapseEmptyLines: false,
      ...options
    };

    // Reset state
    this.macros.clear();
    this.errors = [];
    this.tokens = [];
    this.expansionDepth = 0;
    this.expansionChain.clear();

    // Parse the input
    const parseResult = parseMacro(input, {
      preserveComments: !opts.stripComments
    });

    // Only add parse errors, not expansion errors from macro definitions
    this.errors.push(...parseResult.errors);
    this.tokens.push(...parseResult.tokens);

    // Collect macro definitions (first pass - just store them)
    this.collectMacroDefinitions(parseResult.ast);
    
    // Validate all macro definitions (second pass - check references)
    this.validateAllMacros();

    // Save errors from parsing/collection phase
    const definitionErrors = [...this.errors];

    // Clear errors before expansion
    this.errors = [];

    // Expand the AST
    const expanded = this.expandProgram(parseResult.ast);

    // Combine definition errors with expansion errors
    this.errors = [...definitionErrors, ...this.errors];

    // Post-process if needed
    let result = expanded;
    if (opts.collapseEmptyLines) {
      result = this.collapseEmptyLines(result);
    }

    // Convert macro definitions to the expected format
    const macroDefinitions: MacroDefinition[] = Array.from(this.macros.values()).map(node => ({
      name: node.name,
      parameters: node.parameters,
      body: this.nodeToString(node.body),
      sourceLocation: {
        line: node.position.line - 1,
        column: node.position.column - 1,
        length: node.position.end - node.position.start
      }
    }));

    return {
      expanded: result,
      errors: this.errors,
      tokens: this.tokens,
      macros: macroDefinitions
    };
  }

  private collectMacroDefinitions(ast: ProgramNode): void {
    for (const statement of ast.statements) {
      if (statement.type === 'MacroDefinition') {
        if (this.macros.has(statement.name)) {
          this.errors.push({
            type: 'syntax_error',
            message: `Duplicate macro definition: '${statement.name}'`,
            location: {
              line: statement.position.line - 1,
              column: statement.position.column - 1,
              length: statement.position.end - statement.position.start
            }
          });
        } else {
          this.macros.set(statement.name, statement);
        }
      }
    }
  }

  private validateAllMacros(): void {
    // Second pass - validate all macro definitions now that we have all of them
    for (const macro of this.macros.values()) {
      this.validateMacroDefinition(macro);
    }
  }

  private validateMacroDefinition(macro: MacroDefinitionNode): void {
    // Create a set of parameter names for this macro
    const paramSet = new Set(macro.parameters || []);
    
    // Validate the body
    this.validateNodes(macro.body, paramSet, macro.position);
  }

  private validateNodes(nodes: BodyNode[], validParams: Set<string>, macroPosition: { line: number; column: number }): void {
    for (const node of nodes) {
      if (node.type === 'MacroInvocation') {
        const invocation = node as MacroInvocationNode;
        // Check if the macro exists
        if (!this.macros.has(invocation.name)) {
          this.errors.push({
            type: 'undefined',
            message: `Macro '${invocation.name}' is not defined`,
            location: {
              line: invocation.position.line - 1,
              column: invocation.position.column - 1,
              length: invocation.position.end - invocation.position.start
            }
          });
        }
        // Validate arguments if any
        if (invocation.arguments) {
          for (const arg of invocation.arguments) {
            this.validateExpression(arg, validParams, macroPosition);
          }
        }
      } else if (node.type === 'BuiltinFunction') {
        const builtin = node as BuiltinFunctionNode;
        // Validate builtin function arguments
        for (const arg of builtin.arguments) {
          this.validateExpression(arg, validParams, macroPosition);
        }
      }
    }
  }

  private validateExpression(expr: ExpressionNode, validParams: Set<string>, macroPosition: { line: number; column: number }): void {
    switch (expr.type) {
      case 'MacroInvocation':
        const invocation = expr as MacroInvocationNode;
        if (!this.macros.has(invocation.name)) {
          this.errors.push({
            type: 'undefined',
            message: `Macro '${invocation.name}' is not defined`,
            location: {
              line: invocation.position.line - 1,
              column: invocation.position.column - 1,
              length: invocation.position.end - invocation.position.start
            }
          });
        }
        break;
      
      case 'BuiltinFunction':
        const builtin = expr as BuiltinFunctionNode;
        // For validation purposes, check if arguments would be valid
        if (builtin.name === 'repeat') {
          if (builtin.arguments.length === 2) {
            const countArg = builtin.arguments[0];
            // If it's a text node that's not a parameter, check if it's a valid number
            if (countArg.type === 'Text') {
              const text = (countArg as TextNode).value;
              if (!validParams.has(text) && isNaN(parseInt(text, 10))) {
                this.errors.push({
                  type: 'syntax_error',
                  message: `Invalid repeat count: ${text}`,
                  location: {
                    line: builtin.position.line - 1,
                    column: builtin.position.column - 1,
                    length: builtin.position.end - builtin.position.start
                  }
                });
              }
            }
          }
        }
        // Recursively validate builtin arguments
        for (const arg of builtin.arguments) {
          this.validateExpression(arg, validParams, macroPosition);
        }
        break;
      
      case 'ExpressionList':
        const list = expr as ExpressionListNode;
        for (const item of list.expressions) {
          this.validateExpression(item as ExpressionNode, validParams, macroPosition);
        }
        break;
    }
  }

  private expandProgram(ast: ProgramNode): string {
    const lines: string[] = [];
    
    for (const statement of ast.statements) {
      if (statement.type === 'CodeLine') {
        const expanded = this.expandCodeLine(statement);
        lines.push(expanded);
      } else if (statement.type === 'MacroDefinition') {
        // Macro definitions are replaced with empty lines
        lines.push('');
      }
    }
    
    return lines.join('\n');
  }

  private expandCodeLine(node: CodeLineNode): string {
    return this.expandContentList(node.content);
  }

  private expandContentList(content: ContentNode[]): string {
    let result = '';
    
    for (const node of content) {
      result += this.expandContent(node);
    }
    
    return result;
  }

  private expandContent(node: ContentNode): string {
    switch (node.type) {
      case 'BrainfuckCommand':
        return node.commands;
      
      case 'Text':
        return node.value;
      
      case 'MacroInvocation':
        return this.expandMacroInvocation(node);
      
      case 'BuiltinFunction':
        return this.expandBuiltinFunction(node);
      
      default:
        return '';
    }
  }

  private createInvocationSignature(node: MacroInvocationNode): string {
    // Create a unique signature for this macro invocation
    // Include multiple arguments to better differentiate invocations
    let signature = node.name;
    
    if (node.arguments && node.arguments.length > 0) {
      const argSignatures: string[] = [];
      
      // Include up to 3 arguments in the signature for better differentiation
      const argsToInclude = Math.min(3, node.arguments.length);
      
      for (let i = 0; i < argsToInclude; i++) {
        const arg = node.arguments[i];
        
        if (arg.type === 'Number') {
          argSignatures.push(arg.value.toString());
        } else if (arg.type === 'Identifier') {
          argSignatures.push(arg.name);
        } else if (arg.type === 'Text') {
          const textValue = (arg as any).value;
          if (textValue && textValue.length < 10) {
            argSignatures.push(textValue);
          } else if (textValue) {
            // For longer text, use a hash or first few chars
            argSignatures.push(textValue.substring(0, 8) + '…');
          }
        } else if (arg.type === 'BrainfuckCommand') {
          const commands = (arg as any).commands;
          if (commands && commands.length < 10) {
            argSignatures.push(commands);
          } else if (commands) {
            argSignatures.push(commands.substring(0, 8) + '…');
          }
        } else if (arg.type === 'MacroInvocation') {
          // For macro invocations as arguments, include the macro name
          argSignatures.push(`@${(arg as any).name}`);
        } else {
          // For complex expressions, use a type indicator
          argSignatures.push(`<${arg.type}>`);
        }
      }
      
      if (node.arguments.length > argsToInclude) {
        argSignatures.push('...');
      }
      
      signature += `(${argSignatures.join(', ')})`;
    }
    
    return signature;
  }

  private expandMacroInvocation(node: MacroInvocationNode): string {
    const macro = this.macros.get(node.name);
    
    if (!macro) {
      this.errors.push({
        type: 'undefined',
        message: `Macro '${node.name}' is not defined`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      return this.nodeToString([node]);
    }

    // Create a signature for this specific invocation
    const invocationSignature = this.createInvocationSignature(node);

    // Check for circular dependencies using the full signature
    if (this.expansionChain.has(invocationSignature)) {
      const chain = Array.from(this.expansionChain).join(' → ');
      this.errors.push({
        type: 'circular_dependency',
        message: `Circular macro dependency detected: ${chain} → ${invocationSignature}`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      return `@${node.name}`;
    }

    // Check expansion depth
    this.expansionDepth++;
    if (this.expansionDepth > this.maxExpansionDepth) {
      this.errors.push({
        type: 'syntax_error',
        message: `Maximum macro expansion depth exceeded`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      return `@${node.name}`;
    }

    this.expansionChain.add(invocationSignature);

    let expandedBody: string;
    
    if (node.arguments) {
      // Parameterized macro
      if (!macro.parameters) {
        this.errors.push({
          type: 'parameter_mismatch',
          message: `Macro '${node.name}' does not accept parameters`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        expandedBody = `@${node.name}(${this.expressionsToString(node.arguments)})`;
      } else if (node.arguments.length !== macro.parameters.length) {
        this.errors.push({
          type: 'parameter_mismatch',
          message: `Macro '${node.name}' expects ${macro.parameters.length} parameter(s), got ${node.arguments.length}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        expandedBody = `@${node.name}(${this.expressionsToString(node.arguments)})`;
      } else {
        // Substitute parameters
        expandedBody = this.substituteParameters(macro, node.arguments);
      }
    } else {
      // Simple macro
      if (macro.parameters && macro.parameters.length > 0) {
        this.errors.push({
          type: 'parameter_mismatch',
          message: `Macro '${node.name}' expects ${macro.parameters.length} parameter(s), got 0`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        expandedBody = `@${node.name}`;
      } else {
        expandedBody = this.expandBodyNodes(macro.body);
      }
    }

    this.expansionChain.delete(invocationSignature);
    this.expansionDepth--;

    return expandedBody;
  }

  private substituteParameters(macro: MacroDefinitionNode, args: ExpressionNode[]): string {
    const substitutions = new Map<string, string>();
    
    // Build substitution map
    for (let i = 0; i < macro.parameters!.length; i++) {
      const param = macro.parameters![i];
      const arg = args[i];
      // Expand the argument first
      const expandedArg = this.expandExpression(arg);
      substitutions.set(param, expandedArg);
    }
    
    // Perform substitution in the body
    return this.expandBodyNodesWithSubstitutions(macro.body, substitutions);
  }

  private expandBodyNodesWithSubstitutions(nodes: BodyNode[], substitutions: Map<string, string>): string {
    let result = '';
    
    for (const node of nodes) {
      if (node.type === 'Text') {
        // Check if this text contains any parameter references
        let text = node.value;
        // Sort by length descending to handle longer names first
        const sortedSubstitutions = Array.from(substitutions.entries())
          .sort((a, b) => b[0].length - a[0].length);
        
        for (const [param, value] of sortedSubstitutions) {
          // Simple replacement - replace all occurrences of the parameter
          text = text.split(param).join(value);
        }
        result += text;
      } else if (node.type === 'BuiltinFunction') {
        // Handle builtin functions with substitutions in their arguments
        const builtinNode = node as BuiltinFunctionNode;
        const expandedArgs = builtinNode.arguments.map(arg => 
          this.substituteInExpression(arg, substitutions)
        );
        
        const modifiedNode = {
          ...builtinNode,
          arguments: expandedArgs
        };
        
        result += this.expandBuiltinFunction(modifiedNode);
      } else if (node.type === 'MacroInvocation') {
        // Handle macro invocations with substitutions in their arguments
        const invocationNode = node as MacroInvocationNode;
        let modifiedNode = invocationNode;
        
        if (invocationNode.arguments) {
          const expandedArgs = invocationNode.arguments.map(arg => 
            this.substituteInExpression(arg, substitutions)
          );
          modifiedNode = {
            ...invocationNode,
            arguments: expandedArgs
          };
        }
        
        result += this.expandMacroInvocation(modifiedNode);
      } else {
        // For other node types, expand normally
        result += this.expandContent(node as ContentNode);
      }
    }
    
    return result;
  }

  private substituteInExpression(expr: ExpressionNode, substitutions: Map<string, string>): ExpressionNode {
    switch (expr.type) {
      case 'Identifier':
        const name = (expr as IdentifierNode).name;
        if (substitutions.has(name)) {
          // Convert to a number if possible, otherwise keep as text
          const value = substitutions.get(name)!;
          const num = parseInt(value, 10);
          if (!isNaN(num) && value.trim() === num.toString()) {
            return {
              type: 'Number',
              value: num,
              position: expr.position
            };
          }
          return {
            type: 'Text',
            value: value,
            position: expr.position
          } as any;
        }
        return expr;
      
      case 'Text':
        // Check if this text contains any parameter references
        const textNode = expr as TextNode;
        let text = textNode.value;
        
        // First check if the entire text is a parameter name
        if (substitutions.has(text)) {
          const value = substitutions.get(text)!;
          // Try to parse as number if it looks like one
          const num = parseInt(value, 10);
          if (!isNaN(num) && value.trim() === num.toString()) {
            return {
              type: 'Number',
              value: num,
              position: textNode.position
            };
          }
          return {
            type: 'Text',
            value: value,
            position: textNode.position
          } as TextNode;
        }
        
        // Otherwise check for parameter references within the text
        // Sort by length descending to handle longer names first
        const sortedSubstitutions = Array.from(substitutions.entries())
          .sort((a, b) => b[0].length - a[0].length);
        
        for (const [param, value] of sortedSubstitutions) {
          // Simple replacement - replace all occurrences of the parameter
          text = text.split(param).join(value);
        }
        if (text !== textNode.value) {
          return {
            type: 'Text',
            value: text,
            position: textNode.position
          } as TextNode;
        }
        return expr;
      
      case 'ExpressionList':
        const list = expr as ExpressionListNode;
        return {
          ...list,
          expressions: list.expressions.map(e => 
            this.substituteInExpression(e as any, substitutions) as any
          )
        };
      
      case 'BuiltinFunction':
        const builtin = expr as BuiltinFunctionNode;
        return {
          ...builtin,
          arguments: builtin.arguments.map(arg => 
            this.substituteInExpression(arg, substitutions)
          )
        };
      
      case 'MacroInvocation':
        const invocation = expr as MacroInvocationNode;
        if (invocation.arguments) {
          return {
            ...invocation,
            arguments: invocation.arguments.map(arg => 
              this.substituteInExpression(arg, substitutions)
            )
          };
        }
        return expr;
      
      case 'ArrayLiteral':
        const array = expr as ArrayLiteralNode;
        return {
          ...array,
          elements: array.elements.map(el => 
            this.substituteInExpression(el, substitutions)
          )
        };
      
      default:
        return expr;
    }
  }

  private expandBodyNodes(nodes: BodyNode[]): string {
    let result = '';
    for (const node of nodes) {
      result += this.expandContent(node as ContentNode);
    }
    return result;
  }

  private expandBuiltinFunction(node: BuiltinFunctionNode): string {
    if (node.name === 'repeat') {
      if (node.arguments.length !== 2) {
        this.errors.push({
          type: 'syntax_error',
          message: `repeat() expects exactly 2 arguments, got ${node.arguments.length}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        return this.nodeToString([node]);
      }
      
      // Evaluate count
      const countExpr = this.expandExpression(node.arguments[0]);
      const count = parseInt(countExpr.trim(), 10);
      
      if (isNaN(count) || count < 0) {
        this.errors.push({
          type: 'syntax_error',
          message: `Invalid repeat count: ${countExpr}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        return this.nodeToString([node]);
      }
      
      // Expand content - handle both expression nodes and content nodes
      let content: string;
      const arg = node.arguments[1];
      if (arg.type === 'BrainfuckCommand') {
        content = (arg as any).commands;
      } else {
        content = this.expandExpression(arg);
      }
      return content.repeat(count);
      
    } else if (node.name === 'if') {
      if (node.arguments.length !== 3) {
        this.errors.push({
          type: 'syntax_error',
          message: `if() expects exactly 3 arguments, got ${node.arguments.length}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        return this.nodeToString([node]);
      }
      
      // Evaluate condition
      const conditionExpr = this.expandExpression(node.arguments[0]);
      const condition = parseInt(conditionExpr.trim(), 10);
      
      if (isNaN(condition)) {
        this.errors.push({
          type: 'syntax_error',
          message: `Invalid if condition: ${conditionExpr}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        return this.nodeToString([node]);
      }
      
      // Select branch
      const selectedBranch = condition !== 0 ? node.arguments[1] : node.arguments[2];
      return this.expandExpression(selectedBranch);
      
    } else if (node.name === 'for') {
      if (node.arguments.length !== 3) {
        this.errors.push({
          type: 'syntax_error',
          message: `for() expects exactly 3 arguments, got ${node.arguments.length}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        return this.nodeToString([node]);
      }
      
      // Extract variable name, array, and body
      const varNode = node.arguments[0];
      const arrayNode = node.arguments[1];
      const bodyNode = node.arguments[2];
      
      if (varNode.type !== 'Identifier') {
        this.errors.push({
          type: 'syntax_error',
          message: `Expected variable name in for loop, got ${varNode.type}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        return this.nodeToString([node]);
      }
      
      const varName = (varNode as any).name;
      
      // Evaluate array expression
      let values: string[] = [];
      
      if (arrayNode.type === 'ArrayLiteral') {
        // Handle array literal {1, 2, 3}
        const arrayLiteral = arrayNode as any;
        values = arrayLiteral.elements.map((el: any) => this.expandExpression(el).trim());
      } else {
        // Handle macro invocations that might return arrays
        const expanded = this.expandExpression(arrayNode).trim();
        // Try to parse as comma-separated values
        if (expanded.startsWith('{') && expanded.endsWith('}')) {
          const inner = expanded.slice(1, -1);
          values = inner.split(',').map(v => v.trim());
        } else {
          this.errors.push({
            type: 'syntax_error',
            message: `Invalid array expression in for loop`,
            location: {
              line: node.position.line - 1,
              column: node.position.column - 1,
              length: node.position.end - node.position.start
            }
          });
          return this.nodeToString([node]);
        }
      }
      
      // Expand body for each value
      let result = '';
      for (const value of values) {
        // Create a temporary substitution for the loop variable
        const tempSubstitutions = new Map<string, string>();
        tempSubstitutions.set(varName, value);
        
        // Expand the body with substitution
        if (bodyNode.type === 'ExpressionList') {
          const list = bodyNode as ExpressionListNode;
          const substitutedNodes = list.expressions.map(expr => 
            this.substituteInExpression(expr as any, tempSubstitutions) as any
          );
          result += this.expandContentList(substitutedNodes);
        } else {
          const substituted = this.substituteInExpression(bodyNode, tempSubstitutions);
          result += this.expandExpression(substituted);
        }
      }
      
      return result;
      
    } else if (node.name === 'reverse') {
      if (node.arguments.length !== 1) {
        this.errors.push({
          type: 'syntax_error',
          message: `reverse() expects exactly 1 argument, got ${node.arguments.length}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        return this.nodeToString([node]);
      }
      
      const arrayArg = node.arguments[0];
      
      // Check if it's an array literal
      if (arrayArg.type !== 'ArrayLiteral') {
        // Try to expand it first in case it's a macro that returns an array
        const expanded = this.expandExpression(arrayArg).trim();
        if (expanded.startsWith('{') && expanded.endsWith('}')) {
          // Parse as array and reverse
          const inner = expanded.slice(1, -1);
          const values = inner.split(',').map(v => v.trim());
          return '{' + values.reverse().join(', ') + '}';
        } else {
          this.errors.push({
            type: 'syntax_error',
            message: `reverse() expects an array literal, got ${arrayArg.type}`,
            location: {
              line: node.position.line - 1,
              column: node.position.column - 1,
              length: node.position.end - node.position.start
            }
          });
          return this.nodeToString([node]);
        }
      }
      
      // Handle array literal directly
      const arrayLiteral = arrayArg as ArrayLiteralNode;
      const reversedElements = [...arrayLiteral.elements].reverse();
      
      // Return reversed array as array literal string
      const expandedElements = reversedElements.map(el => this.expandExpression(el));
      return '{' + expandedElements.join(', ') + '}';
    }
    
    return '';
  }

  private expandExpression(expr: ExpressionNode): string {
    switch (expr.type) {
      case 'Number':
        return expr.value.toString();
      
      case 'Identifier':
        return expr.name;
      
      case 'MacroInvocation':
        return this.expandMacroInvocation(expr);
      
      case 'BuiltinFunction':
        return this.expandBuiltinFunction(expr);
      
      case 'ExpressionList':
        return this.expandContentList(expr.expressions);
      
      case 'Text':
        // Handle Text nodes that appear as expressions (e.g., in function arguments)
        return (expr as any).value;
      
      case 'BrainfuckCommand':
        // Handle BF commands that appear as expressions
        return (expr as any).commands;
      
      case 'ArrayLiteral':
        // Return array literal as comma-separated values wrapped in braces
        const array = expr as ArrayLiteralNode;
        const elements = array.elements.map(el => this.expandExpression(el));
        return '{' + elements.join(', ') + '}';
      
      default:
        return '';
    }
  }

  private expressionsToString(expressions: ExpressionNode[]): string {
    return expressions.map(expr => this.nodeToString([expr as any])).join(', ');
  }

  private nodeToString(nodes: ASTNode[]): string {
    let result = '';
    
    for (const node of nodes) {
      switch (node.type) {
        case 'BrainfuckCommand':
          result += (node as BrainfuckCommandNode).commands;
          break;
        case 'Text':
          result += (node as TextNode).value;
          break;
        case 'Number':
          result += (node as NumberNode).value.toString();
          break;
        case 'Identifier':
          result += (node as IdentifierNode).name;
          break;
        case 'MacroInvocation':
          const macro = node as MacroInvocationNode;
          result += `@${macro.name}`;
          if (macro.arguments) {
            result += `(${this.expressionsToString(macro.arguments)})`;
          }
          break;
        case 'BuiltinFunction':
          const builtin = node as BuiltinFunctionNode;
          result += `{${builtin.name}(${this.expressionsToString(builtin.arguments)})}`;
          break;
        case 'ExpressionList':
          result += this.nodeToString((node as ExpressionListNode).expressions as any[]);
          break;
      }
    }
    
    return result;
  }

  private collapseEmptyLines(code: string): string {
    const lines = code.split('\n');
    const nonEmptyLines: string[] = [];
    
    for (const line of lines) {
      // A line is considered non-empty if it has any BF commands (including $ for breakpoints)
      if (line.match(/[><+\-.\[\]$]/)) {
        nonEmptyLines.push(line);
      }
    }

    return nonEmptyLines.join('\n');
  }
}

export function createMacroExpanderV2(): MacroExpander {
  return new MacroExpanderV2();
}