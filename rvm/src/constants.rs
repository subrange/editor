//! Central configuration and constants for the Ripple VM

// Default VM configuration
pub const DEFAULT_BANK_SIZE: u16 = 4096;
pub const DEFAULT_MEMORY_SIZE: usize = 65536 * 65536; // 64K words in 64K banks

// Memory-mapped I/O addresses
pub const MMIO_OUT: usize = 0;       // Output register
pub const MMIO_OUT_FLAG: usize = 1;  // Output ready flag (1 = ready, 0 = busy)

// Memory layout
pub const DATA_SECTION_OFFSET: usize = 2; // Data section starts after MMIO registers (same as MMIO count)

// Instruction encoding
pub const INSTRUCTION_SIZE: usize = 8; // Bytes per instruction

// Binary format magic numbers
pub const MAGIC_RLINK: &[u8] = b"RLINK";

// Debug output configuration
pub const DEBUG_MEMORY_DISPLAY_WORDS: usize = 32; // Number of memory words to display in debug dumps
pub const DEBUG_MEMORY_WORDS_PER_LINE: usize = 8; // Words per line in memory dumps

// VM limits
pub const MAX_REGISTERS: usize = 18;
pub const MIN_MEMORY_SIZE: usize = 256; // Minimum reasonable memory size

// Output flags
pub const OUTPUT_READY: u16 = 1;
pub const OUTPUT_BUSY: u16 = 0;