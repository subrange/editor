# The Editor System: Your Creative Workspace

## Where Code Comes to Life

Welcome to the heart of the IDE – the editor system. If you've ever used VS Code, Sublime, or any modern editor, you'll feel right at home. If you haven't, don't worry – I'll walk you through everything. This isn't just a text editor; it's a sophisticated development environment that understands Brainfuck at a fundamental level and helps you write better code faster.

## The Three Editors: Each with a Purpose

The IDE actually gives you three different editors, each specialized for its own task. Think of them like different tools in a workshop – you wouldn't use a hammer to paint, and you wouldn't use a paintbrush to drive nails.

### The Main Editor: Where the Magic Happens

This is your primary workspace, where Brainfuck code lives and breathes. Open the IDE and you'll see it front and center, ready for action. Type a simple `+` and watch it light up – the editor immediately recognizes it as a value modification command. Type `>` and it turns orange, showing pointer movement. Already, the editor is working with you, not just for you.

But here's what makes it special: this editor is deeply integrated with the debugger. Set a breakpoint by clicking a line number, and a red dot appears. Run your program, and watch as a green rectangle follows your execution, symbol by symbol. It's like having a reading guide that shows you exactly where the computer is in your code.

### The Macro Editor: Your Power Tool

Hidden by default (look for the "Show Macro Editor" button), the Macro Editor is where you escape the constraints of raw Brainfuck. This is where you define your own language on top of Brainfuck.

Let me show you what I mean. Copy this code into the Macro Editor:

```brainfuck
#define clear [-]
#define inc(n) {repeat(n, +)}
#define set(n) @clear @inc(n)

#define HELLO {'H', 'e', 'l', 'l', 'o'}

#define hello {
  {for(c in #HELLO, @set(c) .)}
}

@hello // invoke the macro
```

Neat, right?


### The Assembly Editor: For the Adventurous

When you enable the Assembly workspace in settings, a third editor appears. This one speaks a different language entirely – Ripple VM assembly. If you're curious about how computers really work at the lowest level, this editor lets you write code that's just one step above machine code.

It has its own syntax highlighting for instructions like `LI` (load immediate), `ADD`, `JMP` (jump), and more. It understands labels, so you can write `loop:` and later jump to it. It even has a data section where you can define strings and constants. This isn't just a Brainfuck IDE anymore – it's a complete low-level development environment. Still compiles to Brainfuck.

## Search: Finding Your Way

Press Cmd/Ctrl+F and watch the search bar slide down from the top of the editor. This isn't your grandmother's search function. Start typing and matches light up instantly across your code. The current match glows bright yellow while others are dimmed, and a counter shows "2 of 15 matches" so you always know where you are.

But here's where it gets powerful: click the regex button and now you can search with regular expressions. Want to find all loops that start with a plus? Search for `\[\+`. Want to find all places where you move right and then increment? Search for `>\+`. 

The search is also smart about context. It knows about comments and can optionally ignore them. It can do whole-word matching, so searching for "add" won't match "address". And it remembers your last search, so pressing F3 instantly repeats it.

## Jump to definition and find references: The Power of Macros

Press Cmd/Ctrl+click on any macro name, and the editor jumps to its definition in the Macro Editor. This is incredibly useful when you're deep in a complex macro and need to see how it works.

Press it on a macro definition, and it shows you all places where that macro is invoked in your code. This is like having a superpower – you can see how your macros are used without manually searching for them.

## Refactoring: Changing Code with Confidence

Shift+click on any macro name, and safely rename every occurrence in your code. No more manual find-and-replace that breaks things. The editor understands the context of your macros and updates them everywhere they appear.

## Quick Navigation: Teleportation for Your Code

Press Cmd/Ctrl+P and something magical happens – the Quick Navigation panel appears. This is your teleporter, your way to instantly jump anywhere in your code.

Start typing "set" and it shows you all macros with "set" in the name. Click on `set_value` and boom – you're there. But it's not just for macros. Add special comments like `// MARK: Main Loop` and they appear in Quick Nav too. Now you can organize your code into logical sections and jump between them instantly.

In a 500-line program, this is the difference between scrolling for 30 seconds and jumping instantly. It's the difference between losing your train of thought and maintaining your flow. Once you start using Quick Nav, you'll wonder how you ever lived without it.

## The Minimap: Your Code from 30,000 Feet

Look to the right side of the editor (if it's enabled) and you'll see a tiny version of your entire file – the minimap. This isn't just a scaled-down copy; it's a navigation tool. The highlighted rectangle shows what's currently visible in your main editor. Click anywhere in the minimap to jump there instantly. Drag the rectangle to scroll smoothly through your code.

The minimap is especially useful for long programs. You can see the overall structure – dense sections of code, sparse sections with lots of comments, loops nested within loops. It's like having a map of your program's geography. Pattern recognition kicks in, and you start navigating by the shape of your code rather than reading it.

## Bracket Matching: Never Lose Your Place

In Brainfuck, brackets are everything. One mismatched bracket and your program is broken. The editor has your back. Put your cursor on any `[` and its matching `]` lights up. Put it on a `]` and its `[` appears. If a bracket doesn't have a match, it does not highlight.

## Auto-Expansion: Real-Time Macro Magic

When you're working with macros, enable Auto-Expand and watch the magic happen. Every time you save or modify the Macro Editor, your macros are instantly expanded in the Main Editor. It's like having a compiler that runs constantly in the background.

Write a macro, see the Brainfuck. Fix an error, see the correction immediately. This tight feedback loop makes macro development incredibly fast. You're not guessing what your macro will produce – you're seeing it in real-time.

If there's an error, it appears as a red banner at the top of the editor: "Undefined macro 'test' on line 5". Click the error and it jumps to line 5. Fix it, and the error disappears. It's like having a helpful assistant constantly checking your work.

## Split View: See Everything at Once

The Macro Editor and Main Editor can be visible simultaneously, side by side. Write a macro on the left, see its expansion on the right. Adjust the divider to give more space to whichever you're focusing on. Double-click the divider to reset to 50/50.

This split view is especially powerful when debugging macros. You can see exactly which macro code corresponds to which Brainfuck output. When an error occurs in the expanded code, you can trace it back to the exact macro that generated it.

## Tips from a Power User

After hours of using this editor, here are the techniques that make the biggest difference:

**Use MARK Comments Liberally**: Sprinkle `// MARK: Section Name` comments throughout your code. They become waypoints in Quick Nav, making navigation instant.

**Trust Auto-Expand**: When writing macros, let Auto-Expand run. The immediate feedback catches errors before they become problems.
