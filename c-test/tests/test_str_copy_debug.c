#include <stdio.h>

void str_copy(char* dst, char* src) {
    int i = 0;
    while (src[i] && i < 31) {
        putchar('L'); // Loop iteration marker
        putchar('0' + i); // Show current index
        putchar(':');
        putchar(src[i]); // Show character being copied
        putchar(' ');
        
        dst[i] = src[i];
        i++;
    }
    putchar('E'); // End of loop marker
    putchar('0' + i); // Final index value
    putchar('\n');
    dst[i] = 0;
}

int main() {
    char * src = "ABC";
    char dst[32];
    str_copy(dst, src);

    puts(src);
    puts(dst);

    return 0;
}