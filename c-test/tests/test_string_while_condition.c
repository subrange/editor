// Test while loop with compound condition like str_copy
#include <stdio.h>

void copy_with_while(char* dst, char* src) {
    int i = 0;
    // Use the exact same condition as str_copy
    while (src[i] && i < 31) {
        dst[i] = src[i];
        i++;
    }
    dst[i] = 0;
}

int main() {
    char* src = "ABC";
    char dst[32];
    
    copy_with_while(dst, src);
    
    puts(src);
    puts(dst);
    
    return 0;
}