# RVM - Ripple Virtual Machine

A reference implementation of the Ripple VM architecture that executes programs produced by the ripple-asm assembler and linker.

## Features

- Complete implementation of all Ripple VM instructions
- Support for 18 registers (R0-R15, PC, PCB, RA, RAB)
- 16-bit cell width with wrap-around arithmetic  
- Configurable bank size (default: 4096)
- Memory-mapped I/O for output
- Debug mode for step-by-step execution
- Binary program loader (RLINK format)

## Building

```bash
cargo build --release
```

## Usage

```bash
# Run a binary program
rvm program.bin

# Run with verbose output
rvm -v program.bin

# Run in debug mode (step-by-step)
rvm -d program.bin

# Specify custom bank size
rvm -b 4096 program.bin

# Specify custom memory size (in words)
rvm -m 32768 program.bin

# Or use cargo run
cargo run --release -- program.bin
```

### Debug Mode Commands

When running with `-d` (simple debugger), the following commands are available:
- **Enter** - Step one instruction
- **r** - Run to completion
- **c** - Continue from breakpoint (after BRK)
- **q** - Quit debugger

When running with `-t` (TUI debugger), additional commands are available:
- **Space/s** - Step one instruction
- **b** - Toggle breakpoint at cursor
- **R** - Restart execution from beginning
- **?** - Show help with all keybindings
- And many more professional debugging features!

## Testing

Run the test suite:

```bash
./run_tests.sh
```

The test suite includes:
- `test_hello` - Basic I/O and HALT
- `test_arithmetic` - ADD, MUL, and loops
- `test_bitwise` - AND, OR, XOR, shifts
- `test_comparison` - SLT, SLTU, BLT, BGE
- `test_division` - DIV, MOD and immediate versions
- `test_jumps` - CALL/RET (JAL/JALR)
- `test_memory` - LOAD/STORE operations

## Architecture

The VM implements the Ripple architecture as specified in `docs/ASSEMBLY_FORMAT.md`:

### Registers
- R0: Always reads as 0
- PC: Program counter (offset within bank)
- PCB: Program counter bank
- RA: Return address (low)
- RAB: Return address bank (high)
- R3-R15: General purpose

### Memory
- 64KB address space (65536 16-bit cells)
- Memory-mapped I/O:
  - 0x0000: Output register (write byte to stdout)
  - 0x0001: Output ready flag

### Instruction Set
All opcodes from 0x00 to 0x1F are implemented, including:
- ALU operations (ADD, SUB, AND, OR, XOR, SLL, SRL, SLT, SLTU)
- Immediate operations (ADDI, ANDI, ORI, XORI, LI, SLLI, SRLI)
- Memory operations (LOAD, STORE)
- Control flow (JAL, JALR, BEQ, BNE, BLT, BGE)
- Extended arithmetic (MUL, DIV, MOD and immediate versions)
- Special (NOP, HALT, BRK)

### BRK Instruction Behavior

The BRK (breakpoint) instruction behaves differently depending on the execution mode:

#### Normal Mode (without -d flag)
- Dumps complete VM state to stderr including:
  - All register values (hex and decimal)
  - First 32 words of memory
  - Current instruction details
- **Halts execution immediately**
- Useful for debugging crashes and inspecting state

#### Debug Mode (with -d flag)
- Prints breakpoint notification
- **Pauses execution** without halting
- Allows continuing with 'c' command or single-stepping
- Acts as a traditional debugger breakpoint

## Program Format

The VM loads binary programs in RLINK format:
- Magic: "RLINK" (5 bytes)
- Entry point (4 bytes, little-endian)
- Instruction count (4 bytes, little-endian)
- Instructions (8 bytes each)
- Data size (4 bytes, little-endian)
- Data section (variable length)