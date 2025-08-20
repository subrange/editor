// Debug test for Forth interpreter
#include <stdio.h>

int main() {
    // Test the puts function
    puts("Testing puts");
    
    // Test number printing
    putchar('0' + 7);
    putchar('\n');
    
    // Test string comparison
    char* s1 = ".";
    char* s2 = ".";
    
    int i = 0;
    while (s1[i] && s2[i]) {
        if (s1[i] != s2[i]) {
            putchar('N');
            putchar('\n');
            return 0;
        }
        i++;
    }
    if (s1[i] == s2[i]) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}