void putchar(int c);

int add_internal(int a, int b) {
    return a + b;
}

int add(int a, int b) {
    return add_internal(a, b);
}

int main() {
    int x = 5;
    int y = 10;
    int result = add(x, y);
    
    // Test if addition works correctly
    if (result == 15) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return result;
}