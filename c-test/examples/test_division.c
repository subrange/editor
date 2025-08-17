// Simple division test
#include <stdio.h>

int main() {
    // Test basic division
    int a = 8;
    int b = 4;
    int result = a / b;
    
    // Should print 2
    putchar('0' + result);
    putchar('\n');
    
    // Test division in loop context
    int pos = 0;
    while (pos < 32) {
        int color_index = pos / 4;  // Should be 0,0,0,0,1,1,1,1,2,2,2,2...
        putchar('0' + color_index);
        if ((pos + 1) % 4 == 0) {
            putchar(' ');
        }
        pos++;
    }
    putchar('\n');
    
    // Test some more divisions
    putchar('0' + (10 / 5));  // Should print 2
    putchar(' ');
    putchar('0' + (15 / 3));  // Should print 5
    putchar(' ');
    putchar('0' + (20 / 4));  // Should print 5
    putchar('\n');
    
    return 0;
}