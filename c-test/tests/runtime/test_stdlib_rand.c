// Test standard C library rand() and srand() functions
#include <stdlib.h>
#include <stdio.h>

void print_number(int n) {
    // Simple number printing for positive numbers
    if (n == 0) {
        putchar('0');
        return;
    }
    
    // Buffer to store digits
    char digits[10];
    int idx = 0;
    
    while (n > 0) {
        digits[idx++] = '0' + (n % 10);
        n = n / 10;
    }
    
    // Print in reverse order
    while (idx > 0) {
        putchar(digits[--idx]);
    }
}

int main() {
    // Test with default seed (should be 1)
    puts("Default: ");
    
    int r1 = rand();
    print_number(r1);
    putchar('\n');
    
    // Set a specific seed
    srand(42);
    
    puts("Seed 42: ");
    
    int r2 = rand();
    print_number(r2);
    putchar('\n');
    
    // Verify RAND_MAX constraint
    puts("Max: ");
    
    // Generate many random numbers and check they're all <= 32767
    int all_valid = 1;
    int i;
    for (i = 0; i < 10; i++) {
        int r = rand();
        if (r < 0 || r > 32767) {
            all_valid = 0;
            break;
        }
    }
    
    if (all_valid == 1) {
        puts("OK");
    } else {
        puts("FAIL");
    }
    
    // Reset with seed 1 (standard default)
    srand(1);
    
    puts("Seed 1: ");
    
    int r3 = rand();
    print_number(r3);
    putchar('\n');
    
    return 0;
}