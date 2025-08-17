#ifndef MMIO_H
#define MMIO_H

// Memory-Mapped I/O Interface for Ripple VM
// Provides access to hardware devices at fixed memory addresses

#include <mmio_constants.h>

// MMIO access functions (implemented in mmio.c)
unsigned short mmio_read(unsigned short addr);
void mmio_write(unsigned short addr, unsigned short value);

// TTY functions
void tty_putchar(unsigned char c);

// RNG functions
unsigned short rng_get(void);
unsigned short rng_get_seed(void);
void rng_set_seed(unsigned short seed);

// Display functions
void display_set_mode(unsigned short mode);
void display_enable(void);
void display_clear(void);
void display_flush(void);

// TEXT40 VRAM access
void text40_putchar(int x, int y, unsigned char c);
void text40_puts(int x, int y, const char* s);

#endif // MMIO_H