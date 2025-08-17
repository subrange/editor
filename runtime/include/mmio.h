#ifndef MMIO_H
#define MMIO_H

// Memory-Mapped I/O Interface for Ripple VM
// Provides access to hardware devices at fixed memory addresses

// MMIO Header Addresses (bank 0, words 0..31)
#define MMIO_TTY_OUT       0  // Write: output byte (low 8 bits)
#define MMIO_TTY_STATUS    1  // Read: bit0=ready
#define MMIO_TTY_IN_POP    2  // Read: pop next input byte
#define MMIO_TTY_IN_STATUS 3  // Read: bit0=has_byte
#define MMIO_RNG           4  // Read: next random value
#define MMIO_DISP_MODE     5  // R/W: 0=OFF, 1=TTY, 2=TEXT40
#define MMIO_DISP_STATUS   6  // Read: bit0=ready, bit1=flush_done
#define MMIO_DISP_CTL      7  // R/W: bit0=ENABLE, bit1=CLEAR
#define MMIO_DISP_FLUSH    8  // Write: trigger display flush

// Display modes
#define DISP_MODE_OFF    0
#define DISP_MODE_TTY    1
#define DISP_MODE_TEXT40 2

// Display control bits
#define DISP_CTL_ENABLE (1 << 0)
#define DISP_CTL_CLEAR  (1 << 1)

// Display status bits
#define DISP_STATUS_READY      (1 << 0)
#define DISP_STATUS_FLUSH_DONE (1 << 1)

// TTY status bits
#define TTY_STATUS_READY    (1 << 0)
#define TTY_STATUS_HAS_BYTE (1 << 0)

// TEXT40 display constants
#define TEXT40_BASE   32      // VRAM starts at word 32
#define TEXT40_WIDTH  40
#define TEXT40_HEIGHT 25
#define TEXT40_SIZE   1000    // 40*25 words

// MMIO access functions (implemented in mmio.c)
unsigned short mmio_read(unsigned short addr);
void mmio_write(unsigned short addr, unsigned short value);

// TTY functions
void tty_putchar(unsigned char c);

// RNG function  
unsigned short rng_get(void);

// Display functions
void display_set_mode(unsigned short mode);
void display_enable(void);
void display_clear(void);
void display_flush(void);

// TEXT40 VRAM access
void text40_putchar(int x, int y, unsigned char c);
void text40_puts(int x, int y, const char* s);

#endif // MMIO_H