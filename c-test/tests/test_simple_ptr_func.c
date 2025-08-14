void putchar(int c);

void modify_via_ptr(int *p) {
    *p = 88; // 'X'
}

int main() {
    int value = 65; // 'A'
    
    putchar(value);
    putchar('\n');
    
    modify_via_ptr(&value);
    
    putchar(value);
    putchar('\n');
    
    return 0;
}