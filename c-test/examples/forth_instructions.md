# Forth Interpreter Instructions

## How to Run

```bash
./rct run forth_simple
```

## Basic Usage

The Forth interpreter uses Reverse Polish Notation (RPN). Enter numbers and operators separated by spaces:

```forth
> 5 3 +
 ok
> .
8 ok
```

## Available Commands

### Arithmetic
- `+` - Add top two stack items
- `-` - Subtract (second - top)
- `*` - Multiply top two stack items
- `/` - Divide (second / top)
- `MOD` - Modulo (second % top)

### Comparison (returns -1 for true, 0 for false)
- `=` - Equal
- `<` - Less than
- `>` - Greater than

### Stack Operations
- `DUP` - Duplicate top of stack
- `DROP` - Remove top of stack
- `SWAP` - Swap top two items
- `OVER` - Copy second item to top
- `ROT` - Rotate top three items

### I/O
- `.` - Print and remove top of stack
- `CR` - Print newline
- `EMIT` - Print top of stack as ASCII character
- `.S` - Show entire stack (non-destructive)

### Dictionary
- `WORDS` - List all available words
- `: name ... ;` - Define new word

### System
- `BYE` - Exit interpreter

## Examples

### Basic Math
```forth
> 5 3 + .
8 ok
> 10 4 - .
6 ok
> 6 7 * .
42 ok
```

### Stack Manipulation
```forth
> 1 2 3 .S
Stack:
  1
  2
  3
 ok
> SWAP .S
Stack:
  1
  3
  2
 ok
```

### Define New Words
```forth
> : SQUARE DUP * ;
 ok
> 5 SQUARE .
25 ok

> : DOUBLE 2 * ;
 ok
> 7 DOUBLE .
14 ok
```

### Comparison
```forth
> 5 5 = .
-1 ok
> 3 7 < .
-1 ok
> 10 5 > .
-1 ok
```

## Notes

- The interpreter uses RPN (Reverse Polish Notation)
- All operations work on the stack
- Numbers are pushed onto the stack automatically
- Words (commands) operate on stack values
- Use `.S` to inspect the stack at any time
- Use `WORDS` to see all available commands