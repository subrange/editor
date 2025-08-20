#include <stdio.h>

int main() {
    char* src = "ABC";
    char dst[32];
    
    // Direct loop without function call
    int i = 0;
    while (src[i] && i < 31) {
        putchar('L');
        putchar('0' + i);
        putchar(':');
        putchar(src[i]);
        putchar(' ');
        
        dst[i] = src[i];
        i++;
    }
    putchar('E');
    putchar('0' + i);
    putchar('\n');
    dst[i] = 0;

    puts(src);
    puts(dst);

    return 0;
}