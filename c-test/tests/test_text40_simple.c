// Simple TEXT40 display test

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

int main() {
    // Enable TEXT40 display mode
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Write a simple character at position (0,0)
    text40_putchar(0, 0, 'H');
    text40_putchar(1, 0, 'e');
    text40_putchar(2, 0, 'l');
    text40_putchar(3, 0, 'l');
    text40_putchar(4, 0, 'o');
    
    // Write at position (0,1)
    text40_putchar(0, 1, 'W');
    text40_putchar(1, 1, 'o');
    text40_putchar(2, 1, 'r');
    text40_putchar(3, 1, 'l');
    text40_putchar(4, 1, 'd');
    
    // Flush to display
    display_flush();
    
    // Simple delay loop
    for (int i = 0; i < 100000; i++) {
        int dummy = i;
    }
    
    // Return to normal mode
    display_set_mode(DISP_MODE_OFF);
    
    // Print confirmation
    puts("Simple TEXT40 test completed");
    
    return 0;
}