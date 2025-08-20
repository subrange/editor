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

// TEXT40 colored character functions
void text40_putchar_color(int x, int y, unsigned char c, unsigned char fg, unsigned char bg);
void text40_putchar_attr(int x, int y, unsigned char c, unsigned char attr);
void text40_puts_color(int x, int y, const char* s, unsigned char fg, unsigned char bg);
void text40_puts_attr(int x, int y, const char* s, unsigned char attr);

// TEXT40 attribute functions
void text40_set_attr(int x, int y, unsigned char attr);
unsigned char text40_get_char(int x, int y);
unsigned char text40_get_attr(int x, int y);

// Keyboard input functions (TEXT40 mode only)
int key_pressed(unsigned short key_addr);
int key_up_pressed(void);
int key_down_pressed(void);
int key_left_pressed(void);
int key_right_pressed(void);
int key_z_pressed(void);
int key_x_pressed(void);

#endif // MMIO_H