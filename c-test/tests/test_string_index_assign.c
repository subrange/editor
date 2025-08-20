// Test array indexing with assignment for string literals passed as parameters
#include <stdio.h>

void copy_first_char(char* dst, char* src) {
    // This is the exact pattern that fails
    dst[0] = src[0];
    dst[1] = src[1]; 
    dst[2] = src[2];
    dst[3] = 0;
}

int main() {
    char* src = "ABC";
    char dst[10];
    
    // Copy using array indexing
    copy_first_char(dst, src);
    
    // Print both
    puts(src);
    puts(dst);
    
    return 0;
}