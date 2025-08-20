// Test basic puts functionality
#include <stdio.h>

int main() {
    puts("Hello");
    puts("World");
    
    // Test string literal
    char* msg = "Test";
    puts(msg);
    
    // Test character array
    char arr[10];
    arr[0] = 'A';
    arr[1] = 'r';
    arr[2] = 'r';
    arr[3] = 'a';
    arr[4] = 'y';
    arr[5] = 0;
    puts(arr);
    
    return 0;
}