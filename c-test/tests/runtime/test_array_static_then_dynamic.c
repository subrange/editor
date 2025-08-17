// Test mixing static and dynamic array access
#include <stdio.h>

int main() {
    unsigned char arr[5] = {1, 2, 3, 4, 5};
    
    // First access statically
    putchar('0' + arr[0]);
    putchar('0' + arr[1]);
    putchar('\n');
    
    // Then loop with dynamic index
    for (int i = 0; i < 5; i++) {
        putchar('0' + arr[i]);
    }
    putchar('\n');
    
    return 0;
}