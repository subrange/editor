// Simplified array test
#include <stdio.h>

int main() {
    unsigned char arr[5];
    
    // Initialize
    arr[0] = 1;
    arr[1] = 2; 
    arr[2] = 3;
    arr[3] = 4;
    arr[4] = 5;
    
    // Test with single dynamic access
    for (int i = 0; i < 5; i++) {
        putchar('0' + arr[i]);
    }
    putchar('\n');
    
    return 0;
}