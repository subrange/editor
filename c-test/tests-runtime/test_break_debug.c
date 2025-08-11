void putchar(int c);

int main() {
    putchar('S');
    putchar(':');
    
    for (int i = 0; i < 5; i = i + 1) {
        putchar('0' + i);
        if (i == 2) {
            putchar('B');
            break;
        }
        putchar('.');
    }
    
    putchar('E');
    putchar(10);
    return 0;
}