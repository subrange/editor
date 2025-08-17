// Simplified array indexing test
#include <stdio.h>

int main() {
    unsigned char arr[] = {1, 2, 3, 4, 5};
    
    // Test 1: Direct indexing with literals
    putchar('0' + arr[0]);  // Should print 1
    putchar('0' + arr[1]);  // Should print 2
    putchar('0' + arr[2]);  // Should print 3
    putchar('0' + arr[3]);  // Should print 4
    putchar('0' + arr[4]);  // Should print 5
    putchar('\n');
    
    // Test 2: Indexing with variable
    for (int i = 0; i < 5; i++) {
        putchar('0' + arr[i]);  // Should print 12345
    }
    putchar('\n');
    
    // Test 3: Indexing with computed value
    for (int j = 0; j < 5; j++) {
        int index = j;
        putchar('0' + arr[index]);  // Should print 12345
    }
    putchar('\n');
    
    // Test 4: Division-based indexing
    for (int k = 0; k < 10; k++) {
        int idx = k / 2;  // 0,0,1,1,2,2,3,3,4,4
        putchar('0' + arr[idx]);  // Should print 1122334455
    }
    putchar('\n');
    
    return 0;
}