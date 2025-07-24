# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Brainfuck IDE - a web-based integrated development environment for the Brainfuck programming language. It features a code editor with vim-like keybindings, a visual debugger showing tape state, and an interpreter with step-by-step execution capabilities.

## Development Commands

```bash
# Start development server
npm run dev

# Build for production (includes TypeScript type checking)
npm run build

# Run linting
npm run lint

# Preview production build
npm run preview
```

## Architecture

### State Management
The project uses RxJS BehaviorSubjects for reactive state management instead of traditional React state. Key stores:
- `src/components/editor/editor.store.ts` - Editor state, vim modes, command history
- `src/components/debugger/interpreter.store.ts` - Brainfuck interpreter state and execution

### Key Components
- **Editor**: Implements vim-like modes (normal/insert/command), bracket matching, and syntax tokenization
- **Debugger**: Visual tape display with breakpoint support and step-by-step execution
- **Interpreter**: Configurable tape size and cell size (8/16/32-bit), handles Brainfuck execution

### Patterns
- Command pattern for undo/redo operations
- Observable-based reactive state updates
- Feature-based folder structure under `src/components/`
- Keyboard shortcuts handled via `keybindings.service.ts`

## Tech Stack
- React 19.1.0 with TypeScript
- Vite 7.0.4 for build tooling
- TailwindCSS v4 (using new Vite plugin)
- RxJS 7.8.2 for state management
- @tanstack/react-virtual for virtualized rendering (planned for replacement)

## Notes
- No testing infrastructure currently exists
- The project has custom TailwindCSS utilities (`.h`, `.v` for flex containers)
- Tape viewer uses virtualization for performance with large tapes