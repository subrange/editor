// Test TEXT40 display mode with raw terminal rendering

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

int main() {
    // Enable TEXT40 display mode
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Draw a border
    for (int i = 0; i < 40; i++) {
        text40_putchar(i, 0, '-');
        text40_putchar(i, 24, '-');
    }
    for (int i = 1; i < 24; i++) {
        text40_putchar(0, i, '|');
        text40_putchar(39, i, '|');
    }

    // Draw corners
    text40_putchar(0, 0, '+');
    text40_putchar(39, 0, '+');
    text40_putchar(0, 24, '+');
    text40_putchar(39, 24, '+');
    
    // Write title
    text40_puts(14, 2, "TEXT40 DISPLAY");
    text40_puts(16, 3, "40x25 Mode");
    
    // Write some test patterns
    text40_puts(5, 6, "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    text40_puts(5, 8, "0123456789");
    text40_puts(5, 10, "!@#$%^&*()");
    
    // Create a simple animation area
    text40_puts(10, 12, "Animation Test:");
    
    // Flush initial display
    display_flush();
    
    // Simple animation loop
    char anim_chars[] = {'/', '-', '\\', '|'};
    for (int frame = 0; frame < 20; frame++) {
        text40_putchar(15, 14, anim_chars[frame % 4]);
        text40_putchar(20, 14, anim_chars[(frame + 1) % 4]);
        text40_putchar(25, 14, anim_chars[(frame + 2) % 4]);

        // Update status
        text40_puts(10, 20, "Frame: ");
        text40_putchar(17, 20, '0' + (frame / 10));
        text40_putchar(18, 20, '0' + (frame % 10));

        display_flush();
        delay(10000); // Simple delay
    }
    
    // Final message
    text40_puts(12, 22, "Test Complete!");
    display_flush();
    
    // Keep display for a moment
    delay(50000);
    
    // Return to normal mode
    display_set_mode(DISP_MODE_OFF);
    
    // Print confirmation to normal output
    puts("TEXT40 test completed");
    
    return 0;
}