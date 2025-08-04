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
- **File Tree**: Organize your Brainfuck programs
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

# Preview production build
npm run preview
```

## Usage

### Basic Brainfuck
Write standard Brainfuck code:
```brainfuck
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

### Using Macros
Create more readable Brainfuck with macros:
```brainfuck
#define inc(n) {repeat(n, +)}
#define dec(n) {repeat(n, -)}
#define right(n) {repeat(n, >)}
#define left(n) {repeat(n, <)}
#define clear [-]

// Clear current cell
@clear

// Move right and increment by 10
@right(2) @inc(10)

// Complex macro with conditionals
#define safe_dec(n) {if(n, {repeat(n, -)}, )}
@safe_dec(5)  // Decrements by 5
@safe_dec(0)  // Does nothing
```

## Architecture

### Technology Stack
- **Frontend**: React 19.1.0 + TypeScript
- **Build Tool**: Vite 7.0.4
- **Styling**: TailwindCSS v4
- **State Management**: RxJS (reactive streams)
- **Testing**: Vitest

### Project Structure
```
src/
├── components/
│   ├── editor/          # Editor component with vim modes
│   ├── debugger/        # Visual debugger and interpreter
│   └── sidebar/         # File tree and settings
├── services/
│   ├── macro-expander/  # Macro preprocessing system
│   └── tokenizer/       # Syntax highlighting
├── stores/              # RxJS state management
└── experiments/         # Example Brainfuck programs
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development
```bash
# Run tests
npm run test

# Run linter
npm run lint

# Run tests in watch mode
npm run test
```

## Examples

### Hello World with Macros
```brainfuck
#define print_char(c) [-]{repeat(c, +)}.[-]
#define space @print_char(32)
#define newline @print_char(10)

// Print "Hello World!"
@print_char(72)  // H
@print_char(101) // e
@print_char(108) // l
@print_char(108) // l
@print_char(111) // o
@space
@print_char(87)  // W
@print_char(111) // o
@print_char(114) // r
@print_char(108) // l
@print_char(100) // d
@print_char(33)  // !
@newline
```

### Fibonacci Sequence
```brainfuck
#define clear [-]
#define copy_to(n) [-@right(n)+@left(n)+]@right(n)[-@left(n)+@right(n)]@left(n)
#define right(n) {repeat(n, >)}
#define left(n) {repeat(n, <)}

// Initialize first two Fibonacci numbers
+@right(1)+@left(1)

// Generate next 10 Fibonacci numbers
{repeat(10, @copy_to(2)@right(1)@copy_to(2)[-@left(1)+@right(1)]@right(1))}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by the elegance and simplicity of Brainfuck
- Built with modern web technologies for a smooth development experience
- Special thanks to the Brainfuck community for keeping this esoteric language alive