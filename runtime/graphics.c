// RGB565 Graphics Library Implementation
// Provides functions for drawing in RGB565 graphics mode on Ripple VM

#include "graphics.h"
#include "mmio.h"
#include "mmio_constants.h"

// Current display dimensions
static unsigned char display_width = 0;
static unsigned char display_height = 0;

// Convert RGB888 to RGB565
unsigned short rgb565(unsigned char r, unsigned char g, unsigned char b) {
    return ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3);
}

// Initialize graphics mode
void graphics_init(unsigned char width, unsigned char height) {
    display_width = width;
    display_height = height;
    
    // Set display resolution (hi8=width, lo8=height)
    unsigned short resolution = ((unsigned short)width << 8) | height;
    mmio_write(MMIO_DISP_RESOLUTION, resolution);
    
    // Set display mode to RGB565
    mmio_write(MMIO_DISP_MODE, DISP_MODE_RGB565);
}

// Set a pixel in the back buffer
void set_pixel(unsigned short x, unsigned short y, unsigned short color) {
    if (x >= display_width || y >= display_height) return;
    
    // Calculate address in back buffer
    // Memory layout: 32 MMIO + front_buffer + back_buffer
    // Back buffer starts at: 32 + (width * height)
    unsigned int pixel_count = (unsigned int)display_width * display_height;
    unsigned int back_buffer_start = 32 + pixel_count;
    unsigned int addr = back_buffer_start + y * display_width + x;
    
    mmio_write(addr, color);
}

// Get a pixel from the back buffer
unsigned short get_pixel(unsigned short x, unsigned short y) {
    if (x >= display_width || y >= display_height) return 0;
    
    unsigned int pixel_count = (unsigned int)display_width * display_height;
    unsigned int back_buffer_start = 32 + pixel_count;
    unsigned int addr = back_buffer_start + y * display_width + x;
    
    return mmio_read(addr);
}

// Clear the entire screen
void clear_screen(unsigned short color) {
    for (unsigned short y = 0; y < display_height; y++) {
        for (unsigned short x = 0; x < display_width; x++) {
            set_pixel(x, y, color);
        }
    }
}

// Swap front and back buffers
void graphics_flush(void) {
    mmio_write(MMIO_DISP_FLUSH, 1);
}

// Draw a horizontal line
void draw_hline(unsigned short x, unsigned short y, unsigned short width, unsigned short color) {
    for (unsigned short i = 0; i < width && (x + i) < display_width; i++) {
        set_pixel(x + i, y, color);
    }
}

// Draw a vertical line
void draw_vline(unsigned short x, unsigned short y, unsigned short height, unsigned short color) {
    for (unsigned short i = 0; i < height && (y + i) < display_height; i++) {
        set_pixel(x, y + i, color);
    }
}

// Draw a line using Bresenham's algorithm
void draw_line(unsigned short x1, unsigned short y1, unsigned short x2, unsigned short y2, unsigned short color) {
    int dx = (x2 > x1) ? (x2 - x1) : (x1 - x2);
    int dy = (y2 > y1) ? (y2 - y1) : (y1 - y2);
    int sx = (x1 < x2) ? 1 : -1;
    int sy = (y1 < y2) ? 1 : -1;
    int err = dx - dy;
    
    while (1) {
        set_pixel(x1, y1, color);
        
        if (x1 == x2 && y1 == y2) break;
        
        int e2 = 2 * err;
        if (e2 > -dy) {
            err -= dy;
            x1 += sx;
        }
        if (e2 < dx) {
            err += dx;
            y1 += sy;
        }
    }
}

// Draw a rectangle outline
void draw_rect(unsigned short x, unsigned short y, unsigned short w, unsigned short h, unsigned short color) {
    if (w == 0 || h == 0) return;
    
    draw_hline(x, y, w, color);                    // Top
    if (h > 1) {
        draw_hline(x, y + h - 1, w, color);        // Bottom
    }
    if (h > 2) {
        draw_vline(x, y + 1, h - 2, color);        // Left
        if (w > 1) {
            draw_vline(x + w - 1, y + 1, h - 2, color); // Right
        }
    }
}

// Draw a filled rectangle
void fill_rect(unsigned short x, unsigned short y, unsigned short w, unsigned short h, unsigned short color) {
    for (unsigned short dy = 0; dy < h && (y + dy) < display_height; dy++) {
        for (unsigned short dx = 0; dx < w && (x + dx) < display_width; dx++) {
            set_pixel(x + dx, y + dy, color);
        }
    }
}

// Draw a circle using midpoint algorithm
void draw_circle(unsigned short cx, unsigned short cy, unsigned short radius, unsigned short color) {
    int x = radius;
    int y = 0;
    int err = 0;
    
    while (x >= y) {
        set_pixel(cx + x, cy + y, color);
        set_pixel(cx + y, cy + x, color);
        set_pixel(cx - y, cy + x, color);
        set_pixel(cx - x, cy + y, color);
        set_pixel(cx - x, cy - y, color);
        set_pixel(cx - y, cy - x, color);
        set_pixel(cx + y, cy - x, color);
        set_pixel(cx + x, cy - y, color);
        
        if (err <= 0) {
            y += 1;
            err += 2 * y + 1;
        }
        if (err > 0) {
            x -= 1;
            err -= 2 * x + 1;
        }
    }
}

// Draw a filled circle
void fill_circle(unsigned short cx, unsigned short cy, unsigned short radius, unsigned short color) {
    for (unsigned short r = 0; r <= radius; r++) {
        draw_circle(cx, cy, r, color);
    }
}

// Draw a single character (stub implementation for now)
void draw_char(unsigned short x, unsigned short y, char c, unsigned short color) {
    // TODO: Implement font rendering
    // For now, just draw a small box to represent a character
    if (c != ' ') {
        fill_rect(x, y, 3, 5, color);
    }
}

// Draw a string
void draw_string(unsigned short x, unsigned short y, const char* str, unsigned short color) {
    unsigned short curr_x = x;
    while (*str) {
        draw_char(curr_x, y, *str, color);
        curr_x += 6; // 5 pixels wide + 1 pixel spacing
        str++;
    }
}

// Get display width
unsigned char graphics_width(void) {
    return display_width;
}

// Get display height
unsigned char graphics_height(void) {
    return display_height;
}

// Sine lookup table for fast trigonometric calculations
const unsigned char sin_table[256] = {
  0x80, 0x83, 0x86, 0x89, 0x8C, 0x90, 0x93, 0x96,
  0x99, 0x9C, 0x9F, 0xA2, 0xA5, 0xA8, 0xAB, 0xAE,
  0xB1, 0xB3, 0xB6, 0xB9, 0xBC, 0xBF, 0xC1, 0xC4,
  0xC7, 0xC9, 0xCC, 0xCE, 0xD1, 0xD3, 0xD5, 0xD8,
  0xDA, 0xDC, 0xDE, 0xE0, 0xE2, 0xE4, 0xE6, 0xE8,
  0xEA, 0xEB, 0xED, 0xEF, 0xF0, 0xF1, 0xF3, 0xF4,
  0xF5, 0xF6, 0xF8, 0xF9, 0xFA, 0xFA, 0xFB, 0xFC,
  0xFD, 0xFD, 0xFE, 0xFE, 0xFE, 0xFF, 0xFF, 0xFF,
  0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFE, 0xFE, 0xFD,
  0xFD, 0xFC, 0xFB, 0xFA, 0xFA, 0xF9, 0xF8, 0xF6,
  0xF5, 0xF4, 0xF3, 0xF1, 0xF0, 0xEF, 0xED, 0xEB,
  0xEA, 0xE8, 0xE6, 0xE4, 0xE2, 0xE0, 0xDE, 0xDC,
  0xDA, 0xD8, 0xD5, 0xD3, 0xD1, 0xCE, 0xCC, 0xC9,
  0xC7, 0xC4, 0xC1, 0xBF, 0xBC, 0xB9, 0xB6, 0xB3,
  0xB1, 0xAE, 0xAB, 0xA8, 0xA5, 0xA2, 0x9F, 0x9C,
  0x99, 0x96, 0x93, 0x90, 0x8C, 0x89, 0x86, 0x83,
  0x80, 0x7D, 0x7A, 0x77, 0x74, 0x70, 0x6D, 0x6A,
  0x67, 0x64, 0x61, 0x5E, 0x5B, 0x58, 0x55, 0x52,
  0x4F, 0x4D, 0x4A, 0x47, 0x44, 0x41, 0x3F, 0x3C,
  0x39, 0x37, 0x34, 0x32, 0x2F, 0x2D, 0x2B, 0x28,
  0x26, 0x24, 0x22, 0x20, 0x1E, 0x1C, 0x1A, 0x18,
  0x16, 0x15, 0x13, 0x11, 0x10, 0x0F, 0x0D, 0x0C,
  0x0B, 0x0A, 0x08, 0x07, 0x06, 0x06, 0x05, 0x04,
  0x03, 0x03, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01,
  0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x03,
  0x03, 0x04, 0x05, 0x06, 0x06, 0x07, 0x08, 0x0A,
  0x0B, 0x0C, 0x0D, 0x0F, 0x10, 0x11, 0x13, 0x15,
  0x16, 0x18, 0x1A, 0x1C, 0x1E, 0x20, 0x22, 0x24,
  0x26, 0x28, 0x2B, 0x2D, 0x2F, 0x32, 0x34, 0x37,
  0x39, 0x3C, 0x3F, 0x41, 0x44, 0x47, 0x4A, 0x4D,
  0x4F, 0x52, 0x55, 0x58, 0x5B, 0x5E, 0x61, 0x64,
  0x67, 0x6A, 0x6D, 0x70, 0x74, 0x77, 0x7A, 0x7D
};

// Fast sine function using lookup table
unsigned char fast_sin(unsigned char angle) {
    return sin_table[angle];
}