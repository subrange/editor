#include <stdio.h>

// Global array instead of string literal
char global_array[4] = {'A', 'B', 'C', '\0'};

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
    char dst[32];
    array_copy(dst, global_array);

    puts(global_array);
    puts(dst);

    return 0;
}