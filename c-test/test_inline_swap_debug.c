// Debug inline pointer swap
void putchar(int c);

int main() {
    int a = 1;
    int b = 2;
    
    // Print initial values
    putchar('0' + a);  // Should print '1'
    putchar('0' + b);  // Should print '2'
    putchar(' ');
    
    // Create pointers
    int *pa = &a;
    int *pb = &b;
    
    // Do the swap
    int temp = *pa;
    *pa = *pb;
    *pb = temp;
    
    // Print swapped values
    putchar('0' + a);  // Should print '2'
    putchar('0' + b);  // Should print '1'
    putchar('\n');
    
    return 0;
}