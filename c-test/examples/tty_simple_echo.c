// Simple TTY echo test - no display mode change
// Just reads from TTY input and echoes back

#include <stdio.h>

int main() {
    puts("Simple Echo Test");
    puts("Type and press Enter to see your input echoed");
    puts("Type 'quit' to exit");
    puts("================================");
    
    char buffer[100];
    int i = 0;
    int ch;
    
    while (1) {
        // Read a character
        ch = getchar();
        
        // Echo it immediately
        putchar(ch);
        
        // Handle newline - process the line
        if (ch == '\n') {
            // Check if it's "quit"
            if (i == 4 && buffer[0] == 'q' && buffer[1] == 'u' && 
                buffer[2] == 'i' && buffer[3] == 't') {
                puts("Goodbye!");
                break;
            }
            // Reset buffer
            i = 0;
        } else if (i < 99) {
            // Store in buffer
            buffer[i++] = ch;
        }
    }
    
    return 0;
}