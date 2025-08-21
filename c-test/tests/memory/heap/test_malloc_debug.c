// Debug malloc to understand the issue
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Test what happens with BANK_SIZE comparisons
    unsigned int test_size = 1;
    unsigned int bank_size = 64000;
    
    // Print some debug info
    if (test_size > bank_size) {
        putchar('A');  // Should not print
    } else {
        putchar('B');  // Should print
    }
    putchar('\n');
    
    // Now test malloc
    int* ptr = (int*)malloc(1);
    if (ptr == NULL) {
        putchar('N');  // Failed
    } else {
        putchar('Y');  // Success
        *ptr = 42;
        if (*ptr == 42) {
            putchar('!');  // Write worked
        }
    }
    putchar('\n');
    
    return 0;
}