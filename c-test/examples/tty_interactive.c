// Interactive TTY echo program
// Demonstrates real keyboard input using TTY mode

#include <stdio.h>
#include <mmio.h>

int main() {
    // Set display mode to TTY (mode 1)
    display_set_mode(1);  // DISP_MODE_TTY
    
    puts("Interactive TTY Echo Program");
    puts("Type characters and see them echoed!");
    puts("Press 'q' to quit");
    puts("================================");
    putchar('\n');
    
    int ch;
    while (1) {
        ch = getchar();
        
        // Echo the character
        putchar(ch);
        
        // Handle special characters
        if (ch == '\n') {
            // Already echoed the newline
        } else if (ch == 8) {  // Backspace
            // Move cursor back and clear
            putchar(' ');
            putchar(8);
        } else if (ch == 'q' || ch == 'Q') {
            putchar('\n');
            puts("\nGoodbye!");
            break;
        }
    }
    
    // Reset to OFF mode
    display_set_mode(0);
    
    return 0;
}