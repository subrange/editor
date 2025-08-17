// Simple ternary operator test
#include <stdio.h>

int main() {
    // Test 1: Basic max function
    int a = 5;
    int b = 10;
    int max = (a > b) ? a : b;
    putchar('0' + (max / 10));
    putchar('0' + (max % 10));
    putchar('\n');
    
    // Test 2: Character selection
    char c = (a < b) ? 'T' : 'F';
    putchar(c);
    putchar('\n');
    
    // Test 3: Nested ternary
    int x = 7;
    char result = (x > 10) ? 'A' : (x > 5) ? 'B' : 'C';
    putchar(result);
    putchar('\n');
    
    return 0;
}