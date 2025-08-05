import type { 
  MacroDefinition, 
  MacroExpansionError, 
  MacroToken, 
  MacroExpanderOptions, 
  MacroExpanderResult,
  MacroExpander
} from './macro-expander.ts';
import { parseMacro } from './macro-parser.ts';
import type { 
  ASTNode, ContentNode, MacroInvocationNode, BuiltinFunctionNode, 
  ExpressionNode, BodyNode, ProgramNode, MacroDefinitionNode, 
  CodeLineNode, BrainfuckCommandNode, TextNode, NumberNode, 
  IdentifierNode, ExpressionListNode, ArrayLiteralNode, TuplePatternNode
} from './macro-parser.ts';
import { SourceMapBuilder, type Range, type Position, type SourceMap, type SourceMapEntry } from './source-map.ts';

interface ExpansionContext {
  sourceMapBuilder: SourceMapBuilder;
  currentSourcePosition: Position;
  expansionDepth: number;
  macroCallStack: Array<{
    macroName: string;
    callSite: Range;
    parameters?: Record<string, string>;
  }>;
  expandedLines: string[];
  currentExpandedLine: number;
  currentExpandedColumn: number;
}

export class MacroExpanderV3 implements MacroExpander {
  private macros: Map<string, MacroDefinitionNode> = new Map();
  private errors: MacroExpansionError[] = [];
  private tokens: MacroToken[] = [];
  private expansionChain: Set<string> = new Set();
  private maxExpansionDepth = 100;
  private input: string = '';

  expand(input: string, options?: MacroExpanderOptions): MacroExpanderResult {
    const opts = {
      stripComments: true,
      collapseEmptyLines: false,
      generateSourceMap: false,
      ...options
    };

    // Reset state
    this.macros.clear();
    this.errors = [];
    this.tokens = [];
    this.input = input;
    this.expansionChain.clear();

    // Parse the input
    const parseResult = parseMacro(input, {
      preserveComments: !opts.stripComments
    });

    this.errors.push(...parseResult.errors);
    this.tokens.push(...parseResult.tokens);

    // Collect macro definitions
    this.collectMacroDefinitions(parseResult.ast);
    this.validateAllMacros();

    const definitionErrors = [...this.errors];
    this.errors = [];

    // Create expansion context
    const context: ExpansionContext = {
      sourceMapBuilder: new SourceMapBuilder(),
      currentSourcePosition: { line: 1, column: 1, offset: 0 },
      expansionDepth: 0,
      macroCallStack: [],
      expandedLines: [''],
      currentExpandedLine: 1,
      currentExpandedColumn: 1
    };

    // Expand the AST
    this.expandProgram(parseResult.ast, context, opts.generateSourceMap || false);

    this.errors = [...definitionErrors, ...this.errors];

    // Join expanded lines - the first line (index 0) may contain content  
    let expanded = context.expandedLines.join('\n');
    // Only trim trailing whitespace, preserve leading newlines from macro definitions
    expanded = expanded.replace(/\s+$/, '');

    // Build source map before post-processing
    let sourceMap: SourceMap | undefined;
    if (opts.generateSourceMap) {
      sourceMap = context.sourceMapBuilder.build();
    }

    // Post-process and update source map if needed
    if (opts.collapseEmptyLines && sourceMap) {
      try {
        const collapsedResult = this.collapseEmptyLinesWithSourceMap(expanded, sourceMap);
        expanded = collapsedResult.code;
        sourceMap = collapsedResult.sourceMap;
      } catch (error) {
        console.error('Error in collapseEmptyLinesWithSourceMap:', error);
        // Fall back to simple collapse without updating source map
        expanded = this.collapseEmptyLines(expanded);
        console.warn('Source map may be inaccurate due to line collapsing');
      }
    } else if (opts.collapseEmptyLines) {
      expanded = this.collapseEmptyLines(expanded);
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

    const result: MacroExpanderResult = {
      expanded,
      errors: this.errors,
      tokens: this.tokens,
      macros: macroDefinitions
    };

    if (sourceMap) {
      result.sourceMap = sourceMap;
    }

    return result;
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
    for (const macro of this.macros.values()) {
      this.validateMacroDefinition(macro);
    }
  }

  private validateMacroDefinition(macro: MacroDefinitionNode): void {
    const paramSet = new Set(macro.parameters || []);
    this.validateNodes(macro.body, paramSet, macro.position);
  }

  private validateNodes(nodes: BodyNode[], validParams: Set<string>, macroPosition: { line: number; column: number }): void {
    for (const node of nodes) {
      if (node.type === 'MacroInvocation') {
        const invocation = node as MacroInvocationNode;
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
        if (invocation.arguments) {
          for (const arg of invocation.arguments) {
            this.validateExpression(arg, validParams, macroPosition);
          }
        }
      } else if (node.type === 'BuiltinFunction') {
        const builtin = node as BuiltinFunctionNode;
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
        if (builtin.name === 'repeat' && builtin.arguments.length === 2) {
          const countArg = builtin.arguments[0];
          if (countArg.type === 'Text') {
            const text = (countArg as TextNode).value;
            const mightBeLoopVar = text.length === 1 && /^[a-zA-Z]$/.test(text);
            // Don't validate if it's a valid parameter - it will be substituted later
            if (!validParams.has(text) && !mightBeLoopVar && isNaN(parseInt(text, 10))) {
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
          } else if (countArg.type === 'Identifier') {
            // Identifiers should be valid parameters
            const identifierName = (countArg as IdentifierNode).name;
            if (!validParams.has(identifierName)) {
              this.errors.push({
                type: 'syntax_error', 
                message: `Undefined parameter: ${identifierName}`,
                location: {
                  line: builtin.position.line - 1,
                  column: builtin.position.column - 1,
                  length: builtin.position.end - builtin.position.start
                }
              });
            }
          }
        }
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

  private expandProgram(ast: ProgramNode, context: ExpansionContext, generateSourceMap: boolean): void {
    for (const statement of ast.statements) {
      context.currentSourcePosition = {
        line: statement.position.line,
        column: statement.position.column,
        offset: statement.position.start
      };

      if (statement.type === 'CodeLine') {
        this.expandCodeLine(statement, context, generateSourceMap);
        // Add newline after each code line
        this.appendToExpanded('\n', context, generateSourceMap, null);
      } else if (statement.type === 'MacroDefinition') {
        // Macro definitions are replaced with empty lines
        // But we need to create source map entries for each line of the definition
        const macroDefNode = statement as MacroDefinitionNode;
        const startLine = macroDefNode.position.line;
        
        // Calculate end line by counting newlines in the source text
        const sourceText = this.input.substring(macroDefNode.position.start, macroDefNode.position.end);
        const lineCount = (sourceText.match(/\n/g) || []).length + 1;
        const endLine = startLine + lineCount - 1;
        
        // Create a source map entry for each line in the macro definition
        for (let line = startLine; line <= endLine; line++) {
          if (generateSourceMap) {
            // Map each source line to the current expanded position (empty line)
            const sourceRange: Range = {
              start: { line: line, column: 1 },
              end: { line: line, column: 1000 } // Use a large column to cover the whole line
            };
            this.appendToExpanded('', context, generateSourceMap, sourceRange);
          }
        }
        this.appendToExpanded('\n', context, generateSourceMap, null);
      }
    }
  }

  private expandCodeLine(node: CodeLineNode, context: ExpansionContext, generateSourceMap: boolean): void {
    // Remember the start position of this line's expansion
    const lineExpandedStart = {
      line: context.currentExpandedLine,
      column: context.currentExpandedColumn
    };
    
    for (const content of node.content) {
      context.currentSourcePosition = {
        line: content.position.line,
        column: content.position.column,
        offset: content.position.start
      };
      this.expandContent(content, context, generateSourceMap);
    }
    
    // If this line produced no output but contains content, create a minimal source map entry
    // This ensures every source line with content can have breakpoints
    if (generateSourceMap && node.content.length > 0 &&
        context.currentExpandedLine === lineExpandedStart.line &&
        context.currentExpandedColumn === lineExpandedStart.column) {
      // Find the first non-whitespace content
      const firstContent = node.content.find(c => 
        c.type !== 'Text' || c.value.trim().length > 0
      );
      
      if (firstContent) {
        context.sourceMapBuilder.addMapping({
          expandedRange: {
            start: lineExpandedStart,
            end: lineExpandedStart // Zero-width mapping
          },
          sourceRange: {
            start: {
              line: firstContent.position.line,
              column: firstContent.position.column,
              offset: firstContent.position.start
            },
            end: {
              line: firstContent.position.line,
              column: firstContent.position.column + 1,
              offset: firstContent.position.start + 1
            }
          },
          expansionDepth: context.expansionDepth,
          macroName: context.macroCallStack[context.macroCallStack.length - 1]?.macroName,
          macroCallSite: context.macroCallStack[context.macroCallStack.length - 1]?.callSite,
          macroCallStack: [...context.macroCallStack]
        });
      }
    }
  }

  private expandContent(node: ContentNode, context: ExpansionContext, generateSourceMap: boolean): void {
    // If we're inside a macro expansion, use the macro invocation's source range
    // Otherwise, use the node's own position (for top-level content)
    const currentMacro = context.macroCallStack[context.macroCallStack.length - 1];
    const sourceRange: Range = currentMacro?.callSite || {
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
    
    this.expandContentWithSourceRange(node, context, generateSourceMap, sourceRange);
  }
  
  private expandContentWithSourceRange(node: ContentNode, context: ExpansionContext, generateSourceMap: boolean, sourceRange: Range): void {

    switch (node.type) {
      case 'BrainfuckCommand':
        this.appendToExpanded(node.commands, context, generateSourceMap, sourceRange);
        break;
      
      case 'Text':
        this.appendToExpanded(node.value, context, generateSourceMap, sourceRange);
        break;
      
      case 'MacroInvocation':
        this.expandMacroInvocation(node, context, generateSourceMap, sourceRange);
        break;
      
      case 'BuiltinFunction':
        this.expandBuiltinFunction(node, context, generateSourceMap, sourceRange);
        break;
    }
  }

  private appendToExpanded(
    text: string, 
    context: ExpansionContext, 
    generateSourceMap: boolean,
    sourceRange: Range | null
  ): void {
    if (!text) return;

    const startLine = context.currentExpandedLine;
    const startColumn = context.currentExpandedColumn;

    // Split text into lines for efficient processing
    const lines = text.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      if (i > 0) {
        // Not the first line, so we had a newline
        context.expandedLines.push('');
        context.currentExpandedLine++;
        context.currentExpandedColumn = 1;
      }
      
      if (line.length > 0) {
        const currentLineIndex = context.currentExpandedLine - 1;
        if (currentLineIndex >= context.expandedLines.length) {
          context.expandedLines.push('');
        }
        
        // Append the entire line at once (much faster than char by char)
        context.expandedLines[currentLineIndex] += line;
        context.currentExpandedColumn += line.length;
      }
    }

    // Create source map entry if we have a source range
    // Note: We create entries even for empty text to ensure all source lines are mapped
    if (generateSourceMap && sourceRange) {
      const expandedRange: Range = {
        start: {
          line: startLine,
          column: startColumn
        },
        end: {
          line: context.currentExpandedLine,
          column: context.currentExpandedColumn
        }
      };

      // Get current macro context
      const macroContext = context.macroCallStack[context.macroCallStack.length - 1];

      context.sourceMapBuilder.addMapping({
        expandedRange,
        sourceRange,
        expansionDepth: context.expansionDepth,
        macroName: macroContext?.macroName,
        macroCallSite: macroContext?.callSite,
        parameterValues: macroContext?.parameters,
        macroCallStack: [...context.macroCallStack]
      });
    }
  }

  private createInvocationSignature(node: MacroInvocationNode): string {
    let signature = node.name;
    
    if (node.arguments && node.arguments.length > 0) {
      const argSignatures: string[] = [];
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
          argSignatures.push(`@${(arg as any).name}`);
        } else {
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

  private expandMacroInvocation(
    node: MacroInvocationNode, 
    context: ExpansionContext, 
    generateSourceMap: boolean,
    sourceRange: Range
  ): void {
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
      this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
      return;
    }

    const invocationSignature = this.createInvocationSignature(node);

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
      this.appendToExpanded(`@${node.name}`, context, generateSourceMap, sourceRange);
      return;
    }

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
      this.appendToExpanded(`@${node.name}`, context, generateSourceMap, sourceRange);
      return;
    }

    this.expansionChain.add(invocationSignature);

    // Prepare parameter substitutions if needed
    let parameterValues: Record<string, string> | undefined;
    
    // Check parameter count mismatch
    const expectedParams = macro.parameters?.length || 0;
    const providedArgs = node.arguments?.length || 0;
    
    if (expectedParams !== providedArgs) {
      this.errors.push({
        type: 'parameter_mismatch',
        message: `Macro '${node.name}' expects ${expectedParams} parameter(s), got ${providedArgs}`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      // Return early, don't expand the macro
      this.expansionChain.delete(invocationSignature);
      context.expansionDepth--;
      return;
    }
    
    // If we have parameters and arguments, create substitutions
    if (macro.parameters && node.arguments) {
      parameterValues = {};
      for (let i = 0; i < macro.parameters.length; i++) {
        const param = macro.parameters[i];
        const arg = node.arguments[i];
        parameterValues[param] = this.expandExpressionToString(arg, context);
      }
    }

    // Push macro context
    context.macroCallStack.push({
      macroName: node.name,
      callSite: sourceRange,
      parameters: parameterValues
    });

    // Create a source map entry for the macro invocation itself
    // This ensures we can map from the macro call site to its expansion
    if (generateSourceMap && sourceRange) {
      const expandedStart = {
        line: context.currentExpandedLine,
        column: context.currentExpandedColumn
      };
      
      // Expand macro body
      if (parameterValues) {
        this.expandBodyNodesWithSubstitutions(macro.body, parameterValues, context, generateSourceMap);
      } else {
        this.expandBodyNodes(macro.body, context, generateSourceMap);
      }
      
      // Only create entry if expansion produced something
      if (context.currentExpandedLine !== expandedStart.line || 
          context.currentExpandedColumn !== expandedStart.column) {
        context.sourceMapBuilder.addMapping({
          expandedRange: {
            start: expandedStart,
            end: {
              line: context.currentExpandedLine,
              column: context.currentExpandedColumn
            }
          },
          sourceRange,
          expansionDepth: context.expansionDepth,
          macroName: node.name,
          macroCallSite: context.macroCallStack[context.macroCallStack.length - 2]?.callSite,
          parameterValues,
          macroCallStack: [...context.macroCallStack]
        });
      }
    } else {
      // Expand macro body
      if (parameterValues) {
        this.expandBodyNodesWithSubstitutions(macro.body, parameterValues, context, generateSourceMap);
      } else {
        this.expandBodyNodes(macro.body, context, generateSourceMap);
      }
    }

    // Pop macro context
    context.macroCallStack.pop();
    context.expansionDepth--;
    this.expansionChain.delete(invocationSignature);
  }

  private expandBodyNodes(nodes: BodyNode[], context: ExpansionContext, generateSourceMap: boolean): void {
    for (const node of nodes) {
      // Pass the node's actual position from the macro definition
      const nodeSourceRange: Range = {
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
      
      // Update current source position to the macro definition line
      context.currentSourcePosition = {
        line: node.position.line,
        column: node.position.column,
        offset: node.position.start
      };
      
      this.expandContentWithSourceRange(node, context, generateSourceMap, nodeSourceRange);
    }
  }

  private expandBodyNodesWithSubstitutions(
    nodes: BodyNode[], 
    substitutions: Record<string, string>,
    context: ExpansionContext, 
    generateSourceMap: boolean
  ): void {
    for (const node of nodes) {
      if (node.type === 'Text') {
        let text = node.value;
        const sortedSubstitutions = Object.entries(substitutions)
          .sort((a, b) => b[0].length - a[0].length);
        
        for (const [param, value] of sortedSubstitutions) {
          // Only replace the parameter if it appears as a standalone identifier
          // This prevents replacing 'a' inside 'val' but allows single letters
          // We look for the param surrounded by non-alphanumeric characters or at string boundaries
          const escapedParam = param.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
          const regex = new RegExp(`(^|[^a-zA-Z0-9_])${escapedParam}(?=$|[^a-zA-Z0-9_])`, 'g');
          text = text.replace(regex, (match, prefix) => prefix + value);
        }
        
        const sourceRange: Range = {
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
        
        this.appendToExpanded(text, context, generateSourceMap, sourceRange);
      } else if (node.type === 'BuiltinFunction') {
        const builtinNode = node as BuiltinFunctionNode;
        const modifiedNode = {
          ...builtinNode,
          arguments: builtinNode.arguments.map(arg => 
            this.substituteInExpression(arg, substitutions)
          )
        };
        this.expandBuiltinFunction(modifiedNode, context, generateSourceMap, {
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
        });
      } else if (node.type === 'MacroInvocation') {
        const invocationNode = node as MacroInvocationNode;
        let modifiedNode = invocationNode;
        
        if (invocationNode.arguments) {
          modifiedNode = {
            ...invocationNode,
            arguments: invocationNode.arguments.map(arg => 
              this.substituteInExpression(arg, substitutions)
            )
          };
        }
        
        this.expandMacroInvocation(modifiedNode, context, generateSourceMap, {
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
        });
      } else if (node.type === 'BrainfuckCommand') {
        // Pass through BF commands directly with proper source range
        const commandSourceRange: Range = {
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
        this.expandContentWithSourceRange(node, context, generateSourceMap, commandSourceRange);
      } else {
        // For other node types, expand normally with proper source range
        const otherSourceRange: Range = {
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
        this.expandContentWithSourceRange(node, context, generateSourceMap, otherSourceRange);
      }
    }
  }

  private substituteInExpression(expr: ExpressionNode, substitutions: Record<string, string>): ExpressionNode {
    switch (expr.type) {
      case 'Identifier':
        const name = (expr as IdentifierNode).name;
        if (substitutions[name]) {
          const value = substitutions[name];
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
        const textNode = expr as TextNode;
        let text = textNode.value;
        
        if (substitutions[text]) {
          const value = substitutions[text];
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
        
        const sortedSubstitutions = Object.entries(substitutions)
          .sort((a, b) => b[0].length - a[0].length);
        
        for (const [param, value] of sortedSubstitutions) {
          // Only replace the parameter if it appears as a standalone identifier
          // This prevents replacing 'a' inside 'val' but allows single letters
          // We look for the param surrounded by non-alphanumeric characters or at string boundaries
          const escapedParam = param.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
          const regex = new RegExp(`(^|[^a-zA-Z0-9_])${escapedParam}(?=$|[^a-zA-Z0-9_])`, 'g');
          text = text.replace(regex, (match, prefix) => prefix + value);
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

  private expandBuiltinFunction(
    node: BuiltinFunctionNode, 
    context: ExpansionContext, 
    generateSourceMap: boolean,
    sourceRange: Range
  ): void {
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
        this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        return;
      }
      
      const countExpr = this.expandExpressionToString(node.arguments[0], context);
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
        this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        return;
      }
      
      const contentArg = node.arguments[1];
      // When inside a macro expansion, use the macro invocation's source range
      const currentMacro = context.macroCallStack[context.macroCallStack.length - 1];
      const effectiveSourceRange = currentMacro?.callSite || sourceRange;
      
      for (let i = 0; i < count; i++) {
        if (contentArg.type === 'BrainfuckCommand') {
          this.appendToExpanded((contentArg as any).commands, context, generateSourceMap, effectiveSourceRange);
        } else {
          this.expandExpression(contentArg, context, generateSourceMap);
        }
      }
      
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
        this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        return;
      }
      
      const conditionExpr = this.expandExpressionToString(node.arguments[0], context);
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
        this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        return;
      }
      
      const selectedBranch = condition !== 0 ? node.arguments[1] : node.arguments[2];
      this.expandExpression(selectedBranch, context, generateSourceMap);
      
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
        this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        return;
      }
      
      const varNode = node.arguments[0];
      const arrayNode = node.arguments[1];
      const bodyNode = node.arguments[2];
      
      let varNames: string[] = [];
      let isTuplePattern = false;
      
      if (varNode.type === 'Identifier') {
        varNames = [(varNode as IdentifierNode).name];
      } else if (varNode.type === 'TuplePattern') {
        varNames = (varNode as TuplePatternNode).elements;
        isTuplePattern = true;
      } else {
        this.errors.push({
          type: 'syntax_error',
          message: `Expected variable name or tuple pattern in for loop, got ${varNode.type}`,
          location: {
            line: node.position.line - 1,
            column: node.position.column - 1,
            length: node.position.end - node.position.start
          }
        });
        this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        return;
      }
      
      let values: string[] = [];
      
      if (arrayNode.type === 'ArrayLiteral') {
        const arrayLiteral = arrayNode as any;
        values = arrayLiteral.elements.map((el: any) => this.expandExpressionToString(el, context).trim());
      } else if (arrayNode.type === 'Identifier' || arrayNode.type === 'Text') {
        // Handle identifiers and text nodes that might contain array values
        const expanded = this.expandExpressionToString(arrayNode, context).trim();
        
        // Check if it's an array-like structure
        // Special case: if we're doing tuple destructuring and the value looks like a tuple
        if (isTuplePattern && expanded.startsWith('{') && expanded.endsWith('}') && !this.looksLikeArrayOfArrays(expanded)) {
          // Treat the entire value as a single tuple to destructure
          // e.g., when a = "{1, 0, 1}" in {for((x,y,z) in a, ...)}
          values = [expanded];
        } else if (this.looksLikeArrayOfArrays(expanded)) {
          // Parse as array of arrays without outer braces
          values = this.parseArrayElements(expanded);
        } else if (expanded.startsWith('{') && expanded.endsWith('}')) {
          const inner = expanded.slice(1, -1);
          // Parse array elements considering nested braces
          values = this.parseArrayElements(inner);
          
          // Special case: if we have a single element that looks like a tuple
          // and we're doing tuple destructuring, treat it as a tuple directly
          if (isTuplePattern && values.length === 1 && values[0].startsWith('{') && values[0].endsWith('}')) {
            // This handles the case where {a} expands to {{1,0,1}}
            // We want to treat {1,0,1} as a tuple, not as a single element
            values = [values[0]]; // Keep it as is - the tuple parsing below will handle it
          }
        } else if (expanded.includes(',')) {
          values = expanded.split(',').map(v => v.trim());
        } else {
          // Single value case
          values = [expanded];
        }
      } else {
        const expanded = this.expandExpressionToString(arrayNode, context).trim();
        
        // Check if it's an array-like structure
        // First check if it looks like an array with nested arrays: {1},{2}
        if (this.looksLikeArrayOfArrays(expanded)) {
          // Parse as array of arrays without outer braces
          values = this.parseArrayElements(expanded);
        } else if (expanded.startsWith('{') && expanded.endsWith('}')) {
          const inner = expanded.slice(1, -1);
          // Parse array elements considering nested braces
          values = this.parseArrayElements(inner);
        } else if (expanded.includes(',')) {
          values = expanded.split(',').map(v => v.trim());
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
          this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
          return;
        }
      }
      
      for (const value of values) {
        const tempSubstitutions: Record<string, string> = {};
        
        if (isTuplePattern) {
          // Parse the value as a tuple if it's in the format {a, b, c}
          let tupleElements: string[] = [];
          
          if (value.startsWith('{') && value.endsWith('}')) {
            // Parse tuple elements
            tupleElements = this.parseArrayElements(value.slice(1, -1));
          } else {
            // Try to parse as comma-separated values
            tupleElements = value.split(',').map(v => v.trim());
          }
          
          // Map tuple elements to variable names
          for (let i = 0; i < varNames.length; i++) {
            if (i < tupleElements.length) {
              tempSubstitutions[varNames[i]] = tupleElements[i];
            } else {
              // If not enough elements, use empty string
              tempSubstitutions[varNames[i]] = '';
            }
          }
        } else {
          // Single variable case
          tempSubstitutions[varNames[0]] = value;
        }
        
        if (bodyNode.type === 'ExpressionList') {
          const list = bodyNode as ExpressionListNode;
          const substitutedNodes = list.expressions.map(expr => 
            this.substituteInExpression(expr as any, tempSubstitutions)
          );
          for (const node of substitutedNodes) {
            this.expandExpression(node, context, generateSourceMap);
          }
        } else {
          const substituted = this.substituteInExpression(bodyNode, tempSubstitutions);
          this.expandExpression(substituted, context, generateSourceMap);
        }
      }
      
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
        this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        return;
      }
      
      const arrayArg = node.arguments[0];
      
      if (arrayArg.type === 'ArrayLiteral') {
        const arrayLiteral = arrayArg as ArrayLiteralNode;
        const reversedElements = [...arrayLiteral.elements].reverse();
        const expandedElements = reversedElements.map(el => this.expandExpressionToString(el, context));
        this.appendToExpanded('{' + expandedElements.join(', ') + '}', context, generateSourceMap, sourceRange);
      } else {
        const expanded = this.expandExpressionToString(arrayArg, context).trim();
        
        if (expanded.startsWith('{') && expanded.endsWith('}')) {
          const inner = expanded.slice(1, -1);
          const values = inner.split(',').map(v => v.trim());
          this.appendToExpanded('{' + values.reverse().join(', ') + '}', context, generateSourceMap, sourceRange);
        } else if (expanded.includes(',')) {
          const values = expanded.split(',').map(v => v.trim());
          this.appendToExpanded('{' + values.reverse().join(', ') + '}', context, generateSourceMap, sourceRange);
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
          this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
        }
      }
    } else {
      // Unknown builtin function
      this.errors.push({
        type: 'syntax_error',
        message: `Unknown builtin function: ${node.name}`,
        location: {
          line: node.position.line - 1,
          column: node.position.column - 1,
          length: node.position.end - node.position.start
        }
      });
      this.appendToExpanded(this.nodeToString([node]), context, generateSourceMap, sourceRange);
    }
  }

  private expandExpression(expr: ExpressionNode, context: ExpansionContext, generateSourceMap: boolean): void {
    const sourceRange: Range = {
      start: {
        line: expr.position.line,
        column: expr.position.column,
        offset: expr.position.start
      },
      end: {
        line: expr.position.line,
        column: expr.position.column + (expr.position.end - expr.position.start),
        offset: expr.position.end
      }
    };

    switch (expr.type) {
      case 'Number':
        this.appendToExpanded(expr.value.toString(), context, generateSourceMap, sourceRange);
        break;
      
      case 'Identifier':
        this.appendToExpanded(expr.name, context, generateSourceMap, sourceRange);
        break;
      
      case 'MacroInvocation':
        this.expandMacroInvocation(expr, context, generateSourceMap, sourceRange);
        break;
      
      case 'BuiltinFunction':
        this.expandBuiltinFunction(expr, context, generateSourceMap, sourceRange);
        break;
      
      case 'ExpressionList':
        for (const item of expr.expressions) {
          this.expandContent(item, context, generateSourceMap);
        }
        break;
      
      case 'Text':
        this.appendToExpanded((expr as any).value, context, generateSourceMap, sourceRange);
        break;
      
      case 'BrainfuckCommand':
        this.appendToExpanded((expr as any).commands, context, generateSourceMap, sourceRange);
        break;
      
      case 'ArrayLiteral':
        const array = expr as ArrayLiteralNode;
        this.appendToExpanded('{', context, generateSourceMap, sourceRange);
        for (let i = 0; i < array.elements.length; i++) {
          if (i > 0) {
            this.appendToExpanded(', ', context, generateSourceMap, null);
          }
          this.expandExpression(array.elements[i], context, generateSourceMap);
        }
        this.appendToExpanded('}', context, generateSourceMap, sourceRange);
        break;
    }
  }

  private expandExpressionToString(expr: ExpressionNode, context: ExpansionContext): string {
    // Create a temporary context to capture the expanded string
    const tempContext: ExpansionContext = {
      ...context,
      expandedLines: [''],
      currentExpandedLine: 1,
      currentExpandedColumn: 1,
      sourceMapBuilder: new SourceMapBuilder() // Dummy builder
    };

    this.expandExpression(expr, tempContext, false);
    return tempContext.expandedLines.join('\n').trim();
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
      if (line.match(/[><+\-.\[\]$]/)) {
        nonEmptyLines.push(line);
      }
    }

    return nonEmptyLines.join('\n');
  }

  private looksLikeArrayOfArrays(str: string): boolean {
    // Check if string contains comma-separated items that start and end with braces
    // e.g., "{1},{2}" or "{a,b},{c,d}"
    const trimmed = str.trim();
    if (!trimmed.includes(',')) return false;
    
    // Simple check - if it has the pattern },{
    return trimmed.includes('},{');
  }

  private parseArrayElements(inner: string): string[] {
    const elements: string[] = [];
    let current = '';
    let braceDepth = 0;
    
    for (let i = 0; i < inner.length; i++) {
      const char = inner[i];
      
      if (char === '{') {
        braceDepth++;
        current += char;
      } else if (char === '}') {
        braceDepth--;
        current += char;
      } else if (char === ',' && braceDepth === 0) {
        // Only split at commas that are not inside braces
        if (current.trim()) {
          elements.push(current.trim());
        }
        current = '';
      } else {
        current += char;
      }
    }
    
    // Don't forget the last element
    if (current.trim()) {
      elements.push(current.trim());
    }
    
    return elements;
  }

  private collapseEmptyLinesWithSourceMap(code: string, sourceMap: SourceMap): { code: string; sourceMap: SourceMap } {
    const lines = code.split('\n');
    const nonEmptyLines: string[] = [];
    const lineMapping: Map<number, number> = new Map(); // old line -> new line
    
    let newLineIndex = 0;
    for (let oldLineIndex = 0; oldLineIndex < lines.length; oldLineIndex++) {
      const line = lines[oldLineIndex];
      if (line.match(/[><+\-.\[\]$]/)) {
        nonEmptyLines.push(line);
        lineMapping.set(oldLineIndex + 1, newLineIndex + 1); // Convert to 1-based
        newLineIndex++;
      }
    }

    // Create a new source map with updated line numbers
    const newSourceMapBuilder = new SourceMapBuilder();
    
    // Check if source map has entries
    if (!sourceMap.entries || !Array.isArray(sourceMap.entries)) {
      console.error('Source map has no entries array');
      return {
        code: nonEmptyLines.join('\n'),
        sourceMap: newSourceMapBuilder.build()
      };
    }
    
    for (const entry of sourceMap.entries) {
      // Add safety checks
      if (!entry) {
        console.error('Null/undefined source map entry');
        continue;
      }
      if (!entry.expandedRange) {
        console.error('Source map entry missing expandedRange:', JSON.stringify(entry));
        continue;
      }
      if (!entry.expandedRange.start || !entry.expandedRange.end) {
        console.error('Source map entry expandedRange missing start/end:', JSON.stringify(entry));
        continue;
      }
      
      const newExpandedLine = lineMapping.get(entry.expandedRange.start.line);
      if (newExpandedLine !== undefined) {
        // Update the expanded range to use the new line number
        const newEntry: SourceMapEntry = {
          sourceRange: entry.sourceRange,
          expandedRange: {
            start: {
              line: newExpandedLine,
              column: entry.expandedRange.start.column
            },
            end: {
              line: newExpandedLine,
              column: entry.expandedRange.end.column
            }
          },
          macroName: entry.macroName,
          parameterValues: entry.parameterValues,
          expansionDepth: entry.expansionDepth,
          macroCallSite: entry.macroCallSite,
          macroCallStack: entry.macroCallStack
        };
        
        newSourceMapBuilder.addMapping(newEntry);
      }
    }

    return {
      code: nonEmptyLines.join('\n'),
      sourceMap: newSourceMapBuilder.build()
    };
  }
}

export function createMacroExpanderV3(): MacroExpander {
  return new MacroExpanderV3();
}