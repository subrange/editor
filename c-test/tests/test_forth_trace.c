// Test to trace Forth execution
#include <stdio.h>

char* test_str = ".";

int main() {
    // Test string comparison with "."
    char dot[2];
    dot[0] = '.';
    dot[1] = 0;
    
    // Compare with test_str
    int i = 0;
    while (test_str[i] && dot[i]) {
        if (test_str[i] != dot[i]) {
            puts("Strings differ!");
            return 1;
        }
        i++;
    }
    
    if (test_str[i] == dot[i]) {
        puts("Strings match!");
    } else {
        puts("Length mismatch!");
    }
    
    // Test dictionary lookup simulation
    puts("Testing dictionary lookup:");
    
    // Simulate what should happen
    char word[32];
    word[0] = '.';
    word[1] = 0;
    
    puts("Word to find: ");
    puts(word);
    
    // Test if char comparison works
    if (word[0] == '.') {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}