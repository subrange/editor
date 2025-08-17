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
- After every change of the C compiler, please make sure you add the test case using `rct add` and run it with `rct` to ensure that we don't have any regressions
- After every change of the C compiler, make sure to rebuild it from project root with `cargo build --release` to ensure the latest changes are included
- VM opcodes div, mod, divi, modi, mul, muli, slt, and store have been fixed and are now safe to use
- In c-test tests, use if(condition) putchar('Y') else putchar('N') to make sure we actually have some asserts and can capture it in the test runner

- Rasm, Rlink are placed in src/ripple-asm/target/release/ and can be run from there

## Developing C Compiler

When working on the C compiler, follow these steps:
1. Create a new test file in `c-test/tests/` for tests.
2. Add the test to the suite using `rct add tests/test_name.c "expected output"`.
3. Implement compiler logic
4. Run the test suite with `rct --no-cleanup` to verify changes.
5. Ensure all tests pass.
6. If tests fail, debug using the generated files in `c-test/build/`.
7. Once all tests pass, run `rct clean` to remove build artifacts.

## Directory Structure

```
c-test/
├── tests/                    # Main test cases that should pass
├── tests-known-failures/     # Tests expected to fail (unsupported features)
├── build/                    # Temporary build artifacts (auto-created)
└── *.meta.json              # Test metadata files (created by rct add)

rcc-test/
├── src/
│   ├── main.rs              # Main entry point for rct
│   ├── cli.rs               # CLI argument parsing
│   └── ...                  # Test runner implementation
└── Cargo.toml               # Rust dependencies
```

## Running Tests

```bash
# Run all tests
rct

# Run specific tests (without path or .c extension)
rct test_example test_example2

# Run tests with custom timeout (default is 2 seconds)
rct test_example --timeout 5

# Run with different backend
rct --backend bf  # Use Brainfuck backend
rct --backend rvm # Use RVM backend (default)

# Run with verbose output
rct -v

# Debug mode for RVM
rct -d

# Run tests sequentially
rct --no-parallel
```

## Adding a New Test

### 1. Create a Test File

Create a `.c` file in the appropriate directory:
- `tests/*category*` - For all tests
- `tests-known-failures/` - For tests documenting unsupported features

### 2. Write Test Code

Tests should output predictable text that can be verified. Use `putchar()` for single char output, puts for strings.

```c
// tests/test_example.c
#include <stdio.h>

int main() {
    puts("Hello, World!");  // Expected output: "Hello, World!\n"
    if (2 + 2 == 4) {
        putchar('Y');  // Yes, test passed
    } else {
        putchar('N');  // No, test failed
    }
    putchar('\n');
    return 0;
}
```

### 3. Add Test to Test Suite

```bash
# Add a regular test with expected output
rct add tests/test_new.c "Hello\n"

# Add a test with description
rct add tests/test_new.c "Hello\n" -d "Test greeting output"

# Add a test that doesn't use runtime
rct add tests/test_standalone.c "42" --no-runtime
```

The `rct add` command creates a `.meta.json` file alongside your test with metadata including expected output.

### 4. Run Your Test

```bash
# Run specific test
rct test_new

# Run all tests
rct
```

## Test Guidelines

1. **Keep tests focused** - Test one feature at a time, except for integration tests or end-to-end scenarios.
2. **Use assertions** - Output 'Y' for pass, 'N' for fail conditions
3. **Include newlines** - End output with `\n` for clean formatting

## Understanding Test Output

- ✓ Green: Test passed
- ✗ Red: Test failed
- ✓ Yellow: Test passed with warnings (e.g., pointer provenance issues)
- Known failures: Tests in `tests-known-failures/` are expected to fail

## Debugging Failed Tests

Use `--no-cleanup` flag to keep all generated files in `build/` directory for debugging:

```bash
rct test_name --no-cleanup
```

This preserves in `build/`:
- `.ir` - Intermediate representation files
- `.asm` - Generated assembly
- `.pobj` - Assembled object files
- `.bin` - Final binary output (RVM backend)
- `.disassembly.asm` - Disassembled output of the binary, uses rasm for disassembly

If ran with `--backend bf`, then instead of `.bin` you will have:
- `.bfm` - Linked Brainfuck macro output
- `_expanded.bf` - Expanded macro code

You can then manually inspect or run individual compilation steps.

### Interactive Debugging

For detailed debugging of a single test:

```bash
rct debug test_name
```

This runs the test interactively, showing each compilation step.

### Additional Test Management Commands

```bash
# List all tests
rct list

# Check for orphaned test files
rct check

# Show test suite statistics
rct stats

# Clean build artifacts
rct clean

# Rename a test
rct rename old_name new_name

# Launch interactive TUI
rct tui
```

IMPORTANT: Use `rcc compile file.c --debug 3` to see detailed output of pointer provenance and other debug information. It is VERY helpful.
IMPORTANT: Use "log" crate's `trace!` and `debug!` macros to log detailed information during compilation. This will help you understand how the compiler processes your code.

# RVM — Ripple Virtual Machine

Usage: rvm [OPTIONS] <binary-file>

Run a Ripple VM binary program

You can use `rvm file.bin --verbose` to run the binary with verbose output, which will show you the VM commands executed, and additional information about the state of the VM during execution. Use it for debugging purposes to understand how the binary behaves.

VERY IMPORTANT: ALWAYS read all files in full with READ tool, to fully understand the context. NEVER read in full expanded bf files.
VERY IMPORTANT: Just read the full file. JUST. READ. THE. FULL. FILE.
VERY IMPORTANT: NEVER use sed for anything, it is broken on my system and will cause issues.
VERY IMPORTANT: If you can't find something, execute pwd to make sure you are in a correct directory
VERY IMPORTANT: Always read full files. Never read parts of files. When you read the full file, you understand the context. When you read part of the file, you are blind. Remember — ALWAYS read full files. Do not search in file. Do not grep. Do not use sed. Do not use any other tool that reads only part of the file. ALWAYS read the full file.
VERY IMPORTANT RULES:
1. No silent failures - Always throw
   explicit errors instead of generating
   incorrect code
2. Comprehensive unit tests - Test
   all edge cases and scenarios
3. Conservative implementation -
   Better to fail loudly than silently
   corrupt
IMPORTANT: rcc is a project inside rust workspace, so everything is being built into the project root target/release directory.
IMPORTANT: rct (Ripple C Test runner) is located in target/release/rct after building with `cargo build --release`.