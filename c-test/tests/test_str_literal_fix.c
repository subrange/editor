// Test to verify string literal handling
#include <stdio.h>

void print_chars(char* str) {
    int i = 0;
    while (str[i]) {
        putchar(str[i]);
        i++;
    }
    putchar('\n');
}

int main() {
    char* msg = "Hello";
    print_chars(msg);
    
    // Direct pass
    print_chars("World");
    
    return 0;
}