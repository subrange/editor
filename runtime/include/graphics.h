#ifndef GRAPHICS_H
#define GRAPHICS_H

// RGB565 Graphics Library for Ripple VM
// Provides functions for drawing in RGB565 graphics mode

// RGB565 color format: RRRRRGGGGGGBBBBB (16-bit)
// 5 bits red, 6 bits green, 5 bits blue

// Convert RGB888 (8-bit per channel) to RGB565
unsigned short rgb565(unsigned char r, unsigned char g, unsigned char b);

// Convert RGB values (0-255) to RGB565 using macros for compile-time conversion
#define RGB565(r, g, b) (((r & 0xF8) << 8) | ((g & 0xFC) << 3) | ((b >> 3)))

// Common colors in RGB565 format
// Prefixed with RGB_ to avoid conflicts with TEXT40 palette indices
#define RGB_BLACK     0x0000
#define RGB_WHITE     0xFFFF
#define RGB_RED       0xF800
#define RGB_GREEN     0x07E0
#define RGB_BLUE      0x001F
#define RGB_YELLOW    0xFFE0
#define RGB_CYAN      0x07FF
#define RGB_MAGENTA   0xF81F
#define RGB_ORANGE    0xFD20
#define RGB_PINK      0xF81F
#define RGB_GRAY      0x8410
#define RGB_DARK_GRAY 0x4208
#define RGB_PURPLE    0x8010
#define RGB_BROWN     0xA145

// Initialize graphics mode with specified resolution
// Width and height must fit in available memory: (bank_size - 32) / 2 pixels total
void graphics_init(unsigned char width, unsigned char height);

// Set a single pixel at (x, y) with given color
void set_pixel(unsigned short x, unsigned short y, unsigned short color);

// Get the color of a pixel at (x, y)
unsigned short get_pixel(unsigned short x, unsigned short y);

// Clear the entire screen with a color
void clear_screen(unsigned short color);

// Swap front and back buffers (display the drawn frame)
void graphics_flush(void);

// Drawing primitives

// Draw a horizontal line
void draw_hline(unsigned short x, unsigned short y, unsigned short width, unsigned short color);

// Draw a vertical line
void draw_vline(unsigned short x, unsigned short y, unsigned short height, unsigned short color);

// Draw a line from (x1, y1) to (x2, y2)
void draw_line(unsigned short x1, unsigned short y1, unsigned short x2, unsigned short y2, unsigned short color);

// Draw a rectangle outline
void draw_rect(unsigned short x, unsigned short y, unsigned short w, unsigned short h, unsigned short color);

// Draw a filled rectangle
void fill_rect(unsigned short x, unsigned short y, unsigned short w, unsigned short h, unsigned short color);

// Draw a circle outline
void draw_circle(unsigned short cx, unsigned short cy, unsigned short radius, unsigned short color);

// Draw a filled circle
void fill_circle(unsigned short cx, unsigned short cy, unsigned short radius, unsigned short color);

// Draw a single character at position
void draw_char(unsigned short x, unsigned short y, char c, unsigned short color);

// Draw a string at position
void draw_string(unsigned short x, unsigned short y, const char* str, unsigned short color);

// Utility functions

// Get current display width
unsigned char graphics_width(void);

// Get current display height
unsigned char graphics_height(void);

// Math utilities for graphics

// Fast sine function using lookup table (0-255 input, 0-255 output)
unsigned char fast_sin(unsigned char angle);

#endif // GRAPHICS_H