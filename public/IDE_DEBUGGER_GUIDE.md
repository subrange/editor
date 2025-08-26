# Mastering the Debugger & Execution Environment

## Welcome to Visual Debugging

Remember the last time you tried to debug a Brainfuck program? Staring at a wall of brackets and symbols, mentally tracking which cell you're on, what value it holds, and wondering why your output is completely wrong? Those days are over.

This debugger transforms Brainfuck from a black box into a glass box. You can see everything: every cell, every pointer movement, every value change. It's like having X-ray vision for your code. Let me show you how to use these superpowers.

## The Art of Execution: Five Ways to Run Your Code

Think of the execution modes like gears in a car. Sometimes you need first gear to carefully navigate a tricky section, and sometimes you want fifth gear to cruise down the highway. The IDE gives you five distinct ways to run your programs, each optimized for different scenarios.

### Step Mode: The Microscope

When you're learning Brainfuck or debugging a particularly nasty bug, Step Mode is your best friend. Each press of the step button executes exactly one instruction. You see the pointer move. You see the value change. You understand exactly what `[->+<]` does because you watch it happen, step by step.

This is how I recommend everyone starts with Brainfuck. Write a simple program, maybe just `+++.`, and step through it. Watch cell 0 go from 0 to 1 to 2 to 3. Watch the dot command output the ASCII character for 3 (which is a non-printable character, but you get the idea). This builds intuition that's impossible to get any other way.

### Smooth Mode: The Balanced Approach

Once you understand the basics, Smooth Mode becomes your daily driver. It runs your program at a comfortable pace – fast enough to be practical, slow enough to follow along. The tape updates smoothly, output appears in real-time, and all your breakpoints are respected.

Think of it like watching a movie of your program executing. You can see the pointer sliding across the tape, values incrementing and decrementing, loops spinning. If something goes wrong, you'll see it happen. This mode strikes the perfect balance between speed and visibility.

### Turbo Mode: When You Need Speed

You've debugged your program, you know it works, and now you just need it to run. Turbo Mode throws visualization out the window in favor of raw speed. The display updates infrequently, maybe every thousand operations or so. Your program runs almost as fast as native Brainfuck.

This is perfect for those computation-heavy programs – calculating Fibonacci numbers, generating fractals, or running any algorithm that involves millions of operations. You still get the safety of the IDE (infinite loop detection, memory bounds checking), but without the performance penalty of constant visualization.

### Custom Delay Mode: You're in Control

Sometimes you need something specific. Maybe you're teaching Brainfuck to a class and want a 200ms delay so students can follow along. Maybe you're creating a video and need consistent timing. Or maybe you're debugging a specific section and want it to run at exactly the speed that helps you understand it.

Custom Delay Mode puts you in the driver's seat. Set any delay from 0 to 1000 milliseconds. The setting persists between sessions, so once you find your sweet spot, it's always there waiting for you.

### Rust WASM Mode: Maximum Overdrive

This is where things get serious. Your Brainfuck code is compiled to WebAssembly and runs at near-native speed. We're talking millions of operations per second. That Mandelbrot set generator that takes minutes in regular mode? It finishes in seconds.

The trade-off is that you lose debugging features – no stepping, no breakpoints, no tape visualization during execution. But when you need speed, when you need to run that complex algorithm or process large amounts of data, this mode delivers performance that rivals compiled C code.

## The Memory Tape: Three Ways to See Your Data

The memory tape is the heart of Brainfuck – it's where all your data lives. But not all data is the same. Sometimes you're working with strings, sometimes with numbers, sometimes with complex data structures. That's why we offer three different visualization modes.

### Normal View: The Classic

This is your standard, everyday view. Each cell is displayed with its decimal value, and when that value represents a printable ASCII character, you see that too. Cell 65 shows "65 (A)", cell 10 shows "10 (\n)". The current pointer position is highlighted in blue, making it impossible to lose track of where you are.

This view is perfect for most Brainfuck programming. It gives you all the information you need without overwhelming you. You can see enough cells to understand context, but each cell is large enough to read comfortably.

### Compact View: The Overview

When your program uses hundreds or thousands of cells, Normal View becomes impractical. You'd spend all your time scrolling. Compact View solves this by shrinking the display, fitting many more cells on screen at once.

Think of it like zooming out on a map. You lose some detail – maybe you can't see the ASCII representations – but you gain perspective. You can see patterns in your data, identify which regions of memory are being used, spot that one cell that's different from all the others.

### Lane View: The Game Changer

This is where the debugger gets really innovative. Lane View arranges your memory tape into multiple columns, transforming your linear tape into a 2D grid. Set it to 8 lanes, and suddenly cells 0-7 are the first row, 8-15 are the second row, and so on.

Why is this revolutionary? Because suddenly, complex data structures become visible. That sorting algorithm you're implementing? Watch the values bubble up through the grid. Working with a virtual 2D array? See it as an actual 2D array. Processing image data? View it in a format that actually makes sense.

You can even label the lanes. Working with a record structure where every 4 cells represent a person (age, height, weight, ID)? Label the lanes accordingly and never get confused about which cell holds what.

## Navigation: Getting Around Your Memory

With potentially thousands of cells to work with, navigation becomes crucial. We provide several ways to jump around the tape quickly and precisely.

### The Basics

**Go to Start** instantly jumps to cell 0. Lost in the wilderness of cell 2847? One click brings you home.

**Go to Pointer** centers the view on wherever your pointer currently is. This is incredibly useful during debugging – your pointer might be at cell 500 while you're looking at cell 100.

**Go to End** jumps to the highest cell that's been modified. This helps you understand your program's memory footprint.

### Go to Cell: The Power Tool

But here's where it gets interesting. Click "Go to Cell" and you can type not just a cell number, but a mathematical expression. Need to check cell 256? Type `256`. But what if you want to check the cell at the end of a 16x16 grid? Type `16 * 16 - 1`. Want to see what's 10 cells after the middle of a 100-cell buffer? Type `50 + 10`.

This feature understands that when you're debugging, you're often thinking in terms of calculations and offsets. Instead of doing mental math and typing `127`, you can type `128 - 1` and keep your mental context intact.

## Breakpoints: Pause Where It Matters

Breakpoints are like bookmarks for debugging. They tell the debugger "when you get here, stop and let me look around." They're essential for understanding program flow and catching bugs.

### Setting Breakpoints

The easiest way is to click on any line number in the editor. A red dot appears – you've just set a breakpoint. When your program reaches that line, execution pauses. You can set as many as you want, creating a debugging journey through your code.

But breakpoints can be smarter than just "stop here." You can create conditional breakpoints that only trigger when certain conditions are met. Debugging a loop that runs 1000 times but only fails on iteration 999? Set a conditional breakpoint that only triggers when a counter cell equals 999.

Or you can just use the `$` symbol anywhere in the code — IDE will automatically break there. This is great for quick debugging without cluttering your code with manual breakpoints.

## The Output Panel: More Than Just Text

The output panel might seem simple – it's where your program's output goes – but it's surprisingly sophisticated.

### Beyond Simple Output

When you enable the Assembly workspace, the output panel becomes even more powerful. You get additional tabs:

**VM Output** can be set up to show the memory-mapped output of the VM.

**Disassembly View** shows the actual machine instructions being executed. Yep, it disassembles Brainfuck tape.

## Tips from the Trenches

After years of debugging Brainfuck, here are my hard-won insights:

**Start with Step Mode**: I don't care how confident you are. Step through your program at least once. You'll be surprised what you learn.

**Label Everything**: Use the Marks feature liberally. Every important cell should have a name. Your future self will thank you.

**Save Snapshots Before Experiments**: About to try something risky? Snapshot first. It's like quicksave in a video game.

**Use Lane View for Algorithms**: Implementing quicksort? Use lane view. Working with matrices? Lane view. Any time you have structured data, lane view makes it visible.

**Profile Before Optimizing**: Don't guess what's slow. Run the profiler and know what's slow. Optimize the right thing.

## Your Debugging Journey

The debugger transforms Brainfuck from an exercise in frustration into a genuinely enjoyable puzzle. Every bug becomes a solvable mystery. Every optimization opportunity becomes visible. Every program becomes understandable.

Start with simple programs and use Step Mode to build intuition. Graduate to Smooth Mode for general development. Master breakpoints for efficient debugging. Explore Lane View for complex data structures. Push the limits with Turbo and WASM modes.

Remember: in traditional Brainfuck, you're blindfolded in a dark room. With this debugger, you have night vision goggles and a map. Use them wisely, and there's no bug you can't squash, no program you can't understand, no algorithm you can't implement.

Happy debugging, and may your pointers always point where you expect them to!