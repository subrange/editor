# Brainfuck IDE

A modern, feature-rich web-based IDE for the Brainfuck programming language with advanced macro support, visual debugging, and vim-like editing.

## Features

### 🎯 Advanced Macro System
- **Macro Preprocessor**: Define reusable code with `#define` directives
- **Function-like Macros**: Support for parameterized macros with `@macroName(args)` syntax
- **Built-in Functions**:
  - `{repeat(n, content)}` - Repeat content n times
  - `{if(condition, true_branch, false_branch)}` - Conditional expansion
  - `{for(var in array, body)}` - Iterate over arrays
  - `{reverse(array)}` - Reverse array literals
- **Multiline Macros**: Use `\` for line continuation, or just {} blocks
- **Real-time Expansion**: See expanded code instantly

### ✏️ Editor
- **Bracket Matching**: Visual highlighting of matching `[` and `]`
- **Search & Replace**: Regex-powered search with highlighting
- **Syntax Highlighting**: Intelligent highlighting for Brainfuck and macro syntax
- **Virtual Rendering**: Efficient handling of large files

### 🐛 Visual Debugger
- **Interactive Tape Display**: See memory cells update in real-time
- **Step-by-Step Execution**: Step through code with customizable speed
- **Breakpoints**: Set breakpoints to pause execution
- **Configurable Tape**: Adjustable tape size and cell bit width (8/16/32-bit)
- **Execution Controls**: Run, pause, step, and reset

### 📁 File Management
- **Local Storage**: Automatic saving to browser storage
- **Snapshots**: Save and restore IDE states
- **Import/Export**: Work with `.bf` and `.bfm` (macro-enabled) files

### ⚡ Performance
- **Web Workers**: Non-blocking macro expansion and tokenization
- **Virtualized Rendering**: Smooth scrolling even with large files
- **Optimized Interpreter**: Fast execution with optional WASM acceleration

## Getting Started

### Prerequisites
- Node.js (v18 or higher)
- npm or yarn

### Installation
```bash
# Clone the repository
git clone https://github.com/yourusername/brainfuck-ide.git
cd brainfuck-ide

# Install dependencies
npm install

# Start development server
npm run dev
```

### Building for Production
```bash
# Build the project
npm run build

```

## Architecture

### Technology Stack
- **Frontend**: React 19.1.0 + TypeScript
- **Build Tool**: Vite 7.0.4
- **Styling**: TailwindCSS v4
- **State Management**: RxJS (reactive streams)
- **Testing**: Vitest

## Acknowledgments

- Inspired by the elegance and simplicity of Brainfuck
- Built with modern web technologies for a smooth development experience
- Special thanks to the Brainfuck community for keeping this esoteric language alive