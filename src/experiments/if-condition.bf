//+ // Start on cell that has a one in it. This cell is our condition — 0 for false, non-zero for true
[
  // If current cell is nonzero, then

  // Do some stuff on the right and return to the current cell
  // Let's just move right, increment something, and get back
  >>> // Move right 3 cells
  + // Increment it
  <<< // Mandatory move back to the cell containing the condition

  [-] // Zero it out, so we can exit the loop
]

>>> // Move right three cells again to rest on the cell we incremented

// So basically we made
// if (currentCell)
//   doWork()
//
// move(+3)