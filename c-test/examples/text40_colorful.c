// Colorful TEXT40 display demonstration
// Shows off the PICO-8 color palette with various effects

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

// Simple delay function
void delay(int count) {
    int i;
    for (i = 0; i < count; i++) {
        // Busy wait - just loop
        int dummy = i * 2; // Some work to prevent optimization
    }
}

// Draw a colorful rainbow bar
void draw_rainbow_bar(int y) {
    // Better rainbow sequence avoiding duplicated purples
    unsigned char rainbow_colors[] = {
        COLOR_RED,
        COLOR_ORANGE,
        COLOR_YELLOW,
        COLOR_GREEN,
        COLOR_BLUE,
        COLOR_DARK_BLUE,
        COLOR_DARK_PURPLE,
        COLOR_PINK
    };
    
    // Draw rainbow with 4 chars per color to fit in 32 chars (8 colors * 4 = 32)
    for (int color = 0; color < 8; color++) {
        for (int i = 0; i < 4; i++) {
            text40_putchar_color(4 + color * 4 + i, y, ' ',
                                COLOR_BLACK, rainbow_colors[color]);
        }
    }
}

// Draw a color palette showcase
void draw_color_palette(int x, int y) {
    // Show all 16 colors as blocks
    for (int i = 0; i < 16; i++) {
        int row = i / 8;
        int col = i % 8;
        // Draw 2x2 blocks for each color
        text40_putchar_color(x + col * 3, y + row * 2, ' ', COLOR_BLACK, i);
        text40_putchar_color(x + col * 3 + 1, y + row * 2, ' ', COLOR_BLACK, i);
        text40_putchar_color(x + col * 3, y + row * 2 + 1, ' ', COLOR_BLACK, i);
        text40_putchar_color(x + col * 3 + 1, y + row * 2 + 1, ' ', COLOR_BLACK, i);
        
        // Add color number in contrasting color
        unsigned char fg;
        if (i < 8) {
            fg = COLOR_WHITE;
        } else {
            fg = COLOR_BLACK;
        }
        text40_putchar_color(x + col * 3, y + row * 2, '0' + (i / 10), fg, i);
        text40_putchar_color(x + col * 3 + 1, y + row * 2, '0' + (i % 10), fg, i);
    }
}

int main() {
    // Enable TEXT40 display mode
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Draw a colorful border using different colors for each side
    // Top border in blue
    for (int i = 0; i < 40; i++) {
        text40_putchar_color(i, 0, '=', COLOR_BLUE, COLOR_DARK_BLUE);
    }
    // Bottom border in green
    for (int i = 0; i < 40; i++) {
        text40_putchar_color(i, 24, '=', COLOR_GREEN, COLOR_DARK_GREEN);
    }
    // Left border in red
    for (int i = 1; i < 24; i++) {
        text40_putchar_color(0, i, '|', COLOR_RED, COLOR_BLACK);
    }
    // Right border in yellow
    for (int i = 1; i < 24; i++) {
        text40_putchar_color(39, i, '|', COLOR_YELLOW, COLOR_BLACK);
    }
    
    // Colorful corners
    text40_putchar_color(0, 0, '*', COLOR_WHITE, COLOR_RED);
    text40_putchar_color(39, 0, '*', COLOR_WHITE, COLOR_ORANGE);
    text40_putchar_color(0, 24, '*', COLOR_WHITE, COLOR_PINK);
    text40_putchar_color(39, 24, '*', COLOR_WHITE, COLOR_PEACH);
    
    // Title with gradient background
    text40_puts_color(10, 2, " COLORFUL ", COLOR_WHITE, COLOR_DARK_PURPLE);
    text40_puts_color(20, 2, " TEXT40 ", COLOR_YELLOW, COLOR_DARK_BLUE);
    text40_puts_color(28, 2, " DEMO ", COLOR_PINK, COLOR_DARK_GREEN);
    
    // Draw rainbow bar
    text40_puts_color(2, 4, "Rainbow:", COLOR_WHITE, COLOR_BLACK);
    draw_rainbow_bar(5);
    
    // Color palette showcase
    text40_puts_color(2, 7, "PICO-8 Colors:", COLOR_LIGHT_GRAY, COLOR_BLACK);
    draw_color_palette(2, 9);
    
    // Some styled text examples
    text40_puts_color(2, 14, "Retro", COLOR_ORANGE, COLOR_BROWN);
    text40_puts_color(8, 14, "Gaming", COLOR_GREEN, COLOR_DARK_GREEN);
    text40_puts_color(15, 14, "Style!", COLOR_PINK, COLOR_DARK_PURPLE);

    // Matrix-like effect area
    text40_puts_color(2, 16, "Matrix:", COLOR_GREEN, COLOR_BLACK);
    for (int i = 0; i < 10; i++) {
        unsigned char intensity;
        if (i < 5) {
            intensity = COLOR_GREEN;
        } else {
            intensity = COLOR_DARK_GREEN;
        }
        text40_putchar_color(10 + i, 16, '0' + (i % 10), intensity, COLOR_BLACK);
    }

    // Create colored animation area
    text40_puts_color(2, 18, "Animation:", COLOR_YELLOW, COLOR_BLACK);

    // Flush initial display
    display_flush();

    // Animation loop with color cycling
    char anim_chars[] = {'/', '-', '\\', '|'};
    unsigned char anim_colors[] = {
        COLOR_RED, COLOR_ORANGE, COLOR_YELLOW, COLOR_GREEN,
        COLOR_BLUE, COLOR_INDIGO, COLOR_DARK_PURPLE, COLOR_PINK
    };

    for (int frame = 0; frame < 32; frame++) {
        // Animated spinners with color cycling
        unsigned char color = anim_colors[frame % 8];
        text40_putchar_color(13, 18, anim_chars[frame % 4], color, COLOR_BLACK);
        text40_putchar_color(15, 18, anim_chars[(frame + 1) % 4],
                           anim_colors[(frame + 1) % 8], COLOR_BLACK);
        text40_putchar_color(17, 18, anim_chars[(frame + 2) % 4],
                           anim_colors[(frame + 2) % 8], COLOR_BLACK);

        // Scrolling color blocks
        for (int i = 0; i < 10; i++) {
            unsigned char bg_color = anim_colors[(frame + i) % 8];
            text40_putchar_color(25 + i, 18, ' ', COLOR_BLACK, bg_color);
        }

        // Update frame counter with style
        text40_puts_color(2, 20, "Frame:", COLOR_LIGHT_GRAY, COLOR_BLACK);
        text40_putchar_color(9, 20, '0' + (frame / 10), COLOR_WHITE, COLOR_DARK_GRAY);
        text40_putchar_color(10, 20, '0' + (frame % 10), COLOR_WHITE, COLOR_DARK_GRAY);

        // Progress bar
        int progress = (frame * 20) / 32;
        for (int i = 0; i < 20; i++) {
            if (i < progress) {
                text40_putchar_color(15 + i, 20, '=', COLOR_GREEN, COLOR_DARK_GREEN);
            } else {
                text40_putchar_color(15 + i, 20, '-', COLOR_DARK_GRAY, COLOR_BLACK);
            }
        }

        display_flush();
        delay(8000); // Animation delay
    }

    // Final message with fancy styling
    text40_puts_color(8, 22, "  Test  ", COLOR_WHITE, COLOR_GREEN);
    text40_puts_color(16, 22, " Complete! ", COLOR_BLACK, COLOR_YELLOW);
    display_flush();
    
    // Keep display for a moment
    delay(50000);

    // Return to normal mode
    display_set_mode(DISP_MODE_OFF);
    
    // Print confirmation to normal output
    puts("Colorful TEXT40 test completed!");
    
    return 0;
}