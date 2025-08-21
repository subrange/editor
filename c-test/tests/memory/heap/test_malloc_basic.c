// Basic malloc test - allocate memory and use it
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Test 1: Basic allocation
    int* ptr = (int*)malloc(sizeof(int));
    if (ptr == NULL) {
        putchar('N');  // Failed to allocate
        putchar('\n');
        return 1;
    }
    
    // Test 2: Can we write to allocated memory?
    *ptr = 42;
    
    // Test 3: Can we read back the value?
    if (*ptr == 42) {
        putchar('Y');  // Success
    } else {
        putchar('N');  // Failed
    }
    putchar('\n');
    
    // Test 4: Free the memory (should not crash)
    free(ptr);
    
    // Test 5: Allocate an array
    int* arr = (int*)malloc(5 * sizeof(int));
    if (arr == NULL) {
        putchar('N');  // Failed to allocate array
        putchar('\n');
        return 1;
    }
    
    // Test 6: Initialize array
    for (int i = 0; i < 5; i++) {
        arr[i] = i * 10;
    }
    
    // Test 7: Verify array values
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum += arr[i];
    }
    
    if (sum == 100) {  // 0 + 10 + 20 + 30 + 40 = 100
        putchar('Y');  // Success
    } else {
        putchar('N');  // Failed
    }
    putchar('\n');
    
    free(arr);
    
    return 0;
}