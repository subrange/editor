# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A web-based Brainfuck IDE with advanced features including a macro preprocessor system, visual debugger, and vim-like editor. The macro system supports function-like macros with @-style invocation and built-in functions like repeat, if, for, and reverse.

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
- **18 registers**: R0-R15, PC, PCB, RA, RAB
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
./target/release/rasm assemble test.asm --bank-size 32 --max-immediate 1000000

# Link object files
./target/release/rlink file1.pobj file2.pobj -o program.bin

# Convert to macro format
./target/release/rlink -f macro program.pobj
```

### Key Implementation Files
- `src/assembler.rs` - Main assembler logic, label detection, reference tracking
- `src/linker.rs` - Links object files, resolves label references
- `src/encoder.rs` - Instruction encoding logic
- `src/wasm.rs` - WASM bindings for browser integration
- `src/services/ripple-assembler/assembler.ts` - TypeScript wrapper with automatic linking

### Important Notes
- The assembler automatically runs the linker when unresolved references are detected
- JAL uses absolute instruction indices, branches use relative offsets
- Label references are properly categorized as branch/absolute/data based on instruction type
- The UI integration uses WASM through `src/services/ripple-assembler/`
- Browser caching can be an issue - hard refresh (Cmd+Shift+R) after rebuilding WASM

## Development Guidelines

- **Testing and Execution**
  - Please, do not try to run "npm run dev". If you want to test something, either use vitest (which can actually help us a lot), or create temporary .ts file and run it with npx tsx
- To directly assemble, link, and run .asm file, use rbt file.asm --run. It will help with testing C compiler implementation.
- Make sure to run rbt via gtimeout to not accidentally get stuck in an infinite loop
- After every change of the C compiler, please make sure you add the test case to `python3 run_c_tests.py` and run it to ensure that we don't have any regressions
-  To compile C file to asm, use target/release/rcc compile filename.c (optional -o output.asm) 

- VM currently has bugs with opcodes: slt, for now let's not use them in compiler tests
- In c-code tests, use if(condition) putchar('1') else putchar('N') to make sure we actually have some asserts and can capture it in run_c_tests.py