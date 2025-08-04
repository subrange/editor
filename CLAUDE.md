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

For macro system changes, the expansion logic is in `macro-expander-v2.ts` with comprehensive tests in the same directory.

The interpreter can run in both JavaScript and WASM modes - ensure changes work with both implementations.