// RGB565 Gradient Test
// Tests color gradients to verify RGB565 conversion is working

#include <stdio.h>
#include <graphics.h>

// Display dimensions
#define WIDTH  64
#define HEIGHT 64

int main() {
    // Initialize graphics
    graphics_init(WIDTH, HEIGHT);
    
    puts("Drawing color gradients...");
    
    // Draw horizontal red gradient (top section)
    for (unsigned char x = 0; x < WIDTH; x++) {
        unsigned char red = (x * 255) / WIDTH;
        for (unsigned char y = 0; y < 16; y++) {
            set_pixel(x, y, rgb565(red, 0, 0));
        }
    }
    
    // Draw horizontal green gradient (second section)
    for (unsigned char x = 0; x < WIDTH; x++) {
        unsigned char green = (x * 255) / WIDTH;
        for (unsigned char y = 16; y < 32; y++) {
            set_pixel(x, y, rgb565(0, green, 0));
        }
    }
    
    // Draw horizontal blue gradient (third section)
    for (unsigned char x = 0; x < WIDTH; x++) {
        unsigned char blue = (x * 255) / WIDTH;
        for (unsigned char y = 32; y < 48; y++) {
            set_pixel(x, y, rgb565(0, 0, blue));
        }
    }
    
    // Draw rainbow gradient (bottom section)
    for (unsigned char x = 0; x < WIDTH; x++) {
        // Create rainbow using sine waves
        unsigned char angle = (x * 255) / WIDTH;
        unsigned char r = fast_sin(angle);
        unsigned char g = fast_sin(angle + 85);  // 120 degrees
        unsigned char b = fast_sin(angle + 170); // 240 degrees
        
        for (unsigned char y = 48; y < HEIGHT; y++) {
            set_pixel(x, y, rgb565(r, g, b));
        }
    }
    
    // Show the gradients
    graphics_flush();
    
    // Keep displaying for a while
    for (int i = 0; i < 100000; i++);
    
    puts("Gradient test completed!");
    
    // Now animate through different solid colors
    puts("Testing solid colors...");
    
    for (int frame = 0; frame < 256; frame += 8) {
        // Cycle through hues
        unsigned char r = fast_sin(frame);
        unsigned char g = fast_sin(frame + 85);
        unsigned char b = fast_sin(frame + 170);
        
        clear_screen(rgb565(r, g, b));
        graphics_flush();
        
        // Delay
        for (int i = 0; i < 10000; i++);
    }
    
    puts("Color test completed!");
    
    return 0;
}