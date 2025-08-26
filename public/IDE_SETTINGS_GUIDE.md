# Settings & Configuration: Making the IDE Yours

Comment from the author: with every new tutorial this AI seems to get more and more ridiculous. Nerve center? Control Room? World of fucking customization?

I will probably never rewrite it though.

## The Control Room

Welcome to the nerve center of your IDE – the Settings panel. Click that gear icon in the sidebar and you enter a world of customization that can transform your Brainfuck development experience. Whether you're a speed demon who needs maximum performance or a careful learner who wants to see every step, these settings let you tune the IDE to match exactly how you work.

Every setting you change is automatically saved in your browser's localStorage. Close the IDE, come back a week later, and everything is exactly as you left it. It's like having a personal workspace that remembers you.

## The Macro Settings: Your Code Generation Engine

The macro system is the heart of advanced Brainfuck programming, and these settings control how it behaves. Let's explore what each one does and, more importantly, when you should care.

### Strip Comments: The Clean-Up Crew

When your macros expand into Brainfuck, what happens to all those helpful comments you wrote? This setting decides their fate.

**On by default**, and here's why: When you're actually running Brainfuck code, comments are dead weight. They make your expanded code longer, your files bigger, and scrolling more tedious. With this on, your beautiful commented macro code becomes lean, mean Brainfuck.

But sometimes you want those comments. When you're learning how macros expand, or debugging why something went wrong, those comments are breadcrumbs through the forest. Turn this off and every comment survives the expansion, helping you understand the journey from macro to Brainfuck.

My advice? Keep it on unless you're actively debugging macro expansion. Your scrollbar will thank you.

### Collapse Empty Lines: The Space Saver

Macros often generate code with lots of empty lines – they're artifacts of the expansion process, like the leftover flour on a baker's counter. This setting is your choice: keep the mess for readability, or clean it up for compactness.

**Off by default** because those empty lines often serve a purpose. They separate logical sections, make the code breathable, give your eyes a rest. When you're reading expanded code, these gaps are like paragraph breaks in a book.

But when you're looking at the final product, when you want to see as much code as possible on screen, when you're sharing code with others – flip this on. Your 500-line expanded program might shrink to 300 lines of pure, dense Brainfuck.

### Auto Expand: The Real-Time Compiler

This is the magic setting. With Auto Expand on, every keystroke in the Macro Editor triggers an expansion. You see your Brainfuck being generated in real-time, like watching a 3D printer build your design layer by layer.

**On by default** because the feedback is invaluable. Type a macro invocation, see the Brainfuck instantly. Make a typo, see the error immediately. It's like having a compiler that never stops running, catching mistakes before they become mysteries.

But power comes at a price. If you're working with huge macro files – we're talking thousands of lines – that constant recompilation can make the editor sluggish. Every keystroke triggers a full expansion, and suddenly you're waiting a half-second for each character to appear. That's when you turn this off and expand manually when you're ready.

### Use WASM Expander: The Speed Boost

JavaScript is great, but WebAssembly is faster. Like, 5 to 10 times faster. This setting chooses which engine powers your macro expansion.

**On by default** because who doesn't want free performance? The WASM expander tears through even complex macro expansions in milliseconds. It's the difference between waiting and not waiting, between smooth and stuttering.

So why would you ever turn it off? Debugging. When something goes wrong with macro expansion itself – not your macros, but the expander – the JavaScript version is much easier to debug. You can set breakpoints, inspect variables, understand what's happening. It's like the difference between a race car and a regular car: the race car is faster, but when it breaks, good luck fixing it without specialized tools.

For 99.9% of users, leave this on. For that 0.1% debugging the actual expander, you know who you are.

## Debugger Settings: Your Window into the Machine

The debugger is where you watch your code come alive, and these settings control what you see and how you see it.

### View Mode: Three Ways to Watch

Your memory tape can be visualized in three completely different ways. It's like having three different microscopes for examining your data.

**Normal View** is the goldilocks option – not too much, not too little, just right. Each cell displays its value clearly, shows ASCII characters when relevant, and gives you enough context without overwhelming you. This is where most people live.

**Compact View** is for when you need the big picture. Working with a thousand cells? Normal view would have you scrolling forever. Compact view shrinks everything down, trading detail for overview. You can see patterns, identify regions of activity, spot the one cell that's different. It's like zooming out on a map to see the whole country instead of your street.

**Lane View** is the secret weapon. Instead of a long line of cells, arrange them in columns. Suddenly, your linear tape becomes a 2D grid. Working with matrix math? Use lane view. Implementing a virtual machine with 8-byte words? Set 8 lanes and watch your data align perfectly. This is the view that makes complex data structures visible.

### Show Disassembly: For the Brave

When you enable the Assembly workspace, you unlock the ability to see actual machine instructions. This setting controls whether you want that power.

**Off by default** because most Brainfuck programmers don't need to see VM instructions. It's an extra layer of complexity, another panel taking up space, more information to process.

But if you're debugging at the VM level, if you're trying to understand how your Brainfuck becomes machine code, if you're one of those people who reads assembly for fun (you know who you are), turn this on. The disassembly view shows you exactly what instructions are being executed, what registers hold what values, how the virtual machine is interpreting your code.

### Lane Count: Your Grid Dimension

When you're in Lane View, this decides how many columns you see. It's like choosing between ruled paper, graph paper, or engineering paper.

**Default is 4** because it's a nice, manageable number. Four columns fit well on most screens, make patterns visible without being overwhelming.

But the right number depends on your data. Working with 16-bit values split across two cells? Use 2 lanes. Implementing an 8x8 game board? Use 8 lanes. Processing RGB values? Use 3 lanes. The limit is 32, which is probably more than your screen can handle, but it's there if you need it.

## Assembly Settings: The Power User Zone

These settings only matter if you've enabled the Assembly workspace. If you're just doing Brainfuck, skip this section. But if you're ready to go deeper...

### Show Workspace: The Gateway Drug

This is the master switch. Turn it on and suddenly your IDE grows new powers. An Assembly tab appears, new toolbar buttons materialize, the Output panel gains new abilities. It's like finding a hidden room in a house you thought you knew.

**Off by default** because assembly programming isn't why most people come to a Brainfuck IDE. It adds complexity, uses more memory, might confuse beginners.

But if you're curious about low-level programming, if you want to see how a virtual machine works, if you're implementing a compiler that targets the Ripple VM – flip this switch and enter a new world.

### Auto Compile: The Eager Assistant

When you're writing assembly, this setting controls whether it compiles automatically as you type.

**Off by default** because compilation takes time and CPU. Every change triggers a full assembly, link, and validation pass. For small programs it's instant, but as your code grows, those milliseconds add up.

Turn it on when you're actively developing and want immediate feedback. Every syntax error appears instantly, every successful compile gives you that little dopamine hit. Turn it off when you're doing major refactoring and don't want the distraction of constant compilation.

### Bank Size: Your Memory Architecture

The Ripple VM divides memory into banks. This setting controls how big those banks are. It's deeply technical and honestly, most people should leave it alone.

**Default is 64000** because it's large enough for serious programs but small enough to be manageable. Think of it like choosing the page size in a notebook – too small and you're constantly flipping pages, too large and you waste paper.

If you're implementing something specific that needs different banking, you already know what value you need. If you're not sure, stick with the default.

## Interpreter Settings: The Brainfuck Brain

These control how the Brainfuck interpreter itself behaves. They affect compatibility, safety, and what kinds of programs you can run.

### Wrap Cells: The Overflow Policy

What happens when you increment 255 in an 8-bit cell? This setting decides.

**On by default** because that's standard Brainfuck behavior. 255 + 1 = 0, like a car odometer rolling over. It's predictable, it's what most programs expect, it's mathematically clean (modulo arithmetic).

Turn it off and you enter the wild west. 255 + 1 = 256, cells can hold negative numbers, arithmetic works differently. Some algorithms break, others become possible. It's useful for certain kinds of debugging, but most programs will misbehave.

### Wrap Tape: The Edge of the World

What happens when you move past the last cell of the tape? Do you fall off the edge or wrap around to the beginning?

**On by default** for safety. When you hit the edge, you wrap around. It's like Pac-Man going off one side of the screen and appearing on the other. No crashes, no errors, just seamless continuation.

Turn it off for debugging. Now when you move past the edge, the program stops with an error. This catches pointer bugs that wrapping would hide. If your pointer is at cell 29999 and you move right, wrapping would put you at cell 0 silently. With wrapping off, you get an error that says "hey, you just tried to leave the tape!"

### Cell Size: How Big is a Byte?

Most Brainfuck uses 8-bit cells (0-255), but the IDE supports 16-bit (0-65535) and 32-bit (0-4294967295) cells too.

**8-bit is standard**. It's what Brainfuck was designed for, what most programs expect, what makes sense for ASCII output.

**16-bit** is useful for Unicode, larger numbers, certain algorithms that need more range. Your "Hello World" still works, but now you can also say "你好世界".

**32-bit** is for when you're doing serious computation. Scientific calculations, large number processing, or just because you can. The trade-off is memory usage – each cell takes 4 times as much space.

### Tape Size: Your Memory Budget

How many cells do you get to play with? The default 30000 is traditional, but you can go from 1 (why would you?) to 10 million (why would you?!).

**30000 is plenty** for most programs. It's enough to implement complex algorithms, store substantial data, build virtual machines.

Go smaller for debugging – a 100-cell tape makes it easy to see everything at once. Go larger for ambitious projects – implementing a compiler might need millions of cells.

But remember: larger tapes use more memory, take longer to initialize, and might hit browser limits. Your browser has to allocate a contiguous array for the entire tape, even if you only use 10 cells.

## Performance: The Hidden Settings

Some settings aren't in the panel but happen automatically based on what you're doing.

When your tape exceeds 10,000 cells, the renderer switches to optimized mode. Updates batch together, scrolling becomes virtual, only visible cells render. You don't see a setting for this because the IDE knows when to do it.

When you're in Turbo mode, visualization updates become rare. The IDE prioritizes execution speed over visual feedback. Again, no setting needed – the IDE adapts.

This is the philosophy: manual settings for things you might want to control, automatic optimization for things the IDE can figure out itself.

## Import/Export: Taking Your Settings With You

Your settings live in localStorage, which means they're tied to this browser on this computer. Want to move them somewhere else? Here's how.

### Backing Up Your World

Open your browser's developer console (F12, then click Console). Type:
```javascript
JSON.stringify(localStorage)
```

That spits out all your settings as a text string. Copy it, save it somewhere safe. It's your IDE configuration captured in a bottle.

### Restoring Your Settings

Got that settings string? In the console, type:
```javascript
Object.assign(localStorage, JSON.parse('your-settings-string-here'))
```

Refresh the page and boom – your IDE is exactly as you left it. It's like teleporting your workspace to a new computer.

### Starting Fresh

Sometimes you want to wipe everything clean. In the console:
```javascript
localStorage.clear()
```

Refresh, and you're back to factory defaults. Every setting returns to its original state, like the IDE was just installed.

## Setting Recipes: Configurations for Every Occasion

### The Learner's Setup
- Auto Expand: ON (see expansions immediately)
- Strip Comments: OFF (keep all explanations)
- View Mode: Normal (clearest display)
- Custom Delay: 100ms (slow enough to follow)
- Cell Size: 8-bit (standard Brainfuck)

Perfect for tutorials, understanding how things work, building intuition.

### The Speed Demon
- Auto Expand: OFF (manual control)
- Strip Comments: ON (minimal output)
- View Mode: Compact (maximum visibility)
- Use WASM: ON (fastest expansion)
- Execution: Turbo or WASM mode

When you need maximum performance and know what you're doing.

### The Detective
- Auto Expand: OFF (control when things happen)
- Strip Comments: OFF (preserve all information)
- View Mode: Lane (structured view)
- Show Disassembly: ON (if using assembly)
- Execution: Step mode

For hunting bugs, understanding problems, solving mysteries.

### The Teacher
- Auto Expand: ON (immediate feedback)
- Strip Comments: OFF (keep explanations)
- View Mode: Normal or Lane (depending on lesson)
- Custom Delay: 200ms (comfortable pace)
- Larger font size (if available)

Making Brainfuck understandable for others.

## Troubleshooting: When Settings Go Wrong

**Everything is slow**: Turn off Auto Expand for large files, reduce tape size, use compact view, enable WASM.

**Can't see my code**: Wrong view mode? Too many panels open? Try resetting view settings.

**Macros won't expand**: Check Auto Expand is on, verify WASM is working (try turning it off), look for errors in macro syntax.

**Out of memory**: Reduce tape size, clear snapshots, close unused panels, restart browser.

**Settings won't save**: localStorage might be full or disabled. Check browser settings, clear old data.

## The Philosophy of Settings

Good settings disappear. You configure them once and forget they exist. Bad settings require constant fiddling. The IDE tries hard to make its settings good settings.

That's why some things are automatic (performance optimizations), some have smart defaults (most people never need to change them), and some are prominently placed (the ones you'll actually use).

The goal isn't to give you every possible option – it's to give you the right options. The ones that actually matter for how you work with Brainfuck.

## Your Settings Journey

When you start, you'll probably use defaults for everything. That's fine – they're good defaults, chosen carefully.

As you grow, you'll discover preferences. Maybe you like compact view for large programs. Maybe you prefer manual macro expansion. Maybe 16-bit cells open new possibilities.

Eventually, you'll have your perfect setup. Your IDE will feel like a tailored suit, fitting exactly how you work. Export those settings, guard them carefully. They're not just configuration – they're your personal development environment, tuned through experience.

Welcome to your control room. Every switch, dial, and button is here for a reason. Use them wisely, and the IDE becomes not just a tool, but your tool.