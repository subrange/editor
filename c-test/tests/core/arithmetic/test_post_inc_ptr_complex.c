// Test the exact failing case
void putchar(int c);

int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = arr;
    int y;
    
    // This is the exact sequence from the failing test
    p = arr;  // Reset p to start of array
    y = *(p++);  // y becomes 10, p points to arr[1]
    
    putchar('0' + (y / 10));  // Should print '1'
    putchar('0' + (y % 10));  // Should print '0'
    putchar('\n');
    
    putchar('0' + (*p / 10));  // Should print '2'
    putchar('0' + (*p % 10));  // Should print '0'
    putchar('\n');
    
    // Check the condition from the test
    if (y == 10 && *p == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}