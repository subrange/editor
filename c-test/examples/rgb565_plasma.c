
// RGB565 Plasma Effect Demo
// Shows colorful animated plasma pattern using RGB565 graphics mode

#include <stdio.h>
#include <graphics.h>

// Display dimensions
#define WIDTH  64
#define HEIGHT 64

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
    // Initialize graphics with resolution
    graphics_init(WIDTH, HEIGHT);

    // Animation loop
    unsigned char time = 0;
    for (int frame = 0; frame < 200; frame++) {
        // Draw plasma effect
        draw_plasma(time);

        // Flush display (swap buffers)
        graphics_flush();
        
        // Update time for animation
        time += 2;
        
        // Simple delay
        for (int i = 0; i < 1000; i++);
    }
    
    // Print completion message
    puts("Plasma effect completed!");
    
    return 0;
}