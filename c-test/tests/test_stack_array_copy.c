#include <stdio.h>

void array_copy(char* dst, char* src) {
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
}

int main() {
    // Stack array instead of global
    char src[4] = {'A', 'B', 'C', '\0'};
    char dst[32];
    array_copy(dst, src);

    puts(src);
    puts(dst);

    return 0;
}