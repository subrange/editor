# Ripple VM Memory-Mapped I/O Documentation

## Overview

The Ripple VM implements a memory-mapped I/O (MMIO) system with a dedicated 32-word header at bank 0, addresses 0-31. This provides efficient access to I/O devices, random number generation, and display control without requiring system calls or special instructions.

## Memory Layout

### MMIO Header (Bank 0, Words 0-31)

| Address | Name                | R/W | Description                                      |
|---------|---------------------|-----|--------------------------------------------------|
| 0       | `HDR_TTY_OUT`       | W   | TTY output (low 8 bits written to stdout)        |
| 1       | `HDR_TTY_STATUS`    | R   | TTY status (bit 0: ready flag)                   |
| 2       | `HDR_TTY_IN_POP`    | R   | Pop and read next input byte                     |
| 3       | `HDR_TTY_IN_STATUS` | R   | Input status (bit 0: has byte available)         |
| 4       | `HDR_RNG`           | R   | Read next PRNG value (auto-advances)             |
| 5       | `HDR_RNG_SEED`      | R/W | RNG seed (low 16 bits)                           |
| 6       | `HDR_DISP_MODE`     | R/W | Display mode (0=OFF, 1=TTY, 2=TEXT40)            |
| 7       | `HDR_DISP_STATUS`   | R   | Display status (bit 0: ready, bit 1: flush done) |
| 8       | `HDR_DISP_CTL`      | R/W | Display control (bit 0: enable, bit 1: clear)    |
| 9       | `HDR_DISP_FLUSH`    | W   | Trigger display flush (write non-zero)           |
| 10-31   | Reserved            | -   | Reserved for future use (return 0 on read)       |

### TEXT40 VRAM (Bank 0, Words 32-1031)

- **Location**: Words 32-1031 (1000 words total)
- **Layout**: 40x25 character cells
- **Format**: Each word contains: `(attribute << 8) | ascii_char`
  - Low byte: ASCII character code
  - High byte: Attributes (currently unused, set to 0)

### General Memory (Bank 0, Word 1032+)

Regular data memory starts at word 1032, after the VRAM region.

## Device Details

### TTY I/O

**Output (HDR_TTY_OUT)**
- Write-only register at address 0
- Low 8 bits are sent to stdout immediately
- Sets TTY_STATUS busy flag temporarily (currently instant ready)

**Status (HDR_TTY_STATUS)**
- Read-only register at address 1
- Bit 0: Ready flag (1=ready to accept output, 0=busy)

**Input (HDR_TTY_IN_POP)**
- Read-only register at address 2
- Reading pops one byte from input buffer
- Returns 0 if buffer is empty

**Input Status (HDR_TTY_IN_STATUS)**
- Read-only register at address 3
- Bit 0: Has byte flag (1=byte available, 0=buffer empty)

### Random Number Generator

**RNG (HDR_RNG)**
- Read-only register at address 4
- Each read advances the PRNG state
- Returns a 16-bit pseudorandom value
- Uses Linear Congruential Generator (LCG): `next = (1664525 * prev + 1013904223) mod 2^32`

**RNG Seed (HDR_RNG_SEED)**
- Read/Write register at address 5
- Controls low 16 bits of RNG seed
- Writing sets the seed for reproducible sequences

### Display System

**Display Mode (HDR_DISP_MODE)**
- Read/Write register at address 6
- Values:
  - 0: Display OFF
  - 1: TTY passthrough mode
  - 2: TEXT40 mode (40x25 character display)

**Display Status (HDR_DISP_STATUS)**
- Read-only register at address 7
- Bit 0: Ready flag
- Bit 1: Flush done flag

**Display Control (HDR_DISP_CTL)**
- Read/Write register at address 8
- Bit 0: Enable display
- Bit 1: Clear VRAM (edge-triggered, auto-clears)

**Display Flush (HDR_DISP_FLUSH)**
- Write-only register at address 9
- Writing non-zero triggers display update
- Sets flush_done flag when complete

## Implementation Details

### MMIO Read Handling

The VM intercepts reads to bank 0, addresses 0-1031:
1. Addresses 0-31: MMIO header registers
2. Addresses 32-1031: TEXT40 VRAM (direct memory access)
3. Other banks or addresses > 1031: Regular memory access

```rust
fn handle_mmio_read(&mut self, addr: usize) -> Option<u16> {
    match addr {
        HDR_TTY_OUT => Some(0),  // Write-only
        HDR_TTY_STATUS => Some(if self.output_ready { TTY_READY } else { 0 }),
        HDR_TTY_IN_POP => {
            let value = self.input_buffer.pop_front().unwrap_or(0) as u16;
            self.memory[HDR_TTY_IN_POP] = value;
            Some(value)
        },
        HDR_TTY_IN_STATUS => Some(if !self.input_buffer.is_empty() { TTY_HAS_BYTE } else { 0 }),
        HDR_RNG => {
            self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let value = (self.rng_state >> 16) as u16;
            self.memory[HDR_RNG] = value;
            Some(value)
        },
        // ... other MMIO addresses
        _ => None  // Not MMIO
    }
}
```

### MMIO Write Handling

The VM intercepts writes to bank 0, addresses 0-1031:

```rust
fn handle_mmio_write(&mut self, addr: usize, value: u16) -> bool {
    match addr {
        HDR_TTY_OUT => {
            let byte = (value & 0xFF) as u8;
            io::stdout().write_all(&[byte]);
            io::stdout().flush();
            self.output_buffer.push_back(byte);
            true
        },
        HDR_DISP_CTL => {
            if value & DISP_CLEAR != 0 {
                // Clear VRAM
                for i in TEXT40_BASE_WORD..=TEXT40_LAST_WORD {
                    self.memory[i] = 0;
                }
            }
            if value & DISP_ENABLE != 0 {
                self.display_enabled = true;
            }
            true
        },
        // ... other MMIO addresses
        _ => false  // Not MMIO
    }
}
```

### Memory Access Instructions

LOAD and STORE instructions check for MMIO addresses:

```rust
// LOAD instruction (opcode 0x11)
if bank_val == 0 && addr_val < TEXT40_LAST_WORD as u16 + 1 {
    if let Some(value) = self.handle_mmio_read(addr_val as usize) {
        self.registers[rd] = value;
    } else {
        self.registers[rd] = self.memory[addr_val as usize];
    }
}

// STORE instruction (opcode 0x12)
if bank_val == 0 && addr_val < TEXT40_LAST_WORD as u16 + 1 {
    if !self.handle_mmio_write(addr_val as usize, value) {
        self.memory[addr_val as usize] = value;
    }
}
```

## Usage Examples

### Basic TTY Output
```asm
; Print 'A' to stdout
LI    A0, 'A'
LI    T0, 0        ; Bank 0
LI    T1, 0        ; Address 0 (HDR_TTY_OUT)
STORE A0, T0, T1
```

### Reading Input
```asm
; Check for input and read if available
LI    T0, 0        ; Bank 0
LI    T1, 3        ; HDR_TTY_IN_STATUS
LOAD  T2, T0, T1
ANDI  T2, T2, 1
BEQ   T2, R0, no_input

LI    T1, 2        ; HDR_TTY_IN_POP
LOAD  A0, T0, T1   ; Read the byte
no_input:
```

### TEXT40 Display
```asm
; Initialize TEXT40 display
LI    A0, 2        ; TEXT40 mode
LI    T0, 0        ; Bank 0
LI    T1, 6        ; HDR_DISP_MODE
STORE A0, T0, T1

LI    A0, 1        ; Enable display
LI    T1, 8        ; HDR_DISP_CTL
STORE A0, T0, T1

; Write "Hi" at top-left
LI    A0, 'H'
LI    T1, 32       ; VRAM[0]
STORE A0, T0, T1

LI    A0, 'i'
LI    T1, 33       ; VRAM[1]
STORE A0, T0, T1

; Flush display
LI    A0, 1
LI    T1, 9        ; HDR_DISP_FLUSH
STORE A0, T0, T1
```

### Random Number Generation
```asm
; Get random number
LI    T0, 0        ; Bank 0
LI    T1, 4        ; HDR_RNG
LOAD  A0, T0, T1   ; Random value in A0
```

## C Runtime Integration

The C runtime library uses these MMIO addresses for standard I/O:

```c
// putchar implementation
void putchar(int c) {
    volatile uint16_t* tty_out = (volatile uint16_t*)0;
    volatile uint16_t* tty_status = (volatile uint16_t*)1;
    
    // Wait for ready
    while ((*tty_status & 1) == 0) {
        // Spin wait
    }
    
    // Output character
    *tty_out = c & 0xFF;
}

// getchar implementation
int getchar(void) {
    volatile uint16_t* tty_in_status = (volatile uint16_t*)3;
    volatile uint16_t* tty_in_pop = (volatile uint16_t*)2;
    
    // Wait for input
    while ((*tty_in_status & 1) == 0) {
        // Spin wait
    }
    
    // Read and return byte
    return *tty_in_pop & 0xFF;
}
```

## Design Rationale

1. **Fixed Addresses**: All MMIO addresses are fixed at compile time, eliminating runtime discovery overhead
2. **Bank 0 Only**: MMIO is only active in bank 0, simplifying implementation and preventing conflicts
3. **Minimal Header**: 32-word header provides space for current devices plus 22 reserved words for future expansion
4. **Efficient Access**: Low addresses (0-31) are optimal for Brainfuck-generated code
5. **Backward Compatible**: Legacy MMIO_OUT and MMIO_OUT_FLAG aliases maintained at addresses 0 and 1

## Constants Reference

```rust
// MMIO Header Addresses
pub const HDR_TTY_OUT: usize       = 0;
pub const HDR_TTY_STATUS: usize    = 1;
pub const HDR_TTY_IN_POP: usize    = 2;
pub const HDR_TTY_IN_STATUS: usize = 3;
pub const HDR_RNG: usize           = 4;
pub const HDR_RNG_SEED: usize      = 5;
pub const HDR_DISP_MODE: usize     = 6;
pub const HDR_DISP_STATUS: usize   = 7;
pub const HDR_DISP_CTL: usize      = 8;
pub const HDR_DISP_FLUSH: usize    = 9;

// TEXT40 VRAM
pub const TEXT40_BASE_WORD: usize  = 32;
pub const TEXT40_WORDS: usize      = 40 * 25;
pub const TEXT40_LAST_WORD: usize  = 1031;

// Status Bits
pub const TTY_READY: u16           = 0x0001;
pub const TTY_HAS_BYTE: u16        = 0x0001;
pub const DISP_READY: u16          = 0x0001;
pub const DISP_FLUSH_DONE: u16     = 0x0002;
pub const DISP_ENABLE: u16         = 0x0001;
pub const DISP_CLEAR: u16          = 0x0002;

// Display Modes
pub const DISP_OFF: u16            = 0;
pub const DISP_TTY: u16            = 1;
pub const DISP_TEXT40: u16         = 2;
```

## Future Enhancements

The reserved MMIO addresses (10-31) are available for future devices such as:
- Timer/counter peripherals
- Additional display modes
- Sound generation
- Network I/O
- Persistent storage
- Interrupt controllers

These can be added without breaking existing code since the header layout is fixed.