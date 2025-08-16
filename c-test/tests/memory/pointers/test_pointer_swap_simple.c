// Simple pointer swap test
void putchar(int c);

void swap(int *a, int *b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
    int x = 1;
    int y = 2;
    
    swap(&x, &y);
    
    // After swap: x should be 2, y should be 1
    putchar('0' + x);  // Should print '2'
    putchar('0' + y);  // Should print '1'
    putchar('\n');
    
    return 0;
}