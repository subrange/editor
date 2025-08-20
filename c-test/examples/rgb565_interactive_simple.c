// Very simple interactive demo for RGB565
// Move a square with arrow keys

#include <graphics.h>
#include <mmio.h>

int main() {
    // Initialize display
    graphics_init(64, 64);
    
    // Player position
    int player_x = 30;
    int player_y = 30;
    
    // Simple game loop - just 2000 frames
    for (int frame = 0; frame < 2000; frame++) {
        // Clear screen
        clear_screen(rgb565(10, 10, 30));  // Dark blue
        
        // Read keys and move player
        if (key_left_pressed() && player_x > 0) {
            player_x--;
        }
        if (key_right_pressed() && player_x < 58) {
            player_x++;
        }
        if (key_up_pressed() && player_y > 0) {
            player_y--;
        }
        if (key_down_pressed() && player_y < 58) {
            player_y++;
        }
        
        // Draw border
        draw_rect(0, 0, 64, 64, RGB_WHITE);
        

        
        // Draw some obstacles
        fill_rect(10, 10, 8, 8, RGB_GREEN);
        fill_rect(46, 10, 8, 8, RGB_GREEN);
        fill_rect(10, 46, 8, 8, RGB_GREEN);
        fill_rect(46, 46, 8, 8, RGB_GREEN);
        
        // Draw center target
        fill_rect(29, 29, 6, 6, RGB_YELLOW);

        // Draw player as a red square
        fill_rect(player_x, player_y, 6, 6, RGB_RED);
        
        // Flush to display
        graphics_flush();
        
        // Small delay
        for (int d = 0; d < 5000; d++);
    }
    
    // End screen
    clear_screen(RGB_BLACK);
    fill_rect(20, 28, 24, 8, RGB_GREEN);
    graphics_flush();
    
    return 0;
}