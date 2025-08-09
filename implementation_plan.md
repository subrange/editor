# Ripple C99 Compiler (rcc) - Implementation Plan

## Overview
Build a C99 freestanding compiler targeting Ripple VM ISA using Rust with test-driven development. The compiler will emit assembly for the existing rasm/rlink toolchain.

## Architecture Overview

```
Source (.c) → Lexer → Parser → AST → Type Checker → IR → Optimizer → Code Gen → Assembly (.asm)
                                           ↓
                                    Symbol Table/Types
```

## M1: Backend Skeleton (Weeks 1-3)
**Goal**: ISA emitter, ABI, prologue/epilogue, calls, loads/stores, arithmetic, branches. Hello world works.

### 1.1 Project Setup
- [x] Create Rust project structure with workspace
  - `codegen/` - Ripple assembly generation (start here!)
  - `ir/` - minimal IR for testing backend
  - `driver/` - main compiler binary
  - `common/` - shared types, errors

### 1.2 Ripple ISA Emitter
```rust
// codegen/src/asm.rs
pub enum Reg {
    R0, R1, R2, R3, R4, R5, R6, R7,
    R8, R9, R10, R11, R12, R13, R14, R15,
    PC, PCB, RA, RAB,
}

pub enum AsmInst {
    // Arithmetic
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    AddI(Reg, Reg, i16),
    SubI(Reg, Reg, i16),
    
    // Memory
    Load(Reg, Reg, Reg),  // rd, bank, addr
    Store(Reg, Reg, Reg), // rs, bank, addr
    LI(Reg, i16),
    
    // Control
    Jal(i16, i16),        // bank_imm, addr_imm
    Jalr(Reg, Reg, Reg),  // bank_reg, addr_reg
    Beq(Reg, Reg, String), // rs, rt, label
    Bne(Reg, Reg, String),
    
    // Pseudo
    Label(String),
    Comment(String),
}
```

**Tests first:**
```rust
#[test]
fn test_emit_hello_world() {
    let instrs = vec![
        AsmInst::LI(Reg::R3, 'H' as i16),
        AsmInst::Store(Reg::R3, Reg::R0, Reg::R0),
        AsmInst::LI(Reg::R3, 'i' as i16),
        AsmInst::Store(Reg::R3, Reg::R0, Reg::R0),
        AsmInst::Halt,
    ];
    
    let asm = emit(instrs);
    assert_eq!(asm, "LI R3, 72\nSTORE R3, R0, R0\nLI R3, 105\nSTORE R3, R0, R0\nHALT\n");
}
```

### 1.3 ABI Implementation
- [x] Function prologue generator
- [x] Function epilogue generator
- [x] Call sequence (save/restore)
- [x] Stack frame layout

```rust
// codegen/src/abi.rs
pub struct Frame {
    locals_size: u16,
    saved_regs: Vec<Reg>,
    has_calls: bool,
}

impl Frame {
    pub fn gen_prologue(&self) -> Vec<AsmInst> {
        let mut code = vec![];
        
        // Save FP
        code.push(AsmInst::Store(Reg::R15, Reg::R13, Reg::R14));
        code.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
        
        // Save RA if needed
        if self.has_calls {
            code.push(AsmInst::Store(Reg::RA, Reg::R13, Reg::R14));
            code.push(AsmInst::AddI(Reg::R14, Reg::R14, 1));
        }
        
        // Set new FP
        code.push(AsmInst::Add(Reg::R15, Reg::R14, Reg::R0));
        
        // Allocate locals
        if self.locals_size > 0 {
            code.push(AsmInst::AddI(Reg::R14, Reg::R14, self.locals_size as i16));
        }
        
        code
    }
}
```

### 1.4 Minimal IR for Testing
```rust
// ir/src/lib.rs - Just enough to test backend
pub enum SimpleIR {
    Const(u8, i16),          // temp_id, value
    Add(u8, u8, u8),         // dest, src1, src2
    Store(u8, u8, u8),       // value, bank, addr
    Call(String),
    Return(Option<u8>),
    Label(String),
}
```

### 1.5 Basic Code Generation
- [x] IR to assembly lowering
- [x] Register assignment (simple, no allocation yet)
- [x] Arithmetic operations
- [x] Memory operations
- [x] Function calls

**Test-driven:**
```rust
#[test]
fn test_lower_add() {
    let ir = vec![
        SimpleIR::Const(0, 5),
        SimpleIR::Const(1, 10),
        SimpleIR::Add(2, 0, 1),
    ];
    
    let asm = lower_to_asm(ir);
    assert_contains!(asm, AsmInst::LI(Reg::R3, 5));
    assert_contains!(asm, AsmInst::LI(Reg::R4, 10));
    assert_contains!(asm, AsmInst::Add(Reg::R5, Reg::R3, Reg::R4));
}

#[test]
fn test_putchar_call() {
    let ir = vec![
        SimpleIR::Const(0, 'A' as i16),
        SimpleIR::Store(0, 255, 255), // 255 means R0
    ];
    
    let asm = lower_to_asm(ir);
    assert_contains!(asm, AsmInst::Store(Reg::R3, Reg::R0, Reg::R0));
}
```

## M2: Front End & IR (Weeks 3-7)
**Goal**: Parse C subset, type checking, IR, lowering. Run toy programs (no structs).

### 2.1 Lexer
- [x] Token definitions
- [x] Scanner implementation
- [x] Keywords, operators, literals
- [x] Source location tracking

### 2.2 Parser
- [x] Expression parser (precedence climbing)
- [x] Statement parser
- [x] Declaration parser (functions, variables)
- [x] Type parser (basic types only)

### 2.3 AST Definition
```rust
// ast/src/lib.rs
pub enum Type {
    Void,
    Bool,
    Char(Signedness),
    Short(Signedness),
    Int(Signedness),
    Long(Signedness),
    Pointer(Box<Type>),
    Function(Box<Type>, Vec<Type>),
}

pub enum Expr {
    Literal(i32),
    Identifier(String),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Cast(Type, Box<Expr>),
}

pub enum Stmt {
    Expression(Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Return(Option<Expr>),
}
```

### 2.4 Semantic Analysis
- [x] Symbol table
- [x] Type checking
- [x] Type coercion
- [x] Constant evaluation

### 2.5 Full IR Design
```rust
pub enum IrInst {
    // Data movement
    Move(Temp, Operand),
    Load(Temp, Address),
    Store(Address, Operand),
    
    // Arithmetic
    Add(Temp, Operand, Operand),
    Sub(Temp, Operand, Operand),
    Mul(Temp, Operand, Operand),
    
    // Control flow
    Jump(Label),
    CondJump(Cond, Operand, Operand, Label, Label),
    Call(Option<Temp>, String, Vec<Operand>),
    Return(Option<Operand>),
    
    Label(String),
}
```

### 2.6 AST to IR Lowering
- [x] Expression lowering
- [x] Statement lowering
- [x] Control flow generation
- [x] Temporary management

## M3: Data, Structs, Arrays (Weeks 7-9)
**Goal**: Aggregates, address-of/deref, global data emission, .rodata strings.

### 3.1 Extended Types
- [x] Array types (completed - arrays properly decay to pointers)
- [x] Struct/union types (basic support - inline structs work)
- [x] Sizeof computation (works for all types including structs)
- [ ] Alignment rules
- [x] Array initializers with {} (completed - supports both list and string literal initializers)

### 3.2 Memory Layout
- [x] Global variable allocation (starting at address 100)
- [x] String literal storage (hex-encoded in variable names)
- [ ] Section management (.data, .rodata, .bss)

### 3.3 Address Operations
- [x] Address-of (&) operator
- [x] Dereference (*) operator  
- [x] Array indexing (with pointer arithmetic)
- [x] Array-to-pointer decay (critical for C semantics)
- [x] Struct member access (basic implementation - has memory bank tracking issue for stack-allocated structs)

### 3.4 Data Emission
```rust
#[test]
fn test_string_literal() {
    let c = r#"char *s = "Hello";"#;
    let asm = compile(c);
    assert_contains!(asm, ".section .rodata");
    assert_contains!(asm, ".string \"Hello\"");
}
```

### 3.5 Functions
- [ ] Pointer parameters
- [ ] Inline assembly support

## M4: Runtime + libc mini (Weeks 9-11)
**Goal**: crt0, math helpers, memcpy/memset, puts/putchar.

### 4.1 CRT0 Assembly
- [ ] Stack initialization
- [ ] BSS zeroing
- [ ] Main invocation
- [ ] Exit handling

### 4.2 Built-in Functions
- [ ] putchar implementation
- [ ] puts implementation
- [ ] memcpy/memset
- [ ] 32-bit arithmetic helpers

### 4.3 Linking Support
- [ ] Symbol exports
- [ ] External references
- [ ] Section layout

## M5: Optimizations + Debug (Weeks 11-13)
**Goal**: O1, line maps, symbol dumping for IDE, verify stepping.

### 5.1 Basic Optimizations
- [ ] Constant folding
- [ ] Dead code elimination
- [ ] Copy propagation
- [ ] Local CSE

### 5.2 Debug Information
- [ ] Line number mapping
- [ ] Symbol table emission
- [ ] Variable location tracking

### 5.3 Peephole Optimizer
- [ ] Redundant move elimination
- [ ] Instruction combining
- [ ] Branch optimization

## M6: Toolchain Integration (Week 14)
**Goal**: rcc driver orchestrating rasm/rlink, docs, examples.

### 6.1 Driver Implementation
- [ ] Command-line parsing
- [ ] Build orchestration
- [ ] Error reporting
- [ ] Multi-file compilation

### 6.2 Documentation
- [ ] User manual
- [ ] ABI documentation
- [ ] Example programs

### 6.3 Test Suite
- [ ] Conformance tests
- [ ] Regression tests
- [ ] Performance benchmarks

## Testing Strategy

### Unit Tests
Each module has comprehensive unit tests:
- Lexer: Token recognition, error cases
- Parser: Valid/invalid syntax, error recovery
- Type checker: Type rules, coercions
- IR: Lowering correctness
- Codegen: Instruction selection, ABI compliance

### Integration Tests
End-to-end compilation tests:
```rust
#[test]
fn test_hello_world() {
    let c_code = r#"
        void putchar(int c);
        int main() {
            putchar('H');
            putchar('i');
            putchar('\n');
            return 0;
        }
    "#;
    
    let asm = compile(c_code);
    assert_contains!(asm, "STORE R3, R0, R0"); // putchar
}
```

### Conformance Tests
- C99 standard compliance tests
- ABI compliance tests
- Ripple ISA conformance

## Directory Structure

```
rcc/
├── Cargo.toml (workspace)
├── lexer/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── token.rs
│   │   └── scanner.rs
│   └── tests/
├── parser/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── expr.rs
│   │   ├── stmt.rs
│   │   └── decl.rs
│   └── tests/
├── ast/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── sema/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs
│   │   └── symbols.rs
│   └── tests/
├── ir/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   └── lower.rs
│   └── tests/
├── codegen/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── regalloc.rs
│   │   └── emit.rs
│   └── tests/
├── opt/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── driver/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── runtime/
│   ├── crt0.asm
│   └── libc.asm
└── tests/
    ├── conformance/
    └── integration/
```

## Key Design Decisions

1. **Type Representation**: 16-bit int as default, 32-bit long stored in 2 cells
2. **Pointer Model**: MVP uses flat 16-bit pointers (bank=R0)
3. **ABI**: R3-R8 for args/temps, R9-R12 callee-saved, R13-R15 for stack
4. **Stack**: Grows upward, frame pointer-based
5. **IR**: Simple 3-address code, not full SSA initially
6. **Testing**: Write tests first for each component

## Risk Mitigation

1. **Parser Complexity**: Use parser combinator library (nom) or hand-written recursive descent
2. **Register Pressure**: Simple linear scan with aggressive spilling
3. **32-bit Operations**: Implement as library calls initially
4. **Debug Info**: Start simple with line numbers, enhance incrementally

## Success Criteria

- [x] Compiles hello world
- [ ] Compiles FizzBuzz  
- [x] Compiles recursive Fibonacci
- [ ] Passes basic C99 conformance tests
- [x] Integrates with rasm/rlink toolchain (emits compatible assembly)
- [ ] Generates debuggable code for IDE