// MMIO access functions for Ripple VM
// These provide memory-mapped I/O access without inline assembly

#include <mmio_constants.h>

// Read from MMIO address (bank 0)
unsigned short mmio_read(unsigned short addr) {
    unsigned short result;
    // Use extended inline assembly with output operand
    // %0 is the output (result), %1 is the input (addr)
    __asm__("LOAD %0, R0, %1" : "=r"(result) : "r"(addr));
    return result;
}

// Write to MMIO address (bank 0)
void mmio_write(unsigned short addr, unsigned short value) {
    // Use extended inline assembly with input operands
    // %0 is value, %1 is addr
    __asm__("STORE %0, R0, %1" : : "r"(value), "r"(addr));
}

// TTY output
void tty_putchar(unsigned char c) {
    mmio_write(MMIO_TTY_OUT, c);
}

// Get random number
unsigned short rng_get(void) {
    return mmio_read(MMIO_RNG);
}

// Get current RNG seed (low 16 bits)
unsigned short rng_get_seed(void) {
    return mmio_read(MMIO_RNG_SEED);
}

// Set RNG seed (low 16 bits)
void rng_set_seed(unsigned short seed) {
    mmio_write(MMIO_RNG_SEED, seed);
}

// Display control
void display_set_mode(unsigned short mode) {
    mmio_write(MMIO_DISP_MODE, mode);
}

void display_enable(void) {
    mmio_write(MMIO_DISP_CTL, DISP_CTL_ENABLE);
}

void display_clear(void) {
    mmio_write(MMIO_DISP_CTL, DISP_CTL_ENABLE | DISP_CTL_CLEAR);
}

void display_flush(void) {
    mmio_write(MMIO_DISP_FLUSH, 1);
}

// TEXT40 VRAM access
void text40_putchar(int x, int y, unsigned char c) {
    if (x >= 0 && x < TEXT40_WIDTH && y >= 0 && y < TEXT40_HEIGHT) {
        unsigned short addr = TEXT40_BASE + y * TEXT40_WIDTH + x;
        // In our architecture, char is 16-bit, so mask to get only low 8 bits
        // Write character in low byte, attributes (0) in high byte
        unsigned short value = (c & 0xFF);
        mmio_write(addr, value);
    }
}

void text40_puts(int x, int y, const char* s) {
    int pos = x;
    int i = 0;
    char ch;
    
    // Use index-based access instead of pointer arithmetic
    while ((ch = s[i]) != 0 && pos < TEXT40_WIDTH) {
        // Mask the character to 8 bits since char is 16-bit in our architecture
        text40_putchar(pos, y, ch & 0xFF);
        pos += 1;
        i += 1;
    }
}