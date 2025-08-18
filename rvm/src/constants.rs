//! Central configuration and constants for the Ripple VM

#![allow(dead_code)]

// Default VM configuration
pub const DEFAULT_BANK_SIZE: u16 = 4096;
pub const DEFAULT_MEMORY_SIZE: usize = 65536 * 65536; // 64K words in 64K banks

// Memory-mapped I/O header addresses (bank 0, words 0..31)
pub const HDR_TTY_OUT: usize       = 0;  // Write: low8 → stdout
pub const HDR_TTY_STATUS: usize    = 1;  // Read: bit0=ready
pub const HDR_TTY_IN_POP: usize    = 2;  // Read: pop next input byte
pub const HDR_TTY_IN_STATUS: usize = 3;  // Read: bit0=has_byte
pub const HDR_RNG: usize           = 4;  // Read: next PRNG value
pub const HDR_RNG_SEED: usize      = 5;  // R/W: RNG seed (low 16 bits)
pub const HDR_DISP_MODE: usize     = 6;  // R/W: 0=OFF, 1=TTY, 2=TEXT40
pub const HDR_DISP_STATUS: usize   = 7;  // Read: bit0=ready, bit1=flush_done
pub const HDR_DISP_CTL: usize      = 8;  // R/W: bit0=ENABLE, bit1=CLEAR
pub const HDR_DISP_FLUSH: usize    = 9;  // Write: trigger display flush
// Words 10..31 are reserved for future use

// TEXT40 display constants
pub const TEXT40_BASE_WORD: usize  = 32;         // Start of VRAM
pub const TEXT40_WORDS: usize      = 40 * 25;    // 1000 words (40x25 cells)
pub const TEXT40_LAST_WORD: usize  = TEXT40_BASE_WORD + TEXT40_WORDS - 1; // 1031

// TTY status bits
pub const TTY_READY: u16 = 1 << 0;      // bit0
pub const TTY_HAS_BYTE: u16 = 1 << 0;   // bit0

// Display mode values
pub const DISP_OFF: u16    = 0;
pub const DISP_TTY: u16    = 1;
pub const DISP_TEXT40: u16 = 2;

// Display status bits
pub const DISP_READY: u16      = 1 << 0;  // bit0
pub const DISP_FLUSH_DONE: u16 = 1 << 1;  // bit1

// Display control bits
pub const DISP_ENABLE: u16 = 1 << 0;    // bit0
pub const DISP_CLEAR: u16  = 1 << 1;    // bit1 (edge-triggered)

// Legacy MMIO addresses (kept for compatibility)
pub const MMIO_OUT: usize = HDR_TTY_OUT;       // Output register
pub const MMIO_OUT_FLAG: usize = HDR_TTY_STATUS;  // Output ready flag

// Memory layout
pub const DATA_SECTION_OFFSET: usize = TEXT40_LAST_WORD + 1; // Data section starts after VRAM (word 1032)

// Instruction encoding
pub const INSTRUCTION_SIZE: usize = 8; // Bytes per instruction

// Binary format magic numbers
pub const MAGIC_RLINK: &[u8] = b"RLINK";

// Debug output configuration
pub const DEBUG_MEMORY_DISPLAY_WORDS: usize = 32; // Number of memory words to display in debug dumps
pub const DEBUG_MEMORY_WORDS_PER_LINE: usize = 8; // Words per line in memory dumps

// VM limits
pub const MAX_REGISTERS: usize = 32;
pub const MIN_MEMORY_SIZE: usize = 256; // Minimum reasonable memory size

// Output flags
pub const OUTPUT_READY: u16 = 1;
pub const OUTPUT_BUSY: u16 = 0;

// Theme Colors - PICO-8 Color Palette RGB values
pub const THEME_COLORS: [(u8, u8, u8); 16] = [
    (0, 0, 0),          // 0: Black
    (29, 43, 83),       // 1: Dark Blue
    (126, 37, 83),      // 2: Dark Purple
    (0, 135, 81),       // 3: Dark Green
    (171, 82, 54),      // 4: Brown
    (95, 87, 79),       // 5: Dark Gray
    (194, 195, 199),    // 6: Light Gray
    (255, 241, 232),    // 7: White
    (255, 0, 77),       // 8: Red
    (255, 163, 0),      // 9: Orange
    (255, 236, 39),     // 10: Yellow
    (0, 228, 54),       // 11: Green
    (41, 173, 255),     // 12: Blue
    (131, 118, 156),    // 13: Indigo
    (255, 119, 168),    // 14: Pink
    (255, 204, 170),    // 15: Peach
];