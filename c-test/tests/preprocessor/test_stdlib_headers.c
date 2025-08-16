#include <stdio.h>
#include <stdlib.h>
#include <limits.h>

int main() {
    // Test stdio - putchar from header
    putchar('H');
    putchar('i');
    putchar('\n');
    
    // Test limits - INT_MAX macro
    if (INT_MAX == 32767) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test stdlib - EXIT_SUCCESS macro
    if (EXIT_SUCCESS == 0) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return EXIT_SUCCESS;
}