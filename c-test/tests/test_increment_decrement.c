// Test increment and decrement operators
void putchar(int c);

int main() {
    int x = 5;
    int y;
    
    // Test pre-increment
    y = ++x;  // x becomes 6, y becomes 6
    if (x == 6 && y == 6) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test post-increment
    x = 5;
    y = x++;  // y becomes 5, x becomes 6
    if (x == 6 && y == 5) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test pre-decrement
    x = 5;
    y = --x;  // x becomes 4, y becomes 4
    if (x == 4 && y == 4) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test post-decrement
    x = 5;
    y = x--;  // y becomes 5, x becomes 4
    if (x == 4 && y == 5) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test multiple increments in expression
    x = 5;
    y = x++ + ++x;  // y = 5 + 7 = 12, x becomes 7
    if (x == 7 && y == 12) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test with pointers
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = arr;
    
    // Pre-increment pointer
    y = *(++p);  // p points to arr[1], y becomes 20
    if (y == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Post-increment pointer
    p = arr;
    y = *(p++);  // y becomes 10, p points to arr[1]
    if (y == 10 && *p == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}