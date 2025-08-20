// Test that string literals work correctly when passed to functions
#include <stdio.h>

void print_string(char* str) {
    int i = 0;
    while (str[i]) {
        putchar(str[i]);
        i++;
    }
}

int main() {
    // Test 1: Pass string literal directly
    print_string("Hello");
    putchar('\n');
    
    // Test 2: Store to variable then pass
    char* msg = "World";
    print_string(msg);
    putchar('\n');
    
    return 0;
}