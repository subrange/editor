// Exact copy of the failing test
#include <stdio.h>

int main() {
    unsigned char arr[] = {1, 2, 3, 4, 5};
    
    // Test 1: Direct indexing with literals
    putchar('0' + arr[0]);
    putchar('0' + arr[1]);
    putchar('0' + arr[2]);
    putchar('0' + arr[3]);
    putchar('0' + arr[4]);
    putchar('\n');
    
    // Test 2: Indexing with variable
    for (int i = 0; i < 5; i++) {
        putchar('0' + arr[i]);
    }
    putchar('\n');
    
    // Test 3: Indexing with computed value
    for (int j = 0; j < 5; j++) {
        int index = j;
        putchar('0' + arr[index]);
    }
    putchar('\n');
    
    // Test 4: Division-based indexing
    for (int k = 0; k < 10; k++) {
        int idx = k / 2;
        putchar('0' + arr[idx]);
    }
    putchar('\n');
    
    return 0;
}