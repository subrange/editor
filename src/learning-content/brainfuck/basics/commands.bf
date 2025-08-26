// The 8 Brainfuck commands: Interactive Tutorial
// Run this step by step to see how each command works!

// Command 1: Move pointer right
// The pointer starts at cell 0 Let's move it to cell 4
>>>>
// Pointer is now at cell 4

// Command 2: Move pointer left
// Let's move back to cell 1
<<<<
// Pointer is now at cell 1

// Command 3: Increment cell value
// Let's add 5 to the current cell
+++++
// Cell 1 now contains 5

// Command 4: Decrement cell value
// Let's subtract 2
--
// Cell 1 now contains 3

// Command 5: Output character
// First let's set cell to a printable ASCII value (65 = 'A')
[-]  // Clear cell first
++++++++[>++++++++<-]>+.  // Set to 65 and print 'A'
<

// Command 6: Input character
// This command does not work in Braintease IDE because reasons
// ,

// Commands 7 & 8: Loop constructs
// Open square bracket means: if current cell is 0 then jump forward past matching closing square bracket
// Closing square bracket means: if current cell is not 0 then jump back to matching openind square bracket

// Example: Clear a cell (set to 0)
+++++  // Set cell to 5
[-]    // Loop: decrement until zero
// Cell is now 0

>[-]< // Zero out the second cell as well

// Example: Move value from cell 0 to cell 1
+++++      // Set cell 0 to 5
[->+<]     // Loop: dec cell 0 then inc cell 1 until cell 0 is zero
// Cell 0: 0 Cell 1: 5
