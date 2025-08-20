#include <stdio.h>

int main() {
    char* src = "ABC";
    char dst[32];
    
    // For loop instead of while
    for (int i = 0; i < 3; i++) {
        putchar('L');
        putchar('0' + i);
        putchar(':');
        putchar(src[i]);
        putchar(' ');
        
        dst[i] = src[i];
    }
    putchar('E');
    putchar('\n');
    dst[3] = 0;

    puts(src);
    puts(dst);

    return 0;
}