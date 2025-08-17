// Test edge cases for array allocation
#include <stdio.h>

int main() {
    // Single element incomplete array
    int single[] = {42};
    
    // Empty initializer list (C99 allows this for zero-sized arrays in some contexts)
    // But we should handle gracefully even if not supported
    // int empty[] = {};  // Commented out as may not be supported
    
    // Large incomplete array  
    unsigned char large[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 0};
    
    // Mixed with explicit size and incomplete
    int explicit[3] = {7, 8, 9};
    int implicit[] = {7, 8, 9};
    
    // Test single element
    putchar('0' + (single[0] / 10));
    putchar('0' + (single[0] % 10));
    putchar('\n');
    
    // Test large array
    for (int i = 0; i < 10; i++) {
        putchar('0' + large[i]);
    }
    putchar('\n');
    
    // Compare explicit vs implicit
    for (int i = 0; i < 3; i++) {
        putchar('0' + explicit[i]);
    }
    putchar('\n');
    
    for (int i = 0; i < 3; i++) {
        putchar('0' + implicit[i]);
    }
    putchar('\n');
    
    return 0;
}