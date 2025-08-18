# Tree Navigation Feature - Testing Guide

## Overview
The trace viewer tabs (AST, TypedAST) now support interactive tree navigation with expand/collapse functionality.

## New Keyboard Shortcuts

### When focused on AST (tab 6) or TypedAST (tab 8) tabs:

#### Basic Navigation:
- **j / ↓**: Move to next node in the tree
- **k / ↑**: Move to previous node in the tree

#### Expand/Collapse:
- **Enter / Space**: Toggle expand/collapse on current node
- **h / ←**: Collapse current node
- **l / →**: Expand current node
- **H**: Collapse ALL nodes in the tree
- **L**: Expand ALL nodes in the tree

## Visual Indicators:
- **▼**: Expanded node with children (yellow, or cyan when selected)
- **▶**: Collapsed node with children (yellow, or cyan when selected)
- **Gray background**: Currently selected node
- **Colors**:
  - Cyan: AST node types
  - Blue: Properties
  - Green: Values
  - Magenta: Type information (in TypedAST)

## How to Test:

1. Launch the TUI test runner:
   ```bash
   ./rct
   ```

2. Select a test that has been compiled with trace (e.g., `test_add`)

3. Press **Tab** to focus on the right panel

4. Press **6** to view the AST tab

5. Try the navigation:
   - Use **j/k** to move through nodes
   - Press **Enter** to expand/collapse nodes
   - Press **H** to collapse all, then selectively expand with **l**
   - Press **L** to expand all nodes

6. Press **8** to switch to TypedAST tab and try the same navigation

## Implementation Details:

### Changed Files:
1. **rcc-test/src/tui/ui/trace_viewer.rs**:
   - Modified TreeNode to track expansion state per path
   - Updated flatten() to respect expanded_nodes set
   - Added visual indicators for expand/collapse state

2. **rcc-test/src/tui/app.rs**:
   - Added HashSet for tracking expanded nodes
   - Added tree navigation helper methods
   - Methods for expand/collapse operations

3. **rcc-test/src/tui/runner.rs**:
   - Added keyboard handlers for tree navigation
   - Integrated tree-specific controls when in AST/TypedAST tabs

## Benefits:
- **Better navigation**: Navigate complex AST structures efficiently
- **Focused view**: Collapse irrelevant sections to focus on specific parts
- **Smart defaults**: Trees start with reasonable expand/collapse defaults
- **Persistent state**: Expansion state maintained while navigating

## Example Use Cases:
1. **Debugging type issues**: Collapse all nodes, then expand only the type-related sections
2. **Finding specific functions**: Navigate directly to function definitions
3. **Comparing nodes**: Keep multiple sections expanded for comparison
4. **Large file analysis**: Start collapsed, expand only areas of interest