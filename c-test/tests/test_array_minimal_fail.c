// Minimal test to reproduce the issue
#include <stdio.h>

int main() {
    unsigned char arr[] = {1, 2, 3, 4, 5};
    
    // Print all elements statically first
    putchar('0' + arr[0]);
    putchar('0' + arr[1]);
    putchar('0' + arr[2]);
    putchar('0' + arr[3]);
    putchar('0' + arr[4]);
    putchar('\n');
    
    // Then loop - this is where the bug appears
    for (int i = 0; i < 5; i++) {
        putchar('0' + arr[i]);
    }
    putchar('\n');
    
    return 0;
}