# Welcome to the Braintease IDE

## Your Gateway to Esoteric Programming

Welcome to the most advanced Brainfuck development environment ever created! Whether you're a curious beginner exploring the minimalist beauty of Brainfuck, or an experienced developer pushing the boundaries of what's possible with eight simple commands, this IDE transforms the traditionally challenging experience of Brainfuck programming into something approachable and even enjoyable.

And yes, these tutorials were written by AI — I would absolutely die writing all this by hand. After all, I wrote the custom fucking editor and the whole VM in Brainfuck macro, so I deserve a break, right? However, I will leave some comments here and there. I needed to read all this to make sure it makes sense, so I might as well leave some comments for you, the reader.

## What Makes This IDE Special?

Imagine trying to write a novel using only eight letters of the alphabet. That's essentially what programming in Brainfuck is like. This IDE doesn't change the language itself, but it gives you powerful tools to understand, visualize, and debug your code in ways that were previously impossible.

The IDE brings together three major innovations: a visual debugger that shows you exactly what's happening in memory as your program runs, a sophisticated macro system that lets you write reusable code patterns, and even an assembly language workspace for those ready to dive into low-level virtual machine programming.

## Getting Started: The Editor System

When you first open the IDE, you'll see the main editor – your primary workspace. This isn't just a text editor; it's an intelligent environment that understands Brainfuck at a deep level. As you type, the syntax highlighting helps you distinguish between different types of operations: pointer movements appear in yellow, value modifications in blue, and loops highlight a neighboring `[` or `]` when you place a cursor on them.

But here's where it gets interesting: you actually have access to two editors. The second one, the Macro Editor, is where the magic happens. Instead of writing raw Brainfuck, you can define macros – think of them as custom commands that expand into Brainfuck code. Want to clear a cell? Instead of typing `[-]`, you can define a macro called `clear` and just write `@clear`. It's like having your own personal Brainfuck dialect.

### Your First Program

Let's start with something simple. Copy and paste the following code into the main editor (or the macro editor if you prefer):

```brainfuck
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

This is "Hello World!" in Brainfuck. Intimidating, right? Now click the Run button (the play icon) and watch the magic happen. The program will execute, and you'll see "Hello World!" appear in the output panel. But more importantly, you can watch the memory tape as the program builds up the ASCII values for each letter.

## The Visual Debugger: See Your Code Think

The debugger is where this IDE truly shines. Traditional Brainfuck debugging involves staring at code and mentally tracking pointer positions and cell values. The visual debugger eliminates that cognitive burden entirely.

At the top of the screen, you'll see the memory tape visualization. Each cell shows its current value, and the highlight indicates where your pointer is currently positioned. As your program runs, you can watch the pointer dance across the tape, incrementing and decrementing values, creating patterns that eventually become your output.

### Execution Modes: Choose Your Speed

Not all debugging sessions are the same. Sometimes you want to carefully step through each instruction, and sometimes you just want to see if your program works. That's why IDE offers five different execution modes:

**Step Mode** is perfect when you're learning or debugging a tricky section. Each click advances your program by exactly one instruction, giving you complete control. You can see exactly what each `>`, `<`, `+`, or `-` does to your memory state.

**Smooth Mode** runs your program at a comfortable pace, updating the display frequently enough that you can follow along with the execution. It's like watching a movie of your program running – fast enough to be practical, slow enough to understand.

**Turbo Mode** is for when you know your logic is correct and you just want results. The display updates are minimal, prioritizing execution speed over visualization. Your program runs nearly as fast as native Brainfuck, but you still get the safety net of the IDE's error checking.

**Custom Delay Mode** lets you fine-tune the execution speed. Maybe you want a 50ms delay between operations for a presentation, or perhaps 200ms while you're teaching someone. You're in control.

**Rust WASM Mode** unleashes the full power of WebAssembly. Your Brainfuck code is compiled to near-native speed. This mode can execute millions of operations per second, making even complex Brainfuck programs practical to run. And yes, it runs faster than [great interpreter on copy.sh](https://copy.sh/)

### Breakpoints: Pause Where It Matters

Click any line number in the editor, and you'll see a red dot appear – you've just set a breakpoint. When your program reaches that line during execution, it will pause, allowing you to inspect the current state of memory. This is invaluable when debugging loops or tracking down why a certain cell isn't getting the value you expected.

You can set multiple breakpoints, and the debugger will stop at each one in sequence. Combined with the Step mode, this gives you surgical precision in understanding your program's behavior.

Even more, I specifically added the `$` character to the Brainfuck language, which allows you to set a breakpoint at the current position in the code. This is especially useful when you want to pause execution at a specific point without having to manually set a breakpoint. 

## The Macro System: Write Less, Do More

The macro system transforms Brainfuck from a write-only language into something maintainable and reusable. Open the Macro Editor (it might be hidden by default – look for the button to show it), and let's define our first macro:

```brainfuck
#define clear [-]
#define right >
#define left <
#define inc +
#define dec -
```

Now in your main editor, instead of writing `[-]>+++<`, you can write `@clear @right @inc @inc @inc @left`. It's still Brainfuck under the hood, but now it's readable!

But macros can do so much more. They can take parameters:

```brainfuck
#define move(n) {repeat(n, >)}
#define add(n) {repeat(n, +)}

@move(5)  // Moves right 5 cells
@add(10)  // Adds 10 to current cell
```

They can even use control structures:

```brainfuck
#define set_ascii(char) {
    [-]  // Clear current cell
    {repeat(char, +)}  // Add the ASCII value
}

@set_ascii(65)  // Sets cell to 'A'
.  // Prints 'A'
```

### Auto-Expansion: Real-Time Feedback

Enable Auto-Expand in the settings, and watch as your macros are instantly converted to Brainfuck as you type. This immediate feedback helps you catch errors early and understand exactly what your macros are doing. If there's an error in your macro definition, you'll see it highlighted in red with a helpful error message.

## Memory Visualization: Three Ways to See Your Data

The debugger offers three different ways to visualize your memory tape, each suited to different types of programs:

**Normal View** is the standard cell-by-cell display. Each cell shows its decimal value and, when applicable, its ASCII character representation. This is perfect for most Brainfuck programs, especially those working with text.

**Compact View** shrinks the display to fit more cells on screen. When you're working with programs that use hundreds or thousands of cells, this bird's-eye view helps you see patterns in your data that would be invisible in normal view.

**Lane View** is something special. It arranges your memory tape into multiple columns, like a spreadsheet. If you're implementing a matrix multiplication algorithm or working with 2D data structures, lane view makes it obvious what's happening. Set it to 8 lanes, and suddenly you can develop a RISC-like virtual machine right in Brainfuck. Yep, this is how I did it.

## Snapshots: Save Your Progress

Ever wish you could save the exact state of your program's memory and return to it later? That's what snapshots do. Click the camera icon in the sidebar to open the Snapshots panel. At any point during execution (or even when stopped), you can save a snapshot with a custom name.

These snapshots capture everything: the entire memory tape, the pointer position, even any labels you've added to cells. It's like having save states in a video game. Made a mistake? Load your snapshot and try again. Want to compare two different approaches? Save a snapshot of each and switch between them instantly.

## The Output Panel: More Than Just Text

The output panel might seem simple – it's where your program's output appears – but it has some clever features. You can position it at the bottom of the screen (traditional), on the right side (still traditional, but on the right side).

When you're working with the Assembly workspace, the output panel gains additional tabs. You can see the VM Output (system-level messages) and even a Disassembly view that shows the actual machine instructions being executed. It's like having X-ray vision for your code. (Comment from the author: I just can't handle these AI metaphors. I mean, "X-ray vision"? Really? I will leave it untouched though, so you could also suffer.)

## Settings: Make It Yours

Click the gear icon to access settings, where you can customize almost everything about how the IDE behaves. Here are some key settings to know about:

**Strip Comments** controls whether comments are preserved when macros are expanded. Keep them while learning, remove them for cleaner output. I recommend always keeping it on — I swear you will accidentally put a valid Brainfuck command in a comment, and then wonder why your program behaves unexpectedly.

**Auto Expand** toggles real-time macro expansion. This is usually helpful, but for very large macro files, you might want to disable it for better performance.

**Use WASM Expander** switches between JavaScript and WebAssembly for macro processing. WASM is faster, better, and newer, so just ignore the JavaScript option unless you have a specific reason to use it.

**Cell Size** determines whether each memory cell holds 8-bit (0-255), 16-bit (0-65535), or 32-bit values. Most Brainfuck programs expect 8-bit, but larger cells can be useful for mathematical computations.

And a bunch of other settings — feel free to explore. Some of them may even completely break stuff!

## The Learning Panel: Your Guide

Click the graduation cap icon to open the Learning panel. Here you'll find interactive tutorials that demonstrate different IDE features. Each tutorial is crafted to highlight specific capabilities:

These aren't just static tutorials – they load actual code into the editor and configure the IDE to best demonstrate each concept.

## Keyboard Shortcuts: Work Faster

While the IDE is fully functional with just mouse clicks, keyboard shortcuts can dramatically speed up your workflow:

- **Cmd/Ctrl+F**: Open search in the current editor
- **Cmd/Ctrl+P**: Quick navigation to jump to macros or marked sections

## Advanced Features

### The Assembly Workspace

For those ready to go beyond Brainfuck, the IDE includes a complete assembly language environment for the Ripple Virtual Machine. Enable it in settings to access a RISC-like assembly language with 32 registers, labels, and a proper instruction set. You can write assembly code, compile it to Brainfuck, and even run it directly in the IDE.

## Tips for Success

**Start Simple**: Begin with basic Brainfuck programs before diving into macros. Understanding the fundamentals makes everything else easier.

**Use Comments Liberally**: Brainfuck is notoriously hard to read. Comments are your future self's best friend.

**Label Your Cells**: In the debugger panel, you can assign names to specific memory cells, or lanes/columns in the lane mode. Instead of remembering that cell 7 is your loop counter, label it "counter" and never forget again. You can right click on a cell to set its label, or use the settings panel — it is somewhere there.

**Save Snapshots Often**: Before attempting something complex, save a snapshot. It's much easier than trying to manually restore a specific memory state.

**Experiment with View Modes**: Different programs benefit from different visualizations. A text manipulation program might work best in normal view, while a sorting algorithm might be clearer in lane view.

**Use Mark comments**: The IDE supports marking specific lines with comments. This is useful for leaving notes about what a section of code does, or why you made a particular design choice. Use `// MARK: Your comment here` to create a marked section. You can then quickly navigate to these marks using the quick nav feature (Cmd/Ctrl+P), or use the "Marks" panel to see all your marked sections in one place.

## Common Patterns and Solutions

**Infinite Loops**: If your program seems stuck, pause it and check the current cell value. Often, a loop condition isn't being decremented properly.

**Off-by-One Errors**: Use the step debugger to verify your pointer movements. It's easy to move one cell too far or not far enough.

**Value Overflow**: With 8-bit cells, incrementing 255 gives you 0. This wrapping behavior can cause unexpected results if you're not careful.

## Your Journey Starts Here

This IDE represents A WHOLE MONTH of pain, suffering, real coding, vibe coding, brainfuck coding, meta programming, assembly coding, Rust coding, fucking C compiler coding, and other coding aimed at making Brainfuck not just possible to write, but genuinely enjoyable to work with. Whether you're here to challenge yourself, to learn about low-level programming concepts, or just to have fun with an esoteric language, you have all the tools you need.

Start with the tutorials in the Learning panel, experiment with the different execution modes, and don't be afraid to use the macro system to make your life easier. Brainfuck might be minimal by design, but your development experience doesn't have to be.

Happy coding, and remember: in Brainfuck, every character counts, but with this IDE, you can make them all count for something meaningful.

### Note from the author

Holy hell that was a journey. And I just wanted to make a snake game in Brainfuck — and somehow accidentally reinvented the whole computer science.

I hope you have some fun. After all, Brainfuck is Turing-complete.