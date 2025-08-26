# Brainfuck Advanced Language Layer System Tutorial

A comprehensive guide to the macro preprocessing system for Brainfuck programming.

## Table of Contents
1. [Introduction](#introduction)
2. [Basic Macro Definitions](#basic-macro-definitions)
3. [Macro Invocation](#macro-invocation)
4. [Parametric Macros](#parametric-macros)
5. [Built-in Functions](#built-in-functions)
6. [Array Literals and Operations](#array-literals-and-operations)
7. [Control Flow](#control-flow)
8. [Advanced Features](#advanced-features)
9. [Best Practices](#best-practices)
10. [Common Patterns](#common-patterns)

## Introduction

The Brainfuck Advanced Language Layer System macro preprocessor extends the standard Brainfuck language with powerful macro capabilities, making it easier to write complex programs. The preprocessor expands macros before the Brainfuck interpreter executes the code.

### Key Features
- Macro definitions with optional parameters
- Built-in functions for repetition, conditionals, and iteration
- Array literals and operations
- Source preservation for embedding non-Brainfuck content
- Nested macro invocations

## Basic Macro Definitions

### Simple Macros

Define a macro using `#define` followed by the macro name and body:

```brainfuck
#define right >
#define left <
#define inc +
#define dec -
#define output .
#define input ,
#define loop_start [
#define loop_end ]

// Usage
@right @inc @output
// Expands to: >+.
```

### Multi-line Macros

Use curly braces for multi-line macro definitions:

```brainfuck
#define clear {
    [-]
}

#define move_right {
    [->>+<<]
    >>
}
```

### Line Continuation

Use backslash for line continuation in single-line macros:

```brainfuck
#define long_sequence > + > + > + \
                      < < < - \
                      > > > .
```

## Macro Invocation

Macros are invoked using the `@` symbol (or `#` for aesthetic preference — they are equivalent, but highlighted differently):

```brainfuck
#define cell_zero [-]

@cell_zero    // Clear current cell
#cell_zero    // Alternative syntax
```

## Parametric Macros

### Basic Parameters

Define macros with parameters by adding parentheses after the name:

```brainfuck
#define move(n) {repeat(n, >)}
#define add(n) {repeat(n, +)}
#define subtract(n) {repeat(n, -)}

@move(5)       // Expands to: >>>>>
@add(10)       // Expands to: ++++++++++
```

### Multiple Parameters

```brainfuck
#define set_value(cell, value) {
    @move(cell)
    [-]
    @add(value)
}

@set_value(3, 65)  // Move to cell 3, clear it, set to 65 (ASCII 'A')
```

### Parameter Substitution in Complex Expressions

```brainfuck
#define copy_cell(from, to, temp) {
    @move(from)
    [
        @move(temp) +
        @move(to) +
        @move(from) -
    ]
    @move(temp)
    [
        @move(from) +
        @move(temp) -
    ]
}
```

## Built-in Functions

### {repeat(count, content)}

Repeats content a specified number of times:

```brainfuck
{repeat(5, >)}          // >>>>>
{repeat(10, +)}         // ++++++++++
{repeat(3, [-]>)}       // [-]>[-]>[-]>
```

### {if(condition, true_branch, false_branch)}

Conditional expansion based on a numeric condition:

```brainfuck
#define DEBUG 1
#define log(msg) {if(#DEBUG, msg)}

@log(.)  // Outputs only if DEBUG is 1
```

### {for(variable in array, body)}

Iterates over array elements:

```brainfuck
{for(i in {1, 2, 3}, {repeat(i, +)})}
// Expands to: + ++ +++

#define chars {65, 66, 67}
{for(c in @chars, [-]@add(c).)}
// Sets each cell to ASCII A, B, C and outputs them
```

### {reverse(array)}

Reverses an array:

```brainfuck
{reverse({1, 2, 3, 4, 5})}     // {5, 4, 3, 2, 1}

#define sequence {a, b, c}
{reverse(@sequence)}            // {c, b, a}
```

## Array Literals and Operations

### Array Literals

Arrays are defined using curly braces:

```brainfuck
{1, 2, 3, 4, 5}              // Numeric array
{a, b, c}                     // Text array
{'A', 'B', 'C'}               // Character literals (converted to ASCII)
{0x41, 0x42, 0x43}           // Hexadecimal numbers
```

### Array Operations with Macros

```brainfuck
#define ASCII_LETTERS {65, 66, 67, 68, 69}
#define REVERSED_LETTERS {reverse(@ASCII_LETTERS)}

{for(letter in @REVERSED_LETTERS, [-]@add(letter).)}
// Outputs: EDCBA
```

## Control Flow

### For Loop with Index

Access both value and index in iterations:

```brainfuck
{for(val, idx in {A, B, C}, 
    Index: idx Value: val{br}
)}
// Output:
// Index: 0 Value: A
// Index: 1 Value: B
// Index: 2 Value: C
```

### Conditional Compilation

```brainfuck
#define MODE 1  // 0=debug, 1=release

#define debug_print {if(@MODE, , .)}
#define optimize_loop {if(@MODE, 
    [-],              // Simple clear in release
    [->+<]            // Slower but traceable in debug
)}
```

## Advanced Features

### Nested Macro Invocations

```brainfuck
#define inner(x) {repeat(x, +)}
#define outer(n) @inner(n)>@inner(n)

@outer(3)  // Expands to: +++>+++
```

### Character and String Handling

```brainfuck
#define print_char(c) [-]{repeat(c, +)}.

@print_char('A')     // Prints 'A'
@print_char(0x41)    // Also prints 'A' (hex)
@print_char(65)      // Also prints 'A' (decimal)
```

### Building Complex Data Structures

```brainfuck
#define STRING_HELLO {72, 101, 108, 108, 111}
#define STRING_WORLD {87, 111, 114, 108, 100}

#define print_string(str) {
    {for(char in str, 
        [-]              // Clear cell
        @add(char)       // Set to character
        .                // Output
    )}
}

@print_string(@STRING_HELLO)
@print_char(32)  // Space
@print_string(@STRING_WORLD)
// Output: Hello World
```

## Best Practices

### 1. Use Descriptive Names

```brainfuck
// Good
#define clear_cell [-]
#define move_pointer_right >

// Avoid
#define c [-]
#define m >
```

### 2. Create Abstraction Layers

```brainfuck
// Low level
#define ptr_right >
#define ptr_left <

// Mid level
#define goto_cell(n) {repeat(n, @ptr_right)}
#define return_from_cell(n) {repeat(n, @ptr_left)}

// High level
#define with_cell(n, ops) {
    @goto_cell(n)
    ops
    @return_from_cell(n)
}
```

### 3. Document Your Macros

```brainfuck
// Copies value from current cell to cell+n using cell+n+1 as temp
#define copy_to(n) [>@move(n)+>+<<@move(n)-]>>@move(n)[<<@move(n)+>>@move(n)-]

// Sets cell to ASCII value of digit (0-9)
#define digit_to_ascii(d) [-]{repeat(48, +)}{repeat(d, +)}
```

### 4. Use Constants

```brainfuck
#define BUFFER_SIZE 10
#define ASCII_ZERO 48
#define ASCII_A 65
#define NEWLINE 10

#define allocate_buffer {repeat(#BUFFER_SIZE, >[-]<)}
```

## Error Handling

The macro expander provides helpful error messages:

```brainfuck
// Error: Undefined macro
@undefined_macro

// Error: Parameter mismatch
#define test(a, b) a+b
@test(1)  // Expects 2 parameters, got 1

// Error: Circular dependency (when detection enabled)
#define a @b
#define b @a
@a
```

## CLI Usage

The macro expander can be used from the command line:

```bash
# Expand a file
bfm expand input.bfm > output.bf

# List all macros
bfm list input.bfm

# Validate macro definitions
bfm validate input.bfm
```

## Summary

The Brainfuck Advanced Language Layer System provides powerful abstractions that make Brainfuck programming more manageable:

- **Macros** for code reuse and abstraction
- **Parameters** for flexible, reusable components
- **Built-in functions** for common operations
- **Arrays** for data organization
- **Control flow** for complex logic

By combining these features, you can write maintainable and sophisticated Brainfuck programs that would be impractical with raw Brainfuck alone.

For the godlike-tear example, check out the "Macro Language" -> "Advanced" -> "Ripple VM" item in tutorials.