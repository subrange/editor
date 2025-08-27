// Loop examples in Brainfuck

// Clear first three cells
[-]>[-]>[-]<<

+++++ // Set first cell to 5

// Copy value from cell 0 to cell 1
[->+>+<<] // First we move value from cell 0 to cells 1 and 2
>> // Then we move to cell two
[-<<+>>] // And move the value from it back to cell 0
<< // And finally return to cell 0

Execute this example step by step in the debugger to see how the values move through cells

