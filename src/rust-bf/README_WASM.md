# Brainfuck WASM Module

This Rust Brainfuck interpreter can be compiled to WebAssembly for use in web browsers.

## Features

- Multiple cell sizes (8-bit, 16-bit, 32-bit)
- Configurable tape size
- Cell value wrapping
- Tape pointer wrapping
- Code optimization (Clear, Set, MulAdd, Scan patterns)
- Returns full tape state and pointer position after execution

## Building the WASM Module

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build the WASM module
wasm-pack build --target web --out-dir pkg --no-opt

# Or use the build script
./build-wasm.sh
```

## Using in JavaScript

```javascript
import init, { BrainfuckInterpreter } from './pkg/bf_wasm.js';

// Initialize the WASM module
await init();

// Create an interpreter with default options
const interpreter = new BrainfuckInterpreter();

// Or with custom options
const interpreter = BrainfuckInterpreter.with_options(
    30000,  // tape_size
    8,      // cell_size (8, 16, or 32)
    true,   // wrap (cell values)
    true,   // wrap_tape (pointer wrapping)
    true    // optimize (enable optimizations)
);

// Run a Brainfuck program
const code = '++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.';
const input = new Uint8Array([]); // Optional input
const result = interpreter.run_program(code, input);

// Result contains:
// - tape: Array of cell values (as u32)
// - pointer: Current pointer position
// - output: String output from the program

console.log('Output:', result.output);
console.log('Pointer:', result.pointer);
console.log('Tape:', result.tape);
```

## Testing

Open `test-wasm.html` in a web browser using a local web server:

```bash
# Python 3
python3 -m http.server 8000

# Node.js
npx http-server -p 8000

# Then open http://localhost:8000/test-wasm.html
```

## API Reference

### `BrainfuckInterpreter`

#### Constructor
- `new()` - Create interpreter with default settings
- `with_options(tape_size, cell_size, wrap, wrap_tape, optimize)` - Create with custom settings

#### Methods
- `run_program(code, input)` - Execute Brainfuck code with optional input
  - Returns: `{ tape: number[], pointer: number, output: string }`
- `run_program_with_callback(code, input, outputCallback)` - Execute with real-time output callback
  - `outputCallback(char: string, charCode: number)` - Called for each output character
  - Returns: `{ tape: number[], pointer: number, output: string }`
- `optimize_brainfuck(code)` - Optimize and return Brainfuck code
  - Returns: Optimized Brainfuck string

## Real-time Output Example

```javascript
// Create output callback for real-time display
const outputCallback = (char, charCode) => {
    // char is the character as a string
    // charCode is the ASCII value
    if (charCode === 10) {
        console.log(); // newline
    } else {
        process.stdout.write(char);
    }
};

// Run with callback
const result = interpreter.run_program_with_callback(
    code, 
    inputBytes, 
    outputCallback
);
```

This allows you to capture output as it's generated, perfect for:
- Live streaming output to the UI
- Building interactive debuggers
- Creating progress indicators for long-running programs
- Implementing output filters or transformations

## Command Line Usage

The interpreter can still be used as a command-line tool:

```bash
# Build the CLI binary
cargo build --release

# Run a Brainfuck program
./target/release/bf program.bf

# With options
./target/release/bf program.bf --cell-size 16 --tape-size 50000 --no-wrap

# Show optimized code
./target/release/bf program.bf --emit-optimized

# Disable optimizations
./target/release/bf program.bf --no-optimize
```