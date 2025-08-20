#include <stdio.h>

int main() {
    char* src = "ABC";
    int i = 0;
    
    // Test while loop with compound condition
    while (src[i] && i < 31) {
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