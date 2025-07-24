// Tape setup
// Let's fill commands data
+ // 1: nop command
>
++ // Operand 1
>
+++ // Operand 2
>
++ // 2: Add command

// So now we have a stack. [nop, 2, 3, add]

<<< // Move pointer to cell 0 for easier goto() with absolute addressing

// Let cell 7 contain total number of commands, we call it PBC — backwards program counter. It is required that BPC has two 0 cells on the left
>>>>>>> // goto(BPC) // TODO: Needed for the compiler
++ // poke(BPC, 2)
<<<<<<< // goto(0)

// Main loop
>>>>>>> // goto(BPC)
[ // Loop until we have no commands

  // Go to the next non-zero command on the left
  << // goto(BPC-2), a second zeroed cell to the left

  // Go to the leftmost non-zero cell
  +[<[>-]>[-<+>]<]<

  // Command executor here
  -[ // This will not execute if the current command was 1, and will in other cases. Essentially our nop operation. The case of if(>1)

   -[ // This will not execute if the current command was 1 or 2. So this is the case of (>2). Not necessary here, but nice to have
       [-]
    ]

    // -- EXECUTING COMMAND 2(Add) --

    // This will execute if > 1, so 2
    // And 2 is our addition operation
    // Addition cell will be a cell on the right to the current command cell. Let's call it "anchor" cell.
    > // goto(anchor)

    // Move to the first operand
    << //goto(anchor - 2)

    [>>+<<-] // Move to the cell on the right of the first operand, increment it, move to operand, decrement it, repeat

    >> //goto(anchor)
    <<< // goto(anchor - 3) — the second operand

    [>>>+<<<-]

    >>> // goto(anchor)

    < // goto(command_cell)

    // -- END OF EXECUTING COMMAND 2(Add) --

    [-]
  ]

  > goto(anchor)
  // It contains the result of the command. So here we can do work based on it. For now let's just zero it.
  //[-]
  // Actually no, let's do one more building block.

  // We know that BPC is at the address 7 now. So if we move 7 cells to the right, it is safe to say we will overshoot BPC

  // Now we know the relative offset needed to move our anchor here
  [->>>>>>>+<<<<<<<] // poke(result_stack_top, peek(anchor))


  // We guaranteed that there are no non-zero cells between commands stack and BPC, so:
  +[>[<-]<[->+<]>]> // Go to BPC
  - // Decrement [5]
]

// So now we can go to the top of the result stack and halt
+[>[<-]<[->+<]>]>