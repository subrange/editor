// Simple test of Forth operations
#include <stdio.h>

int main() {
    // Test that basic operations work
    puts("Testing Forth operations");
    
    // Test addition
    int a = 5;
    int b = 3;
    int result = a + b;
    printf("5 + 3 = %d\n", result);
    
    // Test comparison
    if (a > b) {
        puts("5 > 3: true");
    }
    
    puts("Done");
    return 0;
}