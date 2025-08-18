# Compiler Trace JSON Formats

This document describes the JSON formats used when running the Ripple C compiler with the `--trace` flag. The trace flag generates the following JSON files for each compilation stage:

1. `filename.tokens.json` - Lexer output
2. `filename.ast.json` - Parser output  
3. `filename.sem.json` - Semantic analyzer output
4. `filename.tast.json` - Typed AST
5. `filename.ir.json` - Intermediate representation

## Usage

```bash
rcc compile input.c --trace
```

This will generate all five JSON files alongside the normal compilation output.

## 1. Tokens JSON Format (`filename.tokens.json`)

Array of token objects from the lexer:

```json
[
  {
    "token_type": "Int",
    "span": {
      "start": { "line": 1, "column": 1, "offset": 0 },
      "end": { "line": 1, "column": 4, "offset": 3 }
    }
  },
  {
    "token_type": { "Identifier": "main" },
    "span": {
      "start": { "line": 1, "column": 5, "offset": 4 },
      "end": { "line": 1, "column": 9, "offset": 8 }
    }
  }
]
```

### Token Types

- Literals: `IntLiteral(i64)`, `CharLiteral(u8)`, `StringLiteral(String)`
- Keywords: `Int`, `Char`, `Return`, `If`, `While`, etc.
- Operators: `Plus`, `Minus`, `Star`, `Equal`, etc.
- Delimiters: `LeftParen`, `RightParen`, `LeftBrace`, `Semicolon`, etc.
- Special: `EndOfFile`, `Newline`

## 2. AST JSON Format (`filename.ast.json`)

Abstract syntax tree from the parser:

```json
{
  "node_id": 0,
  "items": [
    {
      "Function": {
        "node_id": 1,
        "name": "main",
        "return_type": "Int",
        "parameters": [],
        "body": {
          "node_id": 2,
          "kind": {
            "Compound": [
              {
                "node_id": 3,
                "kind": { "Return": { "IntLiteral": { "value": 0 } } },
                "span": { ... }
              }
            ]
          },
          "span": { ... }
        },
        "storage_class": "Auto",
        "span": { ... },
        "symbol_id": null
      }
    }
  ],
  "span": { ... }
}
```

### AST Node Types

- **TopLevelItem**: `Function`, `Declarations`, `TypeDefinition`
- **Statement**: `Expression`, `Compound`, `Declaration`, `If`, `While`, `For`, `Return`, etc.
- **Expression**: `IntLiteral`, `Identifier`, `Binary`, `Unary`, `Call`, `Assignment`, etc.

## 3. Semantic Analysis JSON Format (`filename.sem.json`)

Symbol table and type information after semantic analysis:

```json
{
  "symbols": [
    {
      "name": "main",
      "symbol_type": "Function(Int, [])",
      "scope_level": 0
    },
    {
      "name": "x",
      "symbol_type": "Int",
      "scope_level": 1
    }
  ],
  "type_definitions": [
    {
      "name": "size_t",
      "definition": "unsigned int"
    }
  ]
}
```

## 4. Typed AST JSON Format (`filename.tast.json`)

AST with full type information and resolved pointer arithmetic:

```json
{
  "items": [
    {
      "Function": {
        "name": "main",
        "return_type": "Int",
        "parameters": [],
        "body": {
          "Compound": [
            {
              "Declaration": {
                "name": "x",
                "decl_type": "Int",
                "initializer": {
                  "IntLiteral": {
                    "value": 42,
                    "expr_type": "Int"
                  }
                },
                "symbol_id": 1
              }
            }
          ]
        }
      }
    }
  ]
}
```

### Key Differences from AST

- All expressions have `expr_type` field
- Pointer arithmetic is explicit: `PointerArithmetic`, `PointerDifference`
- Member access includes computed offsets
- All type references are resolved

## 5. IR JSON Format (`filename.ir.json`)

Low-level intermediate representation:

```json
{
  "name": "test",
  "functions": [
    {
      "name": "main",
      "return_type": "I32",
      "parameters": [],
      "blocks": [
        {
          "id": 0,
          "instructions": [
            { "Store": { "ptr": "%0", "value": { "Constant": 42 } } },
            { "Return": { "value": { "Constant": 0 } } }
          ]
        }
      ],
      "is_definition": true
    }
  ],
  "globals": [
    {
      "name": "global_var",
      "var_type": "I32",
      "initializer": { "Constant": 0 },
      "is_external": false
    }
  ],
  "string_literals": []
}
```

### IR Types

- **Types**: `I8`, `I16`, `I32`, `Ptr(Type)`, `Array(Type, size)`, `Struct(fields)`
- **Instructions**: `Alloca`, `Load`, `Store`, `Binary`, `Call`, `Return`, `Branch`, etc.
- **Values**: `Constant(i64)`, `Register(id)`, `Global(name)`, `ConstantArray(values)`

## Error Handling

If the compiler encounters any AST node or feature that cannot be serialized during tracing, it will throw a `CompilerError` rather than silently skipping or using fallbacks. This ensures trace output is always complete and accurate.

## Example

Given this C file:

```c
int add(int a, int b) {
    return a + b;
}

int main() {
    int result = add(3, 4);
    return result;
}
```

Running `rcc compile example.c --trace` will generate:
- `example.tokens.json` - ~50 tokens
- `example.ast.json` - Full AST with 2 functions
- `example.sem.json` - Symbol table with functions and variables
- `example.tast.json` - Typed AST with resolved types
- `example.ir.json` - IR with basic blocks and SSA instructions