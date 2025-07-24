# Technical Task: Brainfuck Macro Expansion System (@-style)

## Overview
Implement a macro preprocessor for Brainfuck that supports function-like macros with @-style invocation syntax (`@macro` and `@macro(args)`). The system should be written in TypeScript and preserve formatting/indentation of non-macro content.

## Syntax Specification

### Macro Definition
```
#define macroName body
#define macroName(param1, param2, ...) body
```

### Macro Invocation (@-style)
```
@macroName
@macroName(arg1, arg2, ...)
```

### Special Built-ins
- `{repeat(n, content)}` - Repeats content n times

## Requirements

### Core Features

1. **Simple Macro Definition**
    - Support parameter-less macros: `#define clear [-]`
    - Invoked with @ prefix: `@clear`
    - Macros can contain any valid Brainfuck code

2. **Parameterized Macros**
    - Support function-like macros: `#define inc(n) {repeat(n, +)}`
    - Invoked with @ prefix: `@inc(5)`
    - Parameters are substituted in the macro body
    - Handle multiple parameters with comma separation

3. **Nested Macro Expansion**
    - Macros can reference other macros: `#define clear2 @clear > @clear`
    - Recursive expansion until no more macros remain
    - Detect and prevent infinite recursion

4. **Built-in Functions**
    - Implement `{repeat(n, content)}` function
    - Additional built-in functions (if/else conditionals)
    - Extensible architecture for future built-ins

5. **Mixed Content**
    - Process files containing both macro definitions and Brainfuck code
    - Preserve plain Brainfuck sections unchanged
    - Maintain original whitespace and indentation

6. **Syntax Highlighting Reporting**
    - Clear error messages for undefined macros, parameter mismatches, circular dependencies
    - Highlight errors in the source code for better developer experience
    - Reporting Syntax highlighting token ranges for macro definitions, compatible with EditorStore tokenizers format

### Technical Requirements

1. **Input/Output**
    - Input: String containing macro definitions and Brainfuck code
    - Output: String with all macros expanded
    - Preserve line structure and indentation where possible

2. **Error Handling**
    - Undefined macro usage
    - Circular macro dependencies
    - Invalid syntax in macro definitions
    - Parameter count mismatches

3. **Performance**
    - Efficient handling of deeply nested macros
    - Reasonable performance for large files

## Implementation Design

### Module Structure
```typescript
interface MacroExpander {
  expand(input: string): string;
}

interface MacroDefinition {
  name: string;
  parameters?: string[];
  body: string;
}

interface ParsedLine {
  type: 'definition' | 'code';
  content: string;
  indentation: string;
}
```

### Processing Pipeline

1. **Parse Phase**
    - Split input into lines
    - Identify macro definitions vs code lines
    - Extract macro names, parameters, and bodies
    - Preserve indentation information

2. **Collection Phase**
    - Build macro definition map
    - Validate macro syntax
    - Check for duplicate definitions

3. **Expansion Phase**
    - Process code lines
    - Find `@identifier` and `@identifier(...)` patterns
    - Recursively expand macro invocations
    - Handle parameterized substitutions
    - Process built-in functions

4. **Output Phase**
    - Reconstruct output with original formatting
    - Ensure whitespace preservation

### Algorithm Considerations

1. **Macro Invocation Detection**
    - Regex pattern: `/@(\w+)(?:\((.*?)\))?/g`
    - Captures: macro name and optional arguments
    - Handle nested parentheses in arguments
    - @ symbol clearly distinguishes macros from BF code

2. **Parameter Substitution**
    - Parse comma-separated arguments
    - Handle whitespace in arguments
    - Support nested macro calls in arguments: `@add(@inc(2))`

3. **Recursion Protection**
    - Track expansion depth
    - Maintain set of macros in current expansion chain
    - Throw error on circular dependencies

## Example Implementation Flow

Input:
```
#define clear [-]
#define inc(n) {repeat(n, +)}
#define move(n) {repeat(n, >)}
#define clear2 @clear > @clear

// Using macros
@move(5) @inc(10) @clear

// Nested macro
@clear2

// Mixed with plain BF
>> @dec(2) <<< @inc(1)
```

Processing:
1. Parse definitions:
    - `clear` → `[-]`
    - `inc(n)` → `{repeat(n, +)}`
    - `move(n)` → `{repeat(n, >)}`
    - `clear2` → `@clear > @clear`

2. Expand code lines:
    - `@move(5)` → `{repeat(5, >)}` → `>>>>>`
    - `@inc(10)` → `{repeat(10, +)}` → `++++++++++`
    - `@clear` → `[-]`
    - `@clear2` → `@clear > @clear` → `[-] > [-]`

3. Output:
```
>>>>> ++++++++++ [-]
[-] > [-]
>> -- <<< +
```

## Error Handling Examples

```typescript
// Undefined macro
"@undefined_macro(5)" → Error: Macro 'undefined_macro' is not defined

// Parameter mismatch  
"#define inc(n) +"
"@inc()" → Error: Macro 'inc' expects 1 parameter, got 0

// Circular reference
"#define a @b"
"#define b @a"
"@a" → Error: Circular macro dependency detected: a → b → a

// Invalid syntax
"@ macro" → No expansion (space after @)
"macro@" → No expansion (@ not at start)
```

## Edge Cases

1. **@ symbol in plain BF**
    - Standalone @ should be preserved
    - `@` without following identifier is not a macro

2. **Email-like patterns**
    - `user@domain` should not be treated as macro invocation
    - Only `@` at word boundaries followed by valid identifiers

3. **Whitespace handling**
    - `@ clear` (space after @) → no expansion
    - `@clear ` (space after identifier) → expands normally

## Testing Requirements

1. **Unit Tests**
    - Simple macro expansion
    - Parameterized macros
    - Nested macros
    - Built-in functions
    - @-syntax edge cases
    - Error cases

2. **Integration Tests**
    - Complex real-world macro files
    - Performance benchmarks
    - Whitespace preservation
    - Mixed macro/plain BF content

## Deliverables

1. **Core Module**: `macro-expander.ts`
2. **Type Definitions**: Full TypeScript types
3. **Unit Tests**: Comprehensive test suite
4. **Documentation**: Usage examples and API docs
5. **Error Messages**: Clear, actionable error reporting that could be used in EditorStore to highlight issues in the source code

## Implementation Notes

The @-style syntax provides:
- Clear visual distinction from BF operators
- Familiar to developers (decorators, annotations)
- No conflict with existing BF syntax
- Easy to parse and highlight