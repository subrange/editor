// Simple TTY echo program
// Reads characters from input and echoes them back
// Press 'q' to quit

#include <stdio.h>

int main() {
    int ch;
    
    puts("Echo program - type and see your input!");
    puts("Press 'q' to quit");
    putchar('\n');
    
    // Read and echo characters until 'q' is pressed
    while (1) {
        ch = getchar();
        
        // Echo the character
        putchar(ch);
        
        // Quit on 'q'
        if (ch == 'q' || ch == 'Q') {
            putchar('\n');
            puts("Goodbye!");
            break;
        }
    }
    
    return 0;
}