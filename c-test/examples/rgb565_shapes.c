// RGB565 Basic Shapes Demo
// Demonstrates drawing basic shapes and colors in RGB565 graphics mode

#include <stdio.h>
#include <graphics.h>

// Display dimensions (small for quick testing)
#define WIDTH  32
#define HEIGHT 32

int main() {
    // Initialize graphics with resolution
    graphics_init(WIDTH, HEIGHT);
    
    // Clear screen to black
    clear_screen(RGB_BLACK);
    
    // Draw some shapes
    
    // Red rectangle in top-left
    fill_rect(2, 2, 8, 8, RGB_RED);
    
    // Green rectangle outline
    draw_rect(12, 2, 8, 8, RGB_GREEN);
    
    // Blue circle
    draw_circle(6, 20, 5, RGB_BLUE);
    
    // Yellow horizontal lines
    draw_hline(22, 14, 8, RGB_YELLOW);
    draw_hline(22, 16, 8, RGB_YELLOW);
    draw_hline(22, 18, 8, RGB_YELLOW);
    
    // Cyan vertical lines
    draw_vline(24, 22, 8, RGB_CYAN);
    draw_vline(26, 22, 8, RGB_CYAN);
    draw_vline(28, 22, 8, RGB_CYAN);
    
    // Magenta filled circle in bottom-right
    for (unsigned short r = 0; r < 4; r++) {
        draw_circle(20, 20, r, RGB_MAGENTA);
    }
    
    // White border around the screen
    draw_rect(0, 0, WIDTH, HEIGHT, RGB_WHITE);
    
    // Flush display to show the shapes
    graphics_flush();
    
    // Animation: Move a small square around
    unsigned short px = 15, py = 15;
    short dx = 1, dy = 1;
    
    for (int frame = 0; frame < 100; frame++) {
        // Clear the old position
        fill_rect(px, py, 3, 3, RGB_BLACK);
        
        // Update position
        px += dx;
        py += dy;
        
        // Bounce off walls
        if (px <= 1 || px >= WIDTH - 4) dx = -dx;
        if (py <= 1 || py >= HEIGHT - 4) dy = -dy;
        
        // Draw at new position
        fill_rect(px, py, 3, 3, RGB_WHITE);
        
        // Redraw border (in case it was overwritten)
        draw_rect(0, 0, WIDTH, HEIGHT, RGB_WHITE);
        
        // Flush display
        graphics_flush();
        
        // Simple delay
        for (int i = 0; i < 5000; i++);
    }
    
    // Print completion message
    puts("RGB565 shapes demo completed!");
    
    return 0;
}