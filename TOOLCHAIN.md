# Ripple C Toolchain Documentation

## Overview

The Ripple C toolchain provides a complete compilation pipeline from C99 source code to Brainfuck, including:
- **rcc** - C99 compiler (C → Assembly)
- **rasm** - Assembler (Assembly → Object files)
- **rlink** - Linker (Object files → Executable)
- **Runtime library** - Standard C library functions

## Toolchain Components

### 1. RCC - Ripple C Compiler
Compiles C99 source files to Ripple assembly.

```bash
rcc compile source.c -o output.asm
```

Features:
- C99 subset support
- Function-scoped label generation (prevents conflicts)
- Inline assembly support
- No built-in startup code (relies on crt0)

### 2. RASM - Ripple Assembler
Assembles Ripple assembly files to object files (.pobj).

```bash
rasm assemble source.asm -o output.pobj --bank-size 4096 --max-immediate 65535
```

Parameters:
- `--bank-size`: Memory bank size (default: 16, recommended: 4096)
- `--max-immediate`: Maximum immediate value (default/recommended: 65535)

### 3. RLINK - Ripple Linker
Links object files into executables or libraries.

```bash
# Create executable (Brainfuck)
rlink file1.pobj file2.pobj -f macro --standalone -o program.bf

# Create library archive
rlink lib1.pobj lib2.pobj -f archive -o library.par

# Link with libraries
rlink crt0.pobj library.par main.pobj -f macro --standalone -o program.bf
```

Output formats:
- `binary` - Binary executable format
- `text` - Human-readable assembly listing
- `macro` - Brainfuck macro format
- `archive` - Library archive (.par files)

Options:
- `--standalone` - Include CPU emulator template (for macro format)
- `--debug` - Enable debug mode in output

### 4. Runtime Library
Located in `/runtime/`, provides standard C library functions:
- `putchar(int c)` - Output a character
- `puts(char *s)` - Output a string
- `memset(void *s, int c, int n)` - Fill memory
- `memcpy(void *dest, void *src, int n)` - Copy memory

## Building a Multi-File Program

### Step 1: Prepare the Runtime Library

```bash
cd runtime/
make clean
make all

# This creates:
# - libruntime.par (library archive)
# - crt0.pobj (startup code)
```

### Step 2: Write Your Program

**main.c:**
```c
void putchar(int c);  // Declare external function
int add(int a, int b); // Declare function from other file

int main() {
    int result = add(5, 3);
    putchar('0' + result);  // Print '8'
    putchar('\n');
    return 0;
}
```

**math.c:**
```c
int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    int result = 0;
    for (int i = 0; i < b; i++) {
        result += a;
    }
    return result;
}
```

### Step 3: Compile to Assembly

```bash
rcc compile main.c -o main.asm
rcc compile math.c -o math.asm
```

### Step 4: Assemble to Object Files

```bash
rasm assemble main.asm -o main.pobj --bank-size 4096 --max-immediate 65535
rasm assemble math.asm -o math.pobj --bank-size 4096 --max-immediate 65535
```

### Step 5: Link Everything Together

```bash
# Link: startup + runtime + your code
rlink crt0.pobj libruntime.par main.pobj math.pobj -f macro --standalone -o program.bf
```

### Step 6: Run the Program

```bash
# Expand macros and run
bfm expand program.bf | bf
```

## Complete Example Makefile

```makefile
# Tools
RCC = ../target/release/rcc
RASM = ../src/ripple-asm/target/release/rasm
RLINK = ../src/ripple-asm/target/release/rlink

# Settings
BANK_SIZE = 4096
MAX_IMMEDIATE = 65535

# Runtime files
RUNTIME_DIR = ../runtime
CRT0 = $(RUNTIME_DIR)/crt0.pobj
RUNTIME_LIB = $(RUNTIME_DIR)/libruntime.par

# Source files
C_SOURCES = main.c math.c utils.c
ASM_FILES = $(C_SOURCES:.c=.asm)
OBJ_FILES = $(C_SOURCES:.c=.pobj)

# Output
PROGRAM = myprogram.bf

# Build executable
$(PROGRAM): $(OBJ_FILES) $(CRT0) $(RUNTIME_LIB)
	$(RLINK) $(CRT0) $(RUNTIME_LIB) $(OBJ_FILES) -f macro --standalone -o $(PROGRAM)

# Compile C to assembly
%.asm: %.c
	$(RCC) compile $< -o $@

# Assemble to object files
%.pobj: %.asm
	$(RASM) assemble $< -o $@ --bank-size $(BANK_SIZE) --max-immediate $(MAX_IMMEDIATE)

# Run the program
run: $(PROGRAM)
	bfm expand $(PROGRAM) | bf

clean:
	rm -f $(ASM_FILES) $(OBJ_FILES) $(PROGRAM) *.bf

.PHONY: run clean
```

## Creating Your Own Library

### Step 1: Write Library Functions

**mylib.c:**
```c
void print_number(int n) {
    if (n < 0) {
        putchar('-');
        n = -n;
    }
    if (n >= 10) {
        print_number(n / 10);
    }
    putchar('0' + (n % 10));
}

int strlen(char *s) {
    int len = 0;
    while (*s++) len++;
    return len;
}
```

### Step 2: Build Library Archive

```bash
# Compile and assemble
rcc compile mylib.c -o mylib.asm
rasm assemble mylib.asm -o mylib.pobj --bank-size 4096 --max-immediate 65535

# Create archive (can include multiple .pobj files)
rlink mylib.pobj -f archive -o libmylib.par
```

### Step 3: Use the Library

```c
// main.c
void print_number(int n);  // Declare library function

int main() {
    print_number(42);
    putchar('\n');
    return 0;
}
```

```bash
# Compile main
rcc compile main.c -o main.asm
rasm assemble main.asm -o main.pobj --bank-size 4096 --max-immediate 65535

# Link with both runtime and your library
rlink crt0.pobj libruntime.par libmylib.par main.pobj -f macro --standalone -o program.bf

# Run
bfm expand program.bf | bf
```

## Important Notes

1. **Label Uniqueness**: The compiler prefixes labels with function names (e.g., `main_L1`, `add_L2`) to prevent conflicts when linking multiple files.

2. **Startup Code (crt0)**: Required for all programs. Sets up stack and calls main():
   ```asm
   _start:
       LI R13, 0       ; Stack bank
       LI R14, 1000    ; Stack pointer
       LI R15, 1000    ; Frame pointer
       CALL main
       HALT
   ```

3. **Function Declarations**: Always declare external functions before use:
   ```c
   void putchar(int c);  // From runtime
   int myfunc(int x);    // From another file
   ```

4. **Archive Files (.par)**: JSON format containing multiple object files. Can be inspected with:
   ```bash
   cat libruntime.par | jq '.objects[].name'
   ```

5. **Linking Order**: 
   - crt0.pobj must come first (contains _start)
   - Libraries can be in any order
   - Main program typically last

## Troubleshooting

### "Duplicate label" errors
- Ensure you're using the latest compiler that prefixes labels with function names
- Check that you're not defining the same function in multiple files

### "Unresolved reference" errors
- Make sure all required libraries are included in the link command
- Verify function declarations match definitions

### Runtime errors
- Check stack initialization in crt0.asm
- Verify BANK_SIZE and MAX_IMMEDIATE match across all compilations
- Use `--debug` flag in rlink for debugging output

## Quick Reference

```bash
# Complete build pipeline
rcc compile program.c -o program.asm
rasm assemble program.asm -o program.pobj --bank-size 4096 --max-immediate 65535
rlink crt0.pobj libruntime.par program.pobj -f macro --standalone -o program.bf
bfm expand program.bf | bf

# Create library
rlink file1.pobj file2.pobj file3.pobj -f archive -o mylib.par

# Test with rbt (direct assembly execution)
rbt program.asm --run
```