// Test with two loops accessing the same array
#include <stdio.h>

int main() {
    unsigned char arr[5] = {1, 2, 3, 4, 5};
    
    // First loop
    for (int i = 0; i < 5; i++) {
        putchar('0' + arr[i]);
    }
    putchar('\n');
    
    // Second loop
    for (int j = 0; j < 5; j++) {
        putchar('0' + arr[j]);
    }
    putchar('\n');
    
    return 0;
}