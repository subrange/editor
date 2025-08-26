// Hello World in Brainfuck
// This program prints "Hello World!" and a newline

// Initialize cells with the following loop
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]

// The cells now have these values:
// Cell 0: 0
// Cell 1: 0  
// Cell 2: 72  ('H')
// Cell 3: 104 ('h') 
// Cell 4: 88  ('X')
// Cell 5: 32  (space)
// Cell 6: 8

>>.           // Move to cell 2 and print 'H'
>---.         // Move to cell 3, subtract 3, print 'e' (101)
+++++++..+++. // Add 7, print 'l' twice, add 3 more, print 'o'
>>.           // Move to cell 5, print space
<-.           // Move back to cell 4, subtract 1, print 'W' (87)
<.            // Move to cell 3, print 'o' (111)
+++.          // Add 3, print 'r' (114)
------.       // Subtract 6, print 'l' (108)
--------.     // Subtract 8, print 'd' (100)
>>+.          // Move to cell 5, add 1, print '!' (33)
>++.          // Move to cell 6, add 2, print newline (10)