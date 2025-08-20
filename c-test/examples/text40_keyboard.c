// Interactive keyboard input demonstration for TEXT40 mode
// Shows arrow keys and Z/X key states with colorful visual feedback

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Simple delay function
void delay(int count) {
    int i;
    for (i = 0; i < count; i++) {
        int dummy = i * 2; // Some work to prevent optimization
    }
}

// Draw a fancy border around the screen
void draw_border(void) {
    // Top and bottom borders with gradient effect
    for (int i = 0; i < 40; i++) {
        unsigned char color = COLOR_DARK_BLUE + (i % 4);
        text40_putchar_color(i, 0, '=', COLOR_WHITE, color);
        text40_putchar_color(i, 24, '=', COLOR_WHITE, color);
    }
    
    // Side borders
    for (int i = 1; i < 24; i++) {
        text40_putchar_color(0, i, '|', COLOR_BLUE, COLOR_BLACK);
        text40_putchar_color(39, i, '|', COLOR_BLUE, COLOR_BLACK);
    }
    
    // Fancy corners
    text40_putchar_color(0, 0, '+', COLOR_YELLOW, COLOR_RED);
    text40_putchar_color(39, 0, '+', COLOR_YELLOW, COLOR_ORANGE);
    text40_putchar_color(0, 24, '+', COLOR_YELLOW, COLOR_GREEN);
    text40_putchar_color(39, 24, '+', COLOR_YELLOW, COLOR_BLUE);
}

// Draw arrow key visual at specified position
void draw_arrow_key(int x, int y, char arrow, int pressed) {
    unsigned char fg, bg;
    if (pressed) {
        fg = COLOR_BLACK;
        bg = COLOR_YELLOW;  // Bright yellow when pressed
    } else {
        fg = COLOR_LIGHT_GRAY;
        bg = COLOR_DARK_GRAY;  // Dark gray when not pressed
    }
    
    // Draw 3x3 key visual
    for (int dy = 0; dy < 3; dy++) {
        for (int dx = 0; dx < 3; dx++) {
            text40_putchar_color(x + dx, y + dy, ' ', fg, bg);
        }
    }
    
    // Draw arrow symbol in center
    text40_putchar_color(x + 1, y + 1, arrow, fg, bg);
}

// Draw action key (Z or X) visual
void draw_action_key(int x, int y, char key, int pressed) {
    unsigned char fg, bg;
    if (pressed) {
        if (key == 'Z') {
            fg = COLOR_WHITE;
            bg = COLOR_RED;  // Red for Z
        } else {
            fg = COLOR_WHITE;
            bg = COLOR_GREEN;  // Green for X
        }
    } else {
        fg = COLOR_LIGHT_GRAY;
        bg = COLOR_DARK_GRAY;
    }
    
    // Draw 5x3 key visual (wider for action buttons)
    for (int dy = 0; dy < 3; dy++) {
        for (int dx = 0; dx < 5; dx++) {
            text40_putchar_color(x + dx, y + dy, ' ', fg, bg);
        }
    }
    
    // Draw key letter in center
    text40_putchar_color(x + 2, y + 1, key, fg, bg);
}

// Draw status indicator
void draw_status(int x, int y, const char* label, int active) {
    text40_puts_color(x, y, label, COLOR_WHITE, COLOR_BLACK);
    
    if (active) {
        text40_puts_color(x + 7, y, "[ON] ", COLOR_BLACK, COLOR_GREEN);
    } else {
        text40_puts_color(x + 7, y, "[OFF]", COLOR_DARK_GRAY, COLOR_BLACK);
    }
}

// Draw a progress/activity bar
void draw_activity_bar(int x, int y, int width, int level) {
    text40_putchar_color(x, y, '[', COLOR_WHITE, COLOR_BLACK);
    
    for (int i = 0; i < width; i++) {
        if (i < level) {
            unsigned char color;
            if (i < width / 3) {
                color = COLOR_GREEN;
            } else if (i < (width * 2) / 3) {
                color = COLOR_YELLOW;
            } else {
                color = COLOR_RED;
            }
            text40_putchar_color(x + 1 + i, y, '=', color, COLOR_BLACK);
        } else {
            text40_putchar_color(x + 1 + i, y, '-', COLOR_DARK_GRAY, COLOR_BLACK);
        }
    }
    
    text40_putchar_color(x + width + 1, y, ']', COLOR_WHITE, COLOR_BLACK);
}

int main() {
    // Enable TEXT40 display mode
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Draw initial UI
    draw_border();
    
    // Title
    text40_puts_color(8, 2, " KEYBOARD ", COLOR_WHITE, COLOR_DARK_PURPLE);
    text40_puts_color(18, 2, " INPUT ", COLOR_YELLOW, COLOR_DARK_BLUE);
    text40_puts_color(25, 2, " DEMO ", COLOR_PINK, COLOR_DARK_GREEN);
    
    // Instructions
    text40_puts_color(3, 4, "Use Arrow Keys and Z/X buttons!", COLOR_LIGHT_GRAY, COLOR_BLACK);
    text40_puts_color(3, 5, "Watch the visual feedback below:", COLOR_LIGHT_GRAY, COLOR_BLACK);
    
    // Labels for arrow keys
    text40_puts_color(14, 7, "Arrow Keys", COLOR_WHITE, COLOR_BLACK);
    
    // Labels for action keys
    text40_puts_color(5, 18, "Action Z", COLOR_WHITE, COLOR_BLACK);
    text40_puts_color(27, 18, "Action X", COLOR_WHITE, COLOR_BLACK);
    
    // Initial display flush
    display_flush();
    
    // Main interactive loop
    int frame = 0;
    int activity_level = 0;
    int keys_pressed_count = 0;
    
    while (1) {
        // Read keyboard states
        int up = key_up_pressed();
        int down = key_down_pressed();
        int left = key_left_pressed();
        int right = key_right_pressed();
        int z = key_z_pressed();
        int x = key_x_pressed();
        
        // Count total keys pressed
        keys_pressed_count = 0;
        if (up) keys_pressed_count++;
        if (down) keys_pressed_count++;
        if (left) keys_pressed_count++;
        if (right) keys_pressed_count++;
        if (z) keys_pressed_count++;
        if (x) keys_pressed_count++;
        
        // Update activity level
        if (keys_pressed_count > 0) {
            if (activity_level < 20) activity_level++;
        } else {
            if (activity_level > 0) activity_level--;
        }
        
        // Draw arrow keys in cross formation
        draw_arrow_key(17, 9, '^', up);     // Up arrow
        draw_arrow_key(17, 13, 'v', down);  // Down arrow
        draw_arrow_key(13, 11, '<', left);  // Left arrow
        draw_arrow_key(21, 11, '>', right); // Right arrow
        
        // Draw action keys
        draw_action_key(5, 19, 'Z', z);
        draw_action_key(28, 19, 'X', x);
        
        // Update status indicators
        draw_status(3, 16, "UP:   ", up);
        draw_status(14, 16, "DOWN: ", down);
        draw_status(25, 16, "LEFT: ", left);
        draw_status(3, 17, "RIGHT:", right);
        draw_status(14, 17, "Z:    ", z);
        draw_status(25, 17, "X:    ", x);
        
        // Draw activity bar
        text40_puts_color(3, 22, "Activity:", COLOR_YELLOW, COLOR_BLACK);
        draw_activity_bar(13, 22, 20, activity_level);
        
        // Show key press count
        text40_puts_color(3, 23, "Keys:", COLOR_LIGHT_GRAY, COLOR_BLACK);
        text40_putchar_color(9, 23, '0' + keys_pressed_count, 
                           keys_pressed_count > 0 ? COLOR_GREEN : COLOR_DARK_GRAY,
                           COLOR_BLACK);
        
        // Animated spinner to show the program is running
        char spinner[] = {'|', '/', '-', '\\'};
        text40_putchar_color(37, 23, spinner[frame % 4], COLOR_BLUE, COLOR_BLACK);
        
        // Special effects when multiple keys are pressed
        if (keys_pressed_count >= 3) {
            // Rainbow effect on the border when 3+ keys pressed
            unsigned char rainbow[] = {
                COLOR_RED, COLOR_ORANGE, COLOR_YELLOW, 
                COLOR_GREEN, COLOR_BLUE, COLOR_INDIGO
            };
            int color_idx = (frame / 2) % 6;
            
            for (int i = 1; i < 39; i++) {
                text40_putchar_color(i, 0, '=', COLOR_WHITE, rainbow[(color_idx + i) % 6]);
                text40_putchar_color(i, 24, '=', COLOR_WHITE, rainbow[(color_idx + i + 3) % 6]);
            }
        }
        
        // Exit condition: Press all 6 keys simultaneously
        if (keys_pressed_count == 6) {
            // Victory message!
            text40_puts_color(10, 11, " ALL KEYS! ", COLOR_BLACK, COLOR_YELLOW);
            text40_puts_color(10, 12, " AMAZING!! ", COLOR_WHITE, COLOR_RED);
            display_flush();
            delay(50000);
            break;
        }
        
        // Update display
        display_flush();
        
        // Small delay for smooth animation
        delay(2000);
        
        frame++;
        
        // Exit after many frames (timeout)
        if (frame > 1000) {
            break;
        }
    }
    
    // Exit message
    text40_puts_color(8, 11, " Test Complete! ", COLOR_WHITE, COLOR_GREEN);
    display_flush();
    delay(30000);
    
    // Return to normal mode
    display_set_mode(DISP_MODE_OFF);
    
    // Print confirmation
    puts("Keyboard input test completed!");
    
    return 0;
}