// The 8 Brainfuck commands - Interactive Tutorial
// Run this step by step to see how each command works!

// Command 1: > (Move pointer right)
// The pointer starts at cell 0. Let's move it to cell 4
>>>>
// Pointer is now at cell 4

// Command 2: < (Move pointer left)  
// Let's move back to cell 1
<<<
// Pointer is now at cell 1

// Command 3: + (Increment cell value)
// Let's add 5 to the current cell
+++++
// Cell 1 now contains 5

// Command 4: - (Decrement cell value)
// Let's subtract 2
--
// Cell 1 now contains 3

// Command 5: . (Output character)
// First let's set cell to a printable ASCII value (65 = 'A')
[-]  // Clear cell first
++++++++[>++++++++<-]>+.  // Set to 65 and print 'A'
<

// Command 6: , (Input character)
// Uncomment the next line to wait for user input
// , 

// Commands 7 & 8: [ and ] (Loop constructs)
// [ means: if current cell is 0, jump forward past matching ]
// ] means: if current cell is not 0, jump back to matching [

// Example: Clear a cell (set to 0)
+++++  // Set cell to 5
[-]    // Loop: decrement until zero
// Cell is now 0

// Example: Move value from cell 0 to cell 1
+++++      // Set cell 0 to 5
[->+<]     // Loop: dec cell 0, inc cell 1, until cell 0 is zero
// Cell 0: 0, Cell 1: 5

// Example: Multiplication (3 * 4 = 12)
>[-]>[-]>[-]<<<  // Clear cells 1,2,3 and return to 0
+++              // Cell 0 = 3
>++++            // Cell 1 = 4  
[                // While cell 1 is not zero
  <[->+>+<<]     // Move cell 0 to cells 1 and 2
  >>[-<<+>>]     // Move cell 2 back to cell 0
  <-             // Decrement cell 1
]
<                // Result is in cell 0 (12)

// Convert to visible ASCII and print (12 + 48 = 60 = '<')
++++++[>++++++++<-]>.  // Add 48 and print