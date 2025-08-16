# RCT TUI - Test Runner Interface

## Overview
The RCT TUI provides an interactive terminal interface for managing and running tests for the Ripple C Compiler.

## Features

### 1. Test List View
- Browse all available tests with categorization
- See test status indicators (✓ passed, ✗ failed, ⟳ running)
- Category indicators: [C]ore, [A]dvanced, [M]emory, [I]ntegration, [R]untime, [E]xperimental

### 2. Source Code Viewer
- View test C source code with line numbers
- Automatically loads from the test file path

### 3. Generated Files Viewer
- **ASM Tab**: View generated assembly code (`.asm` files from build directory)
- **IR Tab**: View intermediate representation (`.ir` files from build directory)
- Both are generated when tests are run

### 4. Test Execution
- Run individual tests with Enter key
- Run all visible tests with 'r' key
- View real-time output in the Output tab
- Test results are cached and displayed with status indicators

### 5. Debug Integration
- Press 'd' to launch the RVM debugger for the selected test
- Temporarily exits TUI to run the debugger
- Returns to TUI after debugging session

### 6. Categories & Filtering
- Press 'c' to toggle category selection
- Categories include: All, Core, Advanced, Memory, Integration, Runtime, Experimental, Known Failures, Examples
- Press '/' to filter tests by name or description
- Filters work in combination with categories

## Usage

### Launch TUI
```bash
./rct tui
```

### With options:
```bash
# Start with a filter
./rct tui --filter "test_add"

# Start with a category selected
./rct tui --category core
```

## Keyboard Shortcuts

### Navigation
- `j` / `↓` - Move down in test list
- `k` / `↑` - Move up in test list
- `PageDown` - Move down 10 items
- `PageUp` - Move up 10 items
- `Home` - Go to first test
- `End` - Go to last test

### View Controls
- `Tab` - Switch between panes (Test List, Details, Output)
- `1-5` - Switch tabs (Source/ASM/IR/Output/Details)
- `c` - Toggle category selection
- `/` - Enter filter mode
- `Esc` - Clear filter or exit current mode

### Test Execution
- `Enter` - Run selected test
- `d` - Debug selected test (launches RVM debugger)
- `r` - Run all visible tests

### Other
- `?` - Toggle help display
- `q` - Quit TUI

## Test Details View
Shows comprehensive information about the selected test:
- File path
- Whether it uses runtime
- Description (if available)
- Expected output
- Test results (if run)
- Actual output and execution time

## Architecture

The TUI is built with:
- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation

The code is organized into modular components:
- `app.rs` - Application state and data management
- `ui.rs` - UI rendering and layout
- `event.rs` - Event handling system
- `runner.rs` - Test execution and TUI lifecycle

## Implementation Notes

1. **Build Directory Integration**: The TUI reads `.asm` and `.ir` files directly from the build directory after compilation
2. **Real-time Updates**: Test output is captured and displayed in real-time
3. **State Preservation**: Test results are cached during the session
4. **Modular Design**: Each component (test list, code viewer, output) is independently rendered

## Future Enhancements

Potential improvements could include:
- Parallel test execution with progress bars
- Test history and statistics
- Diff view for expected vs actual output
- Syntax highlighting for code views
- Export test results to file
- Watchdog mode for continuous testing