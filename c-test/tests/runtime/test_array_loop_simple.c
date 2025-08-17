// Simplified test to debug array loop issue
#include <stdio.h>

int main() {
    unsigned char arr[3] = {7, 8, 9};
    
    // Simple loop with dynamic index
    for (int i = 0; i < 3; i++) {
        putchar('0' + arr[i]);
    }
    putchar('\n');
    
    return 0;
}