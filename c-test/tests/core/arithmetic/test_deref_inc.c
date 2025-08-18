// Debug dereference of incremented pointer
void putchar(int c);

int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = arr;
    
    // Step by step
    ++p;  // p now points to arr[1]
    int y = *p;  // y should be 20
    
    // Print the value
    putchar('0' + (y / 10));
    putchar('0' + (y % 10));
    putchar('\n');
    
    return 0;
}