# RCC Compiler - Missing C99 Features

This document lists C99 features that are not yet implemented in the RCC compiler, organized by priority and complexity.

## Critical Missing Features (High Priority)

### Preprocessor
- **Stringification operator (`#`)** - Convert macro arguments to strings
- **Token pasting operator (`##`)** - Concatenate tokens
- **Predefined macros** - `__FILE__`, `__LINE__`, `__DATE__`, `__TIME__`, `__STDC__`, etc.
- **Complex conditional expressions** - Full expression evaluation in `#if` directives
- **Error directive** - `#error` and `#warning`
- **Null directive** - Standalone `#`
- **_Pragma operator** - C99 alternative to `#pragma`

### Type System
- **Floating-point types** - `float`, `double`, `long double`
- **Complex types** - `_Complex` and `_Imaginary`
- **Long long** - 64-bit integer type
- **Variable-length arrays (VLAs)** - Runtime-sized arrays
- **Flexible array members** - Struct with `[]` as last member
- **Incomplete array types in structs** - Better support for trailing arrays
- **Restrict qualifier** - `restrict` pointer optimization hint
- **Inline functions** - `inline` keyword support
- **_Bool type improvements** - Full standard compliance

### Declarations
- **Mixed declarations and code** - C99 allows declarations anywhere in blocks
- **Compound literals for all types** - Currently limited support
- **Static array indices in parameters** - `void func(int arr[static 10])`
- **Type qualifiers in array parameters** - `void func(int arr[const])`
- **Complex designated initializers** - Nested and array range initializers

## Standard Library Features (Medium Priority)

### Headers Not Implemented
- `<complex.h>` - Complex number operations
- `<fenv.h>` - Floating-point environment
- `<float.h>` - Floating-point limits
- `<inttypes.h>` - Format conversion of integer types
- `<iso646.h>` - Alternative operator spellings
- `<limits.h>` - Implementation limits
- `<locale.h>` - Localization
- `<math.h>` - Mathematical functions
- `<setjmp.h>` - Non-local jumps
- `<signal.h>` - Signal handling
- `<stdarg.h>` - Variable arguments
- `<stdbool.h>` - Boolean type and values
- `<stddef.h>` - Standard definitions
- `<stdint.h>` - Integer types
- `<stdio.h>` - Full I/O support
- `<stdlib.h>` - General utilities
- `<string.h>` - String operations
- `<tgmath.h>` - Type-generic math
- `<time.h>` - Time and date
- `<wchar.h>` - Wide character support
- `<wctype.h>` - Wide character classification

### I/O Functions
- `printf()` family - Full formatting support
- `scanf()` family - Input parsing
- `getchar()`, `gets()` - Character/line input
- File operations - `fopen()`, `fread()`, `fwrite()`, etc.
- Stream operations - `fseek()`, `ftell()`, `rewind()`

### Memory Management
- **Dynamic allocation** - `malloc()`, `free()`, `calloc()`, `realloc()`
- **Memory operations** - `memcpy()`, `memmove()`, `memset()`, `memcmp()`

### String Functions
- String manipulation - `strcpy()`, `strcat()`, `strlen()`, etc.
- String comparison - `strcmp()`, `strncmp()`, etc.
- String searching - `strchr()`, `strstr()`, etc.
- String conversion - `atoi()`, `strtol()`, etc.

## Language Features (Lower Priority)

### Expressions
- **Comma operator in all contexts** - Currently limited support
- **Compound literals in all contexts** - Function arguments, return values
- **Generic selections** - `_Generic` (C11 feature often expected)

### Qualifiers and Specifiers
- **Type qualifiers on function return types**
- **_Noreturn** - Function attribute (C11)
- **_Alignas/_Alignof** - Alignment control (C11)
- **_Thread_local** - Thread storage (C11)

### Bit Fields
- **Bit fields in structs** - `unsigned int field : 3;`
- **Bit field packing rules**
- **Implementation-defined bit field behavior**

## Compiler Infrastructure

### Optimizations
- **Constant folding** - Compile-time expression evaluation
- **Dead code elimination** - More sophisticated analysis
- **Common subexpression elimination**
- **Loop optimizations**
- **Function inlining**
- **Tail call optimization**

### Diagnostics
- **Better error messages** - More context and suggestions
- **Warning levels** - `-W` flags support
- **Static analysis** - Undefined behavior detection
- **Debug information** - DWARF or similar format

### Platform Support
- **Multiple target architectures** - Currently Ripple VM only
- **Calling conventions** - cdecl, stdcall, etc.
- **ABI compliance** - Standard application binary interface
- **Position-independent code** - For shared libraries

## Advanced Features (Nice to Have)

### GNU C Extensions
- **Statement expressions** - `({ ... })`
- **Nested functions**
- **Case ranges** - `case 1 ... 10:`
- **Zero-length arrays**
- **Attributes** - `__attribute__((...))`

### Compiler Builtins
- **Builtin functions** - `__builtin_expect()`, `__builtin_clz()`, etc.
- **Type traits** - `__has_trivial_copy()`, etc.
- **Overflow checking** - `__builtin_add_overflow()`, etc.

### Linkage and Modules
- **Weak symbols** - `__attribute__((weak))`
- **Visibility control** - hidden, protected, etc.
- **Link-time optimization**
- **Separate compilation units** - Better multi-file support

## Implementation Notes

### Priority Recommendations

1. **Immediate priorities** for C99 compliance:
   - Floating-point types (at least basic support)
   - Variable-length arrays
   - Mixed declarations and code
   - Standard library headers (especially `<stdint.h>`, `<stdbool.h>`)
   - Dynamic memory allocation

2. **Secondary priorities** for usability:
   - Full preprocessor operators (`#`, `##`)
   - Bit fields
   - Complete `printf`/`scanf` families
   - String and memory functions

3. **Long-term goals**:
   - Optimization passes
   - Multiple target support
   - Full standard library
   - GNU C extensions

### Known Limitations

- The current architecture (16-bit Ripple VM) may require special handling for:
  - 64-bit types (`long long`)
  - Floating-point operations (may need software emulation)
  - Some standard library functions that assume 32-bit+ architecture

### Testing Requirements

Each new feature should include:
- Unit tests in the test suite
- Integration tests with existing features
- Compliance tests from C99 test suites
- Performance benchmarks where applicable

## References

- ISO/IEC 9899:1999 (C99 Standard)
- ISO/IEC 9899:2011 (C11 Standard) - for forward compatibility
- GNU C Manual - for common extensions
- POSIX.1-2008 - for POSIX compliance