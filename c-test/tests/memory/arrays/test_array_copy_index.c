// Test array access with copied index
#include <stdio.h>

int main() {
    unsigned char arr[5] = {1, 2, 3, 4, 5};
    
    // Loop with copied index
    for (int i = 0; i < 5; i++) {
        int j = i;  // Copy index to temp
        putchar('0' + arr[j]);
    }
    putchar('\n');
    
    return 0;
}