// Design document for a generalized macro expander that can target multiple backends

// Core interfaces that define the generalized system

export interface MacroBackend {
  // Backend identification
  name: string;
  description: string;
  fileExtensions: string[];

  // Convert expanded AST nodes to target language
  generate(nodes: ExpandedNode[]): string;

  // Backend-specific builtin functions
  builtins?: Map<string, BuiltinFunction>;

  // Validation for backend-specific constraints
  validate?(nodes: ExpandedNode[]): ValidationError[];

  // Optional source map support
  generateWithSourceMap?(nodes: ExpandedNode[]): {
    code: string;
    sourceMap?: any;
  };
}

export interface ExpandedNode {
  type: 'Literal' | 'Instruction' | 'Label' | 'Directive' | 'Comment';
  value: string;
  metadata?: {
    sourceLocation?: SourceLocation;
    macroContext?: MacroContext[];
  };
}

export interface BuiltinFunction {
  name: string;
  minArgs: number;
  maxArgs?: number;
  expand(args: any[], context: ExpansionContext): ExpandedNode[];
}

export interface ValidationError {
  type: 'error' | 'warning';
  message: string;
  location?: SourceLocation;
}

// Example backend implementations

export class BrainfuckBackend implements MacroBackend {
  name = 'brainfuck';
  description = 'Classic Brainfuck language';
  fileExtensions = ['.bf', '.b'];

  generate(nodes: ExpandedNode[]): string {
    return nodes
      .filter((n) => n.type === 'Instruction')
      .map((n) => n.value)
      .join('');
  }

  validate(nodes: ExpandedNode[]): ValidationError[] {
    const errors: ValidationError[] = [];
    for (const node of nodes) {
      if (node.type === 'Instruction') {
        const invalidChars = node.value.replace(/[><+\-.,\[\]]/g, '');
        if (invalidChars) {
          errors.push({
            type: 'error',
            message: `Invalid Brainfuck characters: ${invalidChars}`,
            location: node.metadata?.sourceLocation,
          });
        }
      }
    }
    return errors;
  }
}

export class RippleAssemblyBackend implements MacroBackend {
  name = 'ripple-asm';
  description = 'Ripple VM Assembly Language';
  fileExtensions = ['.asm', '.s'];

  private labelCounter = 0;

  builtins = new Map<string, BuiltinFunction>([
    [
      'align',
      {
        name: 'align',
        minArgs: 1,
        expand: (args) => {
          const alignment = args[0];
          return [{ type: 'Directive', value: `.align ${alignment}` }];
        },
      },
    ],
    [
      'local_label',
      {
        name: 'local_label',
        minArgs: 0,
        maxArgs: 1,
        expand: (args) => {
          const prefix = args[0] || 'L';
          const label = `${prefix}_${this.labelCounter++}`;
          return [{ type: 'Label', value: `${label}:` }];
        },
      },
    ],
  ]);

  generate(nodes: ExpandedNode[]): string {
    const lines: string[] = [];

    for (const node of nodes) {
      switch (node.type) {
        case 'Instruction':
        case 'Label':
        case 'Directive':
          lines.push(node.value);
          break;
        case 'Comment':
          lines.push(`; ${node.value}`);
          break;
        case 'Literal':
          // Literals might need context-aware handling
          if (this.looksLikeInstruction(node.value)) {
            lines.push(node.value);
          } else {
            lines.push(`; ${node.value}`);
          }
          break;
      }
    }

    return lines.join('\n');
  }

  private looksLikeInstruction(value: string): boolean {
    // Simple heuristic - starts with uppercase letter or dot
    return /^[A-Z.]/.test(value.trim());
  }

  validate(nodes: ExpandedNode[]): ValidationError[] {
    const errors: ValidationError[] = [];
    const definedLabels = new Set<string>();
    const usedLabels = new Set<string>();

    // First pass: collect labels
    for (const node of nodes) {
      if (node.type === 'Label') {
        const label = node.value.replace(':', '').trim();
        if (definedLabels.has(label)) {
          errors.push({
            type: 'error',
            message: `Duplicate label: ${label}`,
            location: node.metadata?.sourceLocation,
          });
        }
        definedLabels.add(label);
      }
    }

    // Second pass: check label references
    // (simplified - real implementation would parse instruction operands)

    return errors;
  }
}

// The generalized expander that uses backends

export class GeneralizedMacroExpander {
  private backends = new Map<string, MacroBackend>();

  registerBackend(backend: MacroBackend): void {
    this.backends.set(backend.name, backend);
  }

  expand(
    input: string,
    backendName: string,
    options?: any,
  ): {
    output: string;
    errors: any[];
    sourceMap?: any;
  } {
    const backend = this.backends.get(backendName);
    if (!backend) {
      throw new Error(`Unknown backend: ${backendName}`);
    }

    // Use the existing macro parser and expander
    // but inject backend-specific builtins
    const expandedNodes = this.expandToNodes(input, backend);

    // Validate if backend supports it
    const validationErrors = backend.validate?.(expandedNodes) || [];

    // Generate output
    const output = backend.generate(expandedNodes);

    return {
      output,
      errors: validationErrors,
    };
  }

  private expandToNodes(input: string, backend: MacroBackend): ExpandedNode[] {
    // This would integrate with the existing macro expander
    // but return ExpandedNode[] instead of string

    // Simplified for demonstration:
    const lines = input.split('\n');
    const nodes: ExpandedNode[] = [];

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;

      if (trimmed.endsWith(':')) {
        nodes.push({ type: 'Label', value: trimmed });
      } else if (trimmed.startsWith('.')) {
        nodes.push({ type: 'Directive', value: trimmed });
      } else if (trimmed.startsWith(';') || trimmed.startsWith('//')) {
        nodes.push({ type: 'Comment', value: trimmed.substring(1).trim() });
      } else {
        nodes.push({ type: 'Instruction', value: trimmed });
      }
    }

    return nodes;
  }
}

// Usage example

export function demonstrateUsage() {
  const expander = new GeneralizedMacroExpander();

  // Register backends
  expander.registerBackend(new BrainfuckBackend());
  expander.registerBackend(new RippleAssemblyBackend());

  // Same macro source
  const macroSource = `
    #define TIMES 5
    #define inc(n) {repeat(n, +)}
    
    ; Increment by TIMES
    @inc(@TIMES)
  `;

  // Generate for different backends
  const bfResult = expander.expand(macroSource, 'brainfuck');
  console.log('Brainfuck output:', bfResult.output); // "++++"

  const asmResult = expander.expand(macroSource, 'ripple-asm');
  console.log('Assembly output:', asmResult.output); // "ADDI R3, R3, 5"
}

// Advanced features that could be added:

interface AdvancedFeatures {
  // Multi-pass expansion for complex transformations
  multiPass?: boolean;

  // Plugin system for extending backends
  plugins?: MacroPlugin[];

  // Cross-backend optimization
  optimize?: boolean;

  // Debugging support
  debug?: {
    traceExpansion?: boolean;
    breakpoints?: string[];
  };
}

interface MacroPlugin {
  name: string;

  // Hook into expansion process
  beforeExpand?(ast: any): any;
  afterExpand?(nodes: ExpandedNode[]): ExpandedNode[];

  // Add custom builtins
  builtins?: Map<string, BuiltinFunction>;

  // Transform output
  transformOutput?(output: string, backend: MacroBackend): string;
}

// Example of a unified macro that works across backends
const unifiedMacroExample = `
  #define ZERO(location) {
    #if BACKEND == "brainfuck"
      [-]
    #elif BACKEND == "ripple-asm"
      LI location, 0
    #elif BACKEND == "x86"
      XOR location, location
    #endif
  }
  
  #define LOOP(counter, max, body) {
    #if BACKEND == "brainfuck"
      {repeat(max, body)}
    #elif BACKEND == "ripple-asm"
      @local_label("loop")
      body
      ADDI counter, counter, 1
      BNE counter, max, @current_label(-1)
    #endif
  }
`;
