void putchar(int c);

int main() {
    int m = 0;
    putchar('0' + m);
    putchar(10);
    
    m = 1;
    putchar('0' + m);
    putchar(10);
    
    m = 2;
    if (m == 2) {
        putchar('B');
        putchar(10);
    }
    
    return 0;
}