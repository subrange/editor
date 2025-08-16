# rct - Ripple C Compiler Test Runner

A clean, modular, and efficient test runner for the Ripple C compiler, written in Rust. The binary is named `rct` for convenience.

## Features

- **Parallel Test Execution**: Run tests in parallel for faster feedback
- **Beautiful Output**: Colored output with progress bars and detailed diffs
- **Flexible Backends**: Support for both Brainfuck and Ripple VM execution
- **Test Management**: Add, list, and filter tests easily
- **Debug Mode**: Interactive debugging with the RVM TUI debugger
- **Comprehensive Reporting**: Detailed test results with statistics

## Installation

Build the test runner:

```bash
cd rct
cargo build --release
```

The binary will be available at `target/release/rct`.

## Usage

### Running Tests

Run all tests:
```bash
rct
```

Run specific tests:
```bash
rct test_hello test_add test_array
```

Filter tests by pattern:
```bash
rct run --filter array
```

### Command Line Options

```
rct [OPTIONS] [TEST...] [COMMAND]

Options:
  -b, --backend <BACKEND>      Execution backend [default: rvm] [possible values: bf, rvm]
  -t, --timeout <TIMEOUT>      Timeout in seconds [default: 2]
      --bank-size <BANK_SIZE>  Bank size for assembler [default: 16384]
  -v, --verbose               Show output from test programs
      --no-cleanup            Don't clean up generated files
      --no-parallel           Disable parallel test execution
  -d, --debug                 Use debug mode (RVM with -t flag)
      --tests-file <PATH>     Path to tests.json [default: c-test/tests.json]
      --build-dir <PATH>      Build directory [default: c-test/build]
      --project-root <PATH>   Project root directory
  -h, --help                  Print help
  -V, --version               Print version

Commands:
  run            Run tests (default)
  add            Add a new test to tests.json
  clean          Clean build directory
  list           List all available tests
  debug          Build and run a test interactively
  build-runtime  Build runtime library
  stats          Show test suite statistics
  help           Print help for a command
```

### Adding Tests

Add a new test with expected output:
```bash
rct add tests/my_test.c "Hello World\n"
```

Add a test without runtime:
```bash
rct add tests/minimal.c "OK" --no-runtime
```

Add with description:
```bash
rct add tests/feature.c "PASS" -d "Tests new feature X"
```

### Debugging Tests

Debug a test interactively with the RVM TUI debugger:
```bash
rct debug test_hello
```

### Managing Tests

List all tests:
```bash
rct list
```

List test names only:
```bash
rct list --names-only
```

Show test statistics:
```bash
rct stats
```

Clean build artifacts:
```bash
rct clean
```

## Architecture

The test runner is organized into modular components:

### Core Modules

- **`config`**: Test configuration and JSON handling
- **`command`**: Process execution with timeout support
- **`compiler`**: C to binary compilation pipeline
- **`runner`**: Test execution engine with parallel support
- **`reporter`**: Output formatting and progress reporting
- **`cli`**: Command-line interface and argument parsing

### Execution Flow

1. **Load Configuration**: Read tests from `tests.json`
2. **Build Runtime**: Compile runtime library with specified bank size
3. **Compile Tests**: C → Assembly → Object → Binary/Brainfuck
4. **Execute Tests**: Run with timeout and capture output
5. **Compare Results**: Check output against expected values
6. **Report Results**: Display colored output with diffs

## Test Configuration

Tests are defined in `tests.json`:

```json
{
  "tests": [
    {
      "file": "tests/test_hello.c",
      "expected": "Hello World\n",
      "use_runtime": true,
      "description": "Basic hello world test"
    }
  ],
  "known_failures": [
    {
      "file": "tests-known-failures/unsupported.c",
      "description": "Feature not yet implemented"
    }
  ]
}
```

## Performance

- **Parallel Execution**: Tests run in parallel by default using Rayon
- **Progress Indicators**: Real-time progress bars for long test suites
- **Efficient Cleanup**: Automatic artifact cleanup to save disk space
- **Caching**: Runtime library is built once per session

## Error Handling

- **Timeout Detection**: Tests that run too long are terminated
- **Compilation Errors**: Clear error messages for build failures
- **Output Mismatches**: Detailed diffs showing exact differences
- **Provenance Warnings**: Special handling for pointer provenance issues

## Comparison with Python Runner

| Feature | Python Runner | Rust Runner |
|---------|--------------|-------------|
| Parallel Execution | ❌ | ✅ |
| Type Safety | ❌ | ✅ |
| Progress Bars | ❌ | ✅ |
| Structured CLI | Basic | Advanced (clap) |
| Error Handling | Basic | Comprehensive |
| Performance | Slower | Faster |
| Code Organization | Single file | Modular |
| Test Discovery | Basic | Advanced |

## Contributing

The codebase is organized for easy extension:

1. Add new commands in `cli.rs`
2. Extend test configuration in `config.rs`
3. Add new backends in `compiler.rs`
4. Customize output in `reporter.rs`

## License

Same as the parent project.