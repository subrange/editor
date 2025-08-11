# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

1.
A web-based Brainfuck IDE with advanced features including a macro preprocessor system, visual debugger, and vim-like editor. 
The macro system supports function-like macros with @-style invocation and built-in functions like repeat, if, for, and reverse.
2.
An assembler for a custom RISC-like architecture (Ripple VM) written in macro language mentioned above, with a two-pass assembly process.
3.
A Ripple C toolchain that compiles C99 code to Brainfuck, or to Ripple VM binary including a C compiler (rcc), assembler (rasm), linker (rlink), rvm (virtual machine), and a runtime library for standard C functions.

## Development Commands

```bash
# Start development server
npm run dev

# Build for production
npm run build

# Lint code
npm run lint

# Run tests
npm run test

# Run tests once
npm run test:run
```

## Architecture

### Core Technologies
- React 19.1.0 with TypeScript
- Vite 7.0.4 build system
- TailwindCSS v4 with custom utilities (.h for horizontal flex, .v for vertical flex)
- RxJS for reactive state management (BehaviorSubjects instead of React state)
- Web Workers for macro expansion and tokenization

### Key Stores (RxJS BehaviorSubjects)
- `editor.store.ts` - Editor state, vim modes, cursor position, command history
- `interpreter.store.ts` - Brainfuck execution state, tape, breakpoints
- `search.store.ts` - Search state and results
- `quick-nav.store.ts` - Quick navigation state
- `settings.store.ts` - User preferences

### Major Components

**Editor System**
- Vim-like modal editing (normal/insert/visual/command modes)
- Virtual line rendering for performance
- Real-time bracket matching
- Macro-aware syntax highlighting via viewport tokenizer
- Search with regex support and scroll-to-match

**Macro System** 
- Preprocessor with @-style macro invocation: `@macroName(args)`
- Built-in functions: `{repeat(n, content)}`, `{if(cond, true, false)}`, `{for(var in array, body)}`, `{reverse(array)}`
- Multiline macros with backslash continuation
- Web Worker-based expansion for non-blocking UI
- See `src/services/macro-expander/macro-expander.md` for full documentation

**Debugger**
- Visual tape display with cell highlighting
- Step/run/pause controls with configurable speed
- Breakpoint support
- Configurable tape size and cell bit width (8/16/32)

**File Management**
- Local storage persistence
- File tree sidebar
- Snapshot system for saving states

### Service Layer
- `editor-manager.service.ts` - Centralized editor state coordination
- `keybindings.service.ts` - Global keyboard shortcut handling
- `wasm-interpreter.service.ts` - WASM-based interpreter (if available)

### Testing
Tests use Vitest and are located alongside source files as `*.test.ts`

## Key Patterns

1. **Store Pattern**: All state uses RxJS BehaviorSubjects with a consistent interface:
   ```typescript
   const store$ = new BehaviorSubject(initialState);
   const updateStore = (updates: Partial<State>) => store$.next({...store$.value, ...updates});
   ```

2. **Web Workers**: CPU-intensive operations (macro expansion, tokenization) run in workers to keep UI responsive

3. **Command Pattern**: Editor operations use commands for undo/redo support

4. **Virtual Rendering**: Editor uses virtualization for handling large files efficiently

## Brainfuck Extensions

The IDE supports standard Brainfuck plus:
- `.bfm` files for macro-enabled Brainfuck
- Macro preprocessing before execution
- Visual debugging not available in standard interpreters

## Working with the Codebase

When modifying the editor, be aware that it uses a custom vim implementation - check `editor.store.ts` for mode handling and command processing.

For macro system changes, the expansion logic is in `macro-expander-v3.ts` with comprehensive tests in the same directory.

The interpreter can run in both JavaScript and WASM modes - ensure changes work with both implementations.

## Ripple Assembler

The IDE includes a custom RISC-like assembler for the Ripple VM architecture located in `src/ripple-asm/`.

### Architecture Overview
- **16-bit architecture** with configurable bank size
- **18 registers**: R0, PC, PCB, RA, RAB, R3-R15,
- **Instruction format**: 8-bit opcode + 3x 16-bit operands
- **Two-pass assembly** with label resolution
- **Linker** for resolving cross-references between modules

### Instruction Set
**Arithmetic**: ADD, SUB, MUL, DIV, MOD (register and immediate versions)
**Logical**: AND, OR, XOR, SLL, SRL, SLT, SLTU
**Memory**: LOAD, STORE, LI (load immediate)
**Control**: JAL, JALR, BEQ, BNE, BLT, BGE, BRK, HALT
**Virtual**: MOVE, INC, DEC, PUSH, POP, CALL, RET (expand to real instructions)

### Building and Testing

```bash
# Build WASM module for UI
cd src/ripple-asm
./build-wasm.sh

# Build CLI tools
cargo build --release

# Run tests
cargo test

# Assemble a file
rasm assemble test.asm --bank-size 4096 --max-immediate 65535

# Link object files
rlink file1.pobj file2.pobj -o program.bin

# Convert to macro format
rlink -f macro program.pobj
```

### Key Implementation Files
- `src/assembler.rs` - Main assembler logic, label detection, reference tracking
- `src/linker.rs` - Links object files, resolves label references
- `src/encoder.rs` - Instruction encoding logic
- `src/wasm.rs` - WASM bindings for browser integration
- `src/services/ripple-assembler/assembler.ts` - TypeScript wrapper with automatic linking

### Important Notes
- JAL uses absolute instruction indices, branches use relative offsets
- Label references are properly categorized as branch/absolute/data based on instruction type
- Browser caching can be an issue - hard refresh (Cmd+Shift+R) after rebuilding WASM

## Development Guidelines

- **Testing and Execution**
- After every change of the C compiler, please make sure you add the test case to `python3 c-test/run_tests.py` and run it to ensure that we don't have any regressions
- After every change of the C compiler, make sure to rebuild it from project root with `cargo build --release` to ensure the latest changes are included
- VM opcodes div, mod, divi, modi, mul, muli, slt, and store have been fixed and are now safe to use
- In c-test tests, use if(condition) putchar('1') else putchar('N') to make sure we actually have some asserts and can capture it in run_tests.py

- Rasm, Rlink are placed in src/ripple-asm/target/release/ and can be run from there

## Developing C Compiler

When working on the C compiler, follow these steps:
1. Create a new test file in `c-test/tests/` or `c-test/tests-runtime/`.
2. Add the test to a proper section of `c-test/run_tests.py` with expected output.
3. Implement compiler logic
4. Run the test suite with `python3 c-test/run_tests.py --no-cleanup` to verify changes.
5. Ensure all tests pass.
6. If tests fail, debug using the generated files in `c-test/build/`.
7. Once all tests pass, run `python3 c-test/run_tests.py --clean` to remove build artifacts.

## Directory Structure

```
c-test/
├── tests/                    # Main test cases that should pass
├── tests-runtime/            # Tests requiring full runtime library
├── tests-known-failures/     # Tests expected to fail (unsupported features)
├── build/                    # Temporary build artifacts (auto-created)
└── run_tests.py             # Test runner script
```

## Running Tests

```bash
# Run all tests
python3 run_tests.py

# Run single test file
python3 run_tests.py tests/test_example.c

# Run tests with verbose flag to see what build program outputs
python3 run_tests.py tests/test_example.c --verbose

# Run tests with custom timeout. Default is 2 seconds.
python3 run_tests.py tests/test_example.c --timeout 5

# Run tests without cleanup (keep generated files for debugging)
python3 run_tests.py --no-cleanup

# Clean build directory only
python3 run_tests.py --clean
```

## Adding a New Test

### 1. Create a Test File

Create a `.c` file in the appropriate directory:
- `tests/` - For standard tests using only basic features
- `tests-runtime/` - For tests requiring runtime library functions
- `tests-known-failures/` - For tests documenting unsupported features

### 2. Write Test Code

Tests should output predictable text that can be verified. Use `putchar()` for output:

```c
// tests/test_example.c
void putchar(int c);

int main() {
    if (2 + 2 == 4) {
        putchar('Y');  // Yes, test passed
    } else {
        putchar('N');  // No, test failed
    }
    putchar('\n');
    return 0;
}
```

### 3. Add Test to run_tests.py

Edit `run_tests.py` and add your test to the `tests` list:

```python
tests = [
    # ... existing tests ...
    ("tests/test_example.c", "Y\n", False),  # (file, expected_output, use_runtime)
]
```

Parameters:
- **file**: Path to test C file
- **expected_output**: Exact expected output string
- **use_runtime**: `False` for basic tests, `True` for runtime tests

### 4. Run Your Test

```bash
python3 run_tests.py
```

## Test Guidelines

1. **Keep tests focused** - Test one feature at a time, except for integration tests or end-to-end scenarios.
2. **Use assertions** - Output 'Y' for pass, 'N' for fail conditions
4. **Include newlines** - End output with `\n` for clean formatting

## Understanding Test Output

- ✓ Green: Test passed
- ✗ Red: Test failed
- ✓ Yellow: Test passed with warnings (e.g., pointer provenance issues)
- Known failures: Tests in `tests-known-failures/` are expected to fail

## Debugging Failed Tests

Use `--no-cleanup` to keep intermediate files:

```bash
python3 run_tests.py --no-cleanup
```

This preserves in `build/`:
- `.ir` - Intermediate representation files
- `.asm` - Generated assembly
- `.pobj` - Assembled object files
- `.bin` - Final binary output
- `.disassembly.asm` - Disassembled output of the binary, used rasm for disassembly

If ran with --backend bf, then instead of `.bin` you will have:
- `.bfm` - Linked Brainfuck macro output
- `_expanded.bf` - Expanded macro code

You can then manually inspect or run individual compilation steps.

# RVM — Ripple Virtual Machine

Usage: rvm [OPTIONS] <binary-file>

Run a Ripple VM binary program

You can use `rvm file.bin --verbose` to run the binary with verbose output, which will show you the VM commands executed, and additional information about the state of the VM during execution. Use it for debugging purposes to understand how the binary behaves.

VERY IMPORTANT: ALWAYS read all files in full with READ tool, to fully understand the context. NEVER read in full expanded bf files.
VERY IMPORTANT: Just read the full file. JUST. READ. THE. FULL. FILE.
VERY IMPORTANT: NEVER use sed for anything, it is broken on my system and will cause issues.
VERY IMPORTANT: If you can't find something, execute pwd to make sure you are in a correct directory
