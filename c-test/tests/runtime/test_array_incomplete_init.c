// Test incomplete array declarations with initializers
#include <stdio.h>

// Global incomplete array
unsigned char global_arr[] = {10, 20, 30};

int main() {
    // Local incomplete array
    int local_arr[] = {1, 2, 3, 4};
    
    // Nested array (2D)
    int matrix[][2] = {{1, 2}, {3, 4}, {5, 6}};
    
    // Character array
    char str[] = {'H', 'e', 'l', 'l', 'o', '\0'};
    
    // Test global array
    for (int i = 0; i < 3; i++) {
        putchar('0' + (global_arr[i] / 10));
    }
    putchar('\n');
    
    // Test local array
    for (int i = 0; i < 4; i++) {
        putchar('0' + local_arr[i]);
    }
    putchar('\n');
    
    // Test 2D array
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 2; j++) {
            putchar('0' + matrix[i][j]);
        }
    }
    putchar('\n');
    
    // Test string
    for (int i = 0; str[i]; i++) {
        putchar(str[i]);
    }
    putchar('\n');
    
    return 0;
}