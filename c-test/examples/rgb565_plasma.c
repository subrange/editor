// RGB565 Plasma Effect Demo
// Shows colorful animated plasma pattern using RGB565 graphics mode

#include <stdio.h>
#include <mmio_constants.h>

// Display dimensions
#define WIDTH  64
#define HEIGHT 64

// External MMIO functions from runtime
extern void mmio_write(unsigned short addr, unsigned short value);
extern void display_set_mode(unsigned short mode);
extern void display_set_resolution(unsigned char width, unsigned char height);
extern void display_flush(void);

// RGB565 color conversion
unsigned short rgb565(unsigned char r, unsigned char g, unsigned char b) {
    return ((r & 0xF8) << 8) | ((g & 0xFC) << 3) | (b >> 3);
}

// Fast sine approximation using lookup table
const unsigned char sin_table[256] = {
    128, 131, 134, 137, 140, 144, 147, 150, 153, 156, 159, 162, 165, 168, 171, 174,
    177, 179, 182, 185, 188, 191, 193, 196, 199, 201, 204, 206, 209, 211, 213, 216,
    218, 220, 222, 224, 226, 228, 230, 232, 234, 235, 237, 239, 240, 241, 243, 244,
    245, 246, 248, 249, 250, 250, 251, 252, 253, 253, 254, 254, 254, 255, 255, 255,
    255, 255, 255, 255, 254, 254, 254, 253, 253, 252, 251, 250, 250, 249, 248, 246,
    245, 244, 243, 241, 240, 239, 237, 235, 234, 232, 230, 228, 226, 224, 222, 220,
    218, 216, 213, 211, 209, 206, 204, 201, 199, 196, 193, 191, 188, 185, 182, 179,
    177, 174, 171, 168, 165, 162, 159, 156, 153, 150, 147, 144, 140, 137, 134, 131,
    128, 124, 121, 118, 115, 111, 108, 105, 102, 99, 96, 93, 90, 87, 84, 81,
    78, 76, 73, 70, 67, 64, 62, 59, 56, 54, 51, 49, 46, 44, 42, 39,
    37, 35, 33, 31, 29, 27, 25, 23, 21, 20, 18, 16, 15, 14, 12, 11,
    10, 9, 7, 6, 5, 5, 4, 3, 2, 2, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 1, 1, 1, 2, 2, 3, 4, 5, 5, 6, 7, 9,
    10, 11, 12, 14, 15, 16, 18, 20, 21, 23, 25, 27, 29, 31, 33, 35,
    37, 39, 42, 44, 46, 49, 51, 54, 56, 59, 62, 64, 67, 70, 73, 76,
    78, 81, 84, 87, 90, 93, 96, 99, 102, 105, 108, 111, 115, 118, 121, 124
};

// Fast sine function
unsigned char fast_sin(unsigned char angle) {
    return sin_table[angle];
}

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

// Main plasma effect
void draw_plasma(unsigned char time) {
    for (unsigned char y = 0; y < HEIGHT; y++) {
        for (unsigned char x = 0; x < WIDTH; x++) {
            // Calculate plasma value using multiple sine waves
            unsigned char v1 = fast_sin(x + time);
            unsigned char v2 = fast_sin((y << 1) + time);
            unsigned char v3 = fast_sin((x + y + time) >> 1);
            unsigned char v4 = fast_sin(((x * x + y * y) >> 3) + time);
            
            // Combine the waves
            unsigned char plasma = (v1 + v2 + v3 + v4) >> 2;
            
            // Convert plasma value to RGB565 color
            unsigned char r, g, b;
            
            // Create a color gradient
            if (plasma < 85) {
                r = plasma * 3;
                g = 0;
                b = 255 - plasma * 3;
            } else if (plasma < 170) {
                r = 255;
                g = (plasma - 85) * 3;
                b = 0;
            } else {
                r = 255 - (plasma - 170) * 3;
                g = 255;
                b = (plasma - 170) * 3;
            }
            
            unsigned short color = rgb565(r, g, b);
            set_pixel(x, y, color);
        }
    }
}

int main() {
    // Set display resolution
    display_set_resolution(WIDTH, HEIGHT);
    
    // Set display mode to RGB565
    display_set_mode(DISP_MODE_RGB565);
    
    // Animation loop
    unsigned char time = 0;
    for (int frame = 0; frame < 200; frame++) {
        // Draw plasma effect
        draw_plasma(time);
        
        // Flush display (swap buffers)
        display_flush();
        
        // Update time for animation
        time += 2;
        
        // Simple delay
        for (int i = 0; i < 1000; i++);
    }
    
    // Print completion message
    puts("Plasma effect completed!");
    
    return 0;
}