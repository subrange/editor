// Debug *(++p) as single expression
void putchar(int c);

int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = arr;
    
    // Single expression
    int y = *(++p);  // Should be 20
    
    // Print the value
    putchar('0' + (y / 10));
    putchar('0' + (y % 10));
    putchar('\n');
    
    return 0;
}