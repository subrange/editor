#include <stdio.h>

int main() {
    char* src = "ABC";
    
    // Manually check each access
    putchar('0');
    putchar(':');
    putchar(src[0]);
    putchar('\n');
    
    putchar('1');
    putchar(':');
    putchar(src[1]);
    putchar('\n');
    
    putchar('2');
    putchar(':'); 
    putchar(src[2]);
    putchar('\n');
    
    // Now in a loop
    int i = 0;
    putchar('L');
    putchar('\n');
    if (src[i]) {
        putchar('Y');
        putchar('0');
        putchar('\n');
    }
    
    i = 1;
    if (src[i]) {
        putchar('Y');
        putchar('1');
        putchar('\n');
    } else {
        putchar('N');
        putchar('1');
        putchar('\n');
    }
    
    return 0;
}