/**
 * Macro Expander V4 - Backend-agnostic macro expansion with proper whitespace handling
 * 
 * Key improvements over V3:
 * - Preserves AST structure through expansion
 * - Proper whitespace and newline handling
 * - Pluggable backend system
 * - Better separation of concerns
 */

import type {
  MacroExpansionError,
  MacroToken,
  MacroExpanderOptions,
  MacroExpanderResult,
  MacroExpander,
  MacroDefinition
} from './macro-expander.ts';
import { parseMacro } from './macro-parser.ts';
import type {
  ASTNode,
  ProgramNode,
  MacroDefinitionNode,
  ContentNode,
  BodyNode,
  MacroInvocationNode,
  BuiltinFunctionNode,
  ExpressionNode,
  TextNode,
  NumberNode,
  IdentifierNode,
  ExpressionListNode,
  ArrayLiteralNode,
  TuplePatternNode,
  BrainfuckCommandNode
} from './macro-parser.ts';
import { SourceMapBuilder, type Range, type Position, type SourceMap } from './source-map.ts';

// Backend interface
export interface MacroBackend {
  name: string;
  
  // Convert expanded AST nodes to output
  generate(nodes: ExpandedNode[], options: BackendOptions): string;
  
  // Backend-specific builtin functions
  builtins?: Map<string, BuiltinHandler>;
  
  // Validate nodes for backend-specific rules
  validate?(nodes: ExpandedNode[]): ValidationError[];
}

export interface BackendOptions {
  preserveWhitespace?: boolean;
  preserveNewlines?: boolean;
  preserveComments?: boolean;
  collapseEmptyLines?: boolean;
}

export interface BuiltinHandler {
  name: string;
  expand(args: ExpressionNode[], context: ExpansionContext): ExpandedNode[];
}

export interface ValidationError {
  type: 'error' | 'warning';
  message: string;
  location?: Position;
}

// Expanded nodes preserve more information than raw AST nodes
export interface ExpandedNode {
  type: 'text' | 'command' | 'newline' | 'whitespace' | 'comment';
  value: string;
  source?: Range;
  macroContext?: MacroInvocation[];
}

export interface MacroInvocation {
  name: string;
  callSite: Range;
  parameters?: Record<string, string>;
  parameterNodes?: Record<string, ExpressionNode>;
}

interface ExpansionContext {
  sourceMapBuilder: SourceMapBuilder;
  currentSourcePosition: Position;
  expansionDepth: number;
  macroCallStack: MacroInvocation[];
  expandedNodes: ExpandedNode[];
  backend?: MacroBackend;
  options: MacroExpanderOptions & BackendOptions;
  // Meta-programming state
  invocationCounter: number;
  labelCounters: Map<string, number>;
  currentMacroName?: string;
  currentLineNumber?: number;
}

export class MacroExpanderV4 implements MacroExpander {
  private macros: Map<string, MacroDefinitionNode> = new Map();
  private errors: MacroExpansionError[] = [];
  private tokens: MacroToken[] = [];
  private backend?: MacroBackend;
  private maxExpansionDepth = 100;
  
  constructor(backend?: MacroBackend) {
    this.backend = backend;
  }
  
  setBackend(backend: MacroBackend): void {
    this.backend = backend;
  }
  
  expand(input: string, options?: MacroExpanderOptions & BackendOptions): MacroExpanderResult {
    const opts = {
      stripComments: true,
      collapseEmptyLines: false,
      generateSourceMap: false,
      preserveWhitespace: true,
      preserveNewlines: true,
      ...options
    };
    
    // Reset state
    this.macros.clear();
    this.errors = [];
    this.tokens = [];
    
    // Parse the input
    const parseResult = parseMacro(input, {
      preserveComments: !opts.stripComments,
      preserveWhitespace: opts.preserveWhitespace
    });
    
    this.errors.push(...parseResult.errors);
    this.tokens.push(...parseResult.tokens);
    
    // Collect macro definitions
    this.collectMacroDefinitions(parseResult.ast);
    
    // Create expansion context
    const context: ExpansionContext = {
      sourceMapBuilder: new SourceMapBuilder(),
      currentSourcePosition: { line: 1, column: 1, offset: 0 },
      expansionDepth: 0,
      macroCallStack: [],
      expandedNodes: [],
      backend: this.backend,
      options: opts,
      invocationCounter: 0,
      labelCounters: new Map(),
      currentLineNumber: 1
    };
    
    // Expand the AST
    this.expandProgram(parseResult.ast, context);
    
    // Generate output
    let expanded: string;
    if (this.backend) {
      expanded = this.backend.generate(context.expandedNodes, opts);
    } else {
      // Default behavior - concatenate text nodes
      expanded = this.defaultGenerate(context.expandedNodes, opts);
    }
    
    // Convert macro definitions to expected format
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
    
    const result: MacroExpanderResult = {
      expanded,
      errors: this.errors,
      tokens: this.tokens,
      macros: macroDefinitions
    };
    
    if (opts.generateSourceMap) {
      result.sourceMap = context.sourceMapBuilder.build();
    }
    
    return result;
  }
  
  private collectMacroDefinitions(ast: ProgramNode): void {
    for (const statement of ast.statements) {
      if (statement.type === 'MacroDefinition') {
        const macro = statement as MacroDefinitionNode;
        if (this.macros.has(macro.name)) {
          this.errors.push({
            type: 'syntax_error',
            message: `Duplicate macro definition: '${macro.name}'`,
            location: {
              line: macro.position.line - 1,
              column: macro.position.column - 1,
              length: macro.position.end - macro.position.start
            }
          });
        } else {
          this.macros.set(macro.name, macro);
        }
      }
    }
  }
  
  private expandProgram(ast: ProgramNode, context: ExpansionContext): void {
    for (const statement of ast.statements) {
      if (statement.type === 'CodeLine') {
        this.expandCodeLine(statement, context);
        // Add newline after code line
        context.expandedNodes.push({
          type: 'newline',
          value: '\n'
        });
      } else if (statement.type === 'MacroDefinition') {
        // Skip macro definitions in output
        // But add empty lines to preserve line numbers
        context.expandedNodes.push({
          type: 'newline',
          value: '\n'
        });
      }
    }
  }
  
  private expandCodeLine(node: any, context: ExpansionContext): void {
    for (const content of node.content) {
      this.expandContent(content, context);
    }
  }
  
  private expandContent(node: ContentNode, context: ExpansionContext): void {
    switch (node.type) {
      case 'BrainfuckCommand':
        this.expandBrainfuckCommand(node as BrainfuckCommandNode, context);
        break;
        
      case 'Text':
        this.expandText(node as TextNode, context);
        break;
        
      case 'MacroInvocation':
        this.expandMacroInvocation(node as MacroInvocationNode, context);
        break;
        
      case 'BuiltinFunction':
        this.expandBuiltinFunction(node as BuiltinFunctionNode, context);
        break;
    }
  }
  
  private expandBrainfuckCommand(node: BrainfuckCommandNode, context: ExpansionContext): void {
    context.expandedNodes.push({
      type: 'command',
      value: node.commands,
      source: this.nodeToRange(node)
    });
  }
  
  private expandText(node: TextNode, context: ExpansionContext): void {
    let value = node.value;
    
    // Apply parameter substitution if we're inside a macro/for context
    if (context.macroCallStack.length > 0) {
      const currentContext = context.macroCallStack[context.macroCallStack.length - 1];
      if (currentContext.parameters) {
        // Apply parameter substitutions
        for (const [param, paramValue] of Object.entries(currentContext.parameters)) {
          const regex = new RegExp(`\\b${param}\\b`, 'g');
          value = value.replace(regex, paramValue);
        }
      }
      // Then expand meta-variables
      value = this.expandMetaVariables(value, context);
    }
    
    // Split text into segments preserving whitespace
    let i = 0;
    while (i < value.length) {
      // Collect whitespace
      let whitespace = '';
      while (i < value.length && /\s/.test(value[i])) {
        whitespace += value[i];
        i++;
      }
      
      if (whitespace) {
        // Check for newlines
        const lines = whitespace.split('\n');
        for (let j = 0; j < lines.length; j++) {
          if (j > 0) {
            context.expandedNodes.push({
              type: 'newline',
              value: '\n'
            });
          }
          if (lines[j]) {
            context.expandedNodes.push({
              type: 'whitespace',
              value: lines[j]
            });
          }
        }
      }
      
      // Collect non-whitespace
      let text = '';
      while (i < value.length && !/\s/.test(value[i])) {
        text += value[i];
        i++;
      }
      
      if (text) {
        context.expandedNodes.push({
          type: 'text',
          value: text,
          source: this.nodeToRange(node)
        });
      }
    }
  }
  
  private expandMacroInvocation(node: MacroInvocationNode, context: ExpansionContext): void {
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
      return;
    }
    
    // Increment invocation counter
    context.invocationCounter++;
    
    // Check expansion depth
    context.expansionDepth++;
    if (context.expansionDepth > this.maxExpansionDepth) {
      this.errors.push({
        type: 'syntax_error',
        message: `Maximum macro expansion depth exceeded`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      context.expansionDepth--;
      return;
    }
    
    // Prepare parameter substitutions
    const parameters: Record<string, string> = {};
    const parameterNodes: Record<string, ExpressionNode> = {};
    if (macro.parameters && node.arguments) {
      for (let i = 0; i < macro.parameters.length; i++) {
        if (i < node.arguments.length) {
          parameters[macro.parameters[i]] = this.expandExpressionToString(node.arguments[i], context);
          parameterNodes[macro.parameters[i]] = node.arguments[i];
        }
      }
    }
    
    // Push macro context
    context.macroCallStack.push({
      name: node.name,
      callSite: this.nodeToRange(node),
      parameters,
      parameterNodes
    });
    
    // Save previous macro name and set current
    const previousMacroName = context.currentMacroName;
    context.currentMacroName = node.name;
    
    // Create a new label scope for this invocation
    const savedLabelCounters = new Map(context.labelCounters);
    context.labelCounters.clear();
    
    // Expand macro body with substitutions
    this.expandBodyWithSubstitutions(macro.body, parameters, context);
    
    // Restore previous macro name
    context.currentMacroName = previousMacroName;
    
    // Restore label counters
    context.labelCounters = savedLabelCounters;
    
    // Pop macro context
    context.macroCallStack.pop();
    context.expansionDepth--;
  }
  
  private expandBuiltinFunction(node: BuiltinFunctionNode, context: ExpansionContext): void {
    // Check if backend provides this builtin
    if (context.backend?.builtins?.has(node.name)) {
      const handler = context.backend.builtins.get(node.name)!;
      const expandedNodes = handler.expand(node.arguments, context);
      context.expandedNodes.push(...expandedNodes);
      return;
    }
    
    // Handle standard builtins
    switch (node.name) {
      case 'repeat':
        this.expandRepeat(node, context);
        break;
        
      case 'if':
        this.expandIf(node, context);
        break;
        
      case 'for':
        this.expandFor(node, context);
        break;
        
      default:
        this.errors.push({
          type: 'syntax_error',
          message: `Unknown builtin function: ${node.name}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
    }
  }
  
  private expandRepeat(node: BuiltinFunctionNode, context: ExpansionContext): void {
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
      return;
    }
    
    const countStr = this.expandExpressionToString(node.arguments[0], context);
    const count = parseInt(countStr, 10);
    
    if (isNaN(count) || count < 0) {
      this.errors.push({
        type: 'syntax_error',
        message: `Invalid repeat count: ${countStr}`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      return;
    }
    
    // Repeat the content
    for (let i = 0; i < count; i++) {
      if (i > 0) {
        // Add newline between repetitions
        context.expandedNodes.push({
          type: 'newline',
          value: '\n'
        });
      }
      this.expandExpression(node.arguments[1], context);
    }
  }
  
  private expandIf(node: BuiltinFunctionNode, context: ExpansionContext): void {
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
      return;
    }
    
    const conditionStr = this.expandExpressionToString(node.arguments[0], context);
    const condition = parseInt(conditionStr, 10);
    
    if (isNaN(condition)) {
      this.errors.push({
        type: 'syntax_error',
        message: `Invalid if condition: ${conditionStr}`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      return;
    }
    
    // Expand the selected branch
    const branch = condition !== 0 ? node.arguments[1] : node.arguments[2];
    this.expandExpression(branch, context);
  }
  
  private expandFor(node: BuiltinFunctionNode, context: ExpansionContext): void {
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
      return;
    }
    
    // Extract the variable name
    const varNode = node.arguments[0];
    if (varNode.type !== 'Identifier') {
      this.errors.push({
        type: 'syntax_error',
        message: `for() expects a variable name as first argument`,
        location: {
          line: varNode.position.line - 1,
          column: varNode.position.column - 1,
          length: varNode.position.end - varNode.position.start
        }
      });
      return;
    }
    const varName = (varNode as IdentifierNode).name;
    
    // Extract the array
    const arrayNode = node.arguments[1];
    let array: ExpressionNode[];
    
    if (arrayNode.type === 'ArrayLiteral') {
      array = (arrayNode as ArrayLiteralNode).elements;
    } else if (arrayNode.type === 'Identifier' && context.macroCallStack.length > 0) {
      // Check if this identifier can be substituted with an array
      const currentMacro = context.macroCallStack[context.macroCallStack.length - 1];
      const idName = (arrayNode as IdentifierNode).name;
      
      if (currentMacro.parameterNodes && currentMacro.parameterNodes[idName]) {
        const substitutedNode = currentMacro.parameterNodes[idName];
        if (substitutedNode.type === 'ArrayLiteral') {
          array = (substitutedNode as ArrayLiteralNode).elements;
        } else {
          this.errors.push({
            type: 'syntax_error',
            message: `for() expects an array as second argument, but parameter '${idName}' is not an array`,
            location: {
              line: arrayNode.position.line - 1,
              column: arrayNode.position.column - 1,
              length: arrayNode.position.end - arrayNode.position.start
            }
          });
          return;
        }
      } else {
        this.errors.push({
          type: 'syntax_error',
          message: `for() expects an array as second argument`,
          location: {
            line: arrayNode.position.line - 1,
            column: arrayNode.position.column - 1,
            length: arrayNode.position.end - arrayNode.position.start
          }
        });
        return;
      }
    } else {
      this.errors.push({
        type: 'syntax_error',
        message: `for() expects an array as second argument`,
        location: {
          line: arrayNode.position.line - 1,
          column: arrayNode.position.column - 1,
          length: arrayNode.position.end - arrayNode.position.start
        }
      });
      return;
    }
    
    // Get the body
    const body = node.arguments[2];
    
    // Iterate over the array
    for (let i = 0; i < array.length; i++) {
      if (i > 0) {
        // Add space between iterations
        context.expandedNodes.push({
          type: 'whitespace',
          value: ' '
        });
      }
      
      // Expand array element to string for substitution
      const elementValue = this.expandExpressionToString(array[i], context);
      
      // Create substitution context with the loop variable
      const loopSubstitutions = { [varName]: elementValue };
      
      // If we're already in a macro, merge substitutions
      const currentSubstitutions = context.macroCallStack.length > 0
        ? { ...context.macroCallStack[context.macroCallStack.length - 1].parameters, ...loopSubstitutions }
        : loopSubstitutions;
      
      // Expand the body with substitution
      // We need to push a temporary substitution context
      if (context.macroCallStack.length > 0) {
        // We're inside a macro, update the parameters
        const currentMacro = context.macroCallStack[context.macroCallStack.length - 1];
        const savedParams = currentMacro.parameters;
        const savedParamNodes = currentMacro.parameterNodes;
        currentMacro.parameters = { ...savedParams, [varName]: elementValue };
        currentMacro.parameterNodes = { ...savedParamNodes, [varName]: array[i] };
        
        this.expandExpression(body, context);
        
        // Restore original parameters
        currentMacro.parameters = savedParams;
        currentMacro.parameterNodes = savedParamNodes;
      } else {
        // Create a pseudo-macro context for the for loop
        const forContext: MacroInvocation = {
          name: '__for__',
          callSite: this.nodeToRange(node),
          parameters: { [varName]: elementValue },
          parameterNodes: { [varName]: array[i] }
        };
        
        context.macroCallStack.push(forContext);
        this.expandExpression(body, context);
        context.macroCallStack.pop();
      }
    }
  }
  
  private expandExpression(expr: ExpressionNode, context: ExpansionContext): void {
    switch (expr.type) {
      case 'Text':
        this.expandText(expr as TextNode, context);
        break;
        
      case 'Number':
        context.expandedNodes.push({
          type: 'text',
          value: (expr as NumberNode).value.toString()
        });
        break;
        
      case 'Identifier':
        // Check if this identifier should be substituted
        const idName = (expr as IdentifierNode).name;
        let substitutedValue = idName;
        
        // Check if we have substitutions in the current context
        if (context.macroCallStack.length > 0) {
          const currentContext = context.macroCallStack[context.macroCallStack.length - 1];
          if (currentContext.parameters && currentContext.parameters[idName]) {
            substitutedValue = currentContext.parameters[idName];
          }
        }
        
        context.expandedNodes.push({
          type: 'text',
          value: substitutedValue
        });
        break;
        
      case 'BrainfuckCommand':
        this.expandBrainfuckCommand(expr as BrainfuckCommandNode, context);
        break;
        
      case 'MacroInvocation':
        this.expandMacroInvocation(expr as MacroInvocationNode, context);
        break;
        
      case 'BuiltinFunction':
        this.expandBuiltinFunction(expr as BuiltinFunctionNode, context);
        break;
        
      case 'ExpressionList':
        const list = expr as ExpressionListNode;
        let lastLine: number | undefined;
        for (let i = 0; i < list.expressions.length; i++) {
          const item = list.expressions[i];
          
          // Add spaces or newlines between items
          if (i > 0) {
            if (lastLine !== undefined && item.position && item.position.line > lastLine) {
              // Add newlines between items on different lines
              const newlineCount = item.position.line - lastLine;
              for (let j = 0; j < newlineCount; j++) {
                context.expandedNodes.push({
                  type: 'newline',
                  value: '\n'
                });
              }
            } else {
              // Add space between items on the same line
              context.expandedNodes.push({
                type: 'whitespace',
                value: ' '
              });
            }
          }
          
          this.expandContent(item as ContentNode, context);
          if (item.position) {
            lastLine = item.position.line;
          }
        }
        break;
        
      case 'ArrayLiteral':
        const array = expr as ArrayLiteralNode;
        for (let i = 0; i < array.elements.length; i++) {
          if (i > 0) {
            // Add comma and space between elements
            context.expandedNodes.push({
              type: 'text',
              value: ','
            });
            context.expandedNodes.push({
              type: 'whitespace',
              value: ' '
            });
          }
          this.expandExpression(array.elements[i], context);
        }
        break;
    }
  }
  
  private expandBodyWithSubstitutions(
    body: BodyNode[],
    substitutions: Record<string, string>,
    context: ExpansionContext
  ): void {
    let lastNodeEndLine: number | undefined;
    
    for (const node of body) {
      // Add newlines between nodes that were on different lines
      if (lastNodeEndLine !== undefined && node.position && node.position.line > lastNodeEndLine) {
        const newlineCount = node.position.line - lastNodeEndLine;
        for (let i = 0; i < newlineCount; i++) {
          context.expandedNodes.push({
            type: 'newline',
            value: '\n'
          });
        }
      }
      
      if (node.position) {
        lastNodeEndLine = node.position.line;
      }
      if (node.type === 'Text') {
        // Substitute parameters in text
        let text = (node as TextNode).value;
        
        // First, substitute regular parameters
        for (const [param, value] of Object.entries(substitutions)) {
          // Simple substitution - could be improved with proper tokenization
          const regex = new RegExp(`\\b${param}\\b`, 'g');
          text = text.replace(regex, value);
        }
        
        // Then handle meta-variables
        text = this.expandMetaVariables(text, context);
        
        // Directly add the expanded text instead of recursive call
        // to avoid double meta-expansion
        let i = 0;
        while (i < text.length) {
          // Collect whitespace
          let whitespace = '';
          while (i < text.length && /\s/.test(text[i])) {
            whitespace += text[i];
            i++;
          }
          
          if (whitespace) {
            const lines = whitespace.split('\n');
            for (let j = 0; j < lines.length; j++) {
              if (j > 0) {
                context.expandedNodes.push({ type: 'newline', value: '\n' });
              }
              if (lines[j]) {
                context.expandedNodes.push({ type: 'whitespace', value: lines[j] });
              }
            }
          }
          
          // Collect non-whitespace
          let textSegment = '';
          while (i < text.length && !/\s/.test(text[i])) {
            textSegment += text[i];
            i++;
          }
          
          if (textSegment) {
            context.expandedNodes.push({
              type: 'text',
              value: textSegment,
              source: this.nodeToRange({ ...node, type: 'Text' } as any)
            });
          }
        }
      } else {
        // For other node types, we need to handle parameter substitution specially
        if (node.type === 'BuiltinFunction' && context.macroCallStack.length > 0) {
          // Create a substituted version of the builtin function node
          const builtinNode = node as BuiltinFunctionNode;
          const substituteInExpression = (expr: ExpressionNode): ExpressionNode => {
            if (expr.type === 'Identifier') {
              const idNode = expr as IdentifierNode;
              // First check if we have a node substitution
              const currentMacro = context.macroCallStack[context.macroCallStack.length - 1];
              if (currentMacro.parameterNodes && currentMacro.parameterNodes[idNode.name]) {
                return currentMacro.parameterNodes[idNode.name];
              }
              // Otherwise use string substitution
              if (substitutions[idNode.name]) {
                return {
                  type: 'Text',
                  value: substitutions[idNode.name],
                  position: expr.position
                } as TextNode;
              }
            } else if (expr.type === 'Text') {
              const textNode = expr as TextNode;
              if (substitutions[textNode.value]) {
                return {
                  type: 'Text',
                  value: substitutions[textNode.value],
                  position: expr.position
                } as TextNode;
              }
            } else if (expr.type === 'ArrayLiteral') {
              const arrayNode = expr as ArrayLiteralNode;
              return {
                ...arrayNode,
                elements: arrayNode.elements.map(substituteInExpression)
              };
            } else if (expr.type === 'ExpressionList') {
              const listNode = expr as ExpressionListNode;
              return {
                ...listNode,
                expressions: listNode.expressions.map(substituteInExpression)
              };
            }
            return expr;
          };
          
          const substitutedArgs = builtinNode.arguments.map(substituteInExpression);
          
          // Create a new builtin node with substituted arguments
          const substitutedBuiltin: BuiltinFunctionNode = {
            ...builtinNode,
            arguments: substitutedArgs
          };
          
          this.expandBuiltinFunction(substitutedBuiltin, context);
        } else if (node.type === 'MacroInvocation' && context.macroCallStack.length > 0) {
          // Handle macro invocations with parameter substitution
          const invNode = node as MacroInvocationNode;
          const substitutedArgs = invNode.arguments?.map(arg => {
            if (arg.type === 'Text') {
              const textNode = arg as TextNode;
              if (substitutions[textNode.value]) {
                return {
                  type: 'Text',
                  value: substitutions[textNode.value],
                  position: arg.position
                } as TextNode;
              }
            }
            return arg;
          });
          
          const substitutedInvocation: MacroInvocationNode = {
            ...invNode,
            arguments: substitutedArgs
          };
          
          this.expandMacroInvocation(substitutedInvocation, context);
        } else {
          // For other node types, expand normally
          this.expandContent(node, context);
        }
      }
    }
  }
  
  private expandMetaVariables(text: string, context: ExpansionContext): string {
    // Global invocation counter
    text = text.replace(/__INVOC_COUNT__/g, context.invocationCounter.toString());
    
    // Current macro name
    if (context.currentMacroName) {
      text = text.replace(/__MACRO_NAME__/g, context.currentMacroName);
    }
    
    // Current line number (in the macro definition)
    if (context.currentLineNumber !== undefined) {
      text = text.replace(/__LINE__/g, context.currentLineNumber.toString());
    }
    
    // Unique label generation: __LABEL__(prefix) or __LABEL__prefix
    text = text.replace(/__LABEL__(?:\(([^)]+)\)|(\w+))/g, (match, parenPrefix, simplePrefix) => {
      const prefix = parenPrefix || simplePrefix;
      if (!context.labelCounters.has(prefix)) {
        context.labelCounters.set(prefix, context.invocationCounter);
      }
      return `${prefix}_${context.labelCounters.get(prefix)}`;
    });
    
    // Unique ID: __UID__
    text = text.replace(/__UID__/g, () => {
      return `uid_${context.invocationCounter}_${Math.random().toString(36).substring(2, 9)}`;
    });
    
    // Macro depth
    text = text.replace(/__DEPTH__/g, context.expansionDepth.toString());
    
    // Parent macro name (if nested)
    if (context.macroCallStack.length > 1) {
      const parentMacro = context.macroCallStack[context.macroCallStack.length - 2];
      text = text.replace(/__PARENT_MACRO__/g, parentMacro.name);
    }
    
    // Counter that increments each time it's used: __COUNTER__
    text = text.replace(/__COUNTER__/g, () => {
      const count = context.labelCounters.get('__counter') || 0;
      context.labelCounters.set('__counter', count + 1);
      return count.toString();
    });
    
    return text;
  }
  
  private expandExpressionToString(expr: ExpressionNode, context: ExpansionContext): string {
    // Create a temporary context to capture the expansion
    const tempContext: ExpansionContext = {
      ...context,
      expandedNodes: []
    };
    
    this.expandExpression(expr, tempContext);
    
    // Convert nodes to string, preserving whitespace between tokens
    let result = '';
    let lastWasText = false;
    
    for (const node of tempContext.expandedNodes) {
      if (node.type === 'text' || node.type === 'command') {
        if (lastWasText) {
          result += ' ';
        }
        result += node.value;
        lastWasText = true;
      } else if (node.type === 'whitespace') {
        result += ' ';
        lastWasText = false;
      }
    }
    
    return result;
  }
  
  private nodeToRange(node: ASTNode): Range {
    return {
      start: {
        line: node.position.line,
        column: node.position.column,
        offset: node.position.start
      },
      end: {
        line: node.position.line,
        column: node.position.column + (node.position.end - node.position.start),
        offset: node.position.end
      }
    };
  }
  
  private nodeToString(nodes: ASTNode[]): string {
    // Simple conversion for macro definition bodies
    return nodes.map(node => {
      switch (node.type) {
        case 'Text':
          return (node as TextNode).value;
        case 'BrainfuckCommand':
          return (node as BrainfuckCommandNode).commands;
        default:
          return '';
      }
    }).join('');
  }
  
  private defaultGenerate(nodes: ExpandedNode[], options: BackendOptions): string {
    let result = '';
    
    for (const node of nodes) {
      switch (node.type) {
        case 'text':
        case 'command':
          result += node.value;
          break;
          
        case 'whitespace':
          if (options.preserveWhitespace) {
            result += node.value;
          } else {
            result += ' '; // Normalize to single space
          }
          break;
          
        case 'newline':
          if (options.preserveNewlines) {
            result += node.value;
          }
          break;
          
        case 'comment':
          if (options.preserveComments) {
            result += node.value;
          }
          break;
      }
    }
    
    // Post-process if needed
    if (options.collapseEmptyLines) {
      const lines = result.split('\n');
      const nonEmpty = lines.filter(line => line.trim().length > 0);
      result = nonEmpty.join('\n');
    }
    
    return result;
  }
}

// Export factory function
export function createMacroExpanderV4(backend?: MacroBackend): MacroExpander {
  return new MacroExpanderV4(backend);
}