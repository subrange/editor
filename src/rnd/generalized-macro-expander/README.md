# Generalized Macro Expander R&D

This folder contains research and development work on generalizing the Brainfuck macro expander to support multiple backend languages, starting with Ripple Assembly.

## Current Status

### Working Prototype
- ✅ Basic macro expansion to assembly
- ✅ Preserves spaces in assembly instructions  
- ✅ Handles `repeat`, `if` builtins
- ✅ Backend interface design
- ✅ Assembly backend implementation

### V4 Development
- ✅ Created MacroExpanderV4 with improved architecture
- ✅ AST-preserving expansion pipeline
- ✅ Meta-programming support (design complete)
- ⚠️ Meta-variables (`$INVOC_COUNT`, `$LABEL()`) need parser updates

### Known Issues
1. The current parser treats `$` as a Brainfuck command, making meta-variables difficult
2. Spaces are stripped in curly brace contexts without quotes
3. Full `for` loop implementation pending

## Concept

The core idea is that the macro language itself (with `#define`, `@invocation`, `{repeat}`, `{if}`, `{for}`, etc.) is already backend-agnostic. We can reuse the same parsing and expansion logic but generate different output languages.

```
Input (Macro Language) → Parse → Expand → Backend Generator → Output (Target Language)
```

## Current Architecture

The existing macro expander:
1. Parses macro definitions and invocations
2. Expands them recursively
3. Outputs Brainfuck commands

## Proposed Architecture

```typescript
interface MacroBackend {
  name: string;
  generate(expandedAST: ASTNode[]): string;
  builtins?: Map<string, BuiltinFunction>;
  validate?(nodes: ASTNode[]): ValidationError[];
}
```

## Files in this Directory

- `macro-to-assembly.test.ts` - Test cases exploring macro expansion to assembly
- `console-example.ts` - Runnable examples demonstrating the concept
- `generalized-expander-design.ts` - Design document with interfaces and implementation ideas

## Key Insights

1. **The macro language is already generic** - It doesn't assume Brainfuck semantics
2. **Builtins are portable** - `repeat`, `if`, `for` work for any imperative language
3. **Backend-specific features** - Can be added through custom builtins
4. **Source maps work** - The expansion tracking remains the same

## Example: Same Macro, Different Backends

```macro
#define clear(n) {repeat(n, [-]>)}
@clear(5)
```

**Brainfuck output:**
```bf
[-]>[-]>[-]>[-]>[-]>
```

**Assembly output:**
```asm
LI R3, 0
ADDI R4, R4, 1
LI R3, 0
ADDI R4, R4, 1
; ... repeated 5 times
```

## Next Steps

1. Extract the AST types and parser into a shared module
2. Create a backend interface
3. Implement BrainfuckBackend (refactor existing code)
4. Implement RippleAssemblyBackend
5. Add backend-specific builtins and optimizations

## Benefits

- Write macros once, target multiple platforms
- Reuse complex macro logic across languages
- Enable cross-compilation scenarios
- Support mixed-language projects