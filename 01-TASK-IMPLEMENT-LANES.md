Technical Task: Multi-Lane Visualization for Brainfuck IDE
Overview
Implement a visual lane system for the Brainfuck debugger that allows users to view the tape as multiple interleaved lanes, with color-coding to distinguish between lanes.
Requirements
1. Settings UI

Add a new setting called "Lane Count" or "Tape Lanes" in the Settings panel
Implement a numeric input or slider control with range 1-10
Default value: 1 (current single-lane mode)
Position: Add to the INTERPRETER section below "Cell Size"

2. Visual Behavior
   When lanes > 1:

Color-code tape cells based on their lane assignment
Cell at index i belongs to lane (i % laneCount)
Each lane gets a distinct color from a predefined palette
Colors should be visually distinct but maintain readability with the existing dark theme

Example coloring for 5 lanes:

Lane 0 (cells 0, 5, 10...): Color A
Lane 1 (cells 1, 6, 11...): Color B
Lane 2 (cells 2, 7, 12...): Color C
Lane 3 (cells 3, 8, 13...): Color D
Lane 4 (cells 4, 9, 14...): Color E

3. Color Palette

Provide a set of 10 visually distinct colors that work well with the dark theme, using TailwindCSS color utilities
Colors should not interfere with existing highlights (current cell indicator, breakpoints)
Maintain contrast for readability of cell values

4. Persistence

Store the lane count setting in localStorage
Key: brainfuck-ide-lane-count or similar
Load and apply the setting on page refresh
Handle invalid/missing localStorage values gracefully

5. Implementation Notes

This is purely a visual feature - does not affect interpreter behavior
The tape still functions as a single contiguous array
Navigation commands (<, >) still move one cell at a time
Only the visual representation changes

6. Backward Compatibility

When lane count = 1, the tape should appear exactly as it does currently
No visual changes or performance impact in single-lane mode

Acceptance Criteria

Lane count setting is visible and functional in the Settings panel
Changing lane count immediately updates the tape visualization
Each lane has a distinct, readable color
Setting persists across page reloads
No impact on interpreter execution or performance
Works correctly with all existing features (breakpoints, current cell highlight, etc.)