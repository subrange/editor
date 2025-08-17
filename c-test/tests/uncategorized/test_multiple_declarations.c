#include <stdio.h>

int main() {
    // Test multiple variable declarations on the same line
    int a = 5, b = 10;
    
    // Test with mixed initializations
    int c, d = 20;
    
    // Initialize c
    c = 15;
    
    // Test the values
    if (a == 5) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (b == 10) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (c == 15) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (d == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    
    return 0;
}