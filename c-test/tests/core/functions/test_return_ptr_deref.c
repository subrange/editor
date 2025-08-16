void putchar(int c);

int* return_ptr(int *p) { 
    return p; 
}

int main() {
    int val = 42;
    int result = *return_ptr(&val);
    
    if (result == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}
