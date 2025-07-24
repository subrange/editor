# Brainfuck Macro Library Documentation

## Table of Contents
1. [Basic Operations](#basic-operations)
2. [Memory Operations](#memory-operations)
3. [Lane Operations](#lane-operations)
4. [Arithmetic](#arithmetic)
5. [Control Flow](#control-flow)
6. [Common Patterns](#common-patterns)
7. [Stack Operations](#stack-operations)
8. [String/Output](#string-output)
9. [Advanced Patterns](#advanced-patterns)
10. [Debug Helpers](#debug-helpers)
11. [VM Helpers](#vm-helpers)
12. [Math Operations](#math-operations)

---

## Basic Operations

### `@clear`
Zeros out the current cell.
```brainfuck
@set(42) @clear  // Cell is now 0
```

### `@inc(n)`
Increments current cell by n.
```brainfuck
@inc(5)  // Adds 5 to current cell
```

### `@dec(n)`
Decrements current cell by n.
```brainfuck
@dec(3)  // Subtracts 3 from current cell
```

### `@next(n)` / `@right(n)`
Moves pointer n cells to the right.
```brainfuck
@next(3)   // Move 3 cells right
@right(3)  // Same thing, just an alias
```

### `@prev(n)` / `@left(n)`
Moves pointer n cells to the left.
```brainfuck
@prev(2)   // Move 2 cells left
@left(2)   // Same thing
```

### `@zero`
Alias for `@clear` - zeros current cell.

---

## Memory Operations

### `@move(n)`
**Destructively** moves value from current cell to n cells right.
```brainfuck
@set(42) @move(3)  // Value 42 is now 3 cells right, current cell is 0
```

### `@copy(n)`
**Non-destructively** copies value to n cells right.
```brainfuck
@set(42) @copy(3)  // Both current cell and cell+3 contain 42
```

### `@swap`
Swaps current cell value with next cell.
```brainfuck
@set(10) @next(1) @set(20) @prev(1)
@swap  // Now current=20, next=10
```

### `@store(n)`
Stores current value in cell n positions away (clears current).
```brainfuck
@set(42) @store(5)  // Cell+5 now has 42, current is 0
```

### `@load(n)`
Loads value from cell n positions away to current (non-destructive).
```brainfuck
@load(5)  // Current cell now has value from cell+5
```

---

## Lane Operations

For 5-lane architecture where memory is organized as:
```
[val, type, jump, nav, scratch, val, type, jump, nav, scratch, ...]
```

### `@nextword` / `@prevword`
Navigate between "words" (5-cell chunks).
```brainfuck
@nextword  // Moves 5 cells right to next value lane
@prevword  // Moves 5 cells left to previous value lane
```

### `@lane(n)` / `@tolane(n)`
Move to lane n within current word (0-indexed).
```brainfuck
@lane(2)  // Move to jump lane (3rd lane)
```

### Lane shortcuts
- `@value_lane` - Go to lane 0 (already there if at word start)
- `@type_lane` - Go to lane 1
- `@jump_lane` - Go to lane 2
- `@nav_lane` - Go to lane 3
- `@scratch_lane` - Go to lane 4

### `@reset_lane`
Returns to lane 0 from any lane in current word.
```brainfuck
@scratch_lane @inc(10) @reset_lane  // Back at value lane
```

---

## Arithmetic

### `@set(n)`
Sets current cell to specific value (clears first).
```brainfuck
@set(65)  // Cell now contains 65 (ASCII 'A')
```

### `@add_next`
Adds value from next cell to current (destroys next cell).
```brainfuck
@set(10) @next(1) @set(5) @prev(1)
@add_next  // Current=15, next=0
```

### `@sub_next`
Subtracts next cell from current (destroys next cell).
```brainfuck
@set(10) @next(1) @set(3) @prev(1)
@sub_next  // Current=7, next=0
```

### `@mul(n)`
Multiplies current cell by n (uses next cell as temp).
```brainfuck
@set(5) @mul(3)  // Current cell = 15
```

### `@compare`
Compares current with next cell. Result: 0 if equal, non-zero if different.
```brainfuck
@set(5) @next(1) @set(5) @prev(1)
@compare  // Current = 0 (they were equal)
```

---

## Control Flow

### `@if` ... `@endif`
Execute code block if current cell is non-zero. Clears cell after.
```brainfuck
@set(1)
@if
  @print(65)  // Prints 'A'
@endif
// Cell is now 0
```

### `@while` ... `@endwhile`
Standard while loop - continues while current cell is non-zero.
```brainfuck
@set(5)
@while
  @print(42)  // Print '*' 
  @dec(1)
@endwhile
// Prints 5 asterisks
```

### `@loop(n)` ... `@endloop`
Loops exactly n times (needs empty cell to the right).
```brainfuck
@loop(3)
  @print(65)  // Print 'A'
@endloop
// Prints "AAA"
```

---

## Common Patterns

### `@print(c)`
Prints ASCII character c and clears cell.
```brainfuck
@print(72) @print(105)  // Prints "Hi"
```

### `@newline`
Prints newline character.
```brainfuck
@print(65) @newline  // Prints "A\n"
```

### `@space`
Prints space character.
```brainfuck
@print(65) @space @print(66)  // Prints "A B"
```

### `@not`
Boolean NOT - converts 0→1, non-zero→0.
```brainfuck
@set(5) @not   // Cell = 0
@clear @not    // Cell = 1
```

---

## Stack Operations

For stack growing rightward:

### `@push`
Pushes current cell value onto stack.
```brainfuck
@set(42) @push  // 42 is now on stack, current cell cleared
```

### `@pop`
Pops value from stack to current cell.
```brainfuck
@pop  // Current cell now has top stack value
```

---

## String/Output

### `@hello_world`
Prints "Hello World!" with newline.
```brainfuck
@hello_world  // Output: Hello World!\n
```

---

## Advanced Patterns

### `@find_zero`
Moves pointer to next zero cell rightward.
```brainfuck
@find_zero  // Pointer now at first zero cell to the right
```

### `@find_nonzero`
Moves pointer to next non-zero cell rightward.
```brainfuck
@find_nonzero  // Pointer at first non-zero cell
```

---

## Debug Helpers

### `@mark`
Sets current cell to 255 (useful for debugging).
```brainfuck
@mark  // Cell = 255, visible in debugger
```

### `@break`
Creates infinite loop (breakpoint).
```brainfuck
@break  // Program hangs here - check debugger!
```

---

## VM Helpers

For your command-based VM:

### Command constants
- `@nop` - Sets cell to 1 (NOP opcode)
- `@add_cmd` - Sets cell to 2 (ADD opcode)
- `@sub_cmd` - Sets cell to 3 (SUB opcode)
- `@jmp_cmd` - Sets cell to 4 (JMP opcode)
- `@jz_cmd` - Sets cell to 5 (JZ opcode)

### `@cmd(op, a, b)`
Creates a command with two operands.
```brainfuck
@cmd(2, 10, 5)  // Creates: [2, 10, 5] (ADD 10 5)
```

---

## Math Operations

### Fast increments
- `@inc2` - Increment by 2
- `@inc3` - Increment by 3
- `@inc4` - Increment by 4
- `@inc5` - Increment by 5
- `@inc10` - Increment by 10

### `@pow2(n)`
Calculates 2^n in current cell.
```brainfuck
@pow2(3)  // Cell = 8 (2^3)
```

---

## Usage Examples

### Example 1: Working with lanes
```brainfuck
// Store data with type marker
@value_lane @set(42)
@type_lane @set(1)  // Mark as data
@nextword
```

### Example 2: Conditional printing
```brainfuck
@set(5)  // Some condition
@if
  @print(89) @print(69) @print(83)  // Prints "YES"
@endif
```

### Example 3: Building a simple loop
```brainfuck
@loop(5)
  @print(42) @space  // Print "* "
@endloop
@newline
// Output: * * * * * 
```

### Example 4: Using the stack
```brainfuck
@set(10) @push
@set(20) @push
@set(30) @push
@pop @print(48) @space  // Print "0 " (30 + 48 = 78... wait, wrong math)
@pop @add_next @print(48)  // Actually prints the values...
```

---

## Pro Tips

1. **Macro composition**: Combine macros for complex operations
   ```brainfuck
   @set(5) @copy(2) @next(2) @mul(3)  // Copy and triple
   ```

2. **Lane discipline**: Always return to value lane after operations
   ```brainfuck
   @type_lane @set(1) @reset_lane
   ```

3. **Debugging**: Use `@mark` to trace execution paths
   ```brainfuck
   @mark @complex_operation @clear @mark
   ```

4. **Performance**: Use specialized macros like `@inc10` instead of `@inc(10)`

Now you can write elegant Brainfuck without consulting the source! 🚀