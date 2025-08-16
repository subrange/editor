# C Test Suite for Ripple C Compiler

This directory contains the test suite for the Ripple C compiler (rcc), which compiles C99 code to Brainfuck.

## Directory Structure

```
c-test/
├── tests/                    # Main test cases that should pass
├── tests-known-failures/     # Tests expected to fail (unsupported features)
└── build/                    # Temporary build artifacts (auto-created)
```

## Running Tests

```bash
# Run all tests
rct

# Run specific tests (without .c extension)
rct test_add test_bool

# Run with different backend (default is rvm)
rct --backend bf

# Run with verbose output
rct -v

# Run with custom timeout (default is 2 seconds)
rct --timeout 5

# Keep build artifacts for debugging
rct --no-cleanup

# Run tests sequentially instead of in parallel
rct --no-parallel

# Debug mode (RVM with -t flag)
rct -d
```

## Adding a New Test

### 1. Create a Test File

Create a `.c` file in the appropriate directory:
- `tests/`

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

### 3. Add Test to Suite

```bash
# Add a test with expected output
rct add tests/test_example.c "Y\n"

# Add a test with description
rct add tests/test_example.c "Y\n" -d "Tests basic arithmetic"

# Add a test that doesn't use runtime
rct add tests/test_standalone.c "42" --no-runtime
```

### 4. Run Your Test

```bash
# Run a specific test
rct test_example

# Run multiple tests
rct test_example test_arithmetic

# Run all tests matching a pattern
rct run -f "test_*"
```

## Test Guidelines

1. **Keep tests focused** - Test one feature at a time
2. **Use assertions** - Output 'Y' for pass, 'N' for fail conditions
4. **Include newlines** - End output with `\n` for clean formatting

## Prerequisites

Before running tests, ensure you have:

```bash
# Build the C compiler and tools
cd .. && cargo build --release

# Build the assembler
cd ../src/ripple-asm && cargo build --release && cd ..

# Build rbt tool
cd ../rbt && cargo build --release && cd ..


# Install required tools
brew install coreutils  # for gtimeout
npm install -g @ahineya/bfm bf  # Brainfuck tools
```

## Additional Commands

```bash
# List all available tests
rct list

# List only test names
rct list --names-only

# Include known failures in listing
rct list --include-failures

# Check for test files not added to suite
rct check

# Show test suite statistics
rct stats

# Clean build directory
rct clean

# Build runtime library
rct build-runtime

# Debug a single test interactively
rct debug test_example

# Rename a test (updates both .c and .meta.json)
rct rename old_test new_test

# Launch interactive TUI for test management
rct tui
```

## Understanding Test Output

- ✓ Green: Test passed
- ✗ Red: Test failed  
- ✓ Yellow: Test passed with warnings (e.g., pointer provenance issues)
- Known failures: Tests in `tests-known-failures/` are expected to fail

## Debugging Failed Tests

Use `--no-cleanup` to keep intermediate files:

```bash
rct --no-cleanup test_example
```

This preserves in `build/`:
- `.asm` - Generated assembly
- `.pobj` - Assembled object files
- `.bin` - Final binary (RVM backend)
- `.bfm` - Linked Brainfuck macro output (BF backend)
- `_expanded.bf` - Expanded macro code (BF backend)
- `.disassembly.asm` - Disassembled binary (for debugging)

You can then manually inspect or run individual compilation steps.