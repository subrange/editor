# Brainfuck Macro Expander - Rust Implementation

A Rust implementation of the Brainfuck macro expansion system, compatible with the TypeScript version used in the Brainfuck IDE.

## Features

✅ **Core Functionality**
- Macro definitions and invocations with `#define` and `@`/`#` prefixes
- Parameter substitution in macros
- Single-line and multiline macro definitions
- Backslash line continuation for macros

✅ **Builtin Functions**
- `{repeat(n, content)}` - Repeats content n times
- `{if(condition, true_branch, false_branch)}` - Conditional expansion
- `{for(var in array, body)}` - Iteration over arrays
- `{reverse(array)}` - Reverses array elements

✅ **Advanced Features**
- Nested macro invocations
- Array literals `{1, 2, 3}`
- Character literals `'A'`
- Hexadecimal numbers `0xFF`
- Source map generation
- Circular dependency detection
- Comment handling

✅ **CLI Tool**
```bash
# Expand a file
bfm expand input.bfm

# List all macros in a file
bfm list input.bfm

# Validate macro definitions
bfm validate input.bfm
```

## Key Implementation Details

### Token Recognition
The lexer recognizes builtin functions as single tokens (e.g., `{repeat` instead of `{` and `repeat`), matching the TypeScript implementation's behavior.

### Macro Body Parsing
- Single-line macros: `#define NAME content`
- Brace-delimited macros: `#define NAME {content}` (braces are delimiters, not part of content)
- Multiline macros: Starting with `{` on a new line after the macro name

### Expression Parsing
The expression parser handles:
- Whitespace and newlines within argument lists
- Comments within expressions
- Nested parentheses and braces
- Array literals and builtin functions

## Testing

Successfully expands complex real-world files like `cpu.bfm` (875KB output).

```bash
cargo test  # Run test suite
cargo run -- expand cpu.bfm > output.bf  # Expand large file
```

## Known Limitations

- Comment preservation in some edge cases
- Complex nested for loops with tuple destructuring (3+ variables)

## Building

```bash
# Debug build
cargo build

# Release build (recommended for large files)
cargo build --release

# Run tests
cargo test
```

## WASM Support

The library includes WASM bindings for browser integration:

```bash
wasm-pack build --target web
```

## License

Part of the Brainfuck IDE project.