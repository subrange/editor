// Test unsigned literal and comparison
#include <stdio.h>

int main() {
    unsigned int bank_size = 64000U;
    unsigned int test_size = 4;
    
    // Test 1: Check if 64000U is stored correctly
    if (bank_size == 64000U) {
        putchar('Y');  // Should print Y
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 2: Check if comparison works
    if (test_size < bank_size) {
        putchar('Y');  // Should print Y (4 < 64000)
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 3: Check subtraction
    unsigned int remaining = bank_size - test_size;
    if (remaining == 63996U) {
        putchar('Y');  // Should print Y
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}