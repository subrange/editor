#include <stdio.h>

int main() {
    char* src = "ABC";
    int i = 0;
    
    // First, test individual accesses work
    putchar(src[0]); // Should print A
    putchar(src[1]); // Should print B
    putchar('\n');
    
    // Now test in a while loop
    i = 0;
    while (i < 3) {
        putchar('L');
        putchar('0' + i);
        putchar(':');
        putchar(src[i]);
        putchar(' ');
        i++;
    }
    putchar('\n');
    
    return 0;
}