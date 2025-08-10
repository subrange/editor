// Test address-of and dereference operators

void putchar(int c);

int main() {
    int x = 42;
    int *ptr = &x;  // Take address of x
    
    // Dereference ptr to get value
    int value = *ptr;
    
    // Modify through pointer
    *ptr = 100;
    
    // x should now be 100
    if (x == 100) {
        putchar('O');
        putchar('K');
        putchar('\n');
    }
    
    return 0;
}