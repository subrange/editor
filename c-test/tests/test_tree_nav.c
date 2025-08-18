// Test file for tree navigation demo
#include <stdio.h>

int add(int a, int b) {
    return a + b;
}

int multiply(int x, int y) {
    int result = 0;
    for (int i = 0; i < y; i++) {
        result = add(result, x);
    }
    return result;
}

int main() {
    int a = 5;
    int b = 6;
    int sum = add(a, b);
    int product = multiply(a, b);
    
    if (sum == 11 && product == 30) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}