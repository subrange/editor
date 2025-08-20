// RGB565 Basic Shapes Demo
// Demonstrates drawing basic shapes and colors in RGB565 graphics mode

#include <stdio.h>
#include<mmio.h>

// MMIO addresses
#define MMIO_DISP_MODE       6
#define MMIO_DISP_FLUSH      9
#define MMIO_DISP_RESOLUTION 16

// Display modes
#define DISP_MODE_RGB565     3

// Display dimensions (small for quick testing)
#define WIDTH  32
#define HEIGHT 32

// RGB565 color conversion
unsigned short rgb565(unsigned char r, unsigned char g, unsigned char b) {
    return ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3);
}

// Predefined colors
#define COLOR_BLACK   rgb565(0, 0, 0)
#define COLOR_WHITE   rgb565(255, 255, 255)
#define COLOR_RED     rgb565(255, 0, 0)
#define COLOR_GREEN   rgb565(0, 255, 0)
#define COLOR_BLUE    rgb565(0, 0, 255)
#define COLOR_YELLOW  rgb565(255, 255, 0)
#define COLOR_CYAN    rgb565(0, 255, 255)
#define COLOR_MAGENTA rgb565(255, 0, 255)

// Set a pixel in the back buffer
void set_pixel(unsigned short x, unsigned short y, unsigned short color) {
    if (x >= WIDTH || y >= HEIGHT) return;
    
    // Calculate address in back buffer
    // Back buffer starts at: 32 (MMIO) + WIDTH*HEIGHT (front buffer)
    unsigned short addr = 32 + WIDTH * HEIGHT + y * WIDTH + x;
    mmio_write(addr, color);
}

// Fill entire screen with a color
void clear_screen(unsigned short color) {
    for (unsigned short y = 0; y < HEIGHT; y++) {
        for (unsigned short x = 0; x < WIDTH; x++) {
            set_pixel(x, y, color);
        }
    }
}

// Draw a horizontal line
void draw_hline(unsigned short x, unsigned short y, unsigned short width, unsigned short color) {
    for (unsigned short i = 0; i < width; i++) {
        set_pixel(x + i, y, color);
    }
}

// Draw a vertical line
void draw_vline(unsigned short x, unsigned short y, unsigned short height, unsigned short color) {
    for (unsigned short i = 0; i < height; i++) {
        set_pixel(x, y + i, color);
    }
}

// Draw a rectangle
void draw_rect(unsigned short x, unsigned short y, unsigned short w, unsigned short h, unsigned short color) {
    draw_hline(x, y, w, color);           // Top
    draw_hline(x, y + h - 1, w, color);   // Bottom
    draw_vline(x, y, h, color);           // Left
    draw_vline(x + w - 1, y, h, color);   // Right
}

// Draw a filled rectangle
void fill_rect(unsigned short x, unsigned short y, unsigned short w, unsigned short h, unsigned short color) {
    for (unsigned short dy = 0; dy < h; dy++) {
        for (unsigned short dx = 0; dx < w; dx++) {
            set_pixel(x + dx, y + dy, color);
        }
    }
}

// Draw a simple circle (using midpoint algorithm)
void draw_circle(unsigned short cx, unsigned short cy, unsigned short r, unsigned short color) {
    short x = r;
    short y = 0;
    short err = 0;
    
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

int main() {
    // Set display resolution
    unsigned short resolution = (WIDTH << 8) | HEIGHT;
    mmio_write(MMIO_DISP_RESOLUTION, resolution);
    
    // Set display mode to RGB565
    mmio_write(MMIO_DISP_MODE, DISP_MODE_RGB565);
    
    // Clear screen to black
    clear_screen(COLOR_BLACK);
    
    // Draw some shapes
    
    // Red rectangle in top-left
    fill_rect(2, 2, 8, 8, COLOR_RED);
    
    // Green rectangle outline
    draw_rect(12, 2, 8, 8, COLOR_GREEN);
    
    // Blue circle
    draw_circle(6, 20, 5, COLOR_BLUE);
    
    // Yellow horizontal lines
    draw_hline(22, 14, 8, COLOR_YELLOW);
    draw_hline(22, 16, 8, COLOR_YELLOW);
    draw_hline(22, 18, 8, COLOR_YELLOW);
    
    // Cyan vertical lines
    draw_vline(24, 22, 8, COLOR_CYAN);
    draw_vline(26, 22, 8, COLOR_CYAN);
    draw_vline(28, 22, 8, COLOR_CYAN);
    
    // Magenta filled circle in bottom-right
    for (unsigned short r = 0; r < 4; r++) {
        draw_circle(20, 20, r, COLOR_MAGENTA);
    }
    
    // White border around the screen
    draw_rect(0, 0, WIDTH, HEIGHT, COLOR_WHITE);
    
    // Flush display to show the shapes
    mmio_write(MMIO_DISP_FLUSH, 1);
    
    // Animation: Move a small square around
    unsigned short px = 15, py = 15;
    short dx = 1, dy = 1;
    
    for (int frame = 0; frame < 100; frame++) {
        // Clear the old position
        fill_rect(px, py, 3, 3, COLOR_BLACK);
        
        // Update position
        px += dx;
        py += dy;
        
        // Bounce off walls
        if (px <= 1 || px >= WIDTH - 4) dx = -dx;
        if (py <= 1 || py >= HEIGHT - 4) dy = -dy;
        
        // Draw at new position
        fill_rect(px, py, 3, 3, COLOR_WHITE);
        
        // Redraw border (in case it was overwritten)
        draw_rect(0, 0, WIDTH, HEIGHT, COLOR_WHITE);
        
        // Flush display
        mmio_write(MMIO_DISP_FLUSH, 1);
        
        // Simple delay
        for (int i = 0; i < 5000; i++);
    }
    
    // Print completion message
    puts("RGB565 shapes demo completed!");
    
    return 0;
}