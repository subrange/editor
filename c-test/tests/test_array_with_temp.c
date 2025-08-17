// Test array access with intermediate variable
#include <stdio.h>

int main() {
    unsigned char arr[5] = {1, 2, 3, 4, 5};
    
    // Loop with intermediate variable
    for (int j = 0; j < 5; j++) {
        int index = j;
        putchar('0' + arr[index]);
    }
    putchar('\n');
    
    return 0;
}