// Test pre-increment pointer (test 6 from original)
void putchar(int c);

int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = arr;
    
    // Pre-increment pointer - this is test 6
    int y = *(++p);  // p points to arr[1], y becomes 20
    
    if (y == 20) {
        putchar('Y');
    } else {
        putchar('N');
        putchar('0' + (y / 10));
        putchar('0' + (y % 10));
    }
    
    return 0;
}