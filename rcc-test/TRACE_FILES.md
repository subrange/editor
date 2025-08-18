# Trace Files in rcc-test

The Ripple C test runner (`rct`) now automatically generates compiler trace files for every test compilation. These files provide detailed visibility into each stage of the compilation process.

## Automatic Trace Generation

When running tests with `rct`, the compiler is invoked with the `--trace` flag, which generates the following JSON files in the `c-test/build/` directory:

- `test_name.pp.tokens.json` - Lexer output (tokenized source)
- `test_name.pp.ast.json` - Parser output (Abstract Syntax Tree)
- `test_name.pp.sem.json` - Semantic analyzer output
- `test_name.pp.tast.json` - Typed AST (with resolved types)
- `test_name.pp.ir.json` - Intermediate Representation

Note: The `.pp` in the filename indicates these are generated from the preprocessed C file.

## Usage

### Running Tests
```bash
# Run a single test - trace files are generated automatically
./rct test_add

# Run with --no-cleanup to preserve all artifacts including trace files
./rct test_add --no-cleanup

# Run multiple tests
./rct test_add test_pointer
```

### Using the TUI
```bash
# Launch the TUI - trace files are generated for each test run
./rct tui

# In the TUI, press Enter on any test to run it
# Trace files will be saved in c-test/build/
```

## File Locations

All trace files are saved in the `c-test/build/` directory alongside other compilation artifacts:
- `.pp.c` - Preprocessed C source
- `.asm` - Generated assembly
- `.ir` - IR in text format
- `.pobj` - Assembled object file
- `.bin` - Final binary (RVM format)
- `.pp.*.json` - Trace JSON files

## Analyzing Trace Files

The JSON files can be used for:
- Debugging compilation issues
- Understanding how the compiler transforms code
- Building visualization tools
- Teaching compiler internals

Example of viewing a trace file:
```bash
# Pretty-print a trace file
jq '.' c-test/build/test_add.pp.tokens.json | head -20

# Check the AST structure
jq '.items[0].Function.name' c-test/build/test_add.pp.ast.json

# View typed AST with resolved types
jq '.items[0].Function.body' c-test/build/test_add.pp.tast.json
```

## Cleanup

By default, test artifacts are cleaned up after successful test runs. To preserve trace files:

1. Use the `--no-cleanup` flag when running tests
2. Or manually copy trace files before they're deleted
3. Failed tests automatically preserve all artifacts for debugging

## Implementation Details

The trace functionality is implemented by:
1. The `rcc` compiler's `--trace` flag (added to rcc-driver)
2. Automatic inclusion of `--trace` in `rcc-test/src/compiler.rs`
3. Trace files are generated for both CLI and TUI test runners

This provides complete visibility into the compilation pipeline without requiring manual intervention.