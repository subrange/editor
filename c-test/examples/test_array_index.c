// Test array indexing with computed index
#include <stdio.h>

int main() {
    // Test array with known values
    unsigned char colors[] = {8, 9, 10, 11, 12, 1, 2, 14};
    
    // Test 1: Direct array access
    putchar('0' + colors[0]);  // Should print 8
    putchar(' ');
    putchar('0' + colors[2]);  // Should print 10  
    putchar(' ');
    putchar('0' + colors[7]);  // Should print 14
    putchar('\n');
    
    // Test 2: Array access with computed index
    int pos = 0;
    while (pos < 32) {
        int color_index = pos / 4;
        unsigned char color = colors[color_index];
        
        // Print the color value
        if (color >= 10) {
            putchar('1');
            putchar('0' + (color - 10));
        } else {
            putchar('0' + color);
        }
        
        // Add separator every 4 chars
        if ((pos + 1) % 4 == 0) {
            putchar(' ');
        }
        
        pos++;
    }
    putchar('\n');
    
    return 0;
}