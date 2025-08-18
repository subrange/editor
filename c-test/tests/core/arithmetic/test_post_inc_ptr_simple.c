// Simple test for post-increment pointer
void putchar(int c);

int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = arr;
    
    // Post-increment pointer
    int y = *(p++);  // y becomes 10, p points to arr[1]
    
    if (y == 10) putchar('Y'); else putchar('N');
    if (*p == 20) putchar('Y'); else putchar('N');
    
    return 0;
}