// First cell is zero cell marker
>

// New experiment. Set up counter to the value corresponding to how far to the right we want to jump
// To jump from the cell 1 to cell 5, we need to jump over three cells.
+++

[
// Move to the next empty value and put 1 there
[>]+

// Move back to 0 cell
[<]

// Move back to the counter and decrement it
>-
]

// Move one cell after the counter
>

// Move towards the end of the trail of ones, and clean the ones
[->]

// We are in the cell 5




-[[>]+[<]>-]>[->]

[[<]+[>]<-]<[-<]



-[[>]+[<]>-]>[->]

>++[-[+>]+[<]>-]>[[-]>]