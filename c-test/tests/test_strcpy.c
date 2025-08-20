#include <stdio.h>
#include <string.h>

int main() {
    char dst[20];
    char* src = "Hello";
    
    // Test strcpy
    strcpy(dst, src);
    puts(dst);
    
    // Test strcat
    strcat(dst, " World");
    puts(dst);
    
    // Test strlen
    int len = strlen(dst);
    printf("Length: %d\n", len);
    
    // Test strcmp
    if (strcmp(dst, "Hello World") == 0) {
        puts("strcmp works!");
    }
    
    return 0;
}