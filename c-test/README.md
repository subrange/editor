# C Test Suite for Ripple C Compiler

This directory contains the test suite for the Ripple C compiler (rcc), which compiles C99 code to Brainfuck.

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

1. **Keep tests focused** - Test one feature at a time
2. **Use assertions** - Output 'Y' for pass, 'N' for fail conditions
3. **Avoid unsupported features**:
   - `mul`, `muli`, `slt` VM opcodes (have bugs)
   - `typedef` declarations
   - Complex pointer provenance
4. **Include newlines** - End output with `\n` for clean formatting

## Prerequisites

Before running tests, ensure you have:

```bash
# Build the C compiler
cd .. && cargo build --release

# Build the assembler
cd ../src/ripple-asm && cargo build --release

# Build rbt tool
cd ../rbt && cargo build --release

# Install required tools
brew install coreutils  # for gtimeout
npm install -g @ahineya/bfm bf  # Brainfuck tools
```

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
- `.asm` - Generated assembly
- `.pobj` - Assembled object files
- `.bf` - Linked Brainfuck output
- `_expanded.bfm` - Expanded macro code

You can then manually inspect or run individual compilation steps.