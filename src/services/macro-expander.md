# Brainfuck Macro Expander Documentation

## Overview

The Brainfuck Macro Expander is a preprocessing system that adds macro capabilities to Brainfuck code. It supports function-like macros with @-style invocation syntax, making Brainfuck code more readable and maintainable.

## API Reference

### Main Interface

```typescript
import { createMacroExpander } from './macro-expander';

const expander = createMacroExpander();
const result = expander.expand(sourceCode);
```

### Types

```typescript
interface MacroExpanderResult {
  expanded: string;        // The expanded Brainfuck code
  errors: MacroExpansionError[];  // Array of errors encountered
  tokens: MacroToken[];    // Tokens for syntax highlighting
}

interface MacroExpansionError {
  type: 'undefined' | 'parameter_mismatch' | 'circular_dependency' | 'syntax_error';
  message: string;
  location?: {
    line: number;      // 0-based line number
    column: number;    // 0-based column number
    length: number;    // Length of the error span
  };
}

interface MacroToken {
  type: 'macro_definition' | 'macro_invocation' | 'builtin_function';
  range: {
    start: number;     // Start position in the source
    end: number;       // End position in the source
  };
  name: string;        // Name of the macro or function
}
```

## Syntax Guide

### Macro Definition

```brainfuck
#define macroName body
#define macroName(param1, param2, ...) body
```

**Multiline Macros**: Use backslash (`\`) for line continuation:
```brainfuck
#define longMacro first_part \
  second_part \
  third_part

#define complex(x, y) {repeat(x, +)} \
  > \
  {repeat(y, -)} \
  < \
  [-]
```

### Macro Invocation

```brainfuck
@macroName
@macroName(arg1, arg2, ...)
```

### Built-in Functions

```brainfuck
{repeat(n, content)}  // Repeats content n times
{if(condition, true_branch, false_branch)}  // Conditional expansion (non-zero = true, zero = false)
```

## Usage Examples

### Basic Macros

```brainfuck
#define clear [-]
#define inc +
#define dec -
#define right >
#define left <
#define print .
#define read ,

// Clear current cell
@clear

// Move right and increment
@right @inc @inc @inc
```

### Parameterized Macros

```brainfuck
#define inc(n) {repeat(n, +)}
#define dec(n) {repeat(n, -)}
#define move(n) {repeat(n, >)}
#define back(n) {repeat(n, <)}

// Increment by 10
@inc(10)

// Move 5 cells right
@move(5)

// Complex movement pattern
@move(3) @inc(5) @back(3)
```

### Conditional Macros with if

```brainfuck
// Define scratch lanes
#define sA 1
#define sB 0

// Select lane based on parameter
#define scratch_lane(n) {if(n, @lane_sA, @lane_sB)}

// Use different operations based on condition
#define safe_dec(n) {if(n, {repeat(n, -)}, )}

// Complex conditional macro
#define move_or_stay(dir, dist) {if(dir, {repeat(dist, >)}, {repeat(dist, <)})}

// Usage examples
@scratch_lane(@sA)  // Expands to @lane_sA
@scratch_lane(@sB)  // Expands to @lane_sB
@safe_dec(5)        // Expands to -----
@safe_dec(0)        // Expands to nothing
@move_or_stay(1, 3) // Expands to >>>
@move_or_stay(0, 3) // Expands to <<<
```

### Nested Macros

```brainfuck
#define clear [-]
#define zero @clear
#define clear_next > @clear <
#define clear_two @clear > @clear

// Using nested macros
@clear_two
@clear_next
```

### Multiline Macros

```brainfuck
// Define a complex Hello World macro using line continuation
#define hello_world ++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++. \
>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>

// Use the multiline macro
@hello_world

// Parameterized multiline macro for copying values
#define copy(n) [-@next(n)+>+ \
  @prev(n)<]@next(n)[- \
  @prev(n)+@next(n)]@prev(n)

// Define next and prev macros
#define next(n) {repeat(n, >)}
#define prev(n) {repeat(n, <)}

// Use the copy macro
@copy(1)  // Copy value from current cell to next cell
```

### Real-World Example: Hello World

```brainfuck
#define inc(n) {repeat(n, +)}
#define dec(n) {repeat(n, -)}
#define right(n) {repeat(n, >)}
#define left(n) {repeat(n, <)}
#define print .
#define newline @inc(10) @print @dec(10)

// Print "Hello World!"
// H (72)
@inc(72) @print
@right(1)

// e (101)
@inc(101) @print

// l (108)
@inc(7) @print

// l (108)
@print

// o (111)
@inc(3) @print

// space (32)
@right(1) @inc(32) @print @left(2)

// Reset for next part
@clear @right(1) @clear @right(1)

// W (87)
@inc(87) @print

// o (111)
@inc(24) @print

// r (114)
@inc(3) @print

// l (108)
@dec(6) @print

// d (100)
@dec(8) @print

// ! (33)
@right(1) @inc(33) @print

@newline
```

## Error Handling

The expander provides detailed error messages with location information:

### Undefined Macro
```brainfuck
@undefined_macro(5)
// Error: Macro 'undefined_macro' is not defined at line 1, column 0
```

### Parameter Mismatch
```brainfuck
#define inc(n) {repeat(n, +)}
@inc()      // Error: Macro 'inc' expects 1 parameter(s), got 0
@inc(5, 10) // Error: Macro 'inc' expects 1 parameter(s), got 2
```

### Circular Dependencies
```brainfuck
#define a @b
#define b @a
@a  // Error: Circular macro dependency detected: a → b → a
```

### Duplicate Definitions
```brainfuck
#define test +
#define test -  // Error: Duplicate macro definition: 'test'
```

## Edge Cases

### Email Patterns
```brainfuck
// Email addresses are NOT treated as macros
user@domain.com  // Remains unchanged
```

### Standalone @ Symbol
```brainfuck
@ alone    // @ symbol preserved
@@@@       // All @ symbols preserved
@ macro    // Space after @ prevents expansion
```

### Mixed Content
```brainfuck
#define inc(n) {repeat(n, +)}

// Mix of macros and regular BF code
+++@inc(5)---[>@inc(10)<-]
```

## Integration with IDE

The macro expander provides token information that can be used for syntax highlighting:

```typescript
const result = expander.expand(sourceCode);

// Use tokens for syntax highlighting
result.tokens.forEach(token => {
  if (token.type === 'macro_definition') {
    // Highlight macro definitions
  } else if (token.type === 'macro_invocation') {
    // Highlight macro invocations
  } else if (token.type === 'builtin_function') {
    // Highlight built-in functions
  }
});

// Display errors in the editor
result.errors.forEach(error => {
  if (error.location) {
    // Highlight error at specific location
    highlightError(error.location.line, error.location.column, error.location.length);
  }
});
```

## Performance Considerations

1. **Recursion Depth**: Maximum expansion depth is limited to 100 to prevent infinite loops
2. **Circular Dependencies**: Detected and reported as errors
3. **Large Files**: The expander processes line by line for efficient memory usage

## Best Practices

1. **Naming Conventions**: Use descriptive macro names
   ```brainfuck
   #define clear_cell [-]     // Good
   #define c [-]              // Less clear
   ```

2. **Parameter Names**: Use meaningful parameter names
   ```brainfuck
   #define move(direction, count) {repeat(count, direction)}
   ```

3. **Organization**: Group related macros together
   ```brainfuck
   // Movement macros
   #define right(n) {repeat(n, >)}
   #define left(n) {repeat(n, <)}
   
   // Arithmetic macros
   #define inc(n) {repeat(n, +)}
   #define dec(n) {repeat(n, -)}
   ```

4. **Documentation**: Comment your macros
   ```brainfuck
   // Clear current cell and the next one
   #define clear_two [-] > [-] <
   ```

## Limitations

1. **Limited Conditional Logic**: Only numeric conditions supported via {if()} builtin
2. **No Variadic Macros**: Fixed number of parameters only
3. **No String Manipulation**: Parameters are treated as literal text
4. **No Macro Concatenation**: Cannot build macro names dynamically