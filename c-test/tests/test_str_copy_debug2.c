#include <stdio.h>

void str_copy(char* dst, char* src) {
    int i = 0;
    putchar('S'); // Start
    putchar('\n');
    
    // First iteration check
    if (src[0]) {
        putchar('F');
        putchar('0');
        putchar(':');
        putchar(src[0]);
        putchar('\n');
    } else {
        putchar('N');
        putchar('0');
        putchar('\n');
    }
    
    // Loop
    while (src[i] && i < 31) {
        putchar('L');
        putchar('0' + i);
        putchar(':');
        putchar(src[i]);
        putchar('\n');
        
        dst[i] = src[i];
        i++;
        
        // Next check
        if (i < 31 && src[i]) {
            putchar('N');
            putchar('0' + i);
            putchar(':');
            putchar(src[i]);
            putchar('\n');
        }
    }
    
    putchar('E');
    putchar('0' + i);
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