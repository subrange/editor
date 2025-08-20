#include <stdio.h>

int main() {
    char* src = "ABC";
    int i = 0;
    
    // Test while loop with src[i] in the condition
    while (src[i]) {
        putchar('L');
        putchar('0' + i);
        putchar(':');
        putchar(src[i]);
        putchar(' ');
        i++;
        if (i >= 10) break; // Safety
    }
    putchar('\n');
    
    return 0;
}