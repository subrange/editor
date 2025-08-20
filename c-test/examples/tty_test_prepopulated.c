// Test TTY input with pre-populated buffer
// This tests if getchar() works when input is available

#include <stdio.h>

int main() {
    puts("Testing getchar with pre-populated input...");
    
    // Try to read 5 characters
    for (int i = 0; i < 5; i++) {
        int ch = getchar();
        puts("Got character:");
        putchar(ch);
        putchar('\n');
    }
    
    puts("Done!");
    return 0;
}