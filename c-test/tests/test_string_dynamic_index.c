// Test dynamic indexing with string literals
#include <stdio.h>

void copy_with_loop(char* dst, char* src) {
    int i;
    // Minimal loop - just copy 3 chars
    for (i = 0; i < 3; i++) {
        dst[i] = src[i];  // This is where it fails
    }
    dst[3] = 0;
}

int main() {
    char* src = "ABC";
    char dst[10];
    
    copy_with_loop(dst, src);
    
    puts(src);
    puts(dst);
    
    return 0;
}