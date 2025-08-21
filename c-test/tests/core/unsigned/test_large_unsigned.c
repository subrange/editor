// Test unsigned comparisons with values >= 32768
#include <stdio.h>

int main() {
    unsigned int value = 32768;
    unsigned int limit = 32768;
    
    // Test 1: Compare with 32768
    if (value <= limit) {
        putchar('Y');  // Should print Y
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 2: Compare with literal
    if (value <= 32768) {
        putchar('Y');  // Should print Y
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 3: Subtraction test
    unsigned int space = 32768 - 100;
    if (space > 32000) {
        putchar('Y');  // Should print Y (32668 > 32000)
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}